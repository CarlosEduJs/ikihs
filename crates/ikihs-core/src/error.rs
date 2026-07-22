use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Theme error: {0}")]
    Theme(String),

    #[error("Scope error: {0}")]
    Scope(String),

    #[error("Engine error: {0}")]
    Engine(String),

    #[error("Grammar not found for language: {0}")]
    GrammarNotFound(String),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}
