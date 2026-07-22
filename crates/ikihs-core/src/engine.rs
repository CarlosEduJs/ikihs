use crate::Error;
use crate::scope::Scope;
use crate::theme::Theme;

pub type LineIndex = usize;
pub type ColumnIndex = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbaColor(pub u8, pub u8, pub u8, pub u8);

impl RgbaColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(r, g, b, a)
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.0, self.1, self.2)
    }
}

#[derive(Debug, Clone)]
pub struct HighlightToken {
    pub start: ColumnIndex,
    pub end: ColumnIndex,
    pub scope: Scope,
    pub color: String,
    pub font_style: String,
}

#[derive(Debug, Clone)]
pub struct HighlightLine {
    pub tokens: Vec<HighlightToken>,
}

#[derive(Debug, Clone)]
pub struct HighlightResult {
    pub lines: Vec<HighlightLine>,
    pub language: String,
}

pub trait HighlightEngine: Send + Sync {
    fn highlight(&self, source: &str, lang: &str, theme: &Theme) -> Result<HighlightResult, Error>;

    fn list_grammars(&self) -> Vec<String>;

    fn has_grammar(&self, lang: &str) -> bool {
        self.list_grammars().contains(&lang.to_string())
    }
}
