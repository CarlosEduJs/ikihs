use crate::scope::{ScopeCategory, ScopeString};
use std::collections::HashMap;

pub trait ScopeMapper: Send + Sync {
    fn classify(&self, scope: &str) -> ScopeCategory;
    fn add_mapping(&mut self, pattern: &str, category: ScopeCategory);
}

pub struct BuiltinScopeMapper {
    mappings: HashMap<ScopeString, ScopeCategory>,
}

impl BuiltinScopeMapper {
    pub fn new() -> Self {
        let mut mappings = HashMap::new();
        Self::insert_defaults(&mut mappings);
        Self { mappings }
    }

    fn insert_defaults(mappings: &mut HashMap<ScopeString, ScopeCategory>) {
        let defaults: Vec<(&str, ScopeCategory)> = vec![
            // Comment
            ("comment", ScopeCategory::Comment),
            ("comment.line", ScopeCategory::Comment),
            ("comment.block", ScopeCategory::Comment),
            ("comment.line.double-slash", ScopeCategory::Comment),
            ("comment.line.number-sign", ScopeCategory::Comment),
            ("comment.block.documentation", ScopeCategory::Comment),
            // String
            ("string", ScopeCategory::String),
            ("string.quoted", ScopeCategory::String),
            ("string.unquoted", ScopeCategory::String),
            ("string.quoted.single", ScopeCategory::String),
            ("string.quoted.double", ScopeCategory::String),
            ("string.quoted.triple", ScopeCategory::String),
            ("string.quoted.other", ScopeCategory::String),
            ("string.other", ScopeCategory::String),
            ("string.other.link", ScopeCategory::String),
            // Keyword
            ("keyword", ScopeCategory::Keyword),
            ("keyword.control", ScopeCategory::Keyword),
            ("keyword.other", ScopeCategory::Keyword),
            ("storage", ScopeCategory::Keyword),
            ("storage.type", ScopeCategory::Keyword),
            ("storage.modifier", ScopeCategory::Keyword),
            // Number
            ("constant.numeric", ScopeCategory::Number),
            ("constant.numeric.integer", ScopeCategory::Number),
            ("constant.numeric.float", ScopeCategory::Number),
            ("constant.numeric.hex", ScopeCategory::Number),
            ("constant.other.numeric", ScopeCategory::Number),
            // Function
            ("entity.name.function", ScopeCategory::Function),
            ("meta.function", ScopeCategory::Function),
            ("meta.function-call", ScopeCategory::Function),
            ("support.function", ScopeCategory::Function),
            ("support.function.builtin", ScopeCategory::Function),
            // Variable
            ("variable", ScopeCategory::Variable),
            ("variable.other", ScopeCategory::Variable),
            ("variable.language", ScopeCategory::Variable),
            // Constant
            ("constant", ScopeCategory::Constant),
            ("constant.language", ScopeCategory::Constant),
            ("constant.character", ScopeCategory::Constant),
            ("constant.other", ScopeCategory::Constant),
            ("support.constant", ScopeCategory::Constant),
            // Type
            ("support.type", ScopeCategory::Type),
            ("support.type.builtin", ScopeCategory::Type),
            ("entity.name.type", ScopeCategory::Type),
            // Class
            ("entity.name.class", ScopeCategory::Class),
            ("support.class", ScopeCategory::Class),
            // Parameter
            ("variable.parameter", ScopeCategory::Parameter),
            ("variable.parameter.function", ScopeCategory::Parameter),
            // Property
            ("variable.other.property", ScopeCategory::Property),
            ("support.variable.property", ScopeCategory::Property),
            // Operator
            ("keyword.operator", ScopeCategory::Operator),
            ("keyword.operator.arithmetic", ScopeCategory::Operator),
            ("keyword.operator.assignment", ScopeCategory::Operator),
            ("keyword.operator.comparison", ScopeCategory::Operator),
            ("keyword.operator.logical", ScopeCategory::Operator),
            // Invalid
            ("invalid", ScopeCategory::Invalid),
            ("invalid.illegal", ScopeCategory::Invalid),
            ("invalid.deprecated", ScopeCategory::Invalid),
        ];

        for (pattern, category) in defaults {
            mappings.insert(pattern.to_string(), category);
        }
    }
}

impl Default for BuiltinScopeMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeMapper for BuiltinScopeMapper {
    fn classify(&self, scope: &str) -> ScopeCategory {
        let segments: Vec<&str> = scope.split('.').collect();
        for len in (1..=segments.len()).rev() {
            let prefix = segments[..len].join(".");
            if let Some(category) = self.mappings.get(&prefix) {
                return category.clone();
            }
        }
        ScopeCategory::Other(scope.to_string())
    }

