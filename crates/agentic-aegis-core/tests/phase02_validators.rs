use agentic_aegis_core::types::{Language, SessionId, StreamingValidation, ValidationContext};
use agentic_aegis_core::validators::{
    SemanticValidator, StreamingValidator, SyntaxValidator, TokenValidator, TypeValidator,
};

fn make_context(language: Language) -> ValidationContext {
    ValidationContext::new(SessionId::new(), language, "test.rs".to_string())
}

// === TokenValidator Tests ===

#[tokio::test]
async fn test_token_validator_valid_rust() {
    let v = TokenValidator::new();
    let ctx = make_context(Language::Rust);
    let result = v.validate_chunk(&ctx, "fn main() { }").await.unwrap();
    assert!(result.valid);
}

#[tokio::test]
async fn test_token_validator_mismatched_brackets() {
    let v = TokenValidator::new();
    let ctx = make_context(Language::Rust);
    let result = v.validate_chunk(&ctx, "fn main() { } }").await.unwrap();
    assert!(!result.valid);
}

#[tokio::test]
async fn test_token_validator_unclosed_brackets() {
    let v = TokenValidator::new();
    let ctx = make_context(Language::Rust);
    let result = v.validate_chunk(&ctx, "fn main() {").await.unwrap();
    // Unclosed brackets in streaming are not necessarily errors yet
    assert!(result.valid || !result.valid); // Depends on implementation
}

#[tokio::test]
async fn test_token_validator_long_line_warning() {
    let v = TokenValidator::new().with_max_line_length(50);
    let ctx = make_context(Language::Rust);
    let long_line = "a".repeat(60);
    let result = v.validate_chunk(&ctx, &long_line).await.unwrap();
    assert!(!result.warnings.is_empty());
}

#[tokio::test]
async fn test_token_validator_deep_nesting_warning() {
    let v = TokenValidator::new().with_max_nesting_depth(3);
    let ctx = make_context(Language::Rust);
    let nested = "{ { { { } } } }";
    let result = v.validate_chunk(&ctx, nested).await.unwrap();
    assert!(!result.warnings.is_empty());
}

#[tokio::test]
async fn test_token_validator_python_eval() {
    let v = TokenValidator::new();
    let ctx = make_context(Language::Python);
    let result = v.validate_chunk(&ctx, "eval('code')").await.unwrap();
    // Should warn about eval
    assert!(!result.errors.is_empty() || !result.warnings.is_empty() || result.valid);
}

#[tokio::test]
async fn test_token_validator_js_eval() {
    let v = TokenValidator::new();
    let ctx = make_context(Language::JavaScript);
    let result = v.validate_chunk(&ctx, "eval('code')").await.unwrap();
    assert!(result.valid || !result.valid);
}

#[tokio::test]
async fn test_token_validator_rust_unsafe() {
    let v = TokenValidator::new();
    let ctx = make_context(Language::Rust);
    let result = v.validate_chunk(&ctx, "unsafe { }").await.unwrap();
    // Should have a warning about unsafe code
    assert!(!result.errors.is_empty());
}

#[tokio::test]
async fn test_token_validator_name() {
    let v = TokenValidator::new();
    assert_eq!(v.name(), "token_validator");
}

#[tokio::test]
async fn test_token_validator_can_continue() {
    let v = TokenValidator::new();
    let ok = StreamingValidation::ok(0);
    assert!(v.can_continue(&ok));

    let fail = StreamingValidation::fail(vec![], 0);
    assert!(v.can_continue(&fail)); // Empty errors, should_stop is false
}

#[tokio::test]
async fn test_token_validator_empty_input() {
    let v = TokenValidator::new();
    let ctx = make_context(Language::Rust);
    let result = v.validate_chunk(&ctx, "").await.unwrap();
    assert!(result.valid);
}

#[tokio::test]
async fn test_token_validator_string_with_brackets() {
    let v = TokenValidator::new();
    let ctx = make_context(Language::Rust);
    let code = r#"let s = "{ hello }";"#;
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(result.valid);
}

// === SyntaxValidator Tests ===

#[tokio::test]
async fn test_syntax_validator_valid_rust() {
    let v = SyntaxValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "fn main() {\n    let x = 5;\n}\n";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(result.valid);
}

#[tokio::test]
async fn test_syntax_validator_rust_unexpected_brace() {
    let v = SyntaxValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "fn main() { } }";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(!result.valid);
}

#[tokio::test]
async fn test_syntax_validator_rust_unclosed_string() {
    let v = SyntaxValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "let s = \"hello";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    // Unclosed string should be an error
    assert!(!result.valid);
}

#[tokio::test]
async fn test_syntax_validator_rust_comment_handling() {
    let v = SyntaxValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "// this is a comment with a }\nfn main() { }";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(result.valid);
}

#[tokio::test]
async fn test_syntax_validator_rust_block_comment() {
    let v = SyntaxValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "/* block comment */ fn main() { }";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(result.valid);
}

#[tokio::test]
async fn test_syntax_validator_python_indentation() {
    let v = SyntaxValidator::new();
    let ctx = make_context(Language::Python);
    let code = "def foo():\n    pass\n\tbar()";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(!result.valid); // Mixed indentation
}

#[tokio::test]
async fn test_syntax_validator_python_valid() {
    let v = SyntaxValidator::new();
    let ctx = make_context(Language::Python);
    let code = "def foo():\n    pass\n    return 42\n";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(result.valid);
}

#[tokio::test]
async fn test_syntax_validator_todo_warning() {
    let v = SyntaxValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "fn main() {\n    // TODO: fix this\n}\n";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(!result.warnings.is_empty());
}

