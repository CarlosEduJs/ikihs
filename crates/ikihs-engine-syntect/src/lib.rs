use std::str::FromStr;

use ikihs_core::Error;
use ikihs_core::engine::{HighlightEngine, HighlightLine, HighlightResult, HighlightToken};
use ikihs_core::scope::Scope;
use ikihs_core::scope::mapper::{BuiltinScopeMapper, ScopeMapper};
use ikihs_core::theme::Theme;

use syntect::highlighting::ScopeSelector;
use syntect::highlighting::ScopeSelectors;
use syntect::highlighting::{
    Color as SyntectColor, FontStyle as SyntectFontStyle, Highlighter,
    StyleModifier as SyntectStyleModifier, Theme as SyntectTheme, ThemeItem, ThemeSettings,
};
use syntect::parsing::{ScopeStack, SyntaxSet};

pub struct SyntectEngine {
    syntax_set: SyntaxSet,
    mapper: Box<dyn ScopeMapper>,
}

impl SyntectEngine {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            mapper: Box::new(BuiltinScopeMapper::new()),
        }
    }

    pub fn with_mapper(mapper: impl ScopeMapper + 'static) -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            mapper: Box::new(mapper),
        }
    }
}

impl Default for SyntectEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HighlightEngine for SyntectEngine {
    fn highlight(&self, source: &str, lang: &str, theme: &Theme) -> Result<HighlightResult, Error> {
        let syntax = self
            .syntax_set
            .find_syntax_by_token(lang)
            .ok_or_else(|| Error::GrammarNotFound(lang.to_string()))?;

        let syntect_theme = convert_theme(theme)?;
        let highlighter = Highlighter::new(&syntect_theme);

        let mut parse_state = syntect::parsing::ParseState::new(syntax);
        let mut scope_stack = ScopeStack::new();

        let mut lines = Vec::new();

        for line in source.lines() {
            let ops = parse_state
                .parse_line(line, &self.syntax_set)
                .map_err(|e| Error::Engine(format!("parse error: {}", e)))?;

            let mut tokens = Vec::new();
            let mut last_offset = 0;

            for (offset, op) in &ops {
                if *offset > last_offset {
                    build_token(
                        &mut tokens,
                        line,
                        last_offset,
                        *offset,
                        &scope_stack,
                        &highlighter,
                        &*self.mapper,
                    );
                }

                scope_stack
                    .apply(op)
                    .map_err(|e| Error::Engine(format!("scope error: {}", e)))?;

                last_offset = *offset;
            }

            if last_offset < line.len() {
                build_token(
                    &mut tokens,
                    line,
                    last_offset,
                    line.len(),
                    &scope_stack,
                    &highlighter,
                    &*self.mapper,
                );
            }

            lines.push(HighlightLine { tokens });
        }

        Ok(HighlightResult {
            lines,
            language: syntax.name.clone(),
        })
    }

    fn list_grammars(&self) -> Vec<String> {
        self.syntax_set
            .syntaxes()
            .iter()
            .map(|s| s.name.clone())
            .collect()
    }

    fn has_grammar(&self, lang: &str) -> bool {
        self.syntax_set.find_syntax_by_token(lang).is_some()
    }
}

fn build_token(
    tokens: &mut Vec<HighlightToken>,
    _line: &str,
    start: usize,
    end: usize,
    scope_stack: &ScopeStack,
    highlighter: &Highlighter,
    mapper: &dyn ScopeMapper,
) {
    let scope_str = scope_stack.to_string().trim().to_string();
    let category = classify_scope_stack(scope_stack, mapper);
    let style = highlighter.style_for_stack(scope_stack.as_slice());

    tokens.push(HighlightToken {
        start,
        end,
        scope: Scope::new(&scope_str).with_category(category),
        color: syntect_color_to_hex(style.foreground),
        font_style: syntect_font_style_to_string(style.font_style),
    });
}

