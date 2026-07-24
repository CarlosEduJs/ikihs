mod mapper;

use std::collections::HashMap;
use std::sync::Mutex;

use ikihs_core::engine::{HighlightEngine, HighlightLine, HighlightResult, HighlightToken};
use ikihs_core::scope::{Scope, ScopeCategory};
use ikihs_core::scope::mapper::{BuiltinScopeMapper, ScopeMapper};
use ikihs_core::theme::Theme;
use ikihs_core::Error;
use tree_sitter::{Node, Parser, TreeCursor};

pub struct TreeSitterEngine {
    parser_javascript: Mutex<Parser>,
    parser_jsx: Mutex<Parser>,
    parser_typescript: Mutex<Parser>,
    parser_tsx: Mutex<Parser>,
}

impl TreeSitterEngine {
    pub fn new() -> Self {
        let mut p_js = Parser::new();
        p_js
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .expect("Failed to load JavaScript grammar");

        let mut p_jsx = Parser::new();
        p_jsx
            .set_language(&tree_sitter_javascript::LANGUAGE.into())
            .expect("Failed to load JSX grammar");

        let mut p_ts = Parser::new();
        p_ts
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .expect("Failed to load TypeScript grammar");

        let mut p_tsx = Parser::new();
        p_tsx
            .set_language(&tree_sitter_typescript::LANGUAGE_TSX.into())
            .expect("Failed to load TSX grammar");

        Self {
            parser_javascript: Mutex::new(p_js),
            parser_jsx: Mutex::new(p_jsx),
            parser_typescript: Mutex::new(p_ts),
            parser_tsx: Mutex::new(p_tsx),
        }
    }

    /// Build a fallback ScopeCategory → color lookup from the theme.
    fn build_color_map(theme: &Theme) -> HashMap<ScopeCategory, String> {
        let mapper = BuiltinScopeMapper::new();
        #[derive(Clone)]
        struct Candidate {
            dots: usize,
            index: usize,
            color: String,
        }
        let mut best: HashMap<ScopeCategory, Candidate> = HashMap::new();

        for (i, tc) in theme.token_colors.iter().enumerate() {
            if let Some(ref fg) = tc.settings.foreground {
                for scope_str in &tc.scope {
                    let cat = mapper.classify(scope_str);
                    if matches!(cat, ScopeCategory::Other(_)) {
                        continue;
                    }
                    let dots = scope_str.matches('.').count();
                    let cand = Candidate { dots, index: i, color: fg.clone() };
                    let better = best.get(&cat).map_or(true, |cur| {
                        cand.dots < cur.dots || (cand.dots == cur.dots && cand.index < cur.index)
                    });
                    if better {
                        best.insert(cat, cand);
                    }
                }
            }
        }

        let mut map: HashMap<ScopeCategory, String> = best
            .into_iter()
            .map(|(k, v)| (k, v.color))
            .collect();

        map.entry(ScopeCategory::Operator).or_insert("#D4D4D4".into());
        map.entry(ScopeCategory::Property).or_insert("#9CDCFE".into());
        map.entry(ScopeCategory::Class).or_insert("#4EC9B0".into());
        map.entry(ScopeCategory::Parameter).or_insert("#9CDCFE".into());
        map.insert(ScopeCategory::Constant, "#4FC1FF".into());

        map
    }

    /// Find the best-matching color for `scope_str` from the theme by
    /// prefix matching.  Iterates in reverse (last-wins for same scope)
    /// and prefers the longest (most specific) matching theme scope.
    fn lookup_scope_color(scope_str: &str, theme: &Theme) -> Option<String> {
        let target = scope_str.split(' ').last().unwrap_or(scope_str);
        let mut best_dots: isize = -1;
        let mut best_color = None;

        for tc in theme.token_colors.iter().rev() {
            if let Some(ref fg) = tc.settings.foreground {
                for theme_scope in &tc.scope {
                    if target == *theme_scope
                        || target.starts_with(&format!("{}.", theme_scope))
                    {
                        let dots = theme_scope.matches('.').count() as isize;
                        if dots > best_dots {
                            best_dots = dots;
                            best_color = Some(fg.clone());
                        }
                    }
                }
            }
        }
        best_color
    }

    /// Walk the tree-sitter tree depth-first, collecting flat tokens.
    fn walk_tree(
        root: Node,
        source: &str,
        theme: &Theme,
        color_map: &HashMap<ScopeCategory, String>,
    ) -> Vec<FlatToken> {
        let mut tokens = Vec::new();
        let mut cursor = root.walk();
        visit_children(&mut cursor, source, theme, color_map, &mut tokens);
        tokens
    }
}