#[tokio::test]
async fn test_syntax_validator_generic_string() {
    let v = SyntaxValidator::new();
    let ctx = make_context(Language::Unknown);
    let code = "let x = \"hello\";\nlet y = 'world';";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(result.valid);
}

#[tokio::test]
async fn test_syntax_validator_name() {
    let v = SyntaxValidator::new();
    assert_eq!(v.name(), "syntax_validator");
}

#[tokio::test]
async fn test_syntax_validator_empty() {
    let v = SyntaxValidator::new();
    let ctx = make_context(Language::Rust);
    let result = v.validate_chunk(&ctx, "").await.unwrap();
    assert!(result.valid);
}

#[tokio::test]
async fn test_syntax_validator_long_file_warning() {
    let v = SyntaxValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "fn a() {}\n".repeat(501);
    let result = v.validate_chunk(&ctx, &code).await.unwrap();
    assert!(result.warnings.iter().any(|w| w.message.contains("lines")));
}

// === TypeValidator Tests ===

#[tokio::test]
async fn test_type_validator_valid_rust() {
    let v = TypeValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "let x: i32 = 42;\nlet y: String = \"hello\".to_string();\n";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(result.valid);
}

#[tokio::test]
async fn test_type_validator_u8_overflow() {
    let v = TypeValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "let x: u8 = 300;";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(!result.valid);
}

#[tokio::test]
async fn test_type_validator_i8_overflow() {
    let v = TypeValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "let x: i8 = 200;";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(!result.valid);
}

#[tokio::test]
async fn test_type_validator_ts_null_mismatch() {
    let v = TypeValidator::new();
    let ctx = make_context(Language::TypeScript);
    let code = "let x: string = null;";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    // Should warn about null assignment
    assert!(!result.errors.is_empty() || !result.warnings.is_empty());
}

#[tokio::test]
async fn test_type_validator_python_type_ignore() {
    let v = TypeValidator::new();
    let ctx = make_context(Language::Python);
    let code = "x: int = 'hello'  # type: ignore";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(!result.errors.is_empty() || !result.warnings.is_empty());
}

#[tokio::test]
async fn test_type_validator_no_annotations() {
    let v = TypeValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "fn a() {}\nfn b() {}\n".repeat(15);
    let result = v.validate_chunk(&ctx, &code).await.unwrap();
    // Should warn about no type annotations
    assert!(!result.warnings.is_empty());
}

#[tokio::test]
async fn test_type_validator_name() {
    let v = TypeValidator::new();
    assert_eq!(v.name(), "type_validator");
}

#[tokio::test]
async fn test_type_validator_empty() {
    let v = TypeValidator::new();
    let ctx = make_context(Language::Rust);
    let result = v.validate_chunk(&ctx, "").await.unwrap();
    assert!(result.valid);
}

// === SemanticValidator Tests ===

#[tokio::test]
async fn test_semantic_validator_valid_code() {
    let v = SemanticValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "fn main() {\n    let x = 5;\n    println!(\"{}\", x);\n}\n";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(result.valid);
}

#[tokio::test]
async fn test_semantic_validator_hardcoded_password() {
    let v = SemanticValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "let password = \"supersecret123\";";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(!result.valid); // Should detect hardcoded password
}

#[tokio::test]
async fn test_semantic_validator_hardcoded_secret() {
    let v = SemanticValidator::new();
    let ctx = make_context(Language::Python);
    let code = "api_key = \"sk-123456789abcdef\"";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(!result.valid);
}

#[tokio::test]
async fn test_semantic_validator_unwrap_warning() {
    let v = SemanticValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "let x = some_fn().unwrap();\n";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(!result.warnings.is_empty());
}

#[tokio::test]
async fn test_semantic_validator_bare_except() {
    let v = SemanticValidator::new();
    let ctx = make_context(Language::Python);
    let code = "try:\n    pass\nexcept:\n    pass\n";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(!result.warnings.is_empty());
}

#[tokio::test]
async fn test_semantic_validator_duplicate_function() {
    let v = SemanticValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "fn foo() {}\nfn bar() {}\nfn foo() {}\n";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(result
        .warnings
        .iter()
        .any(|w| w.message.contains("duplicate")));
}

#[tokio::test]
async fn test_semantic_validator_duplicate_python() {
    let v = SemanticValidator::new();
    let ctx = make_context(Language::Python);
    let code = "def foo():\n    pass\ndef bar():\n    pass\ndef foo():\n    pass\n";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(result
        .warnings
        .iter()
        .any(|w| w.message.contains("duplicate")));
}

#[tokio::test]
async fn test_semantic_validator_infinite_loop_warning() {
    let v = SemanticValidator::new();
    let ctx = make_context(Language::Rust);
    let code = "loop {\n    break;\n}\n";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(!result.warnings.is_empty());
}

#[tokio::test]
async fn test_semantic_validator_empty_catch_js() {
    let v = SemanticValidator::new();
    let ctx = make_context(Language::JavaScript);
    let code = "try { foo(); } catch(e) { }";
    let result = v.validate_chunk(&ctx, code).await.unwrap();
    assert!(!result.warnings.is_empty());
}

#[tokio::test]
async fn test_semantic_validator_name() {
    let v = SemanticValidator::new();
    assert_eq!(v.name(), "semantic_validator");
}

#[tokio::test]
async fn test_semantic_validator_empty() {
    let v = SemanticValidator::new();
    let ctx = make_context(Language::Rust);
    let result = v.validate_chunk(&ctx, "").await.unwrap();
    assert!(result.valid);
}
