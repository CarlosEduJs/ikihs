use ikihs_core::theme::Theme;

pub struct ThemeRegistry {
    themes: Vec<Theme>,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        Self { themes: Vec::new() }
    }

    pub fn register(&mut self, theme: Theme) {
        self.themes.push(theme);
    }

    pub fn get(&self, name: &str) -> Option<&Theme> {
        self.themes.iter().find(|t| t.name == name)
    }

    pub fn all(&self) -> &[Theme] {
        &self.themes
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