/// Named node types that should be treated as atomic leaves,
/// even if tree-sitter gives them children (quotes, content parts).
const ATOMIC_KINDS: &[&str] = &[
    "string",
    "number",
    "comment",
    "regex",
    "hash_bang_line",
    "predefined_type",
    "literal_type",
];

fn visit_children(
    cursor: &mut TreeCursor,
    source: &str,
    theme: &Theme,
    color_map: &HashMap<ScopeCategory, String>,
    tokens: &mut Vec<FlatToken>,
) {
    if !cursor.goto_first_child() {
        return;
    }

    // Detect if we just entered a template_substitution or template_string
    // and emit the unconsumed prefix as the appropriate token.
    if let Some(parent) = cursor.node().parent() {
        let before = parent.start_byte();
        let after = cursor.node().start_byte();
        if before < after {
            let (cat, color) = match parent.kind() {
                "template_substitution" => (ScopeCategory::Keyword, "#569CD6"),
                "template_string" => (ScopeCategory::String, "#CE9178"),
                _ => (ScopeCategory::Other("default".into()), "#D4D4D4"),
            };
            tokens.push(FlatToken {
                start: before,
                end: after,
                category: cat,
                color: color.into(),
            });
        }
    }

    loop {
        let node = cursor.node();
        let kind = node.kind();

        if kind == "template_string" || kind == "template_substitution" {
            visit_children(cursor, source, theme, color_map, tokens);
        } else if ATOMIC_KINDS.contains(&kind) {
            emit_leaf(node, cursor, source, theme, color_map, tokens);
        } else if node.child_count() == 0 {
            emit_leaf(node, cursor, source, theme, color_map, tokens);
        } else {
            visit_children(cursor, source, theme, color_map, tokens);
        }

        if !cursor.goto_next_sibling() {
            // After the last child of a template node, emit the trailing gap.
            if let Some(parent) = cursor.node().parent() {
                let before = cursor.node().end_byte();
                let after = parent.end_byte();
                if before < after {
                    let (cat, color) = match parent.kind() {
                        "template_substitution" => (ScopeCategory::Keyword, "#569CD6"),
                        "template_string" => (ScopeCategory::String, "#CE9178"),
                        _ => (ScopeCategory::Other("default".into()), "#D4D4D4"),
                    };
                    tokens.push(FlatToken {
                        start: before,
                        end: after,
                        category: cat,
                        color: color.into(),
                    });
                }
            }
            break;
        }
    }

    cursor.goto_parent();
}

fn emit_leaf(
    node: Node,
    cursor: &TreeCursor,
    source: &str,
    theme: &Theme,
    color_map: &HashMap<ScopeCategory, String>,
    tokens: &mut Vec<FlatToken>,
) {
    let kind = node.kind();
    let start = node.start_byte();
    let end = node.end_byte();
    if start >= end {
        return;
    }

    let text = &source[start..end];
    let parent = node.parent();
    let parent_kind = parent.as_ref().map(|p| p.kind());
    let grandparent_kind = parent.as_ref().and_then(|p| p.parent()).map(|gp| gp.kind());
    let field_name = cursor.field_name();
    let is_named = node.is_named();

    let (mut category, mut scope_string) =
        mapper::node_to_category(kind, text, parent_kind, grandparent_kind, field_name, is_named);

    if category == ScopeCategory::Variable && kind == "identifier" {
        if let Some(p) = parent.as_ref() {
            if p.kind() == "variable_declarator" {
                if let Some(gp) = p.parent() {
                    if gp.kind() == "lexical_declaration" {
                        if let Some(first) = gp.child(0) {
                            if &source[first.start_byte()..first.end_byte()] == "const" {
                                category = ScopeCategory::Constant;
                                scope_string = "source.ts variable.other.constant".into();
                            }
                        }
                    }
                }
            }
        }
    }

    // Primary: scope-prefix match against the theme's token_colors.
    // Fallback: category-based color map.
    let color = TreeSitterEngine::lookup_scope_color(&scope_string, theme)
        .or_else(|| {
            color_map.get(&category).cloned()
        })
        .unwrap_or_else(|| "#D4D4D4".to_string());

    // Merge with previous token if same category and adjacent
    if let Some(last) = tokens.last_mut() {
        if last.category == category && last.end == start {
            last.end = end;
            return;
        }
    }

    tokens.push(FlatToken {
        start,
        end,
        category,
        color,
    });
}

struct FlatToken {
    start: usize,
    end: usize,
    category: ScopeCategory,
    color: String,
}

