// use super::ast::*;
// use crate::lexer::{Lexer, Token, TokenKind};
// use crate::parser::{ParseError, Parser};

use rython_to_ir::ast::*;
use rython_to_ir::lexer::*;
use rython_to_ir::parser::*;

fn tk(kind: TokenKind, value: &str) -> Token {
    Token::new(kind, value.to_string(), 0, 0)
}

fn eof() -> Token {
    tk(TokenKind::Eof, "EOF")
}

fn toks(parts: Vec<Token>) -> Vec<Token> {
    let mut out = parts;
    out.push(eof());
    out
}

fn parse_expr_from(parts: Vec<Token>) -> Result<Expr, ParseError> {
    let mut p = Parser::new(toks(parts));
    p.parse_expr()
}

fn parse_stmt_from(parts: Vec<Token>) -> Result<Stmt, ParseError> {
    let mut p = Parser::new(toks(parts));
    p.parse_statement()
}

fn parse_items_from(parts: Vec<Token>) -> Result<Vec<Item>, ParseError> {
    let mut p = Parser::new(toks(parts));
    p.parse()
}

fn parse_expr_source(source: &str) -> Result<Expr, ParseError> {
    let mut p = Parser::new(Lexer::create_tokens(source.to_string()).expect("lexing failed"));
    p.parse_expr()
}

fn parse_stmt_source(source: &str) -> Result<Stmt, ParseError> {
    let mut p = Parser::new(Lexer::create_tokens(source.to_string()).expect("lexing failed"));
    p.parse_statement()
}

fn parse_items_source(source: &str) -> Result<Vec<Item>, ParseError> {
    let mut p = Parser::new(Lexer::create_tokens(source.to_string()).expect("lexing failed"));
    p.parse()
}

fn dbg_eq<T: std::fmt::Debug, U: std::fmt::Debug>(left: T, right: U) {
    assert_eq!(format!("{:#?}", left), format!("{:#?}", right));
}

fn assert_unexpected_token(err: ParseError, expected: TokenKind, found: TokenKind, idx: usize) {
    match err {
        ParseError::UnexpectedToken {
            expected: e,
            found: f,
            token_idx,
        } => {
            assert_eq!(e, expected);
            assert_eq!(f, found);
            assert_eq!(token_idx, idx);
        }
        other => panic!("expected UnexpectedToken, got {other:?}"),
    }
}

fn assert_unexpected_top_level(err: ParseError, found: TokenKind) {
    match err {
        ParseError::UnexpectedTopLevel { found: f, .. } => assert_eq!(f, found),
        other => panic!("expected UnexpectedTopLevel, got {other:?}"),
    }
}

fn assert_unexpected_expr_start(err: ParseError, found: TokenKind) {
    match err {
        ParseError::UnexpectedExprStart { found: f, .. } => assert_eq!(f, found),
        other => panic!("expected UnexpectedExprStart, got {other:?}"),
    }
}

fn assert_invalid_assignment_target(err: ParseError) {
    match err {
        ParseError::InvalidAssignmentTarget { .. } => {}
        other => panic!("expected InvalidAssignmentTarget, got {other:?}"),
    }
}

fn assert_unexpected_eof(err: ParseError) {
    match err {
        ParseError::UnexpectedEof { .. } => {}
        other => panic!("expected UnexpectedEof, got {other:?}"),
    }
}

fn tokens_eq_ignoring_span(actual: &[Token], expected: &[Token]) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected.iter())
            .all(|(a, b)| a.kind == b.kind && a.value == b.value)
}

macro_rules! lexer_case {
    ($name:ident, $input:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let tokens = Lexer::create_tokens($input.to_string()).expect("lexing failed");
            let expected: Vec<Token> = $expected;
            assert!(
                tokens_eq_ignoring_span(&tokens, &expected),
                "tokens differ\n  actual:   {:?}\n  expected: {:?}",
                tokens,
                expected
            );
        }
    };
}

macro_rules! expr_case {
    ($name:ident, $tokens:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let expr = parse_expr_from($tokens).unwrap();
            dbg_eq(expr, $expected);
        }
    };
}

macro_rules! stmt_case {
    ($name:ident, $tokens:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let stmt = parse_stmt_from($tokens).unwrap();
            dbg_eq(stmt, $expected);
        }
    };
}

macro_rules! item_case {
    ($name:ident, $tokens:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let items = parse_items_from($tokens).unwrap();
            dbg_eq(items, $expected);
        }
    };
}

macro_rules! error_case_expr {
    ($name:ident, $tokens:expr, $check:expr) => {
        #[test]
        fn $name() {
            let err = parse_expr_from($tokens).unwrap_err();
            $check(err);
        }
    };
}

macro_rules! error_case_stmt {
    ($name:ident, $tokens:expr, $check:expr) => {
        #[test]
        fn $name() {
            let err = parse_stmt_from($tokens).unwrap_err();
            $check(err);
        }
    };
}

macro_rules! error_case_items {
    ($name:ident, $tokens:expr, $check:expr) => {
        #[test]
        fn $name() {
            let err = parse_items_from($tokens).unwrap_err();
            $check(err);
        }
    };
}

#[test]
fn parser_starts_empty() {
    let p = Parser::new(vec![eof()]);
    assert_eq!(p.current_idx, 0);
}

#[test]
fn parser_current_and_peek_and_expect_current_have_exact_behavior() {
    let p = Parser::new(toks(vec![
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Plus, "+"),
    ]));
    assert_eq!(p.current().unwrap(), tk(TokenKind::Ident, "x"));
    assert_eq!(p.peek().unwrap(), tk(TokenKind::Plus, "+"));
    p.expect_current(TokenKind::Ident).unwrap();
}

#[test]
fn parser_peek_past_end_returns_eof_error() {
    let p = Parser::new(vec![tk(TokenKind::Ident, "x")]);
    assert_unexpected_eof(p.peek().unwrap_err());
}

#[test]
fn parser_advance_past_end_returns_eof_error() {
    let mut p = Parser::new(vec![tk(TokenKind::Ident, "x")]);
    assert_unexpected_eof(p.advance().unwrap_err());
}

#[test]
fn parser_expect_current_reports_exact_token() {
    let p = Parser::new(toks(vec![tk(TokenKind::Plus, "+")]));
    let err = p.expect_current(TokenKind::Minus).unwrap_err();
    assert_unexpected_token(err, TokenKind::Minus, TokenKind::Plus, 0);
}

// LEXER TESTS
lexer_case!(
    lexer_keyword_true,
    "true",
    vec![tk(TokenKind::True, "true"), eof()]
);
lexer_case!(
    lexer_keyword_false,
    "false",
    vec![tk(TokenKind::False, "false"), eof()]
);
lexer_case!(
    lexer_keyword_char,
    "char",
    vec![tk(TokenKind::Char, "char"), eof()]
);
lexer_case!(
    lexer_keyword_null,
    "null",
    vec![tk(TokenKind::Null, "null"), eof()]
);
lexer_case!(lexer_keyword_if, "if", vec![tk(TokenKind::If, "if"), eof()]);
lexer_case!(
    lexer_keyword_else,
    "else",
    vec![tk(TokenKind::Else, "else"), eof()]
);
lexer_case!(
    lexer_keyword_return,
    "return",
    vec![tk(TokenKind::Return, "return"), eof()]
);
lexer_case!(
    lexer_keyword_loop,
    "loop",
    vec![tk(TokenKind::Loop, "loop"), eof()]
);
lexer_case!(
    lexer_keyword_while,
    "while",
    vec![tk(TokenKind::While, "while"), eof()]
);
lexer_case!(
    lexer_keyword_any,
    "any",
    vec![tk(TokenKind::Any, "any"), eof()]
);
lexer_case!(
    lexer_keyword_let,
    "let",
    vec![tk(TokenKind::Let, "let"), eof()]
);
lexer_case!(lexer_keyword_fn, "fn", vec![tk(TokenKind::Fn, "fn"), eof()]);
lexer_case!(
    lexer_keyword_this,
    "this",
    vec![tk(TokenKind::This, "this"), eof()]
);
lexer_case!(lexer_keyword_in, "in", vec![tk(TokenKind::In, "in"), eof()]);
lexer_case!(
    lexer_keyword_import,
    "import",
    vec![tk(TokenKind::Import, "import"), eof()]
);
lexer_case!(
    lexer_keyword_struct,
    "struct",
    vec![tk(TokenKind::Struct, "struct"), eof()]
);
lexer_case!(
    lexer_keyword_trait,
    "trait",
    vec![tk(TokenKind::Trait, "trait"), eof()]
);
lexer_case!(
    lexer_keyword_global,
    "global",
    vec![tk(TokenKind::Global, "global"), eof()]
);
lexer_case!(
    lexer_keyword_const,
    "const",
    vec![tk(TokenKind::Const, "const"), eof()]
);
lexer_case!(
    lexer_keyword_impl,
    "impl",
    vec![tk(TokenKind::Impl, "impl"), eof()]
);
lexer_case!(
    lexer_keyword_for,
    "for",
    vec![tk(TokenKind::For, "for"), eof()]
);
lexer_case!(
    lexer_keyword_continue,
    "continue",
    vec![tk(TokenKind::Continue, "continue"), eof()]
);
lexer_case!(
    lexer_keyword_break,
    "break",
    vec![tk(TokenKind::Break, "break"), eof()]
);
lexer_case!(
    lexer_keyword_variant,
    "variant",
    vec![tk(TokenKind::Variant, "variant"), eof()]
);
lexer_case!(
    lexer_keyword_and,
    "and",
    vec![tk(TokenKind::And, "and"), eof()]
);
lexer_case!(lexer_keyword_or, "or", vec![tk(TokenKind::Or, "or"), eof()]);
lexer_case!(
    lexer_keyword_operator,
    "operator",
    vec![tk(TokenKind::Operator, "operator"), eof()]
);
lexer_case!(
    lexer_keyword_asm,
    "asm {}",
    vec![tk(TokenKind::Asm, ""), eof()]
);

