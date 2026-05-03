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

    assert!(result.is_err());
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



//! Comprehensive parser tests.
//!
//! Assumption: `Token` can be constructed as `Token { kind, value }` and the
//! `value` field accepts `String`.
//! If your lexer token type has more fields or a different constructor, update
//! the small helper functions below and keep the test cases as-is.

use super::*;

fn tok(kind: TokenKind, value: impl Into<String>) -> Token {
    Token { kind, value: value.into() }
}

fn ident(name: &str) -> Token { tok(TokenKind::Ident, name) }
fn int_lit(n: i64) -> Token { tok(TokenKind::Int, n.to_string()) }
fn float_lit(n: f64) -> Token { tok(TokenKind::Float, n.to_string()) }
fn str_lit(s: &str) -> Token { tok(TokenKind::StringLiteral, s) }
fn sym(kind: TokenKind) -> Token { tok(kind, "") }
fn eof() -> Token { tok(TokenKind::Eof, "") }

fn parser(tokens: Vec<Token>) -> Parser { Parser::new(tokens) }

fn parse_expr(tokens: Vec<Token>) -> Result<Expr, ParseError> {
    let mut p = parser(tokens);
    p.parse_expr()
}

fn parse_stmt(tokens: Vec<Token>) -> Result<Stmt, ParseError> {
    let mut p = parser(tokens);
    p.parse_statement()
}

#[test]
fn advance_on_eof_returns_error() {
let mut p = parser(vec![eof()]);
assert!(matches!(p.current(), Ok(Token { kind: TokenKind::Eof, .. })));
assert!(matches!(p.advance(), Err(ParseError::UnexpectedEof)));
}

#[test]
fn peek_on_single_token_returns_error() {
let p = parser(vec![eof()]);
assert!(matches!(p.peek(), Err(ParseError::UnexpectedEof)));
}

#[test]
fn expect_current_success() {
let p = parser(vec![ident("x"), eof()]);
assert!(p.expect_current(TokenKind::Ident).is_ok());
}

#[test]
fn expect_current_error_reports_found_kind() {
let p = parser(vec![ident("x"), eof()]);
match p.expect_current(TokenKind::Int) {
    Err(ParseError::UnexpectedToken { expected, found, token_idx }) => {
        assert_eq!(expected, TokenKind::Int);
        assert_eq!(found, TokenKind::Ident);
        assert_eq!(token_idx, 0);
    }
    other => panic!("unexpected result: {other:?}"),
}
}

#[test]
fn parse_primary_int_literal() {
let expr = parse_expr(vec![int_lit(42), eof()]).unwrap();
assert!(matches!(expr, Expr::IntLiteral(_)));
}

#[test]
fn parse_primary_float_literal() {
let expr = parse_expr(vec![float_lit(3.5), eof()]).unwrap();
assert!(matches!(expr, Expr::FloatLiteral(_)));
}

#[test]
fn parse_primary_true_literal() {
let expr = parse_expr(vec![sym(TokenKind::True), eof()]).unwrap();
assert!(matches!(expr, Expr::BoolLiteral(true)));
}

#[test]
fn parse_primary_false_literal() {
let expr = parse_expr(vec![sym(TokenKind::False), eof()]).unwrap();
assert!(matches!(expr, Expr::BoolLiteral(false)));
}

#[test]
fn parse_primary_string_literal() {
let expr = parse_expr(vec![str_lit("hi"), eof()]).unwrap();
assert!(matches!(expr, Expr::StringLiteral(_)));
}

#[test]
fn parse_primary_variable() {
let expr = parse_expr(vec![ident("x"), eof()]).unwrap();
assert!(matches!(expr, Expr::Variable(_)));
}

#[test]
fn parse_primary_grouping() {
let expr = parse_expr(vec![sym(TokenKind::LParen), int_lit(1), sym(TokenKind::RParen), eof()]).unwrap();
assert!(matches!(expr, Expr::Grouping(_)));
}

#[test]
fn parse_primary_empty_list() {
let expr = parse_expr(vec![sym(TokenKind::LBracket), sym(TokenKind::RBracket), eof()]).unwrap();
assert!(matches!(expr, Expr::ListLiteral(elements) if elements.is_empty()));
}

#[test]
fn parse_primary_single_element_list() {
let expr = parse_expr(vec![sym(TokenKind::LBracket), int_lit(1), sym(TokenKind::RBracket), eof()]).unwrap();
assert!(matches!(expr, Expr::ListLiteral(elements) if elements.len() == 1));
}

#[test]
fn parse_primary_two_element_list() {
let expr = parse_expr(vec![sym(TokenKind::LBracket), int_lit(1), sym(TokenKind::Comma), int_lit(2), sym(TokenKind::RBracket), eof()]).unwrap();
assert!(matches!(expr, Expr::ListLiteral(elements) if elements.len() == 2));
}

#[test]
fn parse_primary_empty_struct_literal() {
let expr = parse_expr(vec![ident("Point"), sym(TokenKind::LBrace), sym(TokenKind::RBrace), eof()]).unwrap();
assert!(matches!(expr, Expr::StructLiteral { struct_name, arguments } if struct_name == "Point" && arguments.is_empty()));
}

#[test]
fn parse_primary_two_field_struct_literal() {
let expr = parse_expr(vec![ident("Point"), sym(TokenKind::LBrace), ident("x"), sym(TokenKind::Colon), int_lit(1), sym(TokenKind::Comma), ident("y"), sym(TokenKind::Colon), int_lit(2), sym(TokenKind::RBrace), eof()]).unwrap();
assert!(matches!(expr, Expr::StructLiteral { struct_name, arguments } if struct_name == "Point" && arguments.len() == 2));
}

#[test]
fn parse_unary_neg_int() {
let expr = parse_expr(vec![sym(TokenKind::Minus), int_lit(1), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::Neg, .. }));
}

#[test]
fn parse_unary_bitnot_int() {
let expr = parse_expr(vec![sym(TokenKind::Tilde), int_lit(1), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::BitNot, .. }));
}

#[test]
fn parse_unary_neg_float() {
let expr = parse_expr(vec![sym(TokenKind::Minus), float_lit(1.0), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::Neg, .. }));
}

#[test]
fn parse_unary_bitnot_float() {
let expr = parse_expr(vec![sym(TokenKind::Tilde), float_lit(1.0), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::BitNot, .. }));
}

#[test]
fn parse_unary_neg_true() {
let expr = parse_expr(vec![sym(TokenKind::Minus), sym(TokenKind::True), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::Neg, .. }));
}

#[test]
fn parse_unary_bitnot_true() {
let expr = parse_expr(vec![sym(TokenKind::Tilde), sym(TokenKind::True), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::BitNot, .. }));
}

