use ikihs_core::scope::ScopeCategory;

pub fn node_to_category(
    kind: &str,
    text: &str,
    parent_kind: Option<&str>,
    grandparent_kind: Option<&str>,
    field_name: Option<&str>,
    is_named: bool,
) -> (ScopeCategory, String) {
    if is_named {
        let (mut category, mut scope_string) =
            named_node(kind, parent_kind, grandparent_kind, field_name);
        if kind == "property_identifier"
            && text == "constructor"
            && parent_kind == Some("method_definition")
        {
            category = ScopeCategory::Keyword;
            scope_string = "source.ts keyword.constructor".into();
        }
        (category, scope_string)
    } else if text == ":" && parent_kind == Some("pair") {
        (
            ScopeCategory::Property,
            "punctuation.separator.colon.property".into(),
        )
    } else if text == "${" {
        (ScopeCategory::Keyword, "source.ts keyword.template".into())
    } else if text == "}" && parent_kind == Some("template_substitution") {
        (ScopeCategory::Keyword, "source.ts keyword.template".into())
    } else {
        anonymous_token(text)
    }
}

fn named_node(
    kind: &str,
    parent_kind: Option<&str>,
    grandparent_kind: Option<&str>,
    field_name: Option<&str>,
) -> (ScopeCategory, String) {
    match kind {
        "type_identifier" => (ScopeCategory::Type, "source.ts entity.name.type".into()),
        "predefined_type" => (ScopeCategory::Type, "source.ts support.type".into()),
        "literal_type" => (ScopeCategory::String, "source.ts string.quoted".into()),

        "string" => (ScopeCategory::String, "source.ts string.quoted".into()),
        "string_fragment" => (
            ScopeCategory::String,
            "source.ts string.quoted.template".into(),
        ),
        "template_string" => (
            ScopeCategory::String,
            "source.ts string.quoted.template".into(),
        ),
        "number" => (ScopeCategory::Number, "source.ts constant.numeric".into()),

        "comment" => (ScopeCategory::Comment, "source.ts comment.line".into()),
        "hash_bang_line" => (
            ScopeCategory::Comment,
            "source.ts comment.line.hashbang".into(),
        ),

        "property_identifier" => match (parent_kind, grandparent_kind) {
            (Some("method_definition"), _) => (
                ScopeCategory::Function,
                "source.ts entity.name.function".into(),
            ),
            (Some("member_expression"), Some("call_expression")) => (
                ScopeCategory::Function,
                "source.ts meta.function-call.property".into(),
            ),
            _ => (
                ScopeCategory::Property,
                "source.ts variable.other.property".into(),
            ),
        },
        "private_property_identifier" => (
            ScopeCategory::Property,
            "source.ts variable.other.property.private".into(),
        ),

        "identifier" => match (parent_kind, field_name) {
            (
                Some(
                    "function_declaration"
                    | "generator_function_declaration"
                    | "method_definition"
                    | "method_signature"
                    | "abstract_method_signature"
                    | "function_expression"
                    | "generator_function",
                ),
                Some("name"),
            ) => (
                ScopeCategory::Function,
                "source.ts entity.name.function".into(),
            ),
            (Some("call_expression"), Some("function")) => (
                ScopeCategory::Function,
                "source.ts meta.function-call".into(),
            ),
            (Some("function_declaration" | "function_expression"), _) => {
                (ScopeCategory::Function, "source.ts meta.function".into())
            }
            (Some("variable_declarator"), Some("name"))
            | (Some("assignment_expression"), Some("left")) => (
                ScopeCategory::Variable,
                "source.ts variable.other.readwrite".into(),
            ),
            (Some("formal_parameter" | "required_parameter" | "optional_parameter"), _) => (
                ScopeCategory::Parameter,
                "source.ts variable.parameter".into(),
            ),
            (Some("class_declaration" | "abstract_class_declaration"), Some("name"))
            | (Some("class"), Some("name")) => {
                (ScopeCategory::Class, "source.ts entity.name.class".into())
            }
            (Some("member_expression"), Some("property")) => (
                ScopeCategory::Property,
                "source.ts variable.other.property".into(),
            ),
            (Some("catch_clause"), Some("parameter")) => (
                ScopeCategory::Parameter,
                "source.ts variable.parameter".into(),
            ),
            (
                Some(
                    "type_annotation"
                    | "type_alias_declaration"
                    | "type_parameter"
                    | "type_arguments",
                ),
                _,
            ) => (ScopeCategory::Type, "source.ts entity.name.type".into()),
            (_, _) => (
                ScopeCategory::Variable,
                "source.ts variable.other.readwrite".into(),
            ),
        },

        "this" | "super" => (ScopeCategory::Keyword, "source.ts keyword.this".into()),

        "array_pattern" | "object_pattern" | "rest_pattern" => (
            ScopeCategory::Parameter,
            "source.ts variable.parameter".into(),
        ),

        _ => (ScopeCategory::Other(kind.to_string()), "source.ts".into()),
    }
}

