use std::fs;
use std::io::Read;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use serde_json::json;

use ikihs_core::engine::HighlightEngine;
use ikihs_core::theme::Theme;
use ikihs_engine_composite::CompositeEngine;
use ikihs_themes::vscode_theme::VscodeThemeParser;

#[derive(Parser)]
#[command(name = "ikihs", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Highlight {
        source: Option<PathBuf>,
        #[arg(short = 'l', long = "lang")]
        lang: Option<String>,
        #[arg(short = 't', long = "theme", default_value = "dark-plus.json")]
        theme: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Highlight {
            source,
            lang,
            theme,
        }) => cmd_highlight(source, lang, theme),
        None => {
            println!("ikihs {}", env!("CARGO_PKG_VERSION"));
        }
    }
}

fn cmd_highlight(source_path: Option<PathBuf>, lang: Option<String>, theme_path: PathBuf) {
    let source = match source_path {
        Some(path) => fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("error: cannot read '{}': {}", path.display(), e);
            std::process::exit(1);
        }),
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf).unwrap();
            buf
        }
    };

    let theme_json = fs::read_to_string(&theme_path).unwrap_or_else(|e| {
        eprintln!("error: cannot read theme '{}': {}", theme_path.display(), e);
        std::process::exit(1);
    });

    let theme: Theme = VscodeThemeParser::parse_json(&theme_json).unwrap_or_else(|e| {
        eprintln!("error: theme parse failed: {}", e);
        std::process::exit(1);
    });

    let lang = lang.as_deref().unwrap_or("rs");

    let engine = CompositeEngine::new();
    let result = engine.highlight(&source, lang, &theme).unwrap_or_else(|e| {
        eprintln!("error: highlight failed: {}", e);
        std::process::exit(1);
    });

    let fg = theme
        .colors
        .get("editor.foreground")
        .cloned()
        .unwrap_or_else(|| "#000000".into());
    let bg = theme
        .colors
        .get("editor.background")
        .cloned()
        .unwrap_or_else(|| "#ffffff".into());

    output_json(&source, &result, &fg, &bg);
}

fn output_json(source: &str, result: &ikihs_core::engine::HighlightResult, fg: &str, bg: &str) {
    let mut lines_json = Vec::new();
    let mut source_offset = 0;

    for line_tokens in &result.lines {
        let mut line_json = Vec::new();
        for token in &line_tokens.tokens {
            let abs_start = source_offset + token.start;
            let abs_end = source_offset + token.end;
            let text = &source[abs_start..abs_end];
            line_json.push(json!({
                "content": text,
                "offset": abs_start,
                "color": token.color,
                "fontStyle": token.font_style,
                "scope": token.scope.raw(),
                "category": token.scope.category().to_string(),
            }));
        }
        lines_json.push(json!(line_json));

        let remaining = &source[source_offset..];
        let line_end = remaining
            .find('\n')
            .map(|i| i + 1)
            .unwrap_or(remaining.len());
        source_offset += line_end;
    }

    let output = json!({
        "tokens": lines_json,
        "fg": fg,
        "bg": bg,
        "language": result.language,
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
