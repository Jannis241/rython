use rython_to_ir::lexer::*;
use rython_to_ir::parser::*;
use rython_to_ir::ast::*;

fn token(kind: TokenKind, value: &str) -> Token {
    Token {
        kind,
        value: value.to_string(),
    }
}

// ====================== IMPORT TESTS ======================

#[test]
fn test_parse_simple_import() {
    let input = vec![
        token(TokenKind::Import, "import"),
        token(TokenKind::Ident, "std"),
        token(TokenKind::Semicolon, ";"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.len(), 1);
}

#[test]
fn test_parse_import_with_dots() {
    let input = vec![
        token(TokenKind::Import, "import"),
        token(TokenKind::Ident, "std"),
        token(TokenKind::Dot, "."),
        token(TokenKind::Ident, "io"),
        token(TokenKind::Dot, "."),
        token(TokenKind::Ident, "read"),
        token(TokenKind::Semicolon, ";"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        Item::Import(imp) => assert_eq!(imp.import_name, "std.io.read"),
        _ => panic!("Expected Import item"),
    }
}

#[test]
fn test_parse_multiple_imports() {
    let input = vec![
        token(TokenKind::Import, "import"),
        token(TokenKind::Ident, "std"),
        token(TokenKind::Semicolon, ";"),
        token(TokenKind::Import, "import"),
        token(TokenKind::Ident, "math"),
        token(TokenKind::Semicolon, ";"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.len(), 2);
}

#[test]
fn test_parse_import_missing_semicolon() {
    let input = vec![
        token(TokenKind::Import, "import"),
        token(TokenKind::Ident, "std"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_err());
}

#[test]
fn test_parse_import_missing_ident() {
    let input = vec![
        token(TokenKind::Import, "import"),
        token(TokenKind::Semicolon, ";"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_err());
}

#[test]
fn test_parse_import_dot_without_ident() {
    let input = vec![
        token(TokenKind::Import, "import"),
        token(TokenKind::Ident, "std"),
        token(TokenKind::Dot, "."),
        token(TokenKind::Semicolon, ";"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_err());
}

// ====================== VARIANT TESTS ======================

#[test]
fn test_parse_simple_variant() {
    let input = vec![
        token(TokenKind::Variant, "variant"),
        token(TokenKind::Ident, "Color"),
        token(TokenKind::LBrace, "{"),
        token(TokenKind::Ident, "Red"),
        token(TokenKind::RBrace, "}"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        Item::Variant(v) => {
            assert_eq!(v.variant_name, "Color");
            assert_eq!(v.cases.len(), 1);
        }
        _ => panic!("Expected Variant item"),
    }
}

#[test]
fn test_parse_variant_multiple_cases() {
    let input = vec![
        token(TokenKind::Variant, "variant"),
        token(TokenKind::Ident, "Color"),
        token(TokenKind::LBrace, "{"),
        token(TokenKind::Ident, "Red"),
        token(TokenKind::Comma, ","),
        token(TokenKind::Ident, "Green"),
        token(TokenKind::Comma, ","),
        token(TokenKind::Ident, "Blue"),
        token(TokenKind::RBrace, "}"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        Item::Variant(v) => {
            assert_eq!(v.variant_name, "Color");
            assert_eq!(v.cases.len(), 3);
        }
        _ => panic!("Expected Variant item"),
    }
}

#[test]
fn test_parse_variant_single_case_no_comma() {
    let input = vec![
        token(TokenKind::Variant, "variant"),
        token(TokenKind::Ident, "Option"),
        token(TokenKind::LBrace, "{"),
        token(TokenKind::Ident, "None"),
        token(TokenKind::RBrace, "}"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
}

#[test]
fn test_parse_variant_trailing_comma() {
    let input = vec![
        token(TokenKind::Variant, "variant"),
        token(TokenKind::Ident, "Status"),
        token(TokenKind::LBrace, "{"),
        token(TokenKind::Ident, "Active"),
        token(TokenKind::Comma, ","),
        token(TokenKind::Ident, "Inactive"),
        token(TokenKind::Comma, ","),
        token(TokenKind::RBrace, "}"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
}

#[test]
fn test_parse_variant_missing_name() {
    let input = vec![
        token(TokenKind::Variant, "variant"),
        token(TokenKind::LBrace, "{"),
        token(TokenKind::RBrace, "}"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_err());
}

#[test]
fn test_parse_variant_missing_lbrace() {
    let input = vec![
        token(TokenKind::Variant, "variant"),
        token(TokenKind::Ident, "Color"),
        token(TokenKind::Ident, "Red"),
        token(TokenKind::RBrace, "}"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_err());
}

#[test]
fn test_parse_variant_missing_rbrace() {
    let input = vec![
        token(TokenKind::Variant, "variant"),
        token(TokenKind::Ident, "Color"),
        token(TokenKind::LBrace, "{"),
        token(TokenKind::Ident, "Red"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_err());
}

#[test]
fn test_parse_variant_missing_comma_between_cases() {
    let input = vec![
        token(TokenKind::Variant, "variant"),
        token(TokenKind::Ident, "Color"),
        token(TokenKind::LBrace, "{"),
        token(TokenKind::Ident, "Red"),
        token(TokenKind::Ident, "Green"),
        token(TokenKind::RBrace, "}"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_err());
}

#[test]
fn test_parse_variant_empty_body() {
    let input = vec![
        token(TokenKind::Variant, "variant"),
        token(TokenKind::Ident, "Empty"),
        token(TokenKind::LBrace, "{"),
        token(TokenKind::RBrace, "}"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
    let ast = result.unwrap();
    match &ast[0] {
        Item::Variant(v) => assert_eq!(v.cases.len(), 0),
        _ => panic!("Expected Variant item"),
    }
}

// ====================== TOP-LEVEL ERROR TESTS ======================

#[test]
fn test_parse_unexpected_token_at_top_level() {
    let input = vec![
        token(TokenKind::Plus, "+"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_err());
    match result {
        Err(ParseError::UnexpectedTopLevel(_)) => {}
        _ => panic!("Expected UnexpectedTopLevel error"),
    }
}

#[test]
fn test_parse_eof_only() {
    let input = vec![token(TokenKind::Eof, "")];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.len(), 0);
}

#[test]
fn test_parse_semicolons_skipped() {
    let input = vec![
        token(TokenKind::Semicolon, ";"),
        token(TokenKind::Semicolon, ";"),
        token(TokenKind::Import, "import"),
        token(TokenKind::Ident, "std"),
        token(TokenKind::Semicolon, ";"),
        token(TokenKind::Semicolon, ";"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.len(), 1);
}

// ====================== MIXED STATEMENT TESTS ======================

#[test]
fn test_parse_import_and_variant() {
    let input = vec![
        token(TokenKind::Import, "import"),
        token(TokenKind::Ident, "std"),
        token(TokenKind::Semicolon, ";"),
        token(TokenKind::Variant, "variant"),
        token(TokenKind::Ident, "Color"),
        token(TokenKind::LBrace, "{"),
        token(TokenKind::Ident, "Red"),
        token(TokenKind::RBrace, "}"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.len(), 2);
}

#[test]
fn test_parse_variant_and_import() {
    let input = vec![
        token(TokenKind::Variant, "variant"),
        token(TokenKind::Ident, "Status"),
        token(TokenKind::LBrace, "{"),
        token(TokenKind::Ident, "On"),
        token(TokenKind::RBrace, "}"),
        token(TokenKind::Import, "import"),
        token(TokenKind::Ident, "sys"),
        token(TokenKind::Semicolon, ";"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
    let ast = result.unwrap();
    assert_eq!(ast.len(), 2);
}

// ====================== UNIMPLEMENTED FEATURE STUBS ======================

#[test]
fn test_parse_trait_stub() {
    let input = vec![
        token(TokenKind::Trait, "trait"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
}

#[test]
fn test_parse_struct_stub() {
    let input = vec![
        token(TokenKind::Struct, "struct"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
}

#[test]
fn test_parse_fn_stub() {
    let input = vec![
        token(TokenKind::Fn, "fn"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
}

#[test]
fn test_parse_impl_stub() {
    let input = vec![
        token(TokenKind::Impl, "impl"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
}

// ====================== EDGE CASES ======================

#[test]
fn test_parse_eof_during_import() {
    let input = vec![
        token(TokenKind::Import, "import"),
        token(TokenKind::Ident, "std"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_err());
}

#[test]
fn test_parse_eof_during_variant() {
    let input = vec![
        token(TokenKind::Variant, "variant"),
        token(TokenKind::Ident, "Color"),
        token(TokenKind::LBrace, "{"),
        token(TokenKind::Ident, "Red"),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_err());
}

#[test]
fn test_parse_variant_case_eof_during_comma_loop() {
    let input = vec![
        token(TokenKind::Variant, "variant"),
        token(TokenKind::Ident, "Option"),
        token(TokenKind::LBrace, "{"),
        token(TokenKind::Ident, "Some"),
        token(TokenKind::Comma, ","),
        token(TokenKind::Eof, ""),
    ];

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_err());
}

#[test]
fn test_parse_variant_with_many_cases() {
    let mut input = vec![
        token(TokenKind::Variant, "variant"),
        token(TokenKind::Ident, "Large"),
        token(TokenKind::LBrace, "{"),
    ];

    for i in 0..100 {
        input.push(token(TokenKind::Ident, &format!("Case{}", i)));
        if i < 99 {
            input.push(token(TokenKind::Comma, ","));
        }
    }

    input.push(token(TokenKind::RBrace, "}"));
    input.push(token(TokenKind::Eof, ""));

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
    let ast = result.unwrap();
    match &ast[0] {
        Item::Variant(v) => assert_eq!(v.cases.len(), 100),
        _ => panic!("Expected Variant item"),
    }
}

#[test]
fn test_parse_long_import_path() {
    let mut input = vec![token(TokenKind::Import, "import")];

    for i in 0..20 {
        input.push(token(TokenKind::Ident, &format!("module{}", i)));
        if i < 19 {
            input.push(token(TokenKind::Dot, "."));
        }
    }

    input.push(token(TokenKind::Semicolon, ";"));
    input.push(token(TokenKind::Eof, ""));

    let mut parser = Parser::new(input);
    let result = parser.parse();

    assert!(result.is_ok());
    let ast = result.unwrap();
    match &ast[0] {
        Item::Import(imp) => {
            let parts: Vec<&str> = imp.import_name.split('.').collect();
            assert_eq!(parts.len(), 20);
        }
        _ => panic!("Expected Import item"),
    }
}