fn anonymous_token(text: &str) -> (ScopeCategory, String) {
    match text {
        // Control-flow keywords → keyword.control (#C586C0, purple)
        "if" | "else" | "for" | "while" | "do" | "return" | "switch" | "case" | "default"
        | "break" | "continue" | "throw" | "try" | "catch" | "finally" => {
            (ScopeCategory::Keyword, "source.ts keyword.control".into())
        }

        // Declaration/storage keywords → keyword (#569CD6, blue)
        "const" | "let" | "var" | "function" | "class" | "interface" | "type" | "enum"
        | "extends" | "implements" | "abstract" | "async" | "await" | "yield" | "new"
        | "typeof" | "instanceof" | "keyof" | "readonly" | "static" | "public" | "private"
        | "protected" | "declare" | "as" | "satisfies" | "in" | "of" | "using" | "with"
        | "module" | "namespace" | "from" | "require" | "import" | "export" | "get" | "set" => {
            (ScopeCategory::Keyword, "source.ts keyword".into())
        }

        "this" | "super" | "null" | "undefined" | "true" | "false" => {
            (ScopeCategory::Keyword, "source.ts keyword.control".into())
        }

        "+" | "-" | "*" | "/" | "%" | "**" | "++" | "--" | "==" | "===" | "!=" | "!==" | "<"
        | ">" | "<=" | ">=" | "&&" | "||" | "!" | "??" | "&" | "|" | "^" | "~" | "<<" | ">>"
        | ">>>" | "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "**=" | "&=" | "|=" | "^=" | "<<="
        | ">>=" | ">>>=" | "&&=" | "||=" | "??=" | "?" | "=>" | "..." | "?." => {
            (ScopeCategory::Operator, "source.ts keyword.operator".into())
        }

        "`" => (
            ScopeCategory::String,
            "source.ts string.quoted.template.backtick".into(),
        ),

        _ => (ScopeCategory::Other(text.to_string()), "source.ts".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_identifier() {
        let (cat, _) = named_node("type_identifier", None, None, None);
        assert_eq!(cat, ScopeCategory::Type);
    }

    #[test]
    fn test_function_name() {
        let (cat, _) = named_node(
            "identifier",
            Some("function_declaration"),
            None,
            Some("name"),
        );
        assert_eq!(cat, ScopeCategory::Function);
    }

    #[test]
    fn test_variable_name() {
        let (cat, _) = named_node(
            "identifier",
            Some("variable_declarator"),
            None,
            Some("name"),
        );
        assert_eq!(cat, ScopeCategory::Variable);
    }

    #[test]
    fn test_parameter_name() {
        let (cat, _) = named_node("identifier", Some("required_parameter"), None, None);
        assert_eq!(cat, ScopeCategory::Parameter);
    }

    #[test]
    fn test_property() {
        let (cat, _) = named_node("property_identifier", None, None, None);
        assert_eq!(cat, ScopeCategory::Property);
    }

    #[test]
    fn test_method() {
        let (cat, _) = named_node("property_identifier", Some("method_definition"), None, None);
        assert_eq!(cat, ScopeCategory::Function);
    }

    #[test]
    fn test_call_property() {
        let (cat, _) = named_node(
            "property_identifier",
            Some("member_expression"),
            Some("call_expression"),
            None,
        );
        assert_eq!(cat, ScopeCategory::Function);
    }

    #[test]
    fn test_class_name() {
        let (cat, _) = named_node("identifier", Some("class_declaration"), None, Some("name"));
        assert_eq!(cat, ScopeCategory::Class);
    }
}
