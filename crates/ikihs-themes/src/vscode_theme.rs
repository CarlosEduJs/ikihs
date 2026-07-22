use ikihs_core::Error;
use ikihs_core::theme::{Theme, TokenColor, TokenSettings};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct VSCodeThemeJson {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub theme_type: Option<String>,
    pub colors: Option<HashMap<String, String>>,
    #[serde(rename = "tokenColors")]
    pub token_colors: Option<Vec<VSCodeTokenColor>>,
    #[serde(rename = "semanticHighlighting")]
    pub semantic_highlighting: Option<bool>,
    #[serde(rename = "semanticTokenColors")]
    pub semantic_token_colors: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct VSCodeTokenColor {
    pub scope: OneOrManyScopes,
    pub settings: TokenSettings,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OneOrManyScopes {
    Single(String),
    Many(Vec<String>),
}

pub struct VscodeThemeParser;

impl VscodeThemeParser {
    pub fn parse_json(json: &str) -> Result<Theme, Error> {
        let vsc: VSCodeThemeJson = serde_json::from_str(json)?;

        let name = vsc.name.unwrap_or_else(|| "untitled".into());

        let colors = vsc.colors.unwrap_or_default();

        let token_colors = vsc
            .token_colors
            .unwrap_or_default()
            .into_iter()
            .map(|tc| {
                let scopes = match tc.scope {
                    OneOrManyScopes::Single(s) => vec![s],
                    OneOrManyScopes::Many(v) => v,
                };
                TokenColor {
                    scope: scopes,
                    settings: tc.settings,
                }
            })
            .collect();

        Ok(Theme {
            name,
            theme_type: vsc.theme_type,
            colors,
            token_colors,
        })
    }
}