lexer_case!(
    lexer_ident_with_underscore_and_digits,
    "abc_123",
    vec![tk(TokenKind::Ident, "abc_123"), eof()]
);
lexer_case!(
    lexer_ident_respects_keywords_only_exact_match,
    "truex falsey global_value const2 operator2",
    vec![
        tk(TokenKind::Ident, "truex"),
        tk(TokenKind::Ident, "falsey"),
        tk(TokenKind::Ident, "global_value"),
        tk(TokenKind::Ident, "const2"),
        tk(TokenKind::Ident, "operator2"),
        eof()
    ]
);

lexer_case!(lexer_int_zero, "0", vec![tk(TokenKind::Int, "0"), eof()]);
lexer_case!(
    lexer_int_large,
    "1234567890",
    vec![tk(TokenKind::Int, "1234567890"), eof()]
);
lexer_case!(
    lexer_float_simple,
    "1.5",
    vec![tk(TokenKind::Float, "1.5"), eof()]
);
lexer_case!(
    lexer_float_multiple_dots,
    "1.2.3",
    vec![
        tk(TokenKind::Float, "1.2"),
        tk(TokenKind::Dot, "."),
        tk(TokenKind::Int, "3"),
        eof()
    ]
);
lexer_case!(
    lexer_float_trailing_dot,
    "7.",
    vec![tk(TokenKind::Float, "7."), eof()]
);
lexer_case!(
    lexer_int_followed_by_ident_splits,
    "12abc",
    vec![tk(TokenKind::Int, "12"), tk(TokenKind::Ident, "abc"), eof()]
);

lexer_case!(
    lexer_string_empty,
    "\"\"",
    vec![tk(TokenKind::StringLiteral, ""), eof()]
);
lexer_case!(
    lexer_string_basic,
    "\"hello\"",
    vec![tk(TokenKind::StringLiteral, "hello"), eof()]
);
lexer_case!(
    lexer_string_escaped_quote,
    "\"a\\\"b\"",
    vec![tk(TokenKind::StringLiteral, "a\"b"), eof()]
);
lexer_case!(
    lexer_string_escaped_backslash,
    "\"a\\\\b\"",
    vec![tk(TokenKind::StringLiteral, "a\\b"), eof()]
);
lexer_case!(
    lexer_string_newline_escape,
    "\"a\\nb\"",
    vec![tk(TokenKind::StringLiteral, "a\nb"), eof()]
);
lexer_case!(
    lexer_string_tab_escape,
    "\"a\\tb\"",
    vec![tk(TokenKind::StringLiteral, "a\tb"), eof()]
);
lexer_case!(
    lexer_string_cr_escape,
    "\"a\\rb\"",
    vec![tk(TokenKind::StringLiteral, "a\rb"), eof()]
);
lexer_case!(
    lexer_string_unknown_escape_keeps_backslash,
    "\"a\\qb\"",
    vec![tk(TokenKind::StringLiteral, "a\\qb"), eof()]
);

#[test]
fn lexer_char_empty_errors() {
    assert!(Lexer::create_tokens("''".to_string()).is_err());
}
lexer_case!(
    lexer_char_basic,
    "'x'",
    vec![tk(TokenKind::Char, "x"), eof()]
);
lexer_case!(
    lexer_char_escaped_newline,
    "'\\n'",
    vec![tk(TokenKind::Char, "\n"), eof()]
);
lexer_case!(
    lexer_char_escaped_quote,
    "'\\''",
    vec![tk(TokenKind::Char, "'"), eof()]
);
lexer_case!(
    lexer_char_escaped_backslash,
    "'\\\\'",
    vec![tk(TokenKind::Char, "\\"), eof()]
);

lexer_case!(
    lexer_ignores_spaces_and_newlines_and_tabs,
    "  \n\tlet   x",
    vec![tk(TokenKind::Let, "let"), tk(TokenKind::Ident, "x"), eof()]
);
lexer_case!(
    lexer_ignores_line_comments,
    "let x // comment here\n return",
    vec![
        tk(TokenKind::Let, "let"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Return, "return"),
        eof()
    ]
);

lexer_case!(lexer_plus, "+", vec![tk(TokenKind::Plus, "+"), eof()]);
lexer_case!(lexer_minus, "-", vec![tk(TokenKind::Minus, "-"), eof()]);
lexer_case!(lexer_star, "*", vec![tk(TokenKind::Star, "*"), eof()]);
lexer_case!(lexer_slash, "/", vec![tk(TokenKind::Slash, "/"), eof()]);
lexer_case!(lexer_percent, "%", vec![tk(TokenKind::Percent, "%"), eof()]);
lexer_case!(lexer_eq, "=", vec![tk(TokenKind::Eq, "="), eof()]);
lexer_case!(lexer_eqeq, "==", vec![tk(TokenKind::EqEq, "=="), eof()]);
lexer_case!(lexer_bang, "!", vec![tk(TokenKind::Bang, "!"), eof()]);
lexer_case!(lexer_bangeq, "!=", vec![tk(TokenKind::BangEq, "!="), eof()]);
lexer_case!(lexer_lt, "<", vec![tk(TokenKind::Lt, "<"), eof()]);
lexer_case!(lexer_lteq, "<=", vec![tk(TokenKind::LtEq, "<="), eof()]);
lexer_case!(lexer_ltlt, "<<", vec![tk(TokenKind::LtLt, "<<"), eof()]);
lexer_case!(lexer_gt, ">", vec![tk(TokenKind::Gt, ">"), eof()]);
lexer_case!(lexer_gteq, ">=", vec![tk(TokenKind::GtEq, ">="), eof()]);
lexer_case!(lexer_gtgt, ">>", vec![tk(TokenKind::GtGt, ">>"), eof()]);
lexer_case!(lexer_amp, "&", vec![tk(TokenKind::Amp, "&"), eof()]);
lexer_case!(lexer_pipe, "|", vec![tk(TokenKind::Pipe, "|"), eof()]);
lexer_case!(lexer_caret, "^", vec![tk(TokenKind::Caret, "^"), eof()]);
lexer_case!(lexer_tilde, "~", vec![tk(TokenKind::Tilde, "~"), eof()]);
lexer_case!(lexer_lparen, "(", vec![tk(TokenKind::LParen, "("), eof()]);
lexer_case!(lexer_rparen, ")", vec![tk(TokenKind::RParen, ")"), eof()]);
lexer_case!(lexer_lbrace, "{", vec![tk(TokenKind::LBrace, "{"), eof()]);
lexer_case!(lexer_rbrace, "}", vec![tk(TokenKind::RBrace, "}"), eof()]);
lexer_case!(
    lexer_lbracket,
    "[",
    vec![tk(TokenKind::LBracket, "["), eof()]
);
lexer_case!(
    lexer_rbracket,
    "]",
    vec![tk(TokenKind::RBracket, "]"), eof()]
);
lexer_case!(lexer_comma, ",", vec![tk(TokenKind::Comma, ","), eof()]);
lexer_case!(
    lexer_semicolon,
    ";",
    vec![tk(TokenKind::Semicolon, ";"), eof()]
);
lexer_case!(lexer_colon, ":", vec![tk(TokenKind::Colon, ":"), eof()]);
lexer_case!(lexer_dot, ".", vec![tk(TokenKind::Dot, "."), eof()]);

#[test]
fn lexer_invalid_char_panics() {
    assert!(Lexer::create_tokens("@".to_string()).is_err());
}

#[test]
fn lexer_complex_program_tokenization() {
    let tokens = Lexer::create_tokens(
        "import a.b; fn add(x: int, y: int) int { return x + y; }".to_string(),
    )
    .expect("lexing failed");
    let expected = vec![
            tk(TokenKind::Import, "import"),
            tk(TokenKind::Ident, "a"),
            tk(TokenKind::Dot, "."),
            tk(TokenKind::Ident, "b"),
            tk(TokenKind::Semicolon, ";"),
            tk(TokenKind::Fn, "fn"),
            tk(TokenKind::Ident, "add"),
            tk(TokenKind::LParen, "("),
            tk(TokenKind::Ident, "x"),
            tk(TokenKind::Colon, ":"),
            tk(TokenKind::Ident, "int"),
            tk(TokenKind::Comma, ","),
            tk(TokenKind::Ident, "y"),
            tk(TokenKind::Colon, ":"),
            tk(TokenKind::Ident, "int"),
            tk(TokenKind::RParen, ")"),
            tk(TokenKind::Ident, "int"),
            tk(TokenKind::LBrace, "{"),
            tk(TokenKind::Return, "return"),
            tk(TokenKind::Ident, "x"),
            tk(TokenKind::Plus, "+"),
            tk(TokenKind::Ident, "y"),
            tk(TokenKind::Semicolon, ";"),
            tk(TokenKind::RBrace, "}"),
            eof(),
    ];
    assert!(
        tokens_eq_ignoring_span(&tokens, &expected),
        "tokens differ\n  actual:   {:?}\n  expected: {:?}",
        tokens,
        expected
    );
}

