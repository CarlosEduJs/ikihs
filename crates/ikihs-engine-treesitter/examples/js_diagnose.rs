use ikihs_core::engine::HighlightEngine;
use std::fs;
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../crates/ikihs-engine-syntect/fixtures");

    let source = fs::read_to_string(dir.join("javascript/functions.js")).unwrap();
    let expected_json = fs::read_to_string(dir.join("javascript/functions.shiki.json")).unwrap();
    let expected: serde_json::Value = serde_json::from_str(&expected_json).unwrap();
    let theme_json = fs::read_to_string(dir.join("dark-plus.json")).unwrap();
    let theme = ikihs_themes::vscode_theme::VscodeThemeParser::parse_json(&theme_json).unwrap();

    let engine = ikihs_engine_treesitter::TreeSitterEngine::new();
    let result = engine.highlight(&source, "javascript", &theme).unwrap();

    let source_bytes = source.len();
    let mut ikihs = vec![""; source_bytes];
    let mut shiki = vec![""; source_bytes];

    // Build ikihs colors
    let source_lines: Vec<&str> = source.lines().collect();
    let mut line_offset = 0;
    for (li, line) in result.lines.iter().enumerate() {
        for t in &line.tokens {
            let abs_start = line_offset + t.start;
            let abs_end = line_offset + t.end;
            for item in &mut ikihs[abs_start..abs_end.min(source_bytes)] {
                *item = &t.color;
            }
        }
        if let Some(line) = source_lines.get(li) {
            line_offset += line.len() + 1;
        }
    }

    // Build shiki colors
    for line_tokens in expected["tokens"].as_array().unwrap() {
        for t in line_tokens.as_array().unwrap() {
            let start = t["offset"].as_i64().unwrap() as usize;
            let end = start + t["content"].as_str().unwrap().len();
            let color = t["color"].as_str().unwrap();
            for item in &mut shiki[start..end.min(source_bytes)] {
                *item = color;
            }
        }
    }

    println!("Diffs for functions.js:");
    for pos in 0..source_bytes {
        let ik = ikihs[pos].to_lowercase();
        let sk = shiki[pos].to_lowercase();
        if ik != sk {
            let ch = if source.as_bytes()[pos] == b' ' {
                '·'
            } else if source.as_bytes()[pos] == b'\n' {
                '⏎'
            } else {
                source.as_bytes()[pos] as char
            };
            println!(
                "  pos {:>3} ({:?}): engine={:<8} shiki={:<8}",
                pos, ch, ikihs[pos], shiki[pos]
            );
        }
    }
}
