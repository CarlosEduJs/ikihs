use ikihs_core::engine::HighlightEngine;

#[test]
fn load_typescript_grammar() {
    let engine = ikihs_engine_syntect::SyntectEngine::new();
    // TypeScript uses the JavaScript grammar via "javascript" token
    assert!(engine.has_grammar("javascript"), "JavaScript grammar should be available from defaults");
    // "typescript" and "ts" should NOT have dedicated grammars
    assert!(!engine.has_grammar("typescript"), "TypeScript grammar is not bundled");
    assert!(!engine.has_grammar("ts"), "No 'ts' grammar token");
}
