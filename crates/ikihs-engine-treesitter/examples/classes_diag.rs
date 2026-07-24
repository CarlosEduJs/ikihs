use ikihs_core::engine::HighlightEngine;
use std::fs;
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../crates/ikihs-engine-syntect/fixtures");

    let source = fs::read_to_string(dir.join("javascript/classes.js")).unwrap();
    let theme_json = fs::read_to_string(dir.join("dark-plus.json")).unwrap();
    let theme = ikihs_themes::vscode_theme::VscodeThemeParser::parse_json(&theme_json).unwrap();
    let expected_json = fs::read_to_string(dir.join("javascript/classes.shiki.json")).unwrap();
    let expected: serde_json::Value = serde_json::from_str(&expected_json).unwrap();

    let engine = ikihs_engine_treesitter::TreeSitterEngine::new();
    let result = engine.highlight(&source, "javascript", &theme).unwrap();

    let source_bytes = source.len();
    let mut engine_c = vec![""; source_bytes];
    let mut shiki_c = vec![""; source_bytes];
    let source_lines: Vec<&str> = source.lines().collect();
    let mut line_offset = 0;

    for (li, line) in result.lines.iter().enumerate() {
        for t in &line.tokens {
            let abs_start = line_offset + t.start;
            let abs_end = line_offset + t.end;
            for item in &mut engine_c[abs_start..abs_end.min(source_bytes)] {
                *item = &t.color;
            }
        }
        if let Some(line) = source_lines.get(li) {
            line_offset += line.len() + 1;
        }
    }

    for line_tokens in expected["tokens"].as_array().unwrap() {
        for t in line_tokens.as_array().unwrap() {
            let start = t["offset"].as_i64().unwrap() as usize;
            let end = start + t["content"].as_str().unwrap().len();
            let color = t["color"].as_str().unwrap();
            for item in &mut shiki_c[start..end.min(source_bytes)] {
                *item = color;
            }
        }
    }

    println!("Diffs:");
    for pos in 0..source_bytes {
        let e = engine_c[pos].to_lowercase();
        let s = shiki_c[pos].to_lowercase();
        if e != s {
            let ch = if source.as_bytes()[pos] == b' ' {
                '·'
            } else if source.as_bytes()[pos] == b'\n' {
                '⏎'
            } else {
                source.as_bytes()[pos] as char
            };
            println!(
                "  pos {:>3} ({:?}): engine={:<8} shiki={:<8}",
                pos, ch, engine_c[pos], shiki_c[pos]
            );
        }
    }

    println!("\nEngine colors for all positions:");
    for pos in 0..source_bytes {
        let ch = if source.as_bytes()[pos] == b' ' {
            '·'
        } else if source.as_bytes()[pos] == b'\n' {
            '⏎'
        } else {
            source.as_bytes()[pos] as char
        };
        println!(
            "  pos {:>3} ({:?}): engine={:<8} shiki={:<8}",
            pos, ch, engine_c[pos], shiki_c[pos]
        );
    }
}