#[test]
fn parse_unary_neg_false() {
let expr = parse_expr(vec![sym(TokenKind::Minus), sym(TokenKind::False), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::Neg, .. }));
}

#[test]
fn parse_unary_bitnot_false() {
let expr = parse_expr(vec![sym(TokenKind::Tilde), sym(TokenKind::False), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::BitNot, .. }));
}

#[test]
fn parse_unary_neg_string() {
let expr = parse_expr(vec![sym(TokenKind::Minus), str_lit("s"), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::Neg, .. }));
}

#[test]
fn parse_unary_bitnot_string() {
let expr = parse_expr(vec![sym(TokenKind::Tilde), str_lit("s"), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::BitNot, .. }));
}

#[test]
fn parse_unary_neg_ident() {
let expr = parse_expr(vec![sym(TokenKind::Minus), ident("x"), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::Neg, .. }));
}

#[test]
fn parse_unary_bitnot_ident() {
let expr = parse_expr(vec![sym(TokenKind::Tilde), ident("x"), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::BitNot, .. }));
}

#[test]
fn parse_unary_neg_grouping() {
let expr = parse_expr(vec![sym(TokenKind::Minus), sym(TokenKind::LParen), int_lit(1), sym(TokenKind::RParen), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::Neg, .. }));
}

#[test]
fn parse_unary_bitnot_grouping() {
let expr = parse_expr(vec![sym(TokenKind::Tilde), sym(TokenKind::LParen), int_lit(1), sym(TokenKind::RParen), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::BitNot, .. }));
}

#[test]
fn parse_unary_neg_list() {
let expr = parse_expr(vec![sym(TokenKind::Minus), sym(TokenKind::LBracket), int_lit(1), sym(TokenKind::RBracket), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::Neg, .. }));
}

#[test]
fn parse_unary_bitnot_list() {
let expr = parse_expr(vec![sym(TokenKind::Tilde), sym(TokenKind::LBracket), int_lit(1), sym(TokenKind::RBracket), eof()]).unwrap();
assert!(matches!(expr, Expr::Unary { op: UnaryOp::BitNot, .. }));
}

#[test]
fn parse_binary_mul_beats_add() {
let expr = parse_expr(vec![int_lit(1), sym(TokenKind::Plus), int_lit(2), sym(TokenKind::Star), int_lit(3), eof()]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Add, .. }));
}

#[test]
fn parse_binary_add_beats_comparison() {
let expr = parse_expr(vec![int_lit(1), sym(TokenKind::Plus), int_lit(2), sym(TokenKind::Lt), int_lit(3), eof()]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Lt, .. }));
}

#[test]
fn parse_binary_shift_between_add_and_comparison() {
let expr = parse_expr(vec![int_lit(1), sym(TokenKind::LtLt), int_lit(2), sym(TokenKind::Plus), int_lit(3), eof()]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Add, .. }));
}

#[test]
fn parse_binary_bitand_before_bitor() {
let expr = parse_expr(vec![int_lit(1), sym(TokenKind::Pipe), int_lit(2), sym(TokenKind::Amp), int_lit(3), eof()]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::BitOr, .. }));
}

#[test]
fn parse_binary_bitxor_between_bitand_and_bitor() {
let expr = parse_expr(vec![int_lit(1), sym(TokenKind::Caret), int_lit(2), sym(TokenKind::Pipe), int_lit(3), eof()]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::BitOr, .. }));
}

#[test]
fn parse_binary_logical_and_before_or() {
let expr = parse_expr(vec![sym(TokenKind::True), sym(TokenKind::Or), sym(TokenKind::False), sym(TokenKind::And), sym(TokenKind::True), eof()]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Or, .. }));
}

#[test]
fn parse_binary_equality_left_assoc() {
let expr = parse_expr(vec![int_lit(1), sym(TokenKind::EqEq), int_lit(2), sym(TokenKind::EqEq), int_lit(3), eof()]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Eq, .. }));
}

#[test]
fn parse_binary_comparison_left_assoc() {
let expr = parse_expr(vec![int_lit(1), sym(TokenKind::Lt), int_lit(2), sym(TokenKind::LtEq), int_lit(3), eof()]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Le, .. }));
}

#[test]
fn parse_assignment_simple_assignment() {
let expr = parse_expr(vec![ident("x"), sym(TokenKind::Eq), int_lit(1), eof()]).unwrap();
assert!(matches!(expr, Expr::Assign { .. }));
}

#[test]
fn parse_assignment_plus_assignment() {
let expr = parse_expr(vec![ident("x"), sym(TokenKind::PlusEq), int_lit(1), eof()]).unwrap();
assert!(matches!(expr, Expr::BinaryOpAssign { .. }));
}

#[test]
fn parse_assignment_minus_assignment() {
let expr = parse_expr(vec![ident("x"), sym(TokenKind::MinusEq), int_lit(1), eof()]).unwrap();
assert!(matches!(expr, Expr::BinaryOpAssign { .. }));
}

#[test]
fn parse_assignment_mul_assignment() {
let expr = parse_expr(vec![ident("x"), sym(TokenKind::StarEq), int_lit(1), eof()]).unwrap();
assert!(matches!(expr, Expr::BinaryOpAssign { .. }));
}

#[test]
fn parse_assignment_div_assignment() {
let expr = parse_expr(vec![ident("x"), sym(TokenKind::SlashEq), int_lit(1), eof()]).unwrap();
assert!(matches!(expr, Expr::BinaryOpAssign { .. }));
}

#[test]
fn invalid_assignment_target_0() {
match parse_expr(vec![int_lit(1), sym(TokenKind::Eq), int_lit(2), eof()]) {
    Err(ParseError::InvalidAssignmentTarget) => {}
    _ => panic!("expected InvalidAssignmentTarget"),
}
}

#[test]
fn invalid_assignment_target_1() {
match parse_expr(vec![sym(TokenKind::LParen), int_lit(1), sym(TokenKind::RParen), sym(TokenKind::Eq), int_lit(2), eof()]) {
    Err(ParseError::InvalidAssignmentTarget) => {}
    _ => panic!("expected InvalidAssignmentTarget"),
}
}

#[test]
fn invalid_assignment_target_2() {
match parse_expr(vec![sym(TokenKind::True), sym(TokenKind::Eq), int_lit(2), eof()]) {
    Err(ParseError::InvalidAssignmentTarget) => {}
    _ => panic!("expected InvalidAssignmentTarget"),
}
}

#[test]
fn invalid_assignment_target_3() {
match parse_expr(vec![sym(TokenKind::LBracket), int_lit(1), sym(TokenKind::RBracket), sym(TokenKind::Eq), int_lit(2), eof()]) {
    Err(ParseError::InvalidAssignmentTarget) => {}
    _ => panic!("expected InvalidAssignmentTarget"),
}
}

