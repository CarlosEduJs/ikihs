use std::fs;
use std::path::PathBuf;

use ikihs_core::engine::HighlightEngine;
use ikihs_core::theme::Theme;
use ikihs_engine_treesitter::TreeSitterEngine;
use ikihs_themes::vscode_theme::VscodeThemeParser;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../crates/ikihs-engine-syntect/fixtures")
}

#[derive(serde::Deserialize)]
struct ShikiFixture {
    #[expect(dead_code)]
    language: String,
    tokens: Vec<Vec<ShikiToken>>,
}

#[derive(serde::Deserialize)]
struct ShikiToken {
    content: String,
    offset: usize,
    color: String,
}

struct MatchResult {
    total: usize,
    exact: usize,
    color_diff: usize,
}

fn compare_by_color(
    source: &str,
    ikihs_lines: &[Vec<ikihs_core::engine::HighlightToken>],
    shiki_lines: &[Vec<ShikiToken>],
) -> MatchResult {
    let source_bytes = source.len();
    let mut ikihs_color = vec![""; source_bytes];
    let mut shiki_color = vec![""; source_bytes];

    let mut line_offset = 0;
    let source_lines: Vec<&str> = source.lines().collect();
    for (li, line_tokens) in ikihs_lines.iter().enumerate() {
        for t in line_tokens {
            let abs_start = line_offset + t.start;
            let abs_end = line_offset + t.end;
            for color in &mut ikihs_color[abs_start..abs_end.min(source_bytes)] {
                *color = &t.color;
            }
        }
        if let Some(line) = source_lines.get(li) {
            line_offset += line.len() + 1;
        }
    }

    for line_tokens in shiki_lines {
        for t in line_tokens {
            let start = t.offset;
            let end = t.offset + t.content.len();
            for color in &mut shiki_color[start..end.min(source_bytes)] {
                *color = &t.color;
            }
        }
    }

    let mut exact = 0;
    let mut color_diff = 0;
    let mut total = 0;

    for pos in 0..source_bytes {
        if ikihs_color[pos].is_empty() && shiki_color[pos].is_empty() {
            continue;
        }
        total += 1;
        if ikihs_color[pos].eq_ignore_ascii_case(shiki_color[pos]) {
            exact += 1;
        } else {
            color_diff += 1;
        }
    }

    MatchResult {
        total,
        exact,
        color_diff,
    }
}

fn load_dark_plus_theme() -> Theme {
    let path = fixture_dir().join("dark-plus.json");
    let json = fs::read_to_string(&path).expect("failed to read dark-plus theme");
    VscodeThemeParser::parse_json(&json).expect("failed to parse dark-plus theme")
}

fn lang_from_path(path: &str) -> &str {
    if path.starts_with("javascript/") || path.starts_with("js/") {
        "javascript"
    } else {
        "typescript"
    }
}

fn source_ext_for_path(path: &str) -> &[&str] {
    if path.starts_with("javascript/") || path.starts_with("js/") {
        &["js", "jsx"]
    } else {
        &["ts", "tsx"]
    }
}

fn run_fixture(path: &str, min_score: usize) {
    let source_path = {
        let dir = fixture_dir().join(path);
        let mut found = None;
        for ext in source_ext_for_path(path) {
            let p = dir.with_extension(ext);
            if p.exists() {
                found = Some(p);
                break;
            }
        }
        found.unwrap_or_else(|| panic!("source not found for {path}"))
    };
    let source = fs::read_to_string(&source_path).unwrap();

    let mut expected_path = fixture_dir().join(path);
    expected_path.set_extension("shiki.json");
    let expected: ShikiFixture =
        serde_json::from_str(&fs::read_to_string(&expected_path).unwrap()).unwrap();

    let engine = TreeSitterEngine::new();
    let theme = load_dark_plus_theme();
    let result = engine
        .highlight(&source, lang_from_path(path), &theme)
        .unwrap();

    let ikihs_lines: Vec<Vec<ikihs_core::engine::HighlightToken>> =
        result.lines.iter().map(|l| l.tokens.clone()).collect();
    let r = compare_by_color(&source, &ikihs_lines, &expected.tokens);

    let score = (r.exact * 100).checked_div(r.total).unwrap_or(100);

    println!(
        "  [{path}] score={score}% exact={} color={} total={}",
        r.exact, r.color_diff, r.total
    );
    assert!(
        score >= min_score,
        "fixture {path} score too low: {score}% (min {min_score}%)"
    );
}

#[test]
fn fixture_typescript_types() {
    run_fixture("typescript/types", 50);
}

#[test]
fn fixture_typescript_generics() {
    run_fixture("typescript/generics", 70);
}

#[test]
fn fixture_javascript_functions() {
    run_fixture("javascript/functions", 60);
}

#[test]
fn fixture_javascript_classes() {
    run_fixture("javascript/classes", 60);
}

#[test]
fn fixture_summary() {
    let fixtures = [
        "typescript/types",
        "typescript/generics",
        "javascript/functions",
        "javascript/classes",
    ];
    let theme = load_dark_plus_theme();
    let engine = TreeSitterEngine::new();
    let mut total_exact = 0;
    let mut total_all = 0;

    println!("\n── Tree-sitter Fixture Summary ──");
    for path in &fixtures {
        let source_path = {
            let dir = fixture_dir().join(path);
            let mut found = None;
            for ext in source_ext_for_path(path) {
                let p = dir.with_extension(ext);
                if p.exists() {
                    found = Some(p);
                    break;
                }
            }
            found.unwrap()
        };
        let source = fs::read_to_string(&source_path).unwrap();
        let mut expected_path = fixture_dir().join(path);
        expected_path.set_extension("shiki.json");
        let expected: ShikiFixture =
            serde_json::from_str(&fs::read_to_string(&expected_path).unwrap()).unwrap();

        let result = engine
            .highlight(&source, lang_from_path(path), &theme)
            .unwrap();
        let ikihs_lines: Vec<Vec<ikihs_core::engine::HighlightToken>> =
            result.lines.iter().map(|l| l.tokens.clone()).collect();
        let r = compare_by_color(&source, &ikihs_lines, &expected.tokens);
        let score = (r.exact * 100).checked_div(r.total).unwrap_or(100);

        total_exact += r.exact;
        total_all += r.total;

        println!(
            "  {:<35} {:3}%  exact={:<3} color={:<3} total={}",
            path, score, r.exact, r.color_diff, r.total
        );
    }

    let overall = (total_exact * 100).checked_div(total_all).unwrap_or(100);
    println!("  ─────────────────────────────────────");
    println!(
        "  {:<35} {:3}%  ({} / {} tokens)",
        "OVERALL", overall, total_exact, total_all
    );
    println!();
}
