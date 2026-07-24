use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use ikihs_core::engine::HighlightEngine;
use ikihs_core::scope::ScopeCategory;
use ikihs_core::scope::mapper::{BuiltinScopeMapper, ScopeMapper};
use ikihs_core::theme::Theme;
use ikihs_engine_treesitter::TreeSitterEngine;
use ikihs_themes::vscode_theme::VscodeThemeParser;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../crates/ikihs-engine-syntect/fixtures")
}

fn load_dark_plus_theme() -> Theme {
    let path = fixture_dir().join("dark-plus.json");
    let json = fs::read_to_string(&path).expect("failed to read dark-plus theme");
    VscodeThemeParser::parse_json(&json).expect("failed to parse dark-plus theme")
}

#[derive(serde::Deserialize)]
struct ShikiFixture {
    language: String,
    tokens: Vec<Vec<ShikiToken>>,
}

#[derive(serde::Deserialize)]
struct ShikiToken {
    content: String,
    offset: usize,
    color: String,
}

#[test]
fn diagnose_color_map() {
    let theme = load_dark_plus_theme();
    let mapper = BuiltinScopeMapper::new();

    println!("── Color map from Theme token_colors ──");
    let mut color_map: HashMap<ScopeCategory, String> = HashMap::new();

    for tc in theme.token_colors.iter().rev() {
        if let Some(ref fg) = tc.settings.foreground {
            for scope_str in &tc.scope {
                let cat = mapper.classify(scope_str);
                if !matches!(cat, ScopeCategory::Other(_)) {
                    let old = color_map.insert(cat.clone(), fg.clone());
                    if old.is_none() || old.as_deref() != Some(fg.as_str()) {
                        println!("  {:<20} → {:<8}  (scope: {})", format!("{:?}", cat), fg, scope_str);
                    }
                } else {
                    println!("  (unmapped scope: {})", scope_str);
                }
            }
        }
    }

    println!("\n── Final color map ──");
    for (cat, color) in &color_map {
        println!("  {:<20} → {}", format!("{:?}", cat), color);
    }
}

fn diagnose_generic(fixture_name: &str) {
    let source_path = fixture_dir().join(&format!("typescript/{}.ts", fixture_name));
    let source = fs::read_to_string(&source_path).unwrap();

    let mut expected_path = fixture_dir().join(&format!("typescript/{}", fixture_name));
    expected_path.set_extension("shiki.json");
    let expected: ShikiFixture =
        serde_json::from_str(&fs::read_to_string(&expected_path).unwrap()).unwrap();

    let engine = TreeSitterEngine::new();
    let theme = load_dark_plus_theme();
    let result = engine.highlight(&source, "typescript", &theme).unwrap();

    let source_bytes = source.len();
    let mut ikihs_color = vec![""; source_bytes];
    let mut ikihs_cat = vec![String::new(); source_bytes];
    let mut line_offset = 0;
    let source_lines: Vec<&str> = source.lines().collect();

    for (li, line) in result.lines.iter().enumerate() {
        for t in &line.tokens {
            let abs_start = line_offset + t.start;
            let abs_end = line_offset + t.end;
            for i in abs_start..abs_end.min(source_bytes) {
                ikihs_color[i] = &t.color;
                ikihs_cat[i] = format!("{:?}", t.scope.category());
            }
        }
        if let Some(line) = source_lines.get(li) {
            line_offset += line.len() + 1;
        }
    }

    let mut shiki_color = vec![""; source_bytes];
    for line_tokens in &expected.tokens {
        for t in line_tokens {
            let start = t.offset;
            let end = t.offset + t.content.len();
            for color in &mut shiki_color[start..end.min(source_bytes)] {
                *color = &t.color;
            }
        }
    }

    println!("\n── DIFF: typescript/{} ──", fixture_name);
    println!("{:>4} {:>6} {:>8} {:>8}  {:<15}  match", "pos", "char", "shiki", "ikihs", "cat");
    for pos in 0..source_bytes {
        let ch = source[pos..].chars().next().unwrap();
        let is_match = ikihs_color[pos].eq_ignore_ascii_case(shiki_color[pos]);
        if !is_match {
            println!(
                "{pos:>4} {:>6} {:>8} {:>8}  {:<15}  DIFF",
                ch,
                shiki_color[pos].to_lowercase(),
                ikihs_color[pos].to_lowercase(),
                ikihs_cat[pos]
            );
        }
    }
}

#[test]
fn diagnose_typescript_generics() {
    diagnose_generic("generics");
}

#[test]
fn diagnose_typescript_types() {
    let source_path = fixture_dir().join("typescript/types.ts");
    let source = fs::read_to_string(&source_path).unwrap();

    let mut expected_path = fixture_dir().join("typescript/types");
    expected_path.set_extension("shiki.json");
    let expected: ShikiFixture =
        serde_json::from_str(&fs::read_to_string(&expected_path).unwrap()).unwrap();

    let engine = TreeSitterEngine::new();
    let theme = load_dark_plus_theme();
    let result = engine.highlight(&source, "typescript", &theme).unwrap();

    // Build ikihs color map
    let source_bytes = source.len();
    let mut ikihs_color = vec![""; source_bytes];
    let mut ikihs_cat = vec![String::new(); source_bytes];
    let mut line_offset = 0;
    let source_lines: Vec<&str> = source.lines().collect();

    for (li, line) in result.lines.iter().enumerate() {
        for t in &line.tokens {
            let abs_start = line_offset + t.start;
            let abs_end = line_offset + t.end;
            for i in abs_start..abs_end.min(source_bytes) {
                ikihs_color[i] = &t.color;
                ikihs_cat[i] = format!("{:?}", t.scope.category());
            }
        }
        if let Some(line) = source_lines.get(li) {
            line_offset += line.len() + 1;
        }
    }

    // Build shiki color map
    let mut shiki_color = vec![""; source_bytes];
    for line_tokens in &expected.tokens {
        for t in line_tokens {
            let start = t.offset;
            let end = t.offset + t.content.len();
            for color in &mut shiki_color[start..end.min(source_bytes)] {
                *color = &t.color;
            }
        }
    }

    println!("\n── DIFF: typescript/types ──");
    println!("{:>4} {:>6} {:>8} {:>8}  {:<15}  match", "pos", "char", "shiki", "ikihs", "cat");
    for pos in 0..source_bytes {
        let ch = source[pos..].chars().next().unwrap();
        let is_match = ikihs_color[pos].eq_ignore_ascii_case(shiki_color[pos]);
        if !is_match {
            println!(
                "{pos:>4} {:>6} {:>8} {:>8}  {:<15}  DIFF",
                ch,
                shiki_color[pos].to_lowercase(),
                ikihs_color[pos].to_lowercase(),
                ikihs_cat[pos]
            );
        }
    }
}