#[test]
fn parse_call_zero_args() {
let expr = parse_expr(vec![ident("f"), sym(TokenKind::LParen), sym(TokenKind::RParen), eof()]).unwrap();
assert!(matches!(expr, Expr::Call { arguments, .. } if arguments.len() == 0));
}

#[test]
fn parse_call_one_arg() {
let expr = parse_expr(vec![ident("f"), sym(TokenKind::LParen), int_lit(1), sym(TokenKind::RParen), eof()]).unwrap();
assert!(matches!(expr, Expr::Call { arguments, .. } if arguments.len() == 1));
}

#[test]
fn parse_call_two_args() {
let expr = parse_expr(vec![ident("f"), sym(TokenKind::LParen), int_lit(1), sym(TokenKind::Comma), int_lit(2), sym(TokenKind::RParen), eof()]).unwrap();
assert!(matches!(expr, Expr::Call { arguments, .. } if arguments.len() == 2));
}

#[test]
fn parse_call_nested_call() {
let expr = parse_expr(vec![ident("f"), sym(TokenKind::LParen), ident("g"), sym(TokenKind::LParen), int_lit(1), sym(TokenKind::RParen), sym(TokenKind::RParen), eof()]).unwrap();
assert!(matches!(expr, Expr::Call { arguments, .. } if arguments.len() == 1));
}

#[test]
fn parse_call_combo_call_then_multiply() {
assert!(parse_expr(vec![ident("f"), sym(TokenKind::LParen), int_lit(1), sym(TokenKind::RParen), sym(TokenKind::Star), int_lit(2), eof()]).is_ok());
}

#[test]
fn parse_call_combo_multiply_then_call() {
assert!(parse_expr(vec![ident("f"), sym(TokenKind::Star), ident("g"), sym(TokenKind::LParen), int_lit(1), sym(TokenKind::RParen), eof()]).is_ok());
}

#[test]
fn parse_call_combo_call_with_binary_arg() {
assert!(parse_expr(vec![ident("f"), sym(TokenKind::LParen), int_lit(1), sym(TokenKind::Plus), int_lit(2), sym(TokenKind::RParen), eof()]).is_ok());
}

#[test]
fn parse_call_combo_call_with_grouped_arg() {
assert!(parse_expr(vec![ident("f"), sym(TokenKind::LParen), sym(TokenKind::LParen), int_lit(1), sym(TokenKind::Plus), int_lit(2), sym(TokenKind::RParen), sym(TokenKind::RParen), eof()]).is_ok());
}

#[test]
fn parse_statement_let_statement() {
let stmt = parse_stmt(vec![sym(TokenKind::Let), ident("x"), sym(TokenKind::Colon), ident("int"), sym(TokenKind::Eq), int_lit(1), sym(TokenKind::Semicolon), eof()]).unwrap();
assert!(matches!(stmt, Stmt::Let(_)));
}

#[test]
fn parse_statement_expr_statement() {
let stmt = parse_stmt(vec![ident("x"), sym(TokenKind::Semicolon), eof()]).unwrap();
assert!(matches!(stmt, Stmt::Expr(_)));
}

#[test]
fn parse_statement_return_without_value() {
let stmt = parse_stmt(vec![sym(TokenKind::Return), sym(TokenKind::Semicolon), eof()]).unwrap();
assert!(matches!(stmt, Stmt::Return(_)));
}

#[test]
fn parse_statement_return_with_value() {
let stmt = parse_stmt(vec![sym(TokenKind::Return), int_lit(1), sym(TokenKind::Semicolon), eof()]).unwrap();
assert!(matches!(stmt, Stmt::Return(_)));
}

#[test]
fn parse_statement_break_statement() {
let stmt = parse_stmt(vec![sym(TokenKind::Break), sym(TokenKind::Semicolon), eof()]).unwrap();
assert!(matches!(stmt, Stmt::Break));
}

#[test]
fn parse_statement_continue_statement() {
let stmt = parse_stmt(vec![sym(TokenKind::Continue), sym(TokenKind::Semicolon), eof()]).unwrap();
assert!(matches!(stmt, Stmt::Continue));
}

#[test]
fn parse_statement_asm_statement() {
let stmt = parse_stmt(vec![sym(TokenKind::Asm), sym(TokenKind::LParen), str_lit("nop"), sym(TokenKind::RParen), sym(TokenKind::Semicolon), eof()]).unwrap();
assert!(matches!(stmt, Stmt::Asm(_)));
}

#[test]
fn parse_statement_while_statement() {
let stmt = parse_stmt(vec![sym(TokenKind::While), sym(TokenKind::True), sym(TokenKind::LBrace), sym(TokenKind::RBrace), eof()]).unwrap();
assert!(matches!(stmt, Stmt::While(_)));
}

#[test]
fn parse_statement_loop_statement() {
let stmt = parse_stmt(vec![sym(TokenKind::Loop), sym(TokenKind::LBrace), sym(TokenKind::RBrace), eof()]).unwrap();
assert!(matches!(stmt, Stmt::Loop(_)));
}

#[test]
fn parse_statement_for_statement() {
let stmt = parse_stmt(vec![sym(TokenKind::For), ident("i"), sym(TokenKind::In), ident("xs"), sym(TokenKind::LBrace), sym(TokenKind::RBrace), eof()]).unwrap();
assert!(matches!(stmt, Stmt::For(_)));
}

#[test]
fn parse_statement_simple_if() {
let stmt = parse_stmt(vec![sym(TokenKind::If), sym(TokenKind::True), sym(TokenKind::LBrace), sym(TokenKind::RBrace), eof()]).unwrap();
assert!(matches!(stmt, Stmt::If(_)));
}

#[test]
fn parse_statement_if_else_block() {
let stmt = parse_stmt(vec![sym(TokenKind::If), sym(TokenKind::True), sym(TokenKind::LBrace), sym(TokenKind::RBrace), sym(TokenKind::Else), sym(TokenKind::LBrace), sym(TokenKind::RBrace), eof()]).unwrap();
assert!(matches!(stmt, Stmt::If(_)));
}

#[test]
fn parse_statement_if_else_if() {
let stmt = parse_stmt(vec![sym(TokenKind::If), sym(TokenKind::True), sym(TokenKind::LBrace), sym(TokenKind::RBrace), sym(TokenKind::Else), sym(TokenKind::If), sym(TokenKind::False), sym(TokenKind::LBrace), sym(TokenKind::RBrace), eof()]).unwrap();
assert!(matches!(stmt, Stmt::If(_)));
}

#[test]
fn parse_statement_error_let_missing_semicolon() {
assert!(parse_stmt(vec![sym(TokenKind::Let), ident("x"), sym(TokenKind::Colon), ident("int"), sym(TokenKind::Eq), int_lit(1), eof()]).is_err());
}

