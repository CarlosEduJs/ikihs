#[test]
fn check_typescript_fallback_to_javascript() {
    let ss = syntect::parsing::SyntaxSet::load_defaults_newlines();
    let syntax = ss.find_syntax_by_token("javascript").unwrap();
    println!(
        "Fallback JS syntax: {:?} (ext: {:?})",
        syntax.name, syntax.file_extensions
    );

    let mut state = syntect::parsing::ParseState::new(syntax);
    let line = "let x = 1;\n";
    match state.parse_line(line, &ss) {
        Ok(ops) => println!("Parse OK: {} ops", ops.len()),
        Err(e) => println!("Parse error: {:?}", e),
    }
}