fn classify_scope_stack(stack: &ScopeStack, mapper: &dyn ScopeMapper) -> ikihs_core::ScopeCategory {
    stack
        .as_slice()
        .iter()
        .filter_map(|scope| {
            let scope_str = scope.to_string();
            let category = mapper.classify(&scope_str);
            if !matches!(category, ikihs_core::ScopeCategory::Other(_)) {
                Some(category)
            } else {
                None
            }
        })
        .last()
        .unwrap_or(ikihs_core::ScopeCategory::Other(
            stack.to_string().trim().to_string(),
        ))
}

fn compatibility_rules() -> Vec<ThemeItem> {
    let mut rules = Vec::new();

    let pairs = [
        // Syntect grammars scope `println!` etc as `support.macro`;
        // map it to function color like `support.function`
        (vec!["support.macro"], "#DCDCAA"),
        // Syntect JS grammar scopes `.name` in `this.name` as `meta.property.object`
        // dark-plus has no rule for it; map to variable color
        (vec!["meta.property.object"], "#9CDCFE"),
        // Python f-string interpolation `{` `}` → keyword color
        (
            vec![
                "punctuation.section.interpolation.begin",
                "punctuation.section.interpolation.end",
            ],
            "#569CD6",
        ),
        // Python decorator `@` → function color
        (vec!["punctuation.definition.annotation"], "#DCDCAA"),
        // Python decorator name `dataclass` → function color
        (vec!["variable.annotation"], "#DCDCAA"),
        // console etc → variable color (Syntect scopes as support.type.object.*,
        // but Shiki treats as variable → #9CDCFE)
        (vec!["support.type.object"], "#9CDCFE"),
    ];

    for (scopes, fg) in &pairs {
        let owned: Vec<String> = scopes.iter().map(|s| s.to_string()).collect();
        let selectors = parse_scope_selectors(&owned);
        let style = SyntectStyleModifier {
            foreground: parse_hex_color(fg),
            background: None,
            font_style: None,
        };
        rules.push(ThemeItem {
            scope: selectors,
            style,
        });
    }

    rules
}

fn convert_theme(theme: &Theme) -> Result<SyntectTheme, Error> {
    let foreground = theme
        .colors
        .get("editor.foreground")
        .and_then(|c| parse_hex_color(c));
    let background = theme
        .colors
        .get("editor.background")
        .and_then(|c| parse_hex_color(c));

    // Syntect uses first-match-wins for same-specificity selectors,
    // but VS Code uses last-match-wins. We reverse theme rules so that
    // later rules in the JSON (which should take priority) are processed first.
    let mut scopes: Vec<ThemeItem> = Vec::new();

    // Add compatibility rules for scopes produced by Syntect's grammars
    // that the dark-plus theme doesn't cover. These must be added BEFORE
    // the theme rules so syntect's first-match-wins picks them for same-specificity.
    scopes.extend(compatibility_rules());

    for tc in theme.token_colors.iter().rev() {
        let selectors = parse_scope_selectors(&tc.scope);
        let style = SyntectStyleModifier {
            foreground: tc.settings.foreground.as_deref().and_then(parse_hex_color),
            background: tc.settings.background.as_deref().and_then(parse_hex_color),
            font_style: parse_font_style(&tc.settings.font_style),
        };
        scopes.push(ThemeItem {
            scope: selectors,
            style,
        });
    }

    Ok(SyntectTheme {
        name: Some(theme.name.clone()),
        author: None,
        settings: ThemeSettings {
            foreground,
            background,
            ..ThemeSettings::default()
        },
        scopes,
    })
}

fn parse_scope_selectors(scopes: &[String]) -> ScopeSelectors {
    let selectors: Vec<ScopeSelector> = scopes
        .iter()
        .filter_map(|s| ScopeStack::from_str(s).ok())
        .map(|path| ScopeSelector {
            path,
            excludes: Vec::new(),
        })
        .collect();
    ScopeSelectors { selectors }
}

fn parse_hex_color(hex: &str) -> Option<SyntectColor> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(SyntectColor { r, g, b, a: 255 })
    } else if hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
        Some(SyntectColor { r, g, b, a })
    } else {
        None
    }
}