#[test]
fn parse_statement_error_return_missing_semicolon() {
assert!(parse_stmt(vec![sym(TokenKind::Return), int_lit(1), eof()]).is_err());
}

#[test]
fn parse_statement_error_break_missing_semicolon() {
assert!(parse_stmt(vec![sym(TokenKind::Break), eof()]).is_err());
}

#[test]
fn parse_statement_error_continue_missing_semicolon() {
assert!(parse_stmt(vec![sym(TokenKind::Continue), eof()]).is_err());
}

#[test]
fn parse_statement_error_asm_missing_paren() {
assert!(parse_stmt(vec![sym(TokenKind::Asm), sym(TokenKind::Semicolon), eof()]).is_err());
}

#[test]
fn parse_top_level_import() {
let items = parser(vec![sym(TokenKind::Import), ident("std"), sym(TokenKind::Dot), ident("io"), sym(TokenKind::Semicolon), eof()]).parse().unwrap();
assert_eq!(items.len(), 1);
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn parse_top_level_variant() {
let items = parser(vec![sym(TokenKind::Variant), ident("Color"), sym(TokenKind::LBrace), ident("Red"), sym(TokenKind::Comma), ident("Green"), sym(TokenKind::Comma), ident("Blue"), sym(TokenKind::RBrace), eof()]).parse().unwrap();
assert_eq!(items.len(), 1);
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn parse_top_level_trait() {
let items = parser(vec![sym(TokenKind::Trait), ident("Printable"), sym(TokenKind::LBrace), sym(TokenKind::Fn), ident("print"), sym(TokenKind::LParen), sym(TokenKind::RParen), sym(TokenKind::Semicolon), sym(TokenKind::RBrace), eof()]).parse().unwrap();
assert_eq!(items.len(), 1);
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn parse_top_level_struct() {
let items = parser(vec![sym(TokenKind::Struct), ident("Point"), sym(TokenKind::LBrace), ident("x"), sym(TokenKind::Colon), ident("int"), sym(TokenKind::Comma), ident("y"), sym(TokenKind::Colon), ident("int"), sym(TokenKind::RBrace), eof()]).parse().unwrap();
assert_eq!(items.len(), 1);
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn parse_top_level_function() {
let items = parser(vec![sym(TokenKind::Fn), ident("f"), sym(TokenKind::LParen), sym(TokenKind::RParen), ident("int"), sym(TokenKind::LBrace), sym(TokenKind::RBrace), eof()]).parse().unwrap();
assert_eq!(items.len(), 1);
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_top_level_impl() {
let items = parser(vec![sym(TokenKind::Impl), ident("Printable"), sym(TokenKind::For), ident("Point"), sym(TokenKind::LBrace), sym(TokenKind::Fn), ident("print"), sym(TokenKind::LParen), sym(TokenKind::RParen), sym(TokenKind::LBrace), sym(TokenKind::RBrace), sym(TokenKind::RBrace), eof()]).parse().unwrap();
assert_eq!(items.len(), 1);
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn parse_top_level_error_0() {
assert!(parser(vec![ident("x"), eof()]).parse().is_err());
}

#[test]
fn parse_top_level_error_1() {
assert!(parser(vec![sym(TokenKind::Let), ident("x"), eof()]).parse().is_err());
}

#[test]
fn parse_top_level_error_2() {
assert!(parser(vec![sym(TokenKind::Import), ident("std"), eof()]).parse().is_err());
}

#[test]
fn parse_top_level_error_3() {
assert!(parser(vec![sym(TokenKind::Semicolon), ident("x"), eof()]).parse().is_err());
}

#[test]
fn parse_generics_function_generics() {
assert!(parser(vec![sym(TokenKind::Fn), ident("f"), sym(TokenKind::Lt), ident("T"), sym(TokenKind::Gt), sym(TokenKind::LParen), sym(TokenKind::RParen), ident("int"), sym(TokenKind::LBrace), sym(TokenKind::RBrace), eof()]).parse().is_ok());
}

#[test]
fn parse_generics_function_multiple_generics() {
assert!(parser(vec![sym(TokenKind::Fn), ident("f"), sym(TokenKind::Lt), ident("T"), sym(TokenKind::Comma), ident("U"), sym(TokenKind::Gt), sym(TokenKind::LParen), sym(TokenKind::RParen), ident("int"), sym(TokenKind::LBrace), sym(TokenKind::RBrace), eof()]).parse().is_ok());
}

#[test]
fn parse_generics_trait_generics() {
assert!(parser(vec![sym(TokenKind::Trait), ident("Show"), sym(TokenKind::Lt), ident("T"), sym(TokenKind::Colon), ident("Display"), sym(TokenKind::Plus), ident("Debug"), sym(TokenKind::Gt), sym(TokenKind::LBrace), sym(TokenKind::Fn), ident("show"), sym(TokenKind::LParen), sym(TokenKind::RParen), sym(TokenKind::Semicolon), sym(TokenKind::RBrace), eof()]).parse().is_ok());
}

#[test]
fn parse_generics_struct_generics() {
assert!(parser(vec![sym(TokenKind::Struct), ident("Boxed"), sym(TokenKind::Lt), ident("T"), sym(TokenKind::Gt), sym(TokenKind::LBrace), sym(TokenKind::RBrace), eof()]).parse().is_ok());
}

#[test]
fn parse_generics_impl_generics_and_args() {
assert!(parser(vec![sym(TokenKind::Impl), sym(TokenKind::Lt), ident("T"), sym(TokenKind::Gt), ident("Trait"), sym(TokenKind::Lt), ident("T"), sym(TokenKind::Gt), sym(TokenKind::For), ident("Thing"), sym(TokenKind::Lt), ident("T"), sym(TokenKind::Gt), sym(TokenKind::LBrace), sym(TokenKind::RBrace), eof()]).parse().is_ok());
}

#[test]
fn parse_many_function_defs_1() {
let name = format!("f{}", 1);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_2() {
let name = format!("f{}", 2);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_3() {
let name = format!("f{}", 3);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_4() {
let name = format!("f{}", 4);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_5() {
let name = format!("f{}", 5);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_6() {
let name = format!("f{}", 6);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_7() {
let name = format!("f{}", 7);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_8() {
let name = format!("f{}", 8);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_9() {
let name = format!("f{}", 9);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_10() {
let name = format!("f{}", 10);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_11() {
let name = format!("f{}", 11);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_12() {
let name = format!("f{}", 12);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_13() {
let name = format!("f{}", 13);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_14() {
let name = format!("f{}", 14);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_15() {
let name = format!("f{}", 15);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_16() {
let name = format!("f{}", 16);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_17() {
let name = format!("f{}", 17);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_18() {
let name = format!("f{}", 18);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_19() {
let name = format!("f{}", 19);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_20() {
let name = format!("f{}", 20);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_21() {
let name = format!("f{}", 21);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_22() {
let name = format!("f{}", 22);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_23() {
let name = format!("f{}", 23);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_24() {
let name = format!("f{}", 24);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_25() {
let name = format!("f{}", 25);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_26() {
let name = format!("f{}", 26);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_27() {
let name = format!("f{}", 27);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_28() {
let name = format!("f{}", 28);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_29() {
let name = format!("f{}", 29);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn parse_many_function_defs_30() {
let name = format!("f{}", 30);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn binary_plus_int_int() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::Plus),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Add, .. }));
}

#[test]
fn binary_plus_int_float() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::Plus),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Add, .. }));
}

#[test]
fn binary_plus_float_int() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::Plus),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Add, .. }));
}

#[test]
fn binary_plus_float_float() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::Plus),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Add, .. }));
}

#[test]
fn binary_plus_true_int() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::Plus),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Add, .. }));
}

#[test]
fn binary_plus_true_float() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::Plus),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Add, .. }));
}

#[test]
fn binary_plus_false_int() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::Plus),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Add, .. }));
}

#[test]
fn binary_plus_false_float() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::Plus),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Add, .. }));
}

