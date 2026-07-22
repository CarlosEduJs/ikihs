use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use ikihs_core::engine::HighlightEngine;
use ikihs_core::theme::Theme;
use ikihs_engine_syntect::SyntectEngine;
use ikihs_themes::vscode_theme::VscodeThemeParser;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
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

fn load_dark_plus_theme() -> Theme {
    let json = include_str!("../fixtures/dark-plus.json");
    VscodeThemeParser::parse_json(json).expect("failed to parse dark-plus theme")
}

fn escape_char(c: char) -> String {
    match c {
        ' ' => "·".to_string(),
        '\n' => "⏎".to_string(),
        '\t' => "→".to_string(),
        '\r' => "←".to_string(),
        c if c.is_control() => format!("�{:02x}", c as u8),
        c => c.to_string(),
    }
}

/// Build a color map per byte position from HighlightTokens (using absolute offsets)
fn build_ikihs_color_map<'a>(
    source: &'a str,
    ikihs_result: &'a ikihs_core::engine::HighlightResult,
) -> Vec<(&'a str, &'a str, String)> {
    let source_bytes = source.len();
    let mut color_map = vec![""; source_bytes];
    let mut scope_map = vec![""; source_bytes];
    let mut cat_map = vec![String::new(); source_bytes];
    let source_str = source;

    let mut line_offset = 0;
    let source_lines: Vec<&str> = source_str.lines().collect();
    for (li, line) in ikihs_result.lines.iter().enumerate() {
        for t in &line.tokens {
            let abs_start = line_offset + t.start;
            let abs_end = line_offset + t.end;
            for i in abs_start..abs_end.min(source_bytes) {
                color_map[i] = &t.color;
                scope_map[i] = t.scope.raw();
                cat_map[i] = t.scope.category().to_string();
            }
        }
        if let Some(line) = source_lines.get(li) {
            line_offset += line.len() + 1;
        }
    }

    color_map
        .into_iter()
        .zip(scope_map)
        .zip(cat_map)
        .map(|((c, s), cat)| (c, s, cat))
        .collect()
}

/// Build Shiki color map from fixture tokens
fn build_shiki_color_map<'a>(source: &'a str, shiki_fixture: &'a ShikiFixture) -> Vec<&'a str> {
    let source_bytes = source.len();
    let mut color_map = vec![""; source_bytes];
    for line_tokens in &shiki_fixture.tokens {
        for t in line_tokens {
            let start = t.offset;
            let end = t.offset + t.content.len();
            for color in &mut color_map[start..end.min(source_bytes)] {
                *color = &t.color;
            }
        }
    }
    color_map
}

fn diagnose_fixture(path: &str) {
    let source_path = fixture_dir().join(path).with_extension(
        if fs::metadata(fixture_dir().join(path).with_extension("rs")).is_ok() {
            "rs"
        } else if fs::metadata(fixture_dir().join(path).with_extension("js")).is_ok() {
            "js"
        } else {
            "py"
        },
    );
    let source = fs::read_to_string(&source_path).unwrap();

    let mut expected_path = fixture_dir().join(path);
    expected_path.set_extension("shiki.json");
    let expected: ShikiFixture =
        serde_json::from_str(&fs::read_to_string(&expected_path).unwrap()).unwrap();

    let engine = SyntectEngine::new();
    let theme = load_dark_plus_theme();
    let result = engine
        .highlight(&source, &expected.language, &theme)
        .unwrap();

    let ikihs_map = build_ikihs_color_map(&source, &result);
    let shiki_map = build_shiki_color_map(&source, &expected);

    let source_bytes = source.len();
    let mut diff_count = 0;
    let mut diff_categories: HashMap<&str, usize> = HashMap::new();

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  DIAGNOSE: {path}");
    println!("═══════════════════════════════════════════════════════════════");
    println!(
        "{:>4} {:>6} {:>8} {:>8}  {:<50} {:<12}  match",
        "pos", "char", "shiki", "ikihs", "scope", "cat"
    );

    for pos in 0..source_bytes {
        let ch = source[pos..].chars().next().unwrap();
        let esc = escape_char(ch);
        let (ikihs_color, scope, cat) = &ikihs_map[pos];
        let shiki_color = shiki_map[pos];
        let is_match = ikihs_color.eq_ignore_ascii_case(shiki_color);

        if !is_match {
            diff_count += 1;
            *diff_categories.entry(cat.as_str()).or_insert(0) += 1;
            println!(
                "{pos:>4} {:>6} {:>8} {:>8}  {:<50} {:<12}  DIFF",
                esc,
                shiki_color.to_lowercase(),
                ikihs_color.to_lowercase(),
                scope,
                cat
            );
        }
    }

    println!("\n── DIFF SUMMARY ──");
    println!("  Total color diffs: {diff_count}/{source_bytes}");
    let mut cats: Vec<_> = diff_categories.iter().collect();
    cats.sort_by(|a, b| b.1.cmp(a.1));
    for (cat, count) in cats {
        println!("    {cat:<15} {count}");
    }
    println!();
}

#[test]
fn diagnose_rust_hello() {
    diagnose_fixture("rust/hello");
}

#[test]
fn diagnose_rust_comments() {
    diagnose_fixture("rust/comments");
}

#[test]
fn diagnose_javascript_functions() {
    diagnose_fixture("javascript/functions");
}

#[test]
fn diagnose_javascript_classes() {
    diagnose_fixture("javascript/classes");
}

#[test]
fn diagnose_python_functions() {
    diagnose_fixture("python/functions");
}

#[test]
fn diagnose_python_decorators() {
    diagnose_fixture("python/decorators");
}
