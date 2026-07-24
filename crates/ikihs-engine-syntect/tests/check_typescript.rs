#[test]
fn check_typescript_fallback_to_javascript() {
    let ss = syntect::parsing::SyntaxSet::load_defaults_newlines();
    let syntax = ss
        .find_syntax_by_token("javascript")
        .expect("JavaScript fallback syntax should be available");
    println!(
        "Fallback JS syntax: {:?} (ext: {:?})",
        syntax.name, syntax.file_extensions
    );
    assert!(
        syntax.name.to_lowercase().contains("javascript")
            || syntax.name.to_lowercase().contains("js"),
        "expected JavaScript syntax, got: {}",
        syntax.name
    );

    let mut state = syntect::parsing::ParseState::new(syntax);
    let line = "let x = 1;\n";
    let ops = state
        .parse_line(line, &ss)
        .expect("parse_line should succeed for simple JS");
    println!("Parse OK: {} ops", ops.len());
    assert!(!ops.is_empty(), "expected at least one parse operation");
}