#[test]
fn binary_minus_int_int() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::Minus),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Sub, .. }));
}

#[test]
fn binary_minus_int_float() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::Minus),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Sub, .. }));
}

#[test]
fn binary_minus_float_int() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::Minus),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Sub, .. }));
}

#[test]
fn binary_minus_float_float() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::Minus),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Sub, .. }));
}

#[test]
fn binary_minus_true_int() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::Minus),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Sub, .. }));
}

#[test]
fn binary_minus_true_float() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::Minus),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Sub, .. }));
}

#[test]
fn binary_minus_false_int() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::Minus),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Sub, .. }));
}

#[test]
fn binary_minus_false_float() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::Minus),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Sub, .. }));
}

#[test]
fn binary_star_int_int() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::Star),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mul, .. }));
}

#[test]
fn binary_star_int_float() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::Star),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mul, .. }));
}

#[test]
fn binary_star_float_int() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::Star),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mul, .. }));
}

#[test]
fn binary_star_float_float() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::Star),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mul, .. }));
}

#[test]
fn binary_star_true_int() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::Star),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mul, .. }));
}

#[test]
fn binary_star_true_float() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::Star),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mul, .. }));
}

#[test]
fn binary_star_false_int() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::Star),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mul, .. }));
}

#[test]
fn binary_star_false_float() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::Star),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mul, .. }));
}

#[test]
fn binary_slash_int_int() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::Slash),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Div, .. }));
}

#[test]
fn binary_slash_int_float() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::Slash),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Div, .. }));
}

#[test]
fn binary_slash_float_int() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::Slash),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Div, .. }));
}

#[test]
fn binary_slash_float_float() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::Slash),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Div, .. }));
}

#[test]
fn binary_slash_true_int() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::Slash),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Div, .. }));
}

#[test]
fn binary_slash_true_float() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::Slash),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Div, .. }));
}

#[test]
fn binary_slash_false_int() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::Slash),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Div, .. }));
}

#[test]
fn binary_slash_false_float() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::Slash),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Div, .. }));
}

#[test]
fn binary_percent_int_int() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::Percent),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mod, .. }));
}

#[test]
fn binary_percent_int_float() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::Percent),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mod, .. }));
}

#[test]
fn binary_percent_float_int() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::Percent),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mod, .. }));
}

#[test]
fn binary_percent_float_float() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::Percent),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mod, .. }));
}

#[test]
fn binary_percent_true_int() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::Percent),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mod, .. }));
}

#[test]
fn binary_percent_true_float() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::Percent),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mod, .. }));
}

#[test]
fn binary_percent_false_int() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::Percent),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mod, .. }));
}

#[test]
fn binary_percent_false_float() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::Percent),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Mod, .. }));
}

#[test]
fn binary_lt_int_int() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::Lt),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Lt, .. }));
}

#[test]
fn binary_lt_int_float() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::Lt),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Lt, .. }));
}

#[test]
fn binary_lt_float_int() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::Lt),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Lt, .. }));
}

#[test]
fn binary_lt_float_float() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::Lt),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Lt, .. }));
}

#[test]
fn binary_lt_true_int() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::Lt),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Lt, .. }));
}

#[test]
fn binary_lt_true_float() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::Lt),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Lt, .. }));
}

#[test]
fn binary_lt_false_int() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::Lt),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Lt, .. }));
}

#[test]
fn binary_lt_false_float() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::Lt),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Lt, .. }));
}

#[test]
fn binary_le_int_int() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::LtEq),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Le, .. }));
}

#[test]
fn binary_le_int_float() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::LtEq),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Le, .. }));
}

#[test]
fn binary_le_float_int() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::LtEq),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Le, .. }));
}

#[test]
fn binary_le_float_float() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::LtEq),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Le, .. }));
}

#[test]
fn binary_le_true_int() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::LtEq),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Le, .. }));
}

#[test]
fn binary_le_true_float() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::LtEq),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Le, .. }));
}

#[test]
fn binary_le_false_int() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::LtEq),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Le, .. }));
}

#[test]
fn binary_le_false_float() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::LtEq),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Le, .. }));
}

#[test]
fn binary_gt_int_int() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::Gt),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Gt, .. }));
}

#[test]
fn binary_gt_int_float() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::Gt),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Gt, .. }));
}

#[test]
fn binary_gt_float_int() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::Gt),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Gt, .. }));
}

#[test]
fn binary_gt_float_float() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::Gt),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Gt, .. }));
}

#[test]
fn binary_gt_true_int() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::Gt),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Gt, .. }));
}

#[test]
fn binary_gt_true_float() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::Gt),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Gt, .. }));
}

#[test]
fn binary_gt_false_int() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::Gt),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Gt, .. }));
}

#[test]
fn binary_gt_false_float() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::Gt),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Gt, .. }));
}

#[test]
fn binary_ge_int_int() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::GtEq),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Ge, .. }));
}

#[test]
fn binary_ge_int_float() {
let expr = parse_expr(vec![
    int_lit(1),
    sym(TokenKind::GtEq),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Ge, .. }));
}

#[test]
fn binary_ge_float_int() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::GtEq),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Ge, .. }));
}

#[test]
fn binary_ge_float_float() {
let expr = parse_expr(vec![
    float_lit(1.5),
    sym(TokenKind::GtEq),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Ge, .. }));
}

#[test]
fn binary_ge_true_int() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::GtEq),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Ge, .. }));
}

#[test]
fn binary_ge_true_float() {
let expr = parse_expr(vec![
    sym(TokenKind::True),
    sym(TokenKind::GtEq),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Ge, .. }));
}