// EXPRESSION TESTS
expr_case!(
    expr_int_literal,
    vec![tk(TokenKind::Int, "42")],
    Expr::IntLiteral("42".into())
);
expr_case!(
    expr_float_literal,
    vec![tk(TokenKind::Float, "4.2")],
    Expr::FloatLiteral("4.2".into())
);
expr_case!(
    expr_true_literal,
    vec![tk(TokenKind::True, "true")],
    Expr::BoolLiteral(true)
);
expr_case!(
    expr_false_literal,
    vec![tk(TokenKind::False, "false")],
    Expr::BoolLiteral(false)
);
expr_case!(
    expr_string_literal,
    vec![tk(TokenKind::StringLiteral, "hi")],
    Expr::StringLiteral("hi".into())
);
expr_case!(
    expr_variable,
    vec![tk(TokenKind::Ident, "x")],
    Expr::Variable("x".into())
);
expr_case!(
    expr_grouping,
    vec![
        tk(TokenKind::LParen, "("),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::RParen, ")")
    ],
    Expr::Grouping(Box::new(Expr::IntLiteral("1".into())))
);
expr_case!(
    expr_list_empty,
    vec![tk(TokenKind::LBracket, "["), tk(TokenKind::RBracket, "]")],
    Expr::ListLiteral(vec![])
);
expr_case!(
    expr_list_two_items,
    vec![
        tk(TokenKind::LBracket, "["),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Comma, ","),
        tk(TokenKind::Int, "2"),
        tk(TokenKind::RBracket, "]")
    ],
    Expr::ListLiteral(vec![
        Box::new(Expr::IntLiteral("1".into())),
        Box::new(Expr::IntLiteral("2".into()))
    ])
);
expr_case!(
    expr_nested_list,
    vec![
        tk(TokenKind::LBracket, "["),
        tk(TokenKind::LBracket, "["),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::RBracket, "]"),
        tk(TokenKind::Comma, ","),
        tk(TokenKind::LBracket, "["),
        tk(TokenKind::Int, "2"),
        tk(TokenKind::Comma, ","),
        tk(TokenKind::Int, "3"),
        tk(TokenKind::RBracket, "]"),
        tk(TokenKind::RBracket, "]")
    ],
    Expr::ListLiteral(vec![
        Box::new(Expr::ListLiteral(vec![Box::new(Expr::IntLiteral(
            "1".into()
        ))])),
        Box::new(Expr::ListLiteral(vec![
            Box::new(Expr::IntLiteral("2".into())),
            Box::new(Expr::IntLiteral("3".into()))
        ]))
    ])
);

expr_case!(
    expr_unary_neg,
    vec![tk(TokenKind::Minus, "-"), tk(TokenKind::Int, "1")],
    Expr::Unary {
        op: UnaryOp::Neg,
        value: Box::new(Expr::IntLiteral("1".into()))
    }
);
expr_case!(
    expr_unary_bitnot,
    vec![tk(TokenKind::Tilde, "~"), tk(TokenKind::Int, "1")],
    Expr::Unary {
        op: UnaryOp::BitNot,
        value: Box::new(Expr::IntLiteral("1".into()))
    }
);
expr_case!(
    expr_double_unary,
    vec![
        tk(TokenKind::Minus, "-"),
        tk(TokenKind::Tilde, "~"),
        tk(TokenKind::Int, "1")
    ],
    Expr::Unary {
        op: UnaryOp::Neg,
        value: Box::new(Expr::Unary {
            op: UnaryOp::BitNot,
            value: Box::new(Expr::IntLiteral("1".into()))
        })
    }
);
expr_case!(
    expr_unary_not,
    vec![tk(TokenKind::Bang, "!"), tk(TokenKind::True, "true")],
    Expr::Unary {
        op: UnaryOp::Not,
        value: Box::new(Expr::BoolLiteral(true))
    }
);

expr_case!(
    expr_call_no_args,
    vec![
        tk(TokenKind::Ident, "f"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")")
    ],
    Expr::Call {
        callee: Box::new(Expr::Variable("f".into())),
        type_args: vec![],
        arguments: vec![]
    }
);
expr_case!(
    expr_call_one_arg,
    vec![
        tk(TokenKind::Ident, "f"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::RParen, ")")
    ],
    Expr::Call {
        callee: Box::new(Expr::Variable("f".into())),
        type_args: vec![],
        arguments: vec![Expr::IntLiteral("1".into())]
    }
);
expr_case!(
    expr_call_multiple_args,
    vec![
        tk(TokenKind::Ident, "f"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Comma, ","),
        tk(TokenKind::Int, "2"),
        tk(TokenKind::Comma, ","),
        tk(TokenKind::Int, "3"),
        tk(TokenKind::RParen, ")")
    ],
    Expr::Call {
        callee: Box::new(Expr::Variable("f".into())),
        type_args: vec![],
        arguments: vec![
            Expr::IntLiteral("1".into()),
            Expr::IntLiteral("2".into()),
            Expr::IntLiteral("3".into())
        ]
    }
);
expr_case!(
    expr_nested_call,
    vec![
        tk(TokenKind::Ident, "f"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::Ident, "g"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::RParen, ")")
    ],
    Expr::Call {
        callee: Box::new(Expr::Variable("f".into())),
        type_args: vec![],
        arguments: vec![Expr::Call {
            callee: Box::new(Expr::Variable("g".into())),
            type_args: vec![],
            arguments: vec![Expr::IntLiteral("1".into())]
        }]
    }
);
expr_case!(
    expr_chained_call,
    vec![
        tk(TokenKind::Ident, "factory"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::RParen, ")")
    ],
    Expr::Call {
        callee: Box::new(Expr::Call {
            callee: Box::new(Expr::Variable("factory".into())),
            type_args: vec![],
            arguments: vec![]
        }),
        type_args: vec![],
        arguments: vec![Expr::IntLiteral("1".into())]
    }
);

expr_case!(
    expr_struct_literal,
    vec![
        tk(TokenKind::Ident, "Point"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Comma, ","),
        tk(TokenKind::Ident, "y"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Int, "2"),
        tk(TokenKind::RBrace, "}")
    ],
    Expr::StructLiteral {
        struct_name: "Point".into(),
        arguments: vec![
            ("x".into(), Expr::IntLiteral("1".into())),
            ("y".into(), Expr::IntLiteral("2".into()))
        ]
    }
);
expr_case!(
    expr_struct_literal_empty,
    vec![
        tk(TokenKind::Ident, "Unit"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}")
    ],
    Expr::StructLiteral {
        struct_name: "Unit".into(),
        arguments: vec![]
    }
);
expr_case!(
    expr_struct_literal_allows_trailing_comma,
    vec![
        tk(TokenKind::Ident, "Point"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Comma, ","),
        tk(TokenKind::RBrace, "}")
    ],
    Expr::StructLiteral {
        struct_name: "Point".into(),
        arguments: vec![("x".into(), Expr::IntLiteral("1".into()))]
    }
);

expr_case!(
    expr_assignment,
    vec![
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Int, "1")
    ],
    Expr::Assign {
        target: Box::new(Expr::Variable("x".into())),
        value: Box::new(Expr::IntLiteral("1".into()))
    }
);
expr_case!(
    expr_assignment_is_right_associative,
    vec![
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Ident, "y"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Int, "1")
    ],
    Expr::Assign {
        target: Box::new(Expr::Variable("x".into())),
        value: Box::new(Expr::Assign {
            target: Box::new(Expr::Variable("y".into())),
            value: Box::new(Expr::IntLiteral("1".into()))
        })
    }
);
expr_case!(
    expr_compound_plus_assign,
    vec![
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::PlusEq, "+="),
        tk(TokenKind::Int, "1")
    ],
    Expr::BinaryOpAssign {
        target: Box::new(Expr::Variable("x".into())),
        binary_op: BinaryOp::Add,
        value: Box::new(Expr::IntLiteral("1".into()))
    }
);
expr_case!(
    expr_compound_minus_assign,
    vec![
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::MinusEq, "-="),
        tk(TokenKind::Int, "1")
    ],
    Expr::BinaryOpAssign {
        target: Box::new(Expr::Variable("x".into())),
        binary_op: BinaryOp::Sub,
        value: Box::new(Expr::IntLiteral("1".into()))
    }
);
expr_case!(
    expr_compound_star_assign,
    vec![
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::StarEq, "*="),
        tk(TokenKind::Int, "1")
    ],
    Expr::BinaryOpAssign {
        target: Box::new(Expr::Variable("x".into())),
        binary_op: BinaryOp::Mul,
        value: Box::new(Expr::IntLiteral("1".into()))
    }
);
expr_case!(
    expr_compound_slash_assign,
    vec![
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::SlashEq, "/="),
        tk(TokenKind::Int, "1")
    ],
    Expr::BinaryOpAssign {
        target: Box::new(Expr::Variable("x".into())),
        binary_op: BinaryOp::Div,
        value: Box::new(Expr::IntLiteral("1".into()))
    }
);
expr_case!(
    expr_compound_assignment_value_can_be_assignment,
    vec![
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::PlusEq, "+="),
        tk(TokenKind::Ident, "y"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Int, "1")
    ],
    Expr::BinaryOpAssign {
        target: Box::new(Expr::Variable("x".into())),
        binary_op: BinaryOp::Add,
        value: Box::new(Expr::Assign {
            target: Box::new(Expr::Variable("y".into())),
            value: Box::new(Expr::IntLiteral("1".into()))
        })
    }
);

