use std::fs;
use std::path::PathBuf;

fn main() {
    let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../crates/ikihs-engine-syntect/fixtures/typescript/types.ts");
    let source = fs::read_to_string(&path).unwrap();

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
        .unwrap();
    let tree = parser.parse(&source, None).unwrap();
    let root = tree.root_node();
    print_tree(root, &source, 0);
}

fn print_tree(node: tree_sitter::Node, source: &str, depth: usize) {
    let indent = "  ".repeat(depth);
    let text = &source[node.start_byte()..node.end_byte()];
    let details = if node.is_named() { "" } else { " (anon)" };
    println!(
        "{}{:?} {:?} [{}-{}]{}",
        indent,
        node.kind(),
        text,
        node.start_byte(),
        node.end_byte(),
        details
    );
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_tree(child, source, depth + 1);
    }
}
