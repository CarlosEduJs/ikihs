use std::collections::HashMap;

use napi_derive::napi;

use ikihs_core::engine::HighlightEngine;
use ikihs_core::theme::Theme;
use ikihs_engine_composite::CompositeEngine;
use ikihs_themes::vscode_theme::VscodeThemeParser;

#[napi(object)]
pub struct JsHighlighterOptions {
    pub themes: Vec<serde_json::Value>,
    pub langs: Vec<String>,
}

#[napi(object)]
pub struct JsThemedToken {
    pub content: String,
    pub offset: i32,
    pub color: Option<String>,
    pub font_style: Option<i32>,
    pub scope: String,
    pub category: String,
}

#[napi]
pub struct JsHighlighter {
    engine: CompositeEngine,
    themes: HashMap<String, Theme>,
    loaded_languages: Vec<String>,
}

#[napi]
impl JsHighlighter {
    #[napi(constructor)]
    pub fn new(options: JsHighlighterOptions) -> napi::Result<Self> {
        let mut themes = HashMap::new();
        for theme_json in &options.themes {
            let json_str = serde_json::to_string(theme_json)
                .map_err(|e| napi::Error::from_reason(format!("invalid theme json: {e}")))?;
            let theme = VscodeThemeParser::parse_json(&json_str)
                .map_err(|e| napi::Error::from_reason(format!("failed to parse theme: {e}")))?;
            let name = theme.name.clone();
            themes.insert(name, theme);
        }

        let engine = CompositeEngine::new();

        for lang in &options.langs {
            if !engine.has_grammar(lang) {
                return Err(napi::Error::from_reason(format!(
                    "language '{lang}' not found. Available: {}",
                    engine.list_grammars().join(", ")
                )));
            }
        }

        Ok(JsHighlighter {
            engine,
            themes,
            loaded_languages: options.langs,
        })
    }

    #[napi]
    pub fn code_to_tokens_base(
        &self,
        code: String,
        lang: String,
        theme_name: String,
    ) -> napi::Result<Vec<Vec<JsThemedToken>>> {
        let theme = self
            .themes
            .get(&theme_name)
            .ok_or_else(|| napi::Error::from_reason(format!("theme '{theme_name}' not found")))?;

        let result = self
            .engine
            .highlight(&code, &lang, theme)
            .map_err(|e| napi::Error::from_reason(format!("highlight error: {e}")))?;

        let source = code.as_str();
        let mut line_byte_start: usize = 0;
        let mut all_lines = Vec::new();

        for (li, line) in result.lines.iter().enumerate() {
            let line_end = source[line_byte_start..]
                .find('\n')
                .map(|i| line_byte_start + i)
                .unwrap_or(source.len());
            let line_content = &source[line_byte_start..line_end];

            let mut tokens_line = Vec::new();
            for t in &line.tokens {
                let start = t.start.min(line_content.len());
                let end = t.end.min(line_content.len());
                if start >= end {
                    continue;
                }
                let content = &line_content[start..end];

                tokens_line.push(JsThemedToken {
                    content: content.to_string(),
                    offset: (line_byte_start + start) as i32,
                    color: Some(t.color.clone()),
                    font_style: font_style_to_shiki(&t.font_style),
                    scope: t.scope.raw().to_string(),
                    category: t.scope.category().to_string(),
                });
            }

            all_lines.push(tokens_line);
            if li + 1 < result.lines.len() {
                line_byte_start = line_end + 1;
            } else {
                line_byte_start = source.len();
            }
        }

        Ok(all_lines)
    }

    #[napi]
    pub fn load_theme(&mut self, theme_json: serde_json::Value) -> napi::Result<()> {
        let json_str = serde_json::to_string(&theme_json)
            .map_err(|e| napi::Error::from_reason(format!("invalid theme json: {e}")))?;
        let theme = VscodeThemeParser::parse_json(&json_str)
            .map_err(|e| napi::Error::from_reason(format!("failed to parse theme: {e}")))?;
        let name = theme.name.clone();
        self.themes.insert(name, theme);
        Ok(())
    }

    #[napi]
    pub fn load_language(&mut self, lang: String) -> napi::Result<()> {
        if !self.engine.has_grammar(&lang) {
            return Err(napi::Error::from_reason(format!(
                "language '{lang}' not found"
            )));
        }
        if !self.loaded_languages.contains(&lang) {
            self.loaded_languages.push(lang);
        }
        Ok(())
    }

    #[napi]
    pub fn get_loaded_languages(&self) -> Vec<String> {
        self.loaded_languages.clone()
    }

    #[napi]
    pub fn get_loaded_themes(&self) -> Vec<String> {
        let mut names: Vec<String> = self.themes.keys().cloned().collect();
        names.sort();
        names
    }

    #[napi]
    pub fn get_theme(&self, theme_name: String) -> napi::Result<serde_json::Value> {
        let theme = self
            .themes
            .get(&theme_name)
            .ok_or_else(|| napi::Error::from_reason(format!("theme '{theme_name}' not found")))?;
        let json_str = serde_json::to_string(theme)
            .map_err(|e| napi::Error::from_reason(format!("serialize error: {e}")))?;
        let value: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| napi::Error::from_reason(format!("deserialize error: {e}")))?;
        Ok(value)
    }

    #[napi]
    pub fn code_to_html(
        &self,
        code: String,
        lang: String,
        theme_name: String,
    ) -> napi::Result<String> {
        let tokens = self.code_to_tokens_base(code, lang, theme_name)?;
        let mut html = String::new();
        for (li, line) in tokens.iter().enumerate() {
            if li > 0 {
                html.push('\n');
            }
            for token in line {
                let escaped = token
                    .content
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                if let Some(ref color) = token.color {
                    html.push_str(&format!(
                        r#"<span style="color:{}">{}</span>"#,
                        color, escaped
                    ));
                } else {
                    html.push_str(&escaped);
                }
            }
        }
        Ok(html)
    }
}

fn font_style_to_shiki(s: &str) -> Option<i32> {
    if s.is_empty() {
        return None;
    }
    let mut result = 0i32;
    if s.contains("italic") {
        result |= 1;
    }
    if s.contains("bold") {
        result |= 2;
    }
    if s.contains("underline") {
        result |= 4;
    }
    if result == 0 {
        None
    } else {
        Some(result)
    }
}