expr_case!(
    expr_precedence_mul_before_add,
    vec![
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Plus, "+"),
        tk(TokenKind::Int, "2"),
        tk(TokenKind::Star, "*"),
        tk(TokenKind::Int, "3")
    ],
    Expr::BinaryOp {
        lhs: Box::new(Expr::IntLiteral("1".into())),
        binary_op: BinaryOp::Add,
        rhs: Box::new(Expr::BinaryOp {
            lhs: Box::new(Expr::IntLiteral("2".into())),
            binary_op: BinaryOp::Mul,
            rhs: Box::new(Expr::IntLiteral("3".into()))
        })
    }
);
expr_case!(
    expr_left_associative_add,
    vec![
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Plus, "+"),
        tk(TokenKind::Int, "2"),
        tk(TokenKind::Plus, "+"),
        tk(TokenKind::Int, "3")
    ],
    Expr::BinaryOp {
        lhs: Box::new(Expr::BinaryOp {
            lhs: Box::new(Expr::IntLiteral("1".into())),
            binary_op: BinaryOp::Add,
            rhs: Box::new(Expr::IntLiteral("2".into()))
        }),
        binary_op: BinaryOp::Add,
        rhs: Box::new(Expr::IntLiteral("3".into()))
    }
);
expr_case!(
    expr_sub_and_add_left_assoc,
    vec![
        tk(TokenKind::Int, "8"),
        tk(TokenKind::Minus, "-"),
        tk(TokenKind::Int, "3"),
        tk(TokenKind::Plus, "+"),
        tk(TokenKind::Int, "1")
    ],
    Expr::BinaryOp {
        lhs: Box::new(Expr::BinaryOp {
            lhs: Box::new(Expr::IntLiteral("8".into())),
            binary_op: BinaryOp::Sub,
            rhs: Box::new(Expr::IntLiteral("3".into()))
        }),
        binary_op: BinaryOp::Add,
        rhs: Box::new(Expr::IntLiteral("1".into()))
    }
);
expr_case!(
    expr_mul_div_mod_left_assoc,
    vec![
        tk(TokenKind::Int, "8"),
        tk(TokenKind::Star, "*"),
        tk(TokenKind::Int, "3"),
        tk(TokenKind::Slash, "/"),
        tk(TokenKind::Int, "2"),
        tk(TokenKind::Percent, "%"),
        tk(TokenKind::Int, "5")
    ],
    Expr::BinaryOp {
        lhs: Box::new(Expr::BinaryOp {
            lhs: Box::new(Expr::BinaryOp {
                lhs: Box::new(Expr::IntLiteral("8".into())),
                binary_op: BinaryOp::Mul,
                rhs: Box::new(Expr::IntLiteral("3".into()))
            }),
            binary_op: BinaryOp::Div,
            rhs: Box::new(Expr::IntLiteral("2".into()))
        }),
        binary_op: BinaryOp::Mod,
        rhs: Box::new(Expr::IntLiteral("5".into()))
    }
);
expr_case!(
    expr_shift_left_assoc,
    vec![
        tk(TokenKind::Int, "1"),
        tk(TokenKind::LtLt, "<<"),
        tk(TokenKind::Int, "2"),
        tk(TokenKind::GtGt, ">>"),
        tk(TokenKind::Int, "3")
    ],
    Expr::BinaryOp {
        lhs: Box::new(Expr::BinaryOp {
            lhs: Box::new(Expr::IntLiteral("1".into())),
            binary_op: BinaryOp::Shl,
            rhs: Box::new(Expr::IntLiteral("2".into()))
        }),
        binary_op: BinaryOp::Shr,
        rhs: Box::new(Expr::IntLiteral("3".into()))
    }
);
expr_case!(
    expr_comparison_chain_left_assoc,
    vec![
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Lt, "<"),
        tk(TokenKind::Int, "2"),
        tk(TokenKind::LtEq, "<="),
        tk(TokenKind::Int, "3"),
        tk(TokenKind::Gt, ">"),
        tk(TokenKind::Int, "4"),
        tk(TokenKind::GtEq, ">="),
        tk(TokenKind::Int, "5")
    ],
    Expr::BinaryOp {
        lhs: Box::new(Expr::BinaryOp {
            lhs: Box::new(Expr::BinaryOp {
                lhs: Box::new(Expr::BinaryOp {
                    lhs: Box::new(Expr::IntLiteral("1".into())),
                    binary_op: BinaryOp::Lt,
                    rhs: Box::new(Expr::IntLiteral("2".into()))
                }),
                binary_op: BinaryOp::Le,
                rhs: Box::new(Expr::IntLiteral("3".into()))
            }),
            binary_op: BinaryOp::Gt,
            rhs: Box::new(Expr::IntLiteral("4".into()))
        }),
        binary_op: BinaryOp::Ge,
        rhs: Box::new(Expr::IntLiteral("5".into()))
    }
);
expr_case!(
    expr_equality_chain_left_assoc,
    vec![
        tk(TokenKind::Int, "1"),
        tk(TokenKind::EqEq, "=="),
        tk(TokenKind::Int, "2"),
        tk(TokenKind::BangEq, "!="),
        tk(TokenKind::Int, "3")
    ],
    Expr::BinaryOp {
        lhs: Box::new(Expr::BinaryOp {
            lhs: Box::new(Expr::IntLiteral("1".into())),
            binary_op: BinaryOp::Eq,
            rhs: Box::new(Expr::IntLiteral("2".into()))
        }),
        binary_op: BinaryOp::Ne,
        rhs: Box::new(Expr::IntLiteral("3".into()))
    }
);
expr_case!(
    expr_bitwise_chain,
    vec![
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Pipe, "|"),
        tk(TokenKind::Int, "2"),
        tk(TokenKind::Caret, "^"),
        tk(TokenKind::Int, "3"),
        tk(TokenKind::Amp, "&"),
        tk(TokenKind::Int, "4")
    ],
    Expr::BinaryOp {
        lhs: Box::new(Expr::IntLiteral("1".into())),
        binary_op: BinaryOp::BitOr,
        rhs: Box::new(Expr::BinaryOp {
            lhs: Box::new(Expr::IntLiteral("2".into())),
            binary_op: BinaryOp::BitXor,
            rhs: Box::new(Expr::BinaryOp {
                lhs: Box::new(Expr::IntLiteral("3".into())),
                binary_op: BinaryOp::BitAnd,
                rhs: Box::new(Expr::IntLiteral("4".into()))
            })
        })
    }
);
expr_case!(
    expr_logical_and_or_chain,
    vec![
        tk(TokenKind::True, "true"),
        tk(TokenKind::Or, "or"),
        tk(TokenKind::True, "true"),
        tk(TokenKind::And, "and"),
        tk(TokenKind::False, "false")
    ],
    Expr::BinaryOp {
        lhs: Box::new(Expr::BoolLiteral(true)),
        binary_op: BinaryOp::Or,
        rhs: Box::new(Expr::BinaryOp {
            lhs: Box::new(Expr::BoolLiteral(true)),
            binary_op: BinaryOp::And,
            rhs: Box::new(Expr::BoolLiteral(false))
        })
    }
);
expr_case!(
    expr_unary_with_call_operand,
    vec![
        tk(TokenKind::Minus, "-"),
        tk(TokenKind::Ident, "f"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")")
    ],
    Expr::Unary {
        op: UnaryOp::Neg,
        value: Box::new(Expr::Call {
            callee: Box::new(Expr::Variable("f".into())),
            type_args: vec![],
            arguments: vec![]
        })
    }
);

expr_case!(
    expr_char_literal,
    vec![tk(TokenKind::Char, "x")],
    Expr::CharLiteral('x')
);
expr_case!(
    expr_null_literal,
    vec![tk(TokenKind::Null, "null")],
    Expr::NullLiteral
);
error_case_expr!(
    expr_trailing_comma_in_list_is_rejected,
    vec![
        tk(TokenKind::LBracket, "["),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Comma, ","),
        tk(TokenKind::RBracket, "]")
    ],
    |err| assert_unexpected_expr_start(err, TokenKind::RBracket)
);
error_case_expr!(
    expr_missing_struct_literal_field_name_after_comma,
    vec![
        tk(TokenKind::Ident, "Point"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Comma, ","),
        tk(TokenKind::Int, "2"),
        tk(TokenKind::RBrace, "}")
    ],
    |err| assert_unexpected_token(err, TokenKind::Ident, TokenKind::Int, 6)
);
error_case_expr!(
    expr_missing_rparen_in_grouping,
    vec![tk(TokenKind::LParen, "("), tk(TokenKind::Int, "1")],
    |err| assert_unexpected_token(err, TokenKind::RParen, TokenKind::Eof, 2)
);
error_case_expr!(
    expr_missing_rbracket_in_list,
    vec![tk(TokenKind::LBracket, "["), tk(TokenKind::Int, "1")],
    |err| assert_unexpected_token(err, TokenKind::RBracket, TokenKind::Eof, 2)
);
error_case_expr!(
    expr_invalid_assignment_target_literal,
    vec![
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Int, "2")
    ],
    |err| assert_invalid_assignment_target(err)
);
error_case_expr!(
    expr_invalid_assignment_target_call,
    vec![
        tk(TokenKind::Ident, "f"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Int, "2")
    ],
    |err| assert_invalid_assignment_target(err)
);
error_case_expr!(
    expr_trailing_plus_yields_eof_expr_error,
    vec![tk(TokenKind::Int, "1"), tk(TokenKind::Plus, "+")],
    |err| assert_unexpected_expr_start(err, TokenKind::Eof)
);
error_case_expr!(
    expr_trailing_eq_yields_eof_expr_error,
    vec![tk(TokenKind::Ident, "x"), tk(TokenKind::Eq, "=")],
    |err| assert_unexpected_expr_start(err, TokenKind::Eof)
);

