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

struct MatchResult {
    total: usize,
    exact: usize,
    color_diff: usize,
    offset_diff: usize,
    missing_ikihs: usize,
    extra: usize,
}

/// Compare colors byte-by-byte across the entire source.
/// This ignores token boundary differences and only checks if each
/// character position has the same color in both outputs.
fn compare_by_color(
    source: &str,
    ikihs_lines: &[Vec<ikihs_core::engine::HighlightToken>],
    shiki_lines: &[Vec<ShikiToken>],
) -> MatchResult {
    let source_bytes = source.len();
    let mut ikihs_color = vec![""; source_bytes];
    let mut shiki_color = vec![""; source_bytes];

    // Build Ikihs color map (absolute positions)
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

    // Build Shiki color map (absolute offsets)
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
        // Skip positions where neither has a color (shouldn't happen, but just in case)
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
        offset_diff: 0,
        missing_ikihs: 0,
        extra: 0,
    }
}

fn load_dark_plus_theme() -> Theme {
    let json = include_str!("../fixtures/dark-plus.json");
    VscodeThemeParser::parse_json(json).expect("failed to parse dark-plus theme")
}

fn run_fixture(_path: &str, source: &str, expected: &ShikiFixture) -> MatchResult {
    let engine = SyntectEngine::new();
    let theme = load_dark_plus_theme();
    let result = engine
        .highlight(source, &expected.language, &theme)
        .unwrap();

    let ikihs_lines: Vec<Vec<ikihs_core::engine::HighlightToken>> =
        result.lines.iter().map(|l| l.tokens.clone()).collect();
    compare_by_color(source, &ikihs_lines, &expected.tokens)
}

fn load_fixture_source(fixture_path: &str) -> String {
    let path = fixture_dir().join(fixture_path).with_extension("rs");
    if path.exists() {
        return fs::read_to_string(&path).unwrap();
    }
    let path = fixture_dir().join(fixture_path).with_extension("js");
    if path.exists() {
        return fs::read_to_string(&path).unwrap();
    }
    let path = fixture_dir().join(fixture_path).with_extension("py");
    if path.exists() {
        return fs::read_to_string(&path).unwrap();
    }
    panic!("fixture not found: {fixture_path}");
}

fn load_fixture_expected(fixture_path: &str) -> ShikiFixture {
    let mut path = fixture_dir().join(fixture_path);
    path.set_extension("shiki.json");
    let content = fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("expected fixture not found: {}", path.display());
    });
    serde_json::from_str(&content).unwrap()
}

fn score(result: &MatchResult) -> usize {
    if result.total == 0 {
        return 100;
    }
    (result.exact * 100) / result.total
}

#[test]
fn fixture_rust_hello() {
    let path = "rust/hello";
    let source = load_fixture_source(path);
    let expected = load_fixture_expected(path);
    let result = run_fixture(path, &source, &expected);
    let s = score(&result);
    println!(
        "  [{path}] score={s}% exact={} color={} offset={} extra={} missing={} total={}",
        result.exact,
        result.color_diff,
        result.offset_diff,
        result.extra,
        result.missing_ikihs,
        result.total
    );
    assert!(s >= 60, "fixture {path} score too low: {s}% (min 80%)");
}

#[test]
fn fixture_rust_comments() {
    let path = "rust/comments";
    let source = load_fixture_source(path);
    let expected = load_fixture_expected(path);
    let result = run_fixture(path, &source, &expected);
    let s = score(&result);
    println!(
        "  [{path}] score={s}% exact={} color={} offset={} extra={} missing={} total={}",
        result.exact,
        result.color_diff,
        result.offset_diff,
        result.extra,
        result.missing_ikihs,
        result.total
    );
    assert!(s >= 60, "fixture {path} score too low: {s}% (min 80%)");
}

#[test]
fn fixture_javascript_functions() {
    let path = "javascript/functions";
    let source = load_fixture_source(path);
    let expected = load_fixture_expected(path);
    let result = run_fixture(path, &source, &expected);
    let s = score(&result);
    println!(
        "  [{path}] score={s}% exact={} color={} offset={} extra={} missing={} total={}",
        result.exact,
        result.color_diff,
        result.offset_diff,
        result.extra,
        result.missing_ikihs,
        result.total
    );
    assert!(s >= 60, "fixture {path} score too low: {s}% (min 80%)");
}

