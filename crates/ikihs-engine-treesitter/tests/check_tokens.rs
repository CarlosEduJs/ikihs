// Run with: cargo test -p ikihs-engine-treesitter --test check_tokens -- --nocapture
use std::fs;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/ikihs-engine-syntect/fixtures")
}

#[test]
fn check_concrete_tokens() {
    // Build boundaries the same way as the engine
    let source = fs::read_to_string(fixture_dir().join("typescript/types.ts")).unwrap();
    
    let mut boundaries: Vec<usize> = vec![0];
    for (i, &b) in source.as_bytes().iter().enumerate() {
        if b == b'\n' {
            boundaries.push(i + 1);
        }
    }
    let last = source.len();
    if *boundaries.last().unwrap() != last {
        boundaries.push(last);
    }
    
    println!("Source length: {}", source.len());
    println!("Boundaries: {:?}", boundaries);
    
    // Now parse with tree-sitter and emit ALL leaf positions
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()).unwrap();
    let tree = parser.parse(&source, None).unwrap();
    
    let mut tokens: Vec<(usize, usize)> = Vec::new();
    let mut cursor = tree.root_node().walk();
    collect_leaves(&mut cursor, &mut tokens);
    
    println!("\nLeaf tokens (start, end):");
    let mut last_end = 0;
    for (s, e) in &tokens {
        let text = &source[*s..*e];
        let gap = s - last_end;
        if gap > 0 {
            println!("  GAP: {}..{} ({:?})", last_end, s, &source[last_end..*s]);
        }
        println!("  {}..{} = {:?}", s, e, text);
        last_end = *e;
    }
    
    // Check if position 9 is covered
    let mut covered = vec![false; source.len()];
    for (s, e) in &tokens {
        for i in *s..*e {
            covered[i] = true;
        }
    }
    
    println!("\nUncovered positions that show #d4d4d4:");
    for pos in [9, 14, 17, 18, 24, 33, 34, 39, 55, 62, 64, 73, 75, 94, 100, 105, 107, 109] {
        if pos < covered.len() {
            let ch = if source.as_bytes()[pos] == b' ' { '·' } else if source.as_bytes()[pos] == b'\n' { '⏎' } else { source.as_bytes()[pos] as char };
            println!("  pos {} ({:?}): covered={}", pos, ch, covered[pos]);
        }
    }
}

fn collect_leaves(cursor: &mut tree_sitter::TreeCursor, tokens: &mut Vec<(usize, usize)>) {
    if cursor.goto_first_child() {
        loop {
            let node = cursor.node();
            let kind = node.kind();
            let is_atomic = matches!(kind, "string" | "number" | "comment" | "template_string"
                | "regex" | "hash_bang_line" | "predefined_type" | "literal_type");
            
            if is_atomic || node.child_count() == 0 {
                tokens.push((node.start_byte(), node.end_byte()));
            } else {
                collect_leaves(cursor, tokens);
            }
            
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}
