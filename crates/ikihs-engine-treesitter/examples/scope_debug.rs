use ikihs_core::scope::ScopeCategory;
use ikihs_core::scope::mapper::{BuiltinScopeMapper, ScopeMapper};
use ikihs_themes::vscode_theme::VscodeThemeParser;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn lookup_scope_color(scope_str: &str, theme: &ikihs_core::theme::Theme) -> Option<String> {
    let target = scope_str.split(' ').last().unwrap_or(scope_str);
    let mut best_dots: isize = -1;
    let mut best_color = None;

    for tc in theme.token_colors.iter().rev() {
        if let Some(ref fg) = tc.settings.foreground {
            for theme_scope in &tc.scope {
                if target == *theme_scope || target.starts_with(&format!("{}.", theme_scope)) {
                    let dots = theme_scope.matches('.').count() as isize;
                    println!(
                        "    match: target={:?} theme={:?} dots={} color={} {}",
                        target,
                        theme_scope,
                        dots,
                        fg,
                        if dots > best_dots { "✓" } else { "" }
                    );
                    if dots > best_dots {
                        best_dots = dots;
                        best_color = Some(fg.clone());
                    }
                }
            }
        }
    }
    best_color
}

fn main() {
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../crates/ikihs-engine-syntect/fixtures");
    let theme_json = fs::read_to_string(dir.join("dark-plus.json")).unwrap();
    let theme = VscodeThemeParser::parse_json(&theme_json).unwrap();

    let test_scopes = vec![
        "source.ts keyword.control",
        "source.ts variable.other.readwrite",
        "source.ts variable.other.constant",
        "source.ts entity.name.function",
        "source.ts entity.name.type",
        "source.ts support.type",
        "source.ts string.quoted",
        "source.ts constant.numeric",
        "source.ts comment.line",
    ];

    for scope in &test_scopes {
        println!("Scope: {}", scope);
        let result = lookup_scope_color(scope, &theme);
        println!("  Result: {:?}", result);
        println!();
    }
}
