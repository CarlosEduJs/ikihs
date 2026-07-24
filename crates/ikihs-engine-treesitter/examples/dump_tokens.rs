use ikihs_core::engine::HighlightEngine;
use std::fs;
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../crates/ikihs-engine-syntect/fixtures");

    let source = fs::read_to_string(dir.join("javascript/functions.js")).unwrap();
    let theme_json = fs::read_to_string(dir.join("dark-plus.json")).unwrap();
    let theme = ikihs_themes::vscode_theme::VscodeThemeParser::parse_json(&theme_json).unwrap();

    let engine = ikihs_engine_treesitter::TreeSitterEngine::new();
    let result = engine.highlight(&source, "javascript", &theme).unwrap();

    let source_bytes = source.len();
    let mut c = vec![""; source_bytes];
    let source_lines: Vec<&str> = source.lines().collect();
    let mut line_offset = 0;

    for (li, line) in result.lines.iter().enumerate() {
        for t in &line.tokens {
            let abs_start = line_offset + t.start;
            let abs_end = line_offset + t.end;
            for i in abs_start..abs_end.min(source_bytes) {
                c[i] = &t.color;
            }
        }
        if let Some(line) = source_lines.get(li) {
            line_offset += line.len() + 1;
        }
    }

    // Compare with shiki
    use std::process::Command;
    let expected_str = Command::new("python3")
        .args([
            "-c",
            &format!(
                r#"
import json
with open('{}') as f:
    data = json.load(f)
with open('{}') as f:
    source = f.read()
c = [''] * len(source)
for line_tokens in data['tokens']:
    for t in line_tokens:
        start = t['offset']
        end = t['offset'] + len(t['content'])
        for i in range(start, min(end, len(source))):
            c[i] = t['color'].lower()
for i, color in enumerate(c):
    if color:
        print(f'{{{{i}}}}{{{{color}}}}')
"#,
                dir.join("javascript/functions.shiki.json")
                    .to_str()
                    .unwrap(),
                dir.join("javascript/functions.js").to_str().unwrap()
            ),
        ])
        .output()
        .unwrap();
    let _output_str = String::from_utf8_lossy(&expected_str.stdout);

    println!("Diffs:");
    for (pos, color) in c.iter().enumerate() {
        if !color.is_empty() && color.to_lowercase() != c[pos].to_lowercase() {
            let ch = if source.as_bytes()[pos] == b' ' {
                '·'
            } else if source.as_bytes()[pos] == b'\n' {
                '⏎'
            } else {
                source.as_bytes()[pos] as char
            };
            println!(
                "  pos {:>3} ({:?}): engine={} shiki={}",
                pos, ch, c[pos], color
            );
        }
    }
}