fn parse_font_style(s: &Option<String>) -> Option<SyntectFontStyle> {
    let s = s.as_ref()?;
    let mut style = SyntectFontStyle::empty();
    for part in s.split_whitespace() {
        match part {
            "bold" => style |= SyntectFontStyle::BOLD,
            "italic" => style |= SyntectFontStyle::ITALIC,
            "underline" => style |= SyntectFontStyle::UNDERLINE,
            _ => {}
        }
    }
    if style.is_empty() { None } else { Some(style) }
}

fn syntect_color_to_hex(color: SyntectColor) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
}

fn syntect_font_style_to_string(style: SyntectFontStyle) -> String {
    let mut parts = Vec::new();
    if style.contains(SyntectFontStyle::BOLD) {
        parts.push("bold");
    }
    if style.contains(SyntectFontStyle::ITALIC) {
        parts.push("italic");
    }
    if style.contains(SyntectFontStyle::UNDERLINE) {
        parts.push("underline");
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ikihs_core::engine::HighlightEngine;
    use ikihs_core::theme::{TokenColor, TokenSettings};
    use std::collections::HashMap;

    fn make_test_theme() -> Theme {
        Theme {
            name: "test".into(),
            theme_type: None,
            colors: {
                let mut c = HashMap::new();
                c.insert("editor.foreground".into(), "#abb2bf".into());
                c.insert("editor.background".into(), "#282c34".into());
                c
            },
            token_colors: vec![
                TokenColor {
                    scope: vec!["comment".into()],
                    settings: TokenSettings {
                        foreground: Some("#5c6370".into()),
                        background: None,
                        font_style: Some("italic".into()),
                    },
                },
                TokenColor {
                    scope: vec!["keyword".into()],
                    settings: TokenSettings {
                        foreground: Some("#c678dd".into()),
                        background: None,
                        font_style: None,
                    },
                },
                TokenColor {
                    scope: vec!["storage".into()],
                    settings: TokenSettings {
                        foreground: Some("#c678dd".into()),
                        background: None,
                        font_style: None,
                    },
                },
                TokenColor {
                    scope: vec!["string".into()],
                    settings: TokenSettings {
                        foreground: Some("#98c379".into()),
                        background: None,
                        font_style: None,
                    },
                },
                TokenColor {
                    scope: vec!["constant.numeric".into()],
                    settings: TokenSettings {
                        foreground: Some("#d19a66".into()),
                        background: None,
                        font_style: None,
                    },
                },
                TokenColor {
                    scope: vec!["entity.name.function".into()],
                    settings: TokenSettings {
                        foreground: Some("#61afef".into()),
                        background: None,
                        font_style: None,
                    },
                },
            ],
        }
    }

    #[test]
    fn test_highlight_rust() {
        let engine = SyntectEngine::new();
        let theme = make_test_theme();
        let source = "fn main() {\n    let x = 42;\n    println!(\"hello\");\n}";

        let result = engine.highlight(source, "Rust", &theme).unwrap();
        assert_eq!(result.lines.len(), 4);
        assert_eq!(result.language, "Rust");

        let line0 = &result.lines[0];
        assert!(!line0.tokens.is_empty(), "first line should have tokens");

        let keyword_tokens: Vec<_> = line0
            .tokens
            .iter()
            .filter(|t| matches!(t.scope.category(), ikihs_core::ScopeCategory::Keyword))
            .collect();
        assert!(
            !keyword_tokens.is_empty(),
            "should find keyword token for 'fn'"
        );

        let span = "fn main() {";
        let fn_token = line0.tokens.iter().find(|t| &span[t.start..t.end] == "fn");
        assert!(fn_token.is_some(), "should have a token for 'fn'");
        assert_eq!(fn_token.unwrap().color, "#C678DD");
    }

    #[test]
    fn test_highlight_javascript() {
        let engine = SyntectEngine::new();
        let theme = make_test_theme();
        let source = "function greet(name) {\n    return \"Hello, \" + name;\n}";

        let result = engine.highlight(source, "JavaScript", &theme).unwrap();
        assert_eq!(result.lines.len(), 3);
        assert_eq!(result.language, "JavaScript");

        let function_tokens: Vec<_> = result.lines[0]
            .tokens
            .iter()
            .filter(|t| matches!(t.scope.category(), ikihs_core::ScopeCategory::Function))
            .collect();
        assert!(!function_tokens.is_empty(), "should find function token");
    }

    #[test]
    fn test_highlight_empty_string() {
        let engine = SyntectEngine::new();
        let theme = make_test_theme();

        let result = engine.highlight("", "Rust", &theme).unwrap();
        assert_eq!(result.lines.len(), 0);
    }

    #[test]
    fn test_highlight_single_line() {
        let engine = SyntectEngine::new();
        let theme = make_test_theme();

        let result = engine.highlight("42", "Rust", &theme).unwrap();
        assert_eq!(result.lines.len(), 1);
    }

    #[test]
    fn test_unknown_language() {
        let engine = SyntectEngine::new();
        let theme = make_test_theme();

        let result = engine.highlight("hello", "nonexistent-lang-xyz", &theme);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::GrammarNotFound(_)));
    }

    #[test]
    fn test_list_grammars() {
        let engine = SyntectEngine::new();
        let grammars = engine.list_grammars();
        assert!(grammars.contains(&"Rust".to_string()));
        assert!(grammars.contains(&"JavaScript".to_string()));
        assert!(grammars.contains(&"Python".to_string()));
    }

    #[test]
    fn test_has_grammar() {
        let engine = SyntectEngine::new();
        assert!(engine.has_grammar("Rust"));
        assert!(engine.has_grammar("rs"));
        assert!(engine.has_grammar("js"));
        assert!(engine.has_grammar("py"));
        assert!(!engine.has_grammar("nonexistent"));
    }

    #[test]
    fn test_comment_italic() {
        let engine = SyntectEngine::new();
        let theme = make_test_theme();
        let source = "// this is a comment\n";

        let result = engine.highlight(source, "Rust", &theme).unwrap();
        let comment_tokens: Vec<_> = result.lines[0]
            .tokens
            .iter()
            .filter(|t| matches!(t.scope.category(), ikihs_core::ScopeCategory::Comment))
            .collect();
        assert!(!comment_tokens.is_empty());
        assert_eq!(comment_tokens[0].font_style, "italic");
        assert_eq!(comment_tokens[0].color, "#5C6370");
    }

    #[test]
    fn test_convert_theme_without_overrides() {
        let theme = Theme {
            name: "empty".into(),
            theme_type: None,
            colors: HashMap::new(),
            token_colors: vec![],
        };
        let syntect_theme = convert_theme(&theme).unwrap();
        assert_eq!(syntect_theme.name, Some("empty".into()));
        assert!(syntect_theme.settings.foreground.is_none());
        assert!(syntect_theme.settings.background.is_none());
        assert_eq!(syntect_theme.scopes.len(), 6);
    }

    #[test]
    fn test_convert_theme_with_colors() {
        let theme = Theme {
            name: "mytheme".into(),
            theme_type: None,
            colors: {
                let mut c = HashMap::new();
                c.insert("editor.foreground".into(), "#ffffff".into());
                c.insert("editor.background".into(), "#000000".into());
                c
            },
            token_colors: vec![TokenColor {
                scope: vec!["comment".into(), "comment.line".into()],
                settings: TokenSettings {
                    foreground: Some("#888888".into()),
                    background: None,
                    font_style: Some("bold italic".into()),
                },
            }],
        };
        let syntect_theme = convert_theme(&theme).unwrap();
        assert_eq!(
            syntect_theme.settings.foreground,
            Some(SyntectColor {
                r: 255,
                g: 255,
                b: 255,
                a: 255
            })
        );
        assert_eq!(
            syntect_theme.settings.background,
            Some(SyntectColor {
                r: 0,
                g: 0,
                b: 0,
                a: 255
            })
        );
        assert_eq!(syntect_theme.scopes.len(), 7);
        assert_eq!(syntect_theme.scopes[6].scope.selectors.len(), 2);
    }

    #[test]
    fn syntaxes_include_rust_and_javascript() {
        let engine = SyntectEngine::new();
        let names = engine.list_grammars();
        println!("Available syntaxes ({}):", names.len());
        for name in names {
            println!("  {}", name);
        }
        // Just a sanity check
        assert!(engine.has_grammar("Rust"));
    }
}