#[test]
fn binary_ge_false_int() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::GtEq),
    int_lit(1),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Ge, .. }));
}

#[test]
fn binary_ge_false_float() {
let expr = parse_expr(vec![
    sym(TokenKind::False),
    sym(TokenKind::GtEq),
    float_lit(1.5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { binary_op: BinaryOp::Ge, .. }));
}

#[test]
fn parse_expr_edge_nested_parentheses() {
assert!(parse_expr(vec![sym(TokenKind::LParen), sym(TokenKind::LParen), int_lit(1), sym(TokenKind::Plus), int_lit(2), sym(TokenKind::RParen), sym(TokenKind::RParen), eof()]).is_ok());
}

#[test]
fn parse_expr_edge_unary_then_binary() {
assert!(parse_expr(vec![sym(TokenKind::Minus), int_lit(1), sym(TokenKind::Plus), int_lit(2), eof()]).is_ok());
}

#[test]
fn parse_expr_edge_binary_then_unary() {
assert!(parse_expr(vec![int_lit(1), sym(TokenKind::Plus), sym(TokenKind::Minus), int_lit(2), eof()]).is_ok());
}

#[test]
fn parse_expr_edge_call_in_binary() {
assert!(parse_expr(vec![ident("f"), sym(TokenKind::LParen), int_lit(1), sym(TokenKind::RParen), sym(TokenKind::Plus), int_lit(2), eof()]).is_ok());
}

#[test]
fn parse_expr_edge_binary_in_call_arg() {
assert!(parse_expr(vec![ident("f"), sym(TokenKind::LParen), int_lit(1), sym(TokenKind::Plus), int_lit(2), sym(TokenKind::RParen), eof()]).is_ok());
}

#[test]
fn parse_expr_edge_list_in_call_arg() {
assert!(parse_expr(vec![ident("f"), sym(TokenKind::LParen), sym(TokenKind::LBracket), int_lit(1), sym(TokenKind::RBracket), sym(TokenKind::RParen), eof()]).is_ok());
}

#[test]
fn parse_expr_error_empty_input() {
assert!(parse_expr(vec![eof()]).is_err());
}

#[test]
fn parse_expr_error_just_operator() {
assert!(parse_expr(vec![sym(TokenKind::Plus), eof()]).is_err());
}

#[test]
fn parse_expr_error_missing_rparen() {
assert!(parse_expr(vec![sym(TokenKind::LParen), int_lit(1), eof()]).is_err());
}

#[test]
fn parse_expr_error_missing_rbracket() {
assert!(parse_expr(vec![sym(TokenKind::LBracket), int_lit(1), eof()]).is_err());
}

#[test]
fn parse_expr_error_missing_rbrace_struct() {
assert!(parse_expr(vec![ident("Point"), sym(TokenKind::LBrace), ident("x"), sym(TokenKind::Colon), int_lit(1), eof()]).is_err());
}

#[test]
fn parse_expr_error_missing_rhs_assignment() {
assert!(parse_expr(vec![ident("x"), sym(TokenKind::Eq), eof()]).is_err());
}

#[test]
fn parse_statement_edge_expr_stmt_call() {
assert!(parse_stmt(vec![ident("f"), sym(TokenKind::LParen), sym(TokenKind::RParen), sym(TokenKind::Semicolon), eof()]).is_ok());
}

#[test]
fn parse_statement_edge_expr_stmt_group() {
assert!(parse_stmt(vec![sym(TokenKind::LParen), int_lit(1), sym(TokenKind::RParen), sym(TokenKind::Semicolon), eof()]).is_ok());
}

#[test]
fn parse_statement_edge_if_with_return() {
assert!(parse_stmt(vec![sym(TokenKind::If), sym(TokenKind::True), sym(TokenKind::LBrace), sym(TokenKind::Return), sym(TokenKind::Semicolon), sym(TokenKind::RBrace), eof()]).is_ok());
}

#[test]
fn parse_statement_edge_while_with_break() {
assert!(parse_stmt(vec![sym(TokenKind::While), sym(TokenKind::True), sym(TokenKind::LBrace), sym(TokenKind::Break), sym(TokenKind::Semicolon), sym(TokenKind::RBrace), eof()]).is_ok());
}

#[test]
fn parse_statement_edge_for_with_continue() {
assert!(parse_stmt(vec![sym(TokenKind::For), ident("x"), sym(TokenKind::In), ident("xs"), sym(TokenKind::LBrace), sym(TokenKind::Continue), sym(TokenKind::Semicolon), sym(TokenKind::RBrace), eof()]).is_ok());
}

#[test]
fn repeated_import_variants_1() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module1"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_2() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module2"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_3() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module3"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_4() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module4"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_5() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module5"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_6() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module6"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_7() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module7"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_8() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module8"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_9() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module9"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_10() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module10"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_11() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module11"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_12() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module12"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_13() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module13"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_14() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module14"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_15() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module15"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_16() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module16"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_17() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module17"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_18() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module18"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_19() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module19"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_20() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module20"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_21() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module21"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_22() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module22"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_23() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module23"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_24() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module24"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_25() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module25"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_26() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module26"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_27() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module27"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_28() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module28"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_29() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module29"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_30() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module30"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_31() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module31"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_32() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module32"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_33() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module33"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_34() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module34"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_35() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module35"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_36() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module36"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_37() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module37"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_38() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module38"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_39() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module39"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_import_variants_40() {
let items = parser(vec![
    sym(TokenKind::Import),
    ident("pkg"),
    sym(TokenKind::Dot),
    ident("module40"),
    sym(TokenKind::Semicolon),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Import(_)));
}

