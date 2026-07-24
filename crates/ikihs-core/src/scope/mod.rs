pub mod mapper;

use crate::scope::mapper::ScopeMapper;
use serde::{Deserialize, Serialize};
use std::fmt;

pub type ScopeString = String;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScopeCategory {
    Comment,
    Keyword,
    String,
    Number,
    Function,
    Variable,
    Constant,
    Type,
    Class,
    Parameter,
    Property,
    Operator,
    Invalid,
    Other(String),
}

impl fmt::Display for ScopeCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScopeCategory::Comment => write!(f, "comment"),
            ScopeCategory::Keyword => write!(f, "keyword"),
            ScopeCategory::String => write!(f, "string"),
            ScopeCategory::Number => write!(f, "number"),
            ScopeCategory::Function => write!(f, "function"),
            ScopeCategory::Variable => write!(f, "variable"),
            ScopeCategory::Constant => write!(f, "constant"),
            ScopeCategory::Type => write!(f, "type"),
            ScopeCategory::Class => write!(f, "class"),
            ScopeCategory::Parameter => write!(f, "parameter"),
            ScopeCategory::Property => write!(f, "property"),
            ScopeCategory::Operator => write!(f, "operator"),
            ScopeCategory::Invalid => write!(f, "invalid"),
            ScopeCategory::Other(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    pub raw: ScopeString,
    pub category: ScopeCategory,
}

impl Scope {
    pub fn new(raw: impl Into<ScopeString>) -> Self {
        let raw: ScopeString = raw.into();
        let category = mapper::BuiltinScopeMapper::new().classify(&raw);
        Self { raw, category }
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn category(&self) -> &ScopeCategory {
        &self.category
    }

    pub fn with_category(mut self, category: ScopeCategory) -> Self {
        self.category = category;
        self
    }
}

impl From<&str> for Scope {
    fn from(raw: &str) -> Self {
        Self::new(raw)
    }
}

impl From<String> for Scope {
    fn from(raw: String) -> Self {
        Self::new(raw)
    }
}
