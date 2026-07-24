use ikihs_core::Error;
use ikihs_core::engine::{HighlightEngine, HighlightResult};
use ikihs_core::theme::Theme;
use ikihs_engine_syntect::SyntectEngine;
use ikihs_engine_treesitter::TreeSitterEngine;

pub struct CompositeEngine {
    syntect: SyntectEngine,
    treesitter: TreeSitterEngine,
}

impl CompositeEngine {
    pub fn new() -> Self {
        Self {
            syntect: SyntectEngine::new(),
            treesitter: TreeSitterEngine::new(),
        }
    }
}

impl Default for CompositeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl HighlightEngine for CompositeEngine {
    fn highlight(&self, source: &str, lang: &str, theme: &Theme) -> Result<HighlightResult, Error> {
        match lang {
            "typescript" | "ts" | "tsx" | "javascript" | "js" | "jsx" => {
                self.treesitter.highlight(source, lang, theme)
            }
            _ => self.syntect.highlight(source, lang, theme),
        }
    }

    fn list_grammars(&self) -> Vec<String> {
        let mut g = self.syntect.list_grammars();
        g.extend(self.treesitter.list_grammars());
        g.sort();
        g.dedup();
        g
    }

    fn has_grammar(&self, lang: &str) -> bool {
        self.syntect.has_grammar(lang) || self.treesitter.has_grammar(lang)
    }
}