#[test]
fn repeated_variant_cases_1() {
let name = format!("Variant{}", 1);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_2() {
let name = format!("Variant{}", 2);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_3() {
let name = format!("Variant{}", 3);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_4() {
let name = format!("Variant{}", 4);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_5() {
let name = format!("Variant{}", 5);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_6() {
let name = format!("Variant{}", 6);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_7() {
let name = format!("Variant{}", 7);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_8() {
let name = format!("Variant{}", 8);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_9() {
let name = format!("Variant{}", 9);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_10() {
let name = format!("Variant{}", 10);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_11() {
let name = format!("Variant{}", 11);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_12() {
let name = format!("Variant{}", 12);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_13() {
let name = format!("Variant{}", 13);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_14() {
let name = format!("Variant{}", 14);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_15() {
let name = format!("Variant{}", 15);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_16() {
let name = format!("Variant{}", 16);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_17() {
let name = format!("Variant{}", 17);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_18() {
let name = format!("Variant{}", 18);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_19() {
let name = format!("Variant{}", 19);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_20() {
let name = format!("Variant{}", 20);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_21() {
let name = format!("Variant{}", 21);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_22() {
let name = format!("Variant{}", 22);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_23() {
let name = format!("Variant{}", 23);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_24() {
let name = format!("Variant{}", 24);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_25() {
let name = format!("Variant{}", 25);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_26() {
let name = format!("Variant{}", 26);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_27() {
let name = format!("Variant{}", 27);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_28() {
let name = format!("Variant{}", 28);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_29() {
let name = format!("Variant{}", 29);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_variant_cases_30() {
let name = format!("Variant{}", 30);
let items = parser(vec![
    sym(TokenKind::Variant),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("A"),
    sym(TokenKind::Comma),
    ident("B"),
    sym(TokenKind::Comma),
    ident("C"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Variant(_)));
}

#[test]
fn repeated_struct_cases_1() {
let name = format!("S{}", 1);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_2() {
let name = format!("S{}", 2);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_3() {
let name = format!("S{}", 3);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_4() {
let name = format!("S{}", 4);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_5() {
let name = format!("S{}", 5);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_6() {
let name = format!("S{}", 6);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_7() {
let name = format!("S{}", 7);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_8() {
let name = format!("S{}", 8);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_9() {
let name = format!("S{}", 9);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_10() {
let name = format!("S{}", 10);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_11() {
let name = format!("S{}", 11);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_12() {
let name = format!("S{}", 12);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_13() {
let name = format!("S{}", 13);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_14() {
let name = format!("S{}", 14);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_15() {
let name = format!("S{}", 15);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_16() {
let name = format!("S{}", 16);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_17() {
let name = format!("S{}", 17);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_18() {
let name = format!("S{}", 18);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_19() {
let name = format!("S{}", 19);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_20() {
let name = format!("S{}", 20);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_21() {
let name = format!("S{}", 21);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_22() {
let name = format!("S{}", 22);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_23() {
let name = format!("S{}", 23);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_24() {
let name = format!("S{}", 24);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_25() {
let name = format!("S{}", 25);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_26() {
let name = format!("S{}", 26);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_27() {
let name = format!("S{}", 27);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_28() {
let name = format!("S{}", 28);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_29() {
let name = format!("S{}", 29);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_struct_cases_30() {
let name = format!("S{}", 30);
let items = parser(vec![
    sym(TokenKind::Struct),
    ident(&name),
    sym(TokenKind::LBrace),
    ident("x"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::Comma),
    ident("y"),
    sym(TokenKind::Colon),
    ident("int"),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Struct(_)));
}

#[test]
fn repeated_trait_cases_1() {
let name = format!("T{}", 1);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_2() {
let name = format!("T{}", 2);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_3() {
let name = format!("T{}", 3);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_4() {
let name = format!("T{}", 4);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_5() {
let name = format!("T{}", 5);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_6() {
let name = format!("T{}", 6);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_7() {
let name = format!("T{}", 7);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_8() {
let name = format!("T{}", 8);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_9() {
let name = format!("T{}", 9);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_10() {
let name = format!("T{}", 10);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_11() {
let name = format!("T{}", 11);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_12() {
let name = format!("T{}", 12);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_13() {
let name = format!("T{}", 13);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_14() {
let name = format!("T{}", 14);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_15() {
let name = format!("T{}", 15);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_16() {
let name = format!("T{}", 16);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_17() {
let name = format!("T{}", 17);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_18() {
let name = format!("T{}", 18);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_19() {
let name = format!("T{}", 19);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_20() {
let name = format!("T{}", 20);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_21() {
let name = format!("T{}", 21);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_22() {
let name = format!("T{}", 22);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_23() {
let name = format!("T{}", 23);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_24() {
let name = format!("T{}", 24);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_25() {
let name = format!("T{}", 25);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_26() {
let name = format!("T{}", 26);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_27() {
let name = format!("T{}", 27);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_28() {
let name = format!("T{}", 28);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_29() {
let name = format!("T{}", 29);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_trait_cases_30() {
let name = format!("T{}", 30);
let items = parser(vec![
    sym(TokenKind::Trait),
    ident(&name),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::Semicolon),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Trait(_)));
}

#[test]
fn repeated_fn_cases_1() {
let name = format!("f{}", 1);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_2() {
let name = format!("f{}", 2);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_3() {
let name = format!("f{}", 3);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_4() {
let name = format!("f{}", 4);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_5() {
let name = format!("f{}", 5);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_6() {
let name = format!("f{}", 6);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_7() {
let name = format!("f{}", 7);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_8() {
let name = format!("f{}", 8);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_9() {
let name = format!("f{}", 9);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_10() {
let name = format!("f{}", 10);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_11() {
let name = format!("f{}", 11);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_12() {
let name = format!("f{}", 12);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_13() {
let name = format!("f{}", 13);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_14() {
let name = format!("f{}", 14);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_15() {
let name = format!("f{}", 15);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_16() {
let name = format!("f{}", 16);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_17() {
let name = format!("f{}", 17);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_18() {
let name = format!("f{}", 18);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_19() {
let name = format!("f{}", 19);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_20() {
let name = format!("f{}", 20);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_21() {
let name = format!("f{}", 21);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_22() {
let name = format!("f{}", 22);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_23() {
let name = format!("f{}", 23);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_24() {
let name = format!("f{}", 24);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_25() {
let name = format!("f{}", 25);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_26() {
let name = format!("f{}", 26);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_27() {
let name = format!("f{}", 27);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_28() {
let name = format!("f{}", 28);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_29() {
let name = format!("f{}", 29);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_fn_cases_30() {
let name = format!("f{}", 30);
let items = parser(vec![
    sym(TokenKind::Fn),
    ident(&name),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    ident("int"),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::Function(_)));
}

#[test]
fn repeated_impl_cases_1() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing1"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_2() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing2"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_3() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing3"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_4() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing4"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_5() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing5"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_6() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing6"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_7() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing7"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_8() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing8"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_9() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing9"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_10() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing10"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_11() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing11"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_12() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing12"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_13() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing13"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_14() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing14"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_15() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing15"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_16() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing16"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_17() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing17"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_18() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing18"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_19() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing19"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_20() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing20"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_21() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing21"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_22() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing22"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_23() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing23"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_24() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing24"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_25() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing25"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_26() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing26"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_27() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing27"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_28() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing28"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_29() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing29"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn repeated_impl_cases_30() {
let items = parser(vec![
    sym(TokenKind::Impl),
    ident("Trait"),
    sym(TokenKind::For),
    ident("Thing30"),
    sym(TokenKind::LBrace),
    sym(TokenKind::Fn),
    ident("f"),
    sym(TokenKind::LParen),
    sym(TokenKind::RParen),
    sym(TokenKind::LBrace),
    sym(TokenKind::RBrace),
    sym(TokenKind::RBrace),
    eof(),
]).parse().unwrap();
assert!(matches!(&items[0], Item::TraitImplementation(_)));
}

#[test]
fn malformed_top_level_0() {
assert!(parser(vec![sym(TokenKind::Fn), ident("f"), sym(TokenKind::LParen), sym(TokenKind::RParen), sym(TokenKind::LBrace), sym(TokenKind::RBrace), eof()]).parse().is_err());
}

#[test]
fn malformed_top_level_1() {
assert!(parser(vec![sym(TokenKind::Struct), ident("S"), sym(TokenKind::LBrace), ident("x"), sym(TokenKind::Colon), ident("int"), eof()]).parse().is_err());
}

#[test]
fn malformed_top_level_2() {
assert!(parser(vec![sym(TokenKind::Trait), ident("T"), sym(TokenKind::LBrace), sym(TokenKind::Fn), ident("f"), sym(TokenKind::LParen), sym(TokenKind::RParen), eof()]).parse().is_err());
}

#[test]
fn malformed_top_level_3() {
assert!(parser(vec![sym(TokenKind::Impl), ident("Trait"), sym(TokenKind::For), ident("Thing"), sym(TokenKind::LBrace), eof()]).parse().is_err());
}

#[test]
fn mixed_precedence_chain_1() {
assert!(parse_expr(vec![int_lit(1), sym(TokenKind::Plus), int_lit(2), sym(TokenKind::Star), int_lit(3), sym(TokenKind::Minus), int_lit(4), eof()]).is_ok());
}

#[test]
fn mixed_precedence_chain_2() {
assert!(parse_expr(vec![int_lit(1), sym(TokenKind::Star), int_lit(2), sym(TokenKind::Plus), int_lit(3), sym(TokenKind::Slash), int_lit(4), eof()]).is_ok());
}

#[test]
fn mixed_precedence_chain_3() {
assert!(parse_expr(vec![int_lit(1), sym(TokenKind::EqEq), int_lit(2), sym(TokenKind::Or), int_lit(3), sym(TokenKind::And), int_lit(4), eof()]).is_ok());
}

#[test]
fn mixed_precedence_chain_4() {
assert!(parse_expr(vec![int_lit(1), sym(TokenKind::Pipe), int_lit(2), sym(TokenKind::Caret), int_lit(3), sym(TokenKind::Amp), int_lit(4), eof()]).is_ok());
}

#[test]
fn mixed_precedence_chain_5() {
assert!(parse_expr(vec![int_lit(1), sym(TokenKind::LtLt), int_lit(2), sym(TokenKind::Plus), int_lit(3), sym(TokenKind::GtGt), int_lit(4), eof()]).is_ok());
}

#[test]
fn filler_expr_chain_1() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(1),
    sym(TokenKind::Plus),
    int_lit(2),
    sym(TokenKind::Comma),
    int_lit(3),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(4),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_2() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(2),
    sym(TokenKind::Plus),
    int_lit(3),
    sym(TokenKind::Comma),
    int_lit(4),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(5),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_3() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(3),
    sym(TokenKind::Plus),
    int_lit(4),
    sym(TokenKind::Comma),
    int_lit(5),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(6),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_4() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(4),
    sym(TokenKind::Plus),
    int_lit(5),
    sym(TokenKind::Comma),
    int_lit(6),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(7),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_5() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(5),
    sym(TokenKind::Plus),
    int_lit(6),
    sym(TokenKind::Comma),
    int_lit(7),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(8),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_6() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(6),
    sym(TokenKind::Plus),
    int_lit(7),
    sym(TokenKind::Comma),
    int_lit(8),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(9),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_7() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(7),
    sym(TokenKind::Plus),
    int_lit(8),
    sym(TokenKind::Comma),
    int_lit(9),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(10),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_8() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(8),
    sym(TokenKind::Plus),
    int_lit(9),
    sym(TokenKind::Comma),
    int_lit(10),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(11),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_9() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(9),
    sym(TokenKind::Plus),
    int_lit(10),
    sym(TokenKind::Comma),
    int_lit(11),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(12),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_10() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(10),
    sym(TokenKind::Plus),
    int_lit(11),
    sym(TokenKind::Comma),
    int_lit(12),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(13),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_11() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(11),
    sym(TokenKind::Plus),
    int_lit(12),
    sym(TokenKind::Comma),
    int_lit(13),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(14),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_12() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(12),
    sym(TokenKind::Plus),
    int_lit(13),
    sym(TokenKind::Comma),
    int_lit(14),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(15),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_13() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(13),
    sym(TokenKind::Plus),
    int_lit(14),
    sym(TokenKind::Comma),
    int_lit(15),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(16),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_14() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(14),
    sym(TokenKind::Plus),
    int_lit(15),
    sym(TokenKind::Comma),
    int_lit(16),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(17),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_15() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(15),
    sym(TokenKind::Plus),
    int_lit(16),
    sym(TokenKind::Comma),
    int_lit(17),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(18),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_16() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(16),
    sym(TokenKind::Plus),
    int_lit(17),
    sym(TokenKind::Comma),
    int_lit(18),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(19),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_17() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(17),
    sym(TokenKind::Plus),
    int_lit(18),
    sym(TokenKind::Comma),
    int_lit(19),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(20),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_18() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(18),
    sym(TokenKind::Plus),
    int_lit(19),
    sym(TokenKind::Comma),
    int_lit(20),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(21),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_19() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(19),
    sym(TokenKind::Plus),
    int_lit(20),
    sym(TokenKind::Comma),
    int_lit(21),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(22),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_20() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(20),
    sym(TokenKind::Plus),
    int_lit(21),
    sym(TokenKind::Comma),
    int_lit(22),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(23),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_21() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(21),
    sym(TokenKind::Plus),
    int_lit(22),
    sym(TokenKind::Comma),
    int_lit(23),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(24),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_22() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(22),
    sym(TokenKind::Plus),
    int_lit(23),
    sym(TokenKind::Comma),
    int_lit(24),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(25),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_23() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(23),
    sym(TokenKind::Plus),
    int_lit(24),
    sym(TokenKind::Comma),
    int_lit(25),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(26),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_24() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(24),
    sym(TokenKind::Plus),
    int_lit(25),
    sym(TokenKind::Comma),
    int_lit(26),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(27),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}

#[test]
fn filler_expr_chain_25() {
let expr = parse_expr(vec![
    ident("f"),
    sym(TokenKind::LParen),
    int_lit(25),
    sym(TokenKind::Plus),
    int_lit(26),
    sym(TokenKind::Comma),
    int_lit(27),
    sym(TokenKind::RParen),
    sym(TokenKind::Star),
    int_lit(28),
    eof(),
]).unwrap();
assert!(matches!(expr, Expr::BinaryOp { .. } | Expr::Call { .. }));
}