// STATEMENT TESTS
stmt_case!(
    stmt_let_basic,
    vec![
        tk(TokenKind::Let, "let"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "int"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Semicolon, ";")
    ],
    Stmt::Let(Let {
        var_name: "x".into(),
        var_type: Type::Named("int".into()),
        value: Expr::IntLiteral("1".into())
    })
);
stmt_case!(
    stmt_let_any_trait_type,
    vec![
        tk(TokenKind::Let, "let"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Any, "any"),
        tk(TokenKind::Ident, "Display"),
        tk(TokenKind::Plus, "+"),
        tk(TokenKind::Ident, "Debug"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Ident, "y"),
        tk(TokenKind::Semicolon, ";")
    ],
    Stmt::Let(Let {
        var_name: "x".into(),
        var_type: Type::AnyTrait(vec![
            TraitBound {
                trait_name: "Display".into(),
                args: vec![]
            },
            TraitBound {
                trait_name: "Debug".into(),
                args: vec![]
            }
        ]),
        value: Expr::Variable("y".into())
    })
);
stmt_case!(
    stmt_let_value_can_be_struct_literal,
    vec![
        tk(TokenKind::Let, "let"),
        tk(TokenKind::Ident, "p"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "Point"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Ident, "Point"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::RBrace, "}"),
        tk(TokenKind::Semicolon, ";")
    ],
    Stmt::Let(Let {
        var_name: "p".into(),
        var_type: Type::Named("Point".into()),
        value: Expr::StructLiteral {
            struct_name: "Point".into(),
            arguments: vec![("x".into(), Expr::IntLiteral("1".into()))]
        }
    })
);
stmt_case!(
    stmt_return_none,
    vec![
        tk(TokenKind::Return, "return"),
        tk(TokenKind::Semicolon, ";")
    ],
    Stmt::Return(Return { return_value: None })
);
stmt_case!(
    stmt_return_value,
    vec![
        tk(TokenKind::Return, "return"),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Semicolon, ";")
    ],
    Stmt::Return(Return {
        return_value: Some(Expr::IntLiteral("1".into()))
    })
);
stmt_case!(
    stmt_break,
    vec![tk(TokenKind::Break, "break"), tk(TokenKind::Semicolon, ";")],
    Stmt::Break
);
stmt_case!(
    stmt_continue,
    vec![
        tk(TokenKind::Continue, "continue"),
        tk(TokenKind::Semicolon, ";")
    ],
    Stmt::Continue
);
stmt_case!(
    stmt_asm,
    vec![
        tk(TokenKind::Asm, "mov ax, bx"),
        tk(TokenKind::Semicolon, ";")
    ],
    Stmt::Asm(Asm {
        asm_code: "mov ax, bx".into()
    })
);
stmt_case!(
    stmt_expr_stmt_call,
    vec![
        tk(TokenKind::Ident, "f"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::Semicolon, ";")
    ],
    Stmt::Expr(Expr::Call {
        callee: Box::new(Expr::Variable("f".into())),
        type_args: vec![],
        arguments: vec![]
    })
);
stmt_case!(
    stmt_expr_stmt_assignment,
    vec![
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::PlusEq, "+="),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Semicolon, ";")
    ],
    Stmt::Expr(Expr::BinaryOpAssign {
        target: Box::new(Expr::Variable("x".into())),
        binary_op: BinaryOp::Add,
        value: Box::new(Expr::IntLiteral("1".into()))
    })
);

stmt_case!(
    stmt_if_simple,
    vec![
        tk(TokenKind::If, "if"),
        tk(TokenKind::True, "true"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Return, "return"),
        tk(TokenKind::Semicolon, ";"),
        tk(TokenKind::RBrace, "}")
    ],
    Stmt::If(If {
        condition: Expr::BoolLiteral(true),
        if_code: Block {
            statements: vec![Stmt::Return(Return { return_value: None })]
        },
        else_code: None
    })
);
stmt_case!(
    stmt_if_else_block,
    vec![
        tk(TokenKind::If, "if"),
        tk(TokenKind::False, "false"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Break, "break"),
        tk(TokenKind::Semicolon, ";"),
        tk(TokenKind::RBrace, "}"),
        tk(TokenKind::Else, "else"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Continue, "continue"),
        tk(TokenKind::Semicolon, ";"),
        tk(TokenKind::RBrace, "}")
    ],
    Stmt::If(If {
        condition: Expr::BoolLiteral(false),
        if_code: Block {
            statements: vec![Stmt::Break]
        },
        else_code: Some(Box::new(Stmt::Block(Block {
            statements: vec![Stmt::Continue]
        })))
    })
);
stmt_case!(
    stmt_if_else_if_chain,
    vec![
        tk(TokenKind::If, "if"),
        tk(TokenKind::True, "true"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}"),
        tk(TokenKind::Else, "else"),
        tk(TokenKind::If, "if"),
        tk(TokenKind::False, "false"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}")
    ],
    Stmt::If(If {
        condition: Expr::BoolLiteral(true),
        if_code: Block { statements: vec![] },
        else_code: Some(Box::new(Stmt::If(If {
            condition: Expr::BoolLiteral(false),
            if_code: Block { statements: vec![] },
            else_code: None
        })))
    })
);
stmt_case!(
    stmt_while_simple,
    vec![
        tk(TokenKind::While, "while"),
        tk(TokenKind::True, "true"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Break, "break"),
        tk(TokenKind::Semicolon, ";"),
        tk(TokenKind::RBrace, "}")
    ],
    Stmt::While(While {
        condition: Expr::BoolLiteral(true),
        inner_code: Block {
            statements: vec![Stmt::Break]
        }
    })
);
stmt_case!(
    stmt_loop_simple,
    vec![
        tk(TokenKind::Loop, "loop"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Continue, "continue"),
        tk(TokenKind::Semicolon, ";"),
        tk(TokenKind::RBrace, "}")
    ],
    Stmt::Loop(Loop {
        inner_code: Block {
            statements: vec![Stmt::Continue]
        }
    })
);
stmt_case!(
    stmt_for_simple,
    vec![
        tk(TokenKind::For, "for"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::In, "in"),
        tk(TokenKind::Ident, "items"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}")
    ],
    Stmt::For(For {
        var_name: "x".into(),
        iterable: Expr::Variable("items".into()),
        inner_code: Block { statements: vec![] }
    })
);
stmt_case!(
    stmt_for_iterable_disallows_struct_literal_without_grouping,
    vec![
        tk(TokenKind::For, "for"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::In, "in"),
        tk(TokenKind::Ident, "Items"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}")
    ],
    Stmt::For(For {
        var_name: "x".into(),
        iterable: Expr::Variable("Items".into()),
        inner_code: Block { statements: vec![] }
    })
);

error_case_stmt!(
    stmt_missing_semicolon_after_break,
    vec![tk(TokenKind::Break, "break")],
    |err| assert_unexpected_token(err, TokenKind::Semicolon, TokenKind::Eof, 1)
);
error_case_stmt!(
    stmt_missing_semicolon_after_continue,
    vec![tk(TokenKind::Continue, "continue")],
    |err| assert_unexpected_token(err, TokenKind::Semicolon, TokenKind::Eof, 1)
);
error_case_stmt!(
    stmt_expr_missing_semicolon,
    vec![
        tk(TokenKind::Ident, "f"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")")
    ],
    |err| assert_unexpected_token(err, TokenKind::Semicolon, TokenKind::Eof, 3)
);
error_case_stmt!(
    stmt_let_missing_colon,
    vec![
        tk(TokenKind::Let, "let"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Ident, "int"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Semicolon, ";")
    ],
    |err| assert_unexpected_token(err, TokenKind::Colon, TokenKind::Ident, 2)
);
error_case_stmt!(
    stmt_let_missing_eq,
    vec![
        tk(TokenKind::Let, "let"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "int"),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Semicolon, ";")
    ],
    |err| assert_unexpected_token(err, TokenKind::Eq, TokenKind::Int, 4)
);
error_case_stmt!(
    stmt_if_missing_brace,
    vec![
        tk(TokenKind::If, "if"),
        tk(TokenKind::True, "true"),
        tk(TokenKind::Return, "return")
    ],
    |err| assert_unexpected_token(err, TokenKind::LBrace, TokenKind::Return, 2)
);
error_case_stmt!(
    stmt_for_missing_in,
    vec![
        tk(TokenKind::For, "for"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Ident, "items")
    ],
    |err| assert_unexpected_token(err, TokenKind::In, TokenKind::Ident, 2)
);
error_case_stmt!(
    stmt_let_missing_type,
    vec![
        tk(TokenKind::Let, "let"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Any, "any"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Semicolon, ";")
    ],
    |err| assert_unexpected_token(err, TokenKind::Ident, TokenKind::Eq, 4)
);
error_case_stmt!(
    stmt_return_missing_value_after_operator,
    vec![
        tk(TokenKind::Return, "return"),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Plus, "+")
    ],
    |err| assert_unexpected_expr_start(err, TokenKind::Eof)
);
error_case_stmt!(
    stmt_asm_requires_parenthesized_string,
    vec![
        tk(TokenKind::Asm, "asm"),
        tk(TokenKind::StringLiteral, "nop"),
        tk(TokenKind::Semicolon, ";")
    ],
    |err| assert_unexpected_token(err, TokenKind::Semicolon, TokenKind::StringLiteral, 1)
);
stmt_case!(
    stmt_return_char_literal,
    vec![
        tk(TokenKind::Return, "return"),
        tk(TokenKind::Char, "x"),
        tk(TokenKind::Semicolon, ";")
    ],
    Stmt::Return(Return {
        return_value: Some(Expr::CharLiteral('x'))
    })
);

