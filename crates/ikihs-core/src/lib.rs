pub mod engine;
pub mod error;
pub mod scope;
pub mod theme;

pub use engine::{HighlightEngine, RgbaColor};
pub use error::Error;
pub use scope::mapper::{BuiltinScopeMapper, ScopeMapper};
pub use scope::{Scope, ScopeCategory};
pub use theme::Theme;
