use std::fs;
use std::path::PathBuf;

fn print_tree(node: tree_sitter::Node, source: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    let text = source[node.start_byte()..node.end_byte()].escape_default().to_string();
    println!("{}{:?} {:?} ({}-{}) named={}", indent, node.kind(), text, node.start_byte(), node.end_byte(), node.is_named());
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_tree(child, source, depth + 1);
    }
}

fn main() {
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../crates/ikihs-engine-syntect/fixtures");
    let source = fs::read_to_string(dir.join("javascript/functions.js")).unwrap();
    
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_javascript::LANGUAGE.into()).unwrap();
    let tree = parser.parse(&source, None).unwrap();
    
    // Just print the template string part
    print_tree(tree.root_node(), &source, 0);
}