// TOP-LEVEL ITEM TESTS
item_case!(
    item_import_simple,
    vec![
        tk(TokenKind::Import, "import"),
        tk(TokenKind::Ident, "foo"),
        tk(TokenKind::Semicolon, ";")
    ],
    vec![Item::Import(Import {
        import_name: "foo".into()
    })]
);
item_case!(
    item_import_dotted,
    vec![
        tk(TokenKind::Import, "import"),
        tk(TokenKind::Ident, "foo"),
        tk(TokenKind::Dot, "."),
        tk(TokenKind::Ident, "bar"),
        tk(TokenKind::Dot, "."),
        tk(TokenKind::Ident, "baz"),
        tk(TokenKind::Semicolon, ";")
    ],
    vec![Item::Import(Import {
        import_name: "foo.bar.baz".into()
    })]
);
item_case!(
    item_global_var,
    vec![
        tk(TokenKind::Global, "global"),
        tk(TokenKind::Ident, "counter"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "int"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Int, "0"),
        tk(TokenKind::Semicolon, ";")
    ],
    vec![Item::GlobalVar(GlobalVar {
        var_name: "counter".into(),
        var_type: Type::Named("int".into()),
        value: Expr::IntLiteral("0".into())
    })]
);
item_case!(
    item_const_var,
    vec![
        tk(TokenKind::Const, "const"),
        tk(TokenKind::Ident, "answer"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "int"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Int, "42"),
        tk(TokenKind::Semicolon, ";")
    ],
    vec![Item::ConstVar(ConstVar {
        var_name: "answer".into(),
        var_type: Type::Named("int".into()),
        value: Expr::IntLiteral("42".into())
    })]
);
error_case_items!(
    item_global_missing_initializer,
    vec![
        tk(TokenKind::Global, "global"),
        tk(TokenKind::Ident, "counter"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "int"),
        tk(TokenKind::Semicolon, ";")
    ],
    |err| assert_unexpected_token(err, TokenKind::Eq, TokenKind::Semicolon, 4)
);
error_case_items!(
    item_const_missing_semicolon,
    vec![
        tk(TokenKind::Const, "const"),
        tk(TokenKind::Ident, "answer"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "int"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Int, "42")
    ],
    |err| assert_unexpected_token(err, TokenKind::Semicolon, TokenKind::Eof, 6)
);
item_case!(
    item_variant_empty,
    vec![
        tk(TokenKind::Variant, "variant"),
        tk(TokenKind::Ident, "Option"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::Variant(Variant {
        variant_name: "Option".into(),
        cases: vec![]
    })]
);
item_case!(
    item_variant_cases,
    vec![
        tk(TokenKind::Variant, "variant"),
        tk(TokenKind::Ident, "Result"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Ident, "Ok"),
        tk(TokenKind::Comma, ","),
        tk(TokenKind::Ident, "Err"),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::Variant(Variant {
        variant_name: "Result".into(),
        cases: vec!["Ok".into(), "Err".into()]
    })]
);
item_case!(
    item_variant_allows_trailing_comma,
    vec![
        tk(TokenKind::Variant, "variant"),
        tk(TokenKind::Ident, "Result"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Ident, "Ok"),
        tk(TokenKind::Comma, ","),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::Variant(Variant {
        variant_name: "Result".into(),
        cases: vec!["Ok".into()]
    })]
);

item_case!(
    item_trait_empty_generics_no_fns,
    vec![
        tk(TokenKind::Trait, "trait"),
        tk(TokenKind::Ident, "Display"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::Trait(Trait {
        trait_name: "Display".into(),
        generic_params: vec![],
        function_signatures: vec![]
    })]
);
item_case!(
    item_trait_one_signature,
    vec![
        tk(TokenKind::Trait, "trait"),
        tk(TokenKind::Ident, "Add"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Fn, "fn"),
        tk(TokenKind::Ident, "add"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::Ident, "rhs"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "Self"),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::Ident, "Self"),
        tk(TokenKind::Semicolon, ";"),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::Trait(Trait {
        trait_name: "Add".into(),
        generic_params: vec![],
        function_signatures: vec![FunctionSignature {
            function_name: "add".into(),
            generic_params: vec![],
            params: vec![Param {
                name: "rhs".into(),
                param_type: Type::Named("Self".into())
            }],
            return_type: Some(Type::Named("Self".into())),
            operator: None
        }]
    })]
);
item_case!(
    item_trait_operator_signature_without_return_type,
    vec![
        tk(TokenKind::Trait, "trait"),
        tk(TokenKind::Ident, "Neg"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Fn, "fn"),
        tk(TokenKind::Operator, "operator"),
        tk(TokenKind::Minus, "-"),
        tk(TokenKind::Ident, "neg"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::Semicolon, ";"),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::Trait(Trait {
        trait_name: "Neg".into(),
        generic_params: vec![],
        function_signatures: vec![FunctionSignature {
            function_name: "neg".into(),
            generic_params: vec![],
            params: vec![],
            return_type: None,
            operator: Some("-".into())
        }]
    })]
);

item_case!(
    item_struct_fields_only,
    vec![
        tk(TokenKind::Struct, "struct"),
        tk(TokenKind::Ident, "Point"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "int"),
        tk(TokenKind::Comma, ","),
        tk(TokenKind::Ident, "y"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "int"),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::Struct(Struct {
        struct_name: "Point".into(),
        generic_params: vec![],
        fields: vec![
            StructField {
                field_name: "x".into(),
                field_type: Type::Named("int".into())
            },
            StructField {
                field_name: "y".into(),
                field_type: Type::Named("int".into())
            }
        ],
        functions: vec![]
    })]
);
item_case!(
    item_struct_allows_trailing_field_comma,
    vec![
        tk(TokenKind::Struct, "struct"),
        tk(TokenKind::Ident, "Point"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "int"),
        tk(TokenKind::Comma, ","),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::Struct(Struct {
        struct_name: "Point".into(),
        generic_params: vec![],
        fields: vec![StructField {
            field_name: "x".into(),
            field_type: Type::Named("int".into())
        }],
        functions: vec![]
    })]
);

item_case!(
    item_fn_simple,
    vec![
        tk(TokenKind::Fn, "fn"),
        tk(TokenKind::Ident, "main"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::Function(Function {
        name: "main".into(),
        generic_params: vec![],
        params: vec![],
        body: Block { statements: vec![] },
        return_type: None,
        operator: None
    })]
);
item_case!(
    item_fn_with_return_type,
    vec![
        tk(TokenKind::Fn, "fn"),
        tk(TokenKind::Ident, "len"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "Array"),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::Ident, "int"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Return, "return"),
        tk(TokenKind::Int, "1"),
        tk(TokenKind::Semicolon, ";"),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::Function(Function {
        name: "len".into(),
        generic_params: vec![],
        params: vec![Param {
            name: "x".into(),
            param_type: Type::Named("Array".into())
        }],
        body: Block {
            statements: vec![Stmt::Return(Return {
                return_value: Some(Expr::IntLiteral("1".into()))
            })]
        },
        return_type: Some(Type::Named("int".into())),
        operator: None
    })]
);
item_case!(
    item_fn_operator_name,
    vec![
        tk(TokenKind::Fn, "fn"),
        tk(TokenKind::Operator, "operator"),
        tk(TokenKind::Plus, "+"),
        tk(TokenKind::Ident, "add"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::Function(Function {
        name: "add".into(),
        generic_params: vec![],
        params: vec![],
        body: Block { statements: vec![] },
        return_type: None,
        operator: Some("+".into())
    })]
);
item_case!(
    item_fn_with_generic_params_and_bounds,
    vec![
        tk(TokenKind::Fn, "fn"),
        tk(TokenKind::Ident, "id"),
        tk(TokenKind::Lt, "<"),
        tk(TokenKind::Ident, "T"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "Clone"),
        tk(TokenKind::Plus, "+"),
        tk(TokenKind::Ident, "Debug"),
        tk(TokenKind::Gt, ">"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::Ident, "value"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "T"),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::Ident, "T"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Return, "return"),
        tk(TokenKind::Ident, "value"),
        tk(TokenKind::Semicolon, ";"),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::Function(Function {
        name: "id".into(),
        generic_params: vec![GenericParam {
            name: "T".into(),
            bounds: vec![
                TraitBound {
                    trait_name: "Clone".into(),
                    args: vec![]
                },
                TraitBound {
                    trait_name: "Debug".into(),
                    args: vec![]
                }
            ]
        }],
        params: vec![Param {
            name: "value".into(),
            param_type: Type::Named("T".into())
        }],
        body: Block {
            statements: vec![Stmt::Return(Return {
                return_value: Some(Expr::Variable("value".into()))
            })]
        },
        return_type: Some(Type::Named("T".into())),
        operator: None
    })]
);
item_case!(
    item_struct_with_method,
    vec![
        tk(TokenKind::Struct, "struct"),
        tk(TokenKind::Ident, "Point"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Ident, "x"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "int"),
        tk(TokenKind::Comma, ","),
        tk(TokenKind::Fn, "fn"),
        tk(TokenKind::Ident, "new"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}"),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::Struct(Struct {
        struct_name: "Point".into(),
        generic_params: vec![],
        fields: vec![StructField {
            field_name: "x".into(),
            field_type: Type::Named("int".into())
        }],
        functions: vec![Function {
            name: "new".into(),
            generic_params: vec![],
            params: vec![],
            body: Block { statements: vec![] },
            return_type: None,
            operator: None
        }]
    })]
);
item_case!(
    item_impl_simple,
    vec![
        tk(TokenKind::Impl, "impl"),
        tk(TokenKind::Ident, "Display"),
        tk(TokenKind::For, "for"),
        tk(TokenKind::Ident, "Point"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Fn, "fn"),
        tk(TokenKind::Ident, "show"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}"),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::TraitImplementation(TraitImplementation {
        generic_params: vec![],
        trait_name: "Display".into(),
        trait_args: vec![],
        struct_name: "Point".into(),
        struct_args: vec![],
        functions: vec![Function {
            name: "show".into(),
            generic_params: vec![],
            params: vec![],
            body: Block { statements: vec![] },
            return_type: None,
            operator: None
        }]
    })]
);
item_case!(
    item_impl_with_generics_and_type_args,
    vec![
        tk(TokenKind::Impl, "impl"),
        tk(TokenKind::Lt, "<"),
        tk(TokenKind::Ident, "T"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "Clone"),
        tk(TokenKind::Gt, ">"),
        tk(TokenKind::Ident, "TraitX"),
        tk(TokenKind::Lt, "<"),
        tk(TokenKind::Ident, "int"),
        tk(TokenKind::Gt, ">"),
        tk(TokenKind::For, "for"),
        tk(TokenKind::Ident, "StructX"),
        tk(TokenKind::Lt, "<"),
        tk(TokenKind::Ident, "T"),
        tk(TokenKind::Gt, ">"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Fn, "fn"),
        tk(TokenKind::Ident, "f"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}"),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::TraitImplementation(TraitImplementation {
        generic_params: vec![GenericParam {
            name: "T".into(),
            bounds: vec![TraitBound {
                trait_name: "Clone".into(),
                args: vec![]
            }]
        }],
        trait_name: "TraitX".into(),
        trait_args: vec![Type::Named("int".into())],
        struct_name: "StructX".into(),
        struct_args: vec![Type::Named("T".into())],
        functions: vec![Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec![],
            body: Block { statements: vec![] },
            return_type: None,
            operator: None
        }]
    })]
);