    fn add_mapping(&mut self, pattern: &str, category: ScopeCategory) {
        self.mappings.insert(pattern.to_string(), category);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Scope;

    #[test]
    fn test_exact_match() {
        let mapper = BuiltinScopeMapper::new();
        assert_eq!(mapper.classify("keyword.control"), ScopeCategory::Keyword);
    }

    #[test]
    fn test_hierarchy_fallback() {
        let mapper = BuiltinScopeMapper::new();
        assert_eq!(
            mapper.classify("entity.name.function.python"),
            ScopeCategory::Function
        );
    }

    #[test]
    fn test_generic_fallback() {
        let mapper = BuiltinScopeMapper::new();
        assert_eq!(
            mapper.classify("keyword.control.if.js"),
            ScopeCategory::Keyword
        );
    }

    #[test]
    fn test_string_variants() {
        let mapper = BuiltinScopeMapper::new();
        assert_eq!(
            mapper.classify("string.quoted.double.js"),
            ScopeCategory::String
        );
        assert_eq!(
            mapper.classify("string.quoted.single.tsx"),
            ScopeCategory::String
        );
    }

    #[test]
    fn test_operator_vs_keyword() {
        let mapper = BuiltinScopeMapper::new();
        assert_eq!(
            mapper.classify("keyword.operator.arithmetic"),
            ScopeCategory::Operator
        );
        assert_eq!(
            mapper.classify("keyword.control.import"),
            ScopeCategory::Keyword
        );
    }

    #[test]
    fn test_no_match_fallback() {
        let mapper = BuiltinScopeMapper::new();
        let result = mapper.classify("some.weird.custom.scope");
        assert_eq!(
            result,
            ScopeCategory::Other("some.weird.custom.scope".into())
        );
    }

    #[test]
    fn test_comment_line() {
        let mapper = BuiltinScopeMapper::new();
        assert_eq!(
            mapper.classify("comment.line.double-slash"),
            ScopeCategory::Comment
        );
        assert_eq!(
            mapper.classify("comment.block.documentation"),
            ScopeCategory::Comment
        );
    }

    #[test]
    fn test_custom_override() {
        let mut mapper = BuiltinScopeMapper::new();
        mapper.add_mapping(
            "entity.name.function.python",
            ScopeCategory::Other("custom".into()),
        );
        assert_eq!(
            mapper.classify("entity.name.function.python"),
            ScopeCategory::Other("custom".into())
        );
        assert_eq!(
            mapper.classify("entity.name.function.rust"),
            ScopeCategory::Function
        );
    }

    #[test]
    fn test_scope_new_defaults() {
        let scope = Scope::new("keyword.control.if");
        assert_eq!(scope.category(), &ScopeCategory::Keyword);
        assert_eq!(scope.raw(), "keyword.control.if");
    }

    #[test]
    fn test_scope_with_category() {
        let scope = Scope::new("some.custom.scope").with_category(ScopeCategory::Variable);
        assert_eq!(scope.category(), &ScopeCategory::Variable);
    }

    #[test]
    fn test_number_mapping() {
        let mapper = BuiltinScopeMapper::new();
        assert_eq!(
            mapper.classify("constant.numeric.hex.rust"),
            ScopeCategory::Number
        );
    }

    #[test]
    fn test_variable_vs_parameter() {
        let mapper = BuiltinScopeMapper::new();
        assert_eq!(
            mapper.classify("variable.parameter.function"),
            ScopeCategory::Parameter
        );
        assert_eq!(
            mapper.classify("variable.other.tsx"),
            ScopeCategory::Variable
        );
    }

    #[test]
    fn test_invalid_stays_invalid() {
        let mapper = BuiltinScopeMapper::new();
        assert_eq!(mapper.classify("invalid.illegal"), ScopeCategory::Invalid);
    }

    #[test]
    fn test_property_mapping() {
        let mapper = BuiltinScopeMapper::new();
        assert_eq!(
            mapper.classify("support.variable.property"),
            ScopeCategory::Property
        );
        assert_eq!(
            mapper.classify("variable.other.property.css"),
            ScopeCategory::Property
        );
    }

    #[test]
    fn test_from_str() {
        let scope: Scope = "comment.line".into();
        assert_eq!(scope.category(), &ScopeCategory::Comment);
    }

    #[test]
    fn test_serde_roundtrip() {
        let scope = Scope::new("string.quoted.double");
        let json = serde_json::to_string(&scope).unwrap();
        let deserialized: Scope = serde_json::from_str(&json).unwrap();
        assert_eq!(scope, deserialized);
    }
}
