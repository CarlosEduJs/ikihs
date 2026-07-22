use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub theme_type: Option<String>,
    pub colors: ThemeColors,
    pub token_colors: Vec<TokenColor>,
}

pub type ThemeColors = std::collections::HashMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenColor {
    pub scope: Vec<String>,
    pub settings: TokenSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenSettings {
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub font_style: Option<String>,
}
