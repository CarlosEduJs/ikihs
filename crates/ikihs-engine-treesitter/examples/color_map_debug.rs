use ikihs_core::scope::ScopeCategory;
use ikihs_core::scope::mapper::{BuiltinScopeMapper, ScopeMapper};
use ikihs_themes::vscode_theme::VscodeThemeParser;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

struct Candidate {
    dots: usize,
    index: usize,
    color: String,
}

fn main() {
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../crates/ikihs-engine-syntect/fixtures");
    let theme_json = fs::read_to_string(dir.join("dark-plus.json")).unwrap();
    let theme = VscodeThemeParser::parse_json(&theme_json).unwrap();
    let mapper = BuiltinScopeMapper::new();

    let mut best: HashMap<ScopeCategory, Candidate> = HashMap::new();

    for (i, tc) in theme.token_colors.iter().enumerate() {
        if let Some(ref fg) = tc.settings.foreground {
            for scope_str in &tc.scope {
                let cat = mapper.classify(scope_str);
                if matches!(cat, ScopeCategory::Other(_)) {
                    continue;
                }
                let dots = scope_str.matches('.').count();
                let cand = Candidate {
                    dots,
                    index: i,
                    color: fg.clone(),
                };
                let better = best.get(&cat).is_none_or(|cur| {
                    cand.dots < cur.dots || (cand.dots == cur.dots && cand.index < cur.index)
                });
                if true {
                    // always print
                    println!(
                        "  {:?} ← {} (dots={}, i={}) scope={}",
                        cat, fg, dots, i, scope_str
                    );
                }
                if better {
                    best.insert(cat, cand);
                }
            }
        }
    }

    println!("\nFinal map:");
    for (cat, cand) in &best {
        println!(
            "  {:?} → {} (dots={}, i={})",
            cat, cand.color, cand.dots, cand.index
        );
    }
}