error_case_items!(
    item_unexpected_top_level,
    vec![tk(TokenKind::Return, "return")],
    |err| assert_unexpected_top_level(err, TokenKind::Return)
);
error_case_items!(
    item_import_missing_semicolon,
    vec![tk(TokenKind::Import, "import"), tk(TokenKind::Ident, "foo")],
    |err| assert_unexpected_token(err, TokenKind::Semicolon, TokenKind::Eof, 2)
);
error_case_items!(
    item_import_trailing_dot_requires_next_ident,
    vec![
        tk(TokenKind::Import, "import"),
        tk(TokenKind::Ident, "foo"),
        tk(TokenKind::Dot, "."),
        tk(TokenKind::Semicolon, ";")
    ],
    |err| assert_unexpected_token(err, TokenKind::Ident, TokenKind::Semicolon, 3)
);
error_case_items!(
    item_variant_missing_name,
    vec![
        tk(TokenKind::Variant, "variant"),
        tk(TokenKind::LBrace, "{")
    ],
    |err| assert_unexpected_token(err, TokenKind::Ident, TokenKind::LBrace, 1)
);
error_case_items!(
    item_struct_missing_brace,
    vec![tk(TokenKind::Struct, "struct"), tk(TokenKind::Ident, "S")],
    |err| assert_unexpected_token(err, TokenKind::LBrace, TokenKind::Eof, 2)
);
error_case_items!(
    item_fn_missing_body_brace,
    vec![
        tk(TokenKind::Fn, "fn"),
        tk(TokenKind::Ident, "f"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")")
    ],
    // After `f()` without a return-type token (e.g. an Ident) and without `{`,
    // parse_type fails with `expected Ident, found Eof`. The previous behaviour
    // silently swallowed that error and reported the missing brace instead, which
    // hid genuine type errors.
    |err| assert_unexpected_token(err, TokenKind::Ident, TokenKind::Eof, 4)
);
error_case_items!(
    item_impl_missing_for,
    vec![
        tk(TokenKind::Impl, "impl"),
        tk(TokenKind::Ident, "Trait"),
        tk(TokenKind::Ident, "Struct")
    ],
    |err| assert_unexpected_token(err, TokenKind::For, TokenKind::Ident, 2)
);
error_case_items!(
    item_let_is_not_valid_top_level,
    vec![tk(TokenKind::Let, "let")],
    |err| assert_unexpected_top_level(err, TokenKind::Let)
);
error_case_items!(
    item_trait_missing_function_semicolon,
    vec![
        tk(TokenKind::Trait, "trait"),
        tk(TokenKind::Ident, "T"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Fn, "fn"),
        tk(TokenKind::Ident, "f"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::RBrace, "}")
    ],
    |err| assert_unexpected_token(err, TokenKind::Ident, TokenKind::RBrace, 7)
);