impl HighlightEngine for TreeSitterEngine {
    fn highlight(
        &self,
        source: &str,
        lang: &str,
        theme: &Theme,
    ) -> Result<HighlightResult, Error> {
        let parser = match lang {
            "javascript" | "js" => &self.parser_javascript,
            "jsx" => &self.parser_jsx,
            "typescript" | "ts" => &self.parser_typescript,
            "tsx" => &self.parser_tsx,
            _ => return Err(Error::GrammarNotFound(lang.to_string())),
        };

        let color_map = Self::build_color_map(theme);

        let tree = parser
            .lock()
            .map_err(|e| Error::Engine(format!("lock error: {e}")))?
            .parse(source, None)
            .ok_or_else(|| Error::Engine("tree-sitter parse failed".into()))?;

        let flat = Self::walk_tree(tree.root_node(), source, theme, &color_map);
        let lines = split_into_highlight_lines(&flat, source);

        Ok(HighlightResult {
            lines,
            language: lang.to_string(),
        })
    }

    fn list_grammars(&self) -> Vec<String> {
        vec![
            "javascript".into(), "js".into(), "jsx".into(),
            "typescript".into(), "ts".into(), "tsx".into(),
        ]
    }
}

/// Convert flat byte-offset tokens into per-line HighlightToken vectors.
/// Fills gaps between tokens on each line with the default foreground
/// color (#D4D4D4) so that every byte position is covered by a token.
/// Uses `source.lines()` to match the same line-splitting that the
/// test comparison code uses, avoiding off-by-one errors with blank lines.
fn split_into_highlight_lines(
    flat: &[FlatToken],
    source: &str,
) -> Vec<HighlightLine> {
    let source_lines: Vec<&str> = source.lines().collect();
    let num_lines = source_lines.len();
    let mut result: Vec<HighlightLine> = Vec::with_capacity(num_lines);

    // Build line→byte-offset table matching source.lines() traversal
    let mut line_offsets: Vec<usize> = Vec::with_capacity(num_lines);
    let mut offset = 0;
    for line_text in &source_lines {
        line_offsets.push(offset);
        offset += line_text.len() + 1;
    }

    for (li, line_text) in source_lines.iter().enumerate() {
        let line_len = line_text.len();
        let line_pos = line_offsets[li];
        let mut pairs: Vec<(usize, usize, String)> = Vec::new();

        for token in flat {
            if token.start >= line_pos && token.start < line_pos + line_len {
                let rel_start = token.start - line_pos;
                let rel_end = line_len.min(token.end - line_pos);
                if rel_start < rel_end {
                    pairs.push((rel_start, rel_end, token.color.clone()));
                }
            }
        }

        pairs.sort_by_key(|&(s, e, _)| (s, e));

        let mut tokens: Vec<HighlightToken> = Vec::new();
        let mut cursor = 0;
        for (s, e, color) in &pairs {
            if *s > cursor {
                tokens.push(HighlightToken {
                    start: cursor,
                    end: *s,
                    scope: Scope::new("").with_category(ScopeCategory::Other("default".into())),
                    color: "#D4D4D4".into(),
                    font_style: String::new(),
                });
            }
            if *s < cursor {
                continue;
            }
            tokens.push(HighlightToken {
                start: *s,
                end: *e,
                scope: Scope::new("").with_category(ScopeCategory::Other("default".into())),
                color: color.clone(),
                font_style: String::new(),
            });
            cursor = *e;
        }
        if cursor < line_len {
            tokens.push(HighlightToken {
                start: cursor,
                end: line_len,
                scope: Scope::new("").with_category(ScopeCategory::Other("default".into())),
                color: "#D4D4D4".into(),
                font_style: String::new(),
            });
        }

        dedup_adjacent(&mut tokens);
        result.push(HighlightLine { tokens });
    }

    result
}

fn dedup_adjacent(tokens: &mut Vec<HighlightToken>) {
    if tokens.len() <= 1 {
        return;
    }
    let mut i = 0;
    while i + 1 < tokens.len() {
        if tokens[i].end >= tokens[i + 1].start
            && tokens[i].color == tokens[i + 1].color
        {
            tokens[i].end = tokens[i + 1].end;
            tokens.remove(i + 1);
        } else {
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_color_map() {
        let theme = Theme {
            name: "test".into(),
            theme_type: None,
            colors: std::collections::HashMap::new(),
            token_colors: vec![],
        };
        let map = TreeSitterEngine::build_color_map(&theme);
        // Fallback entries are always present
        assert_eq!(map.get(&ScopeCategory::Operator).unwrap(), "#D4D4D4");
        assert_eq!(map.get(&ScopeCategory::Constant).unwrap(), "#4FC1FF");
    }
}