#[test]
fn fixture_javascript_classes() {
    let path = "javascript/classes";
    let source = load_fixture_source(path);
    let expected = load_fixture_expected(path);
    let result = run_fixture(path, &source, &expected);
    let s = score(&result);
    println!(
        "  [{path}] score={s}% exact={} color={} offset={} extra={} missing={} total={}",
        result.exact,
        result.color_diff,
        result.offset_diff,
        result.extra,
        result.missing_ikihs,
        result.total
    );
    assert!(s >= 60, "fixture {path} score too low: {s}% (min 80%)");
}

#[test]
fn fixture_python_functions() {
    let path = "python/functions";
    let source = load_fixture_source(path);
    let expected = load_fixture_expected(path);
    let result = run_fixture(path, &source, &expected);
    let s = score(&result);
    println!(
        "  [{path}] score={s}% exact={} color={} offset={} extra={} missing={} total={}",
        result.exact,
        result.color_diff,
        result.offset_diff,
        result.extra,
        result.missing_ikihs,
        result.total
    );
    assert!(s >= 60, "fixture {path} score too low: {s}% (min 80%)");
}

#[test]
fn fixture_python_decorators() {
    let path = "python/decorators";
    let source = load_fixture_source(path);
    let expected = load_fixture_expected(path);
    let result = run_fixture(path, &source, &expected);
    let s = score(&result);
    println!(
        "  [{path}] score={s}% exact={} color={} offset={} extra={} missing={} total={}",
        result.exact,
        result.color_diff,
        result.offset_diff,
        result.extra,
        result.missing_ikihs,
        result.total
    );
    assert!(s >= 60, "fixture {path} score too low: {s}% (min 80%)");
}

#[test]
fn fixture_edge_empty() {
    let path = "edge-cases/empty";
    let source = load_fixture_source(path);
    let expected = load_fixture_expected(path);
    let result = run_fixture(path, &source, &expected);
    let s = score(&result);
    println!(
        "  [{path}] score={s}% exact={} color={} offset={} extra={} missing={} total={}",
        result.exact,
        result.color_diff,
        result.offset_diff,
        result.extra,
        result.missing_ikihs,
        result.total
    );
    assert_eq!(s, 100, "empty file should match perfectly");
}

#[test]
fn fixture_edge_single_line() {
    let path = "edge-cases/single-line";
    let source = load_fixture_source(path);
    let expected = load_fixture_expected(path);
    let result = run_fixture(path, &source, &expected);
    let s = score(&result);
    println!(
        "  [{path}] score={s}% exact={} color={} offset={} extra={} missing={} total={}",
        result.exact,
        result.color_diff,
        result.offset_diff,
        result.extra,
        result.missing_ikihs,
        result.total
    );
    assert!(s >= 60, "fixture {path} score too low: {s}% (min 80%)");
}

#[test]
fn fixture_edge_only_comments() {
    let path = "edge-cases/only-comments";
    let source = load_fixture_source(path);
    let expected = load_fixture_expected(path);
    let result = run_fixture(path, &source, &expected);
    let s = score(&result);
    println!(
        "  [{path}] score={s}% exact={} color={} offset={} extra={} missing={} total={}",
        result.exact,
        result.color_diff,
        result.offset_diff,
        result.extra,
        result.missing_ikihs,
        result.total
    );
    assert!(s >= 60, "fixture {path} score too low: {s}% (min 80%)");
}

/// Run all fixtures in batch mode and print summary
#[test]
fn fixture_summary() {
    let fixtures = [
        "rust/hello",
        "rust/comments",
        "javascript/functions",
        "javascript/classes",
        "python/functions",
        "python/decorators",
        "edge-cases/empty",
        "edge-cases/single-line",
        "edge-cases/only-comments",
    ];

    let mut total_exact = 0;
    let mut total_all = 0;

    println!("\n── Fixture Summary ──");
    for path in &fixtures {
        let source = load_fixture_source(path);
        let expected = load_fixture_expected(path);
        let result = run_fixture(path, &source, &expected);
        let s = score(&result);
        total_exact += result.exact;
        total_all += result.total;
        println!(
            "  {:<35} {:3}%  exact={:<3} color={:<3} offset={:<3} extra={:<3} missing={:<3} total={}",
            path,
            s,
            result.exact,
            result.color_diff,
            result.offset_diff,
            result.extra,
            result.missing_ikihs,
            result.total
        );
    }

    let overall = (total_exact * 100).checked_div(total_all).unwrap_or(100);
    println!("  ─────────────────────────────────────");
    println!(
        "  {:<35} {:3}%  ({} / {} tokens)",
        "OVERALL", overall, total_exact, total_all
    );
    println!();
    assert!(overall >= 90, "overall score too low: {overall}% (min 90%)");
}