item_case!(
    item_trait_generics_and_bounds,
    vec![
        tk(TokenKind::Trait, "trait"),
        tk(TokenKind::Ident, "X"),
        tk(TokenKind::Lt, "<"),
        tk(TokenKind::Ident, "T"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "A"),
        tk(TokenKind::Plus, "+"),
        tk(TokenKind::Ident, "B"),
        tk(TokenKind::Comma, ","),
        tk(TokenKind::Ident, "U"),
        tk(TokenKind::Gt, ">"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::Trait(Trait {
        trait_name: "X".into(),
        generic_params: vec![
            GenericParam {
                name: "T".into(),
                bounds: vec![
                    TraitBound {
                        trait_name: "A".into(),
                        args: vec![]
                    },
                    TraitBound {
                        trait_name: "B".into(),
                        args: vec![]
                    }
                ]
            },
            GenericParam {
                name: "U".into(),
                bounds: vec![]
            }
        ],
        function_signatures: vec![]
    })]
);
item_case!(
    item_struct_generics_and_fields,
    vec![
        tk(TokenKind::Struct, "struct"),
        tk(TokenKind::Ident, "Box"),
        tk(TokenKind::Lt, "<"),
        tk(TokenKind::Ident, "T"),
        tk(TokenKind::Gt, ">"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Ident, "value"),
        tk(TokenKind::Colon, ":"),
        tk(TokenKind::Ident, "T"),
        tk(TokenKind::RBrace, "}")
    ],
    vec![Item::Struct(Struct {
        struct_name: "Box".into(),
        generic_params: vec![GenericParam {
            name: "T".into(),
            bounds: vec![]
        }],
        fields: vec![StructField {
            field_name: "value".into(),
            field_type: Type::Named("T".into())
        }],
        functions: vec![]
    })]
);

#[test]
fn parse_multiple_top_level_items_in_sequence() {
    let items = parse_items_from(vec![
        tk(TokenKind::Import, "import"),
        tk(TokenKind::Ident, "a"),
        tk(TokenKind::Semicolon, ";"),
        tk(TokenKind::Variant, "variant"),
        tk(TokenKind::Ident, "V"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::Ident, "X"),
        tk(TokenKind::RBrace, "}"),
        tk(TokenKind::Struct, "struct"),
        tk(TokenKind::Ident, "S"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}"),
        tk(TokenKind::Fn, "fn"),
        tk(TokenKind::Ident, "f"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}"),
        tk(TokenKind::Impl, "impl"),
        tk(TokenKind::Ident, "T"),
        tk(TokenKind::For, "for"),
        tk(TokenKind::Ident, "S"),
        tk(TokenKind::LBrace, "{"),
        tk(TokenKind::RBrace, "}"),
    ])
    .unwrap();
    assert_eq!(items.len(), 5);
}

#[test]
fn parse_skips_extra_semicolons_top_level() {
    let items = parse_items_from(vec![
        tk(TokenKind::Semicolon, ";"),
        tk(TokenKind::Semicolon, ";"),
    ])
    .unwrap();
    assert!(items.is_empty());
}

#[test]
fn parse_empty_program_with_eof() {
    let items = parse_items_from(vec![]).unwrap();
    assert!(items.is_empty());
}

#[test]
fn parse_expr_after_consuming_current_tokens_is_stable() {
    let mut p = Parser::new(toks(vec![tk(TokenKind::Int, "1")]));
    let expr = p.parse_expr().unwrap();
    dbg_eq(expr, Expr::IntLiteral("1".into()));
    assert_unexpected_eof(p.advance().unwrap_err());
}

#[test]
fn source_expr_parses_through_lexer_and_parser() {
    let expr = parse_expr_source("1 + 2 * f(3)").unwrap();
    dbg_eq(
        expr,
        Expr::BinaryOp {
            lhs: Box::new(Expr::IntLiteral("1".into())),
            binary_op: BinaryOp::Add,
            rhs: Box::new(Expr::BinaryOp {
                lhs: Box::new(Expr::IntLiteral("2".into())),
                binary_op: BinaryOp::Mul,
                rhs: Box::new(Expr::Call {
                    callee: Box::new(Expr::Variable("f".into())),
                    type_args: vec![],
                    arguments: vec![Expr::IntLiteral("3".into())],
                }),
            }),
        },
    );
}

#[test]
fn source_expr_bang_parses_as_unary_not() {
    let expr = parse_expr_source("!true").unwrap();
    dbg_eq(
        expr,
        Expr::Unary {
            op: UnaryOp::Not,
            value: Box::new(Expr::BoolLiteral(true)),
        },
    );
}

#[test]
fn source_stmt_parses_char_literal_return() {
    let stmt = parse_stmt_source("return 'x';").unwrap();
    dbg_eq(
        stmt,
        Stmt::Return(Return {
            return_value: Some(Expr::CharLiteral('x')),
        }),
    );
}

#[test]
fn source_items_parse_full_program_shape() {
    let items = parse_items_source(
        r#"
        import std.io;
        global counter: int = 0;
        const answer: int = 42;
        variant Option { Some, None, }
        struct Point<T> { x: int, y: int, }
        trait Display { fn show(self: Self) string; }
        fn main() int { let x: int = 1; return x; }
        "#,
    )
    .unwrap();

    assert_eq!(items.len(), 7);
    dbg_eq(
        &items[0],
        &Item::Import(Import {
            import_name: "std.io".into(),
        }),
    );
    dbg_eq(
        &items[1],
        &Item::GlobalVar(GlobalVar {
            var_name: "counter".into(),
            var_type: Type::Named("int".into()),
            value: Expr::IntLiteral("0".into()),
        }),
    );
    dbg_eq(
        &items[2],
        &Item::ConstVar(ConstVar {
            var_name: "answer".into(),
            var_type: Type::Named("int".into()),
            value: Expr::IntLiteral("42".into()),
        }),
    );
    dbg_eq(
        &items[3],
        &Item::Variant(Variant {
            variant_name: "Option".into(),
            cases: vec!["Some".into(), "None".into()],
        }),
    );
    match &items[6] {
        Item::Function(function) => {
            assert_eq!(function.name, "main");
            assert_eq!(function.body.statements.len(), 2);
        }
        other => panic!("expected function item, got {other:?}"),
    }
}

#[test]
fn source_items_parse_global_and_const_declarations() {
    let items = parse_items_source("global counter: int = 0; const answer: int = 42;").unwrap();

    dbg_eq(
        items,
        vec![
            Item::GlobalVar(GlobalVar {
                var_name: "counter".into(),
                var_type: Type::Named("int".into()),
                value: Expr::IntLiteral("0".into()),
            }),
            Item::ConstVar(ConstVar {
                var_name: "answer".into(),
                var_type: Type::Named("int".into()),
                value: Expr::IntLiteral("42".into()),
            }),
        ],
    );
}

// ===== Postfix: field access, indexing, method calls =====

#[test]
fn field_access_chains_via_dot() {
    let expr = parse_expr_source("a.b.c").unwrap();
    dbg_eq(
        expr,
        Expr::FieldAccess {
            object: Box::new(Expr::FieldAccess {
                object: Box::new(Expr::Variable("a".into())),
                field_name: "b".into(),
            }),
            field_name: "c".into(),
        },
    );
}

#[test]
fn method_call_is_field_access_then_call() {
    let expr = parse_expr_source("obj.method(1, 2)").unwrap();
    dbg_eq(
        expr,
        Expr::Call {
            callee: Box::new(Expr::FieldAccess {
                object: Box::new(Expr::Variable("obj".into())),
                field_name: "method".into(),
            }),
            type_args: vec![],
            arguments: vec![Expr::IntLiteral("1".into()), Expr::IntLiteral("2".into())],
        },
    );
}

#[test]
fn variant_case_access_is_field_access() {
    // Per the syntax doc, `Option.Some` is variant case construction; at the
    // parser level it lowers to a regular field access on `Option`. Disambiguating
    // variant types from real values is the job of a later semantic pass.
    let expr = parse_expr_source("Option.Some").unwrap();
    dbg_eq(
        expr,
        Expr::FieldAccess {
            object: Box::new(Expr::Variable("Option".into())),
            field_name: "Some".into(),
        },
    );
}

// ===== Generics on call sites (turbofish) =====

// #[test]
// fn turbofish_passes_type_args_to_call() {
//     let expr = parse_expr_source("make::<int>(5)").unwrap();
//     dbg_eq(
//         expr,
//         Expr::Call {
//             callee: Box::new(Expr::Variable("make".into())),
//             type_args: vec![Type::Named("int".into())],
//             arguments: vec![Expr::IntLiteral("5".into())],
//         },
//     );
// }
//
// #[test]
// fn turbofish_supports_multiple_type_args() {
//     let expr = parse_expr_source("pair::<int, bool>()").unwrap();
//     dbg_eq(
//         expr,
//         Expr::Call {
//             callee: Box::new(Expr::Variable("pair".into())),
//             type_args: vec![Type::Named("int".into()), Type::Named("bool".into())],
//             arguments: vec![],
//         },
//     );
// }

// ===== Assignment to lvalues =====

#[test]
fn assignment_to_field_is_allowed() {
    let expr = parse_expr_source("p.name = \"x\"").unwrap();
    dbg_eq(
        expr,
        Expr::Assign {
            target: Box::new(Expr::FieldAccess {
                object: Box::new(Expr::Variable("p".into())),
                field_name: "name".into(),
            }),
            value: Box::new(Expr::StringLiteral("x".into())),
        },
    );
}

#[test]
fn compound_assignment_to_field_is_allowed() {
    let expr = parse_expr_source("p.count += 1").unwrap();
    dbg_eq(
        expr,
        Expr::BinaryOpAssign {
            target: Box::new(Expr::FieldAccess {
                object: Box::new(Expr::Variable("p".into())),
                field_name: "count".into(),
            }),
            binary_op: BinaryOp::Add,
            value: Box::new(Expr::IntLiteral("1".into())),
        },
    );
}

#[test]
fn assignment_to_call_result_is_rejected() {
    let err = parse_expr_from(vec![
        tk(TokenKind::Ident, "f"),
        tk(TokenKind::LParen, "("),
        tk(TokenKind::RParen, ")"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Int, "1"),
    ])
    .unwrap_err();
    assert_invalid_assignment_target(err);
}

#[test]
fn assignment_to_int_literal_is_rejected() {
    let err = parse_expr_from(vec![
        tk(TokenKind::Int, "5"),
        tk(TokenKind::Eq, "="),
        tk(TokenKind::Int, "1"),
    ])
    .unwrap_err();
    assert_invalid_assignment_target(err);
}

// ===== parse_fn_def: missing return type or brace surfaces a real error =====

#[test]
fn fn_with_no_return_type_and_no_body_brace_errors_on_type() {
    // After `f()` the parser should *not* silently treat the missing token as
    // "no return type"; it should fail trying to read a return-type identifier.
    let err = parse_items_source("fn f()").unwrap_err();
    match err {
        ParseError::UnexpectedToken {
            expected: TokenKind::Ident,
            found: TokenKind::Eof,
            ..
        } => {}
        other => panic!("expected UnexpectedToken(Ident, Eof), got {other:?}"),
    }
}

#[test]
fn fn_with_explicit_return_type_then_body_parses_cleanly() {
    let items = parse_items_source("fn add(x: int, y: int) int { return x + y; }").unwrap();
    match &items[..] {
        [Item::Function(f)] => {
            assert_eq!(f.name, "add");
            assert!(matches!(&f.return_type, Some(Type::Named(n)) if n == "int"));
            assert_eq!(f.params.len(), 2);
        }
        other => panic!("expected single function item, got {other:?}"),
    }
}

#[test]
fn fn_without_return_type_but_with_body_parses_as_void() {
    let items = parse_items_source("fn noop() {}").unwrap();
    match &items[..] {
        [Item::Function(f)] => {
            assert_eq!(f.name, "noop");
            assert!(f.return_type.is_none());
        }
        other => panic!("expected single function item, got {other:?}"),
    }
}

// ===== parse_operator_name validation =====

#[test]
fn operator_function_with_known_symbol_parses() {
    let items = parse_items_source("fn operator + add(rhs: int) int { return rhs; }").unwrap();
    match &items[..] {
        [Item::Function(f)] => {
            assert_eq!(f.operator.as_deref(), Some("+"));
            assert_eq!(f.name, "add");
        }
        other => panic!("expected function item, got {other:?}"),
    }
}

#[test]
fn operator_function_with_unknown_symbol_is_rejected() {
    // `+~` lexes as two separate operator tokens but the combined name is not
    // in the known-operator set.
    let err = parse_items_source("fn operator +~ inc() {}").unwrap_err();
    match err {
        ParseError::InvalidOperatorName { name, .. } => assert_eq!(name, "+~"),
        other => panic!("expected InvalidOperatorName, got {other:?}"),
    }
}

#[test]
fn operator_function_with_no_symbol_is_rejected() {
    let err = parse_items_source("fn operator add() {}").unwrap_err();
    match err {
        ParseError::EmptyOperatorName { .. } => {}
        other => panic!("expected EmptyOperatorName, got {other:?}"),
    }
}

#[test]
fn operator_function_with_index_brackets_parses() {
    let items = parse_items_source("fn operator [] get(rhs: int) int { return rhs; }").unwrap();
    match &items[..] {
        [Item::Function(f)] => {
            assert_eq!(f.operator.as_deref(), Some("[]"));
        }
        other => panic!("expected function item, got {other:?}"),
    }
}

#[test]
fn plain_if_does_not_become_if_let() {
    let stmt = parse_stmt_source("if x { return 1; }").unwrap();
    assert!(matches!(stmt, Stmt::If(_)));
}

// ===== print_items must handle every Item variant without panicking =====

#[test]
fn print_items_handles_every_item_variant() {
    let items = vec![
        Item::GlobalVar(GlobalVar {
            var_name: "g".into(),
            var_type: Type::Named("int".into()),
            value: Expr::IntLiteral("0".into()),
        }),
        Item::ConstVar(ConstVar {
            var_name: "c".into(),
            var_type: Type::Named("int".into()),
            value: Expr::IntLiteral("1".into()),
        }),
        Item::Function(Function {
            name: "f".into(),
            generic_params: vec![],
            params: vec![],
            body: Block { statements: vec![] },
            return_type: None,
            operator: None,
        }),
        Item::Trait(Trait {
            trait_name: "T".into(),
            generic_params: vec![],
            function_signatures: vec![],
        }),
        Item::Struct(Struct {
            struct_name: "S".into(),
            generic_params: vec![],
            fields: vec![],
            functions: vec![],
        }),
        Item::Variant(Variant {
            variant_name: "V".into(),
            cases: vec!["A".into(), "B".into()],
        }),
        Item::TraitImplementation(TraitImplementation {
            generic_params: vec![],
            trait_name: "T".into(),
            trait_args: vec![],
            struct_name: "S".into(),
            struct_args: vec![],
            functions: vec![],
        }),
        Item::Import(Import {
            import_name: "std.io".into(),
        }),
        Item::Asm(Asm {
            asm_code: "mov rax, 0".into(),
        }),
    ];
    // Must not panic; the previous implementation's `_ => todo!()` arm
    // would have triggered on Item::Asm.
    print_items(&items);
}
