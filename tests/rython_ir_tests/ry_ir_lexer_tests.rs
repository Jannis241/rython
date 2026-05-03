use rython_to_ir::lexer::{Lexer, Token, TokenKind};

fn lex(input: &str) -> Vec<Token> {
    Lexer::create_tokens(input.to_string())
}

fn assert_tokens(input: &str, expected: &[(TokenKind, &str)]) {
    let actual = lex(input);
    let expected: Vec<Token> = expected
        .iter()
        .map(|(kind, value)| Token::new(kind.clone(), (*value).to_string()))
        .collect();

    assert_eq!(actual, expected, "input: {input:?}");
}

#[test]
fn empty_input_returns_no_tokens() {
    assert_tokens("", &[]);
}

#[test]
fn whitespace_only_input_returns_eof() {
    assert_tokens(" \n\t\r", &[(TokenKind::Eof, "EOF")]);
}

#[test]
fn skips_whitespace_between_tokens() {
    assert_tokens(
        "let\n\tanswer  \r\n42",
        &[
            (TokenKind::Let, "let"),
            (TokenKind::Ident, "answer"),
            (TokenKind::Int, "42"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn lexes_all_keywords() {
    assert_tokens(
        "if else return loop while any let fn this in import struct trait impl for continue break variant and or operator asm",
        &[
            (TokenKind::If, "if"),
            (TokenKind::Else, "else"),
            (TokenKind::Return, "return"),
            (TokenKind::Loop, "loop"),
            (TokenKind::While, "while"),
            (TokenKind::Any, "any"),
            (TokenKind::Let, "let"),
            (TokenKind::Fn, "fn"),
            (TokenKind::This, "this"),
            (TokenKind::In, "in"),
            (TokenKind::Import, "import"),
            (TokenKind::Struct, "struct"),
            (TokenKind::Trait, "trait"),
            (TokenKind::Impl, "impl"),
            (TokenKind::For, "for"),
            (TokenKind::Continue, "continue"),
            (TokenKind::Break, "break"),
            (TokenKind::Variant, "variant"),
            (TokenKind::And, "and"),
            (TokenKind::Or, "or"),
            (TokenKind::Operator, "operator"),
            (TokenKind::Asm, "asm"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn keywords_are_case_sensitive() {
    assert_tokens(
        "If ELSE Return ASM",
        &[
            (TokenKind::Ident, "If"),
            (TokenKind::Ident, "ELSE"),
            (TokenKind::Ident, "Return"),
            (TokenKind::Ident, "ASM"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn keyword_prefixes_and_suffixes_are_identifiers() {
    assert_tokens(
        "ifx xif let_value fn2 operator_overload",
        &[
            (TokenKind::Ident, "ifx"),
            (TokenKind::Ident, "xif"),
            (TokenKind::Ident, "let_value"),
            (TokenKind::Ident, "fn2"),
            (TokenKind::Ident, "operator_overload"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn lexes_identifiers_with_letters_digits_and_underscores_after_first_char() {
    assert_tokens(
        "foo bar_1 CamelCase snake_case ABC xyz",
        &[
            (TokenKind::Ident, "foo"),
            (TokenKind::Ident, "bar_1"),
            (TokenKind::Ident, "CamelCase"),
            (TokenKind::Ident, "snake_case"),
            (TokenKind::Ident, "ABC"),
            (TokenKind::Ident, "xyz"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn lexes_integer_literals() {
    assert_tokens(
        "0 1 42 007 1234567890",
        &[
            (TokenKind::Int, "0"),
            (TokenKind::Int, "1"),
            (TokenKind::Int, "42"),
            (TokenKind::Int, "007"),
            (TokenKind::Int, "1234567890"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn lexes_float_literals() {
    assert_tokens(
        "0.0 1.23 42. 123.456",
        &[
            (TokenKind::Float, "0.0"),
            (TokenKind::Float, "1.23"),
            (TokenKind::Float, "42."),
            (TokenKind::Float, "123.456"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn keeps_multiple_dots_inside_number_literal() {
    assert_tokens("1.2.3", &[(TokenKind::Float, "1.2.3"), (TokenKind::Eof, "EOF")]);
}

#[test]
fn stops_number_before_identifier_text() {
    assert_tokens(
        "123abc",
        &[
            (TokenKind::Int, "123"),
            (TokenKind::Ident, "abc"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn lexes_string_literals() {
    assert_tokens(
        r#""hello" "" "hello world" "123 + if""#,
        &[
            (TokenKind::StringLiteral, "hello"),
            (TokenKind::StringLiteral, ""),
            (TokenKind::StringLiteral, "hello world"),
            (TokenKind::StringLiteral, "123 + if"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn string_literal_keeps_newlines_and_tabs_inside() {
    assert_tokens(
        "\"line 1\n\tline 2\"",
        &[(TokenKind::StringLiteral, "line 1\n\tline 2"), (TokenKind::Eof, "EOF")],
    );
}

#[test]
fn unterminated_string_returns_string_token_until_end() {
    assert_tokens(
        "\"unterminated",
        &[
            (TokenKind::StringLiteral, "unterminated"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn lexes_single_character_operators() {
    assert_tokens(
        "+ - * / % = < > & | ^ ~",
        &[
            (TokenKind::Plus, "+"),
            (TokenKind::Minus, "-"),
            (TokenKind::Star, "*"),
            (TokenKind::Slash, "/"),
            (TokenKind::Percent, "%"),
            (TokenKind::Eq, "="),
            (TokenKind::Lt, "<"),
            (TokenKind::Gt, ">"),
            (TokenKind::Amp, "&"),
            (TokenKind::Pipe, "|"),
            (TokenKind::Caret, "^"),
            (TokenKind::Tilde, "~"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn lexes_compound_operators() {
    assert_tokens(
        "+= -= *= /= == != <= >= << >>",
        &[
            (TokenKind::PlusEq, "+="),
            (TokenKind::MinusEq, "-="),
            (TokenKind::StarEq, "*="),
            (TokenKind::SlashEq, "/="),
            (TokenKind::EqEq, "=="),
            (TokenKind::BangEq, "!="),
            (TokenKind::LtEq, "<="),
            (TokenKind::GtEq, ">="),
            (TokenKind::LtLt, "<<"),
            (TokenKind::GtGt, ">>"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn lexes_delimiters_and_punctuation() {
    assert_tokens(
        "( ) { } [ ] , ; : .",
        &[
            (TokenKind::LParen, "("),
            (TokenKind::RParen, ")"),
            (TokenKind::LBrace, "{"),
            (TokenKind::RBrace, "}"),
            (TokenKind::LBracket, "["),
            (TokenKind::RBracket, "]"),
            (TokenKind::Comma, ","),
            (TokenKind::Semicolon, ";"),
            (TokenKind::Colon, ":"),
            (TokenKind::Dot, "."),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn lexes_small_function_shape() {
    assert_tokens(
        "fn add(a: int, b: int) { return a + b; }",
        &[
            (TokenKind::Fn, "fn"),
            (TokenKind::Ident, "add"),
            (TokenKind::LParen, "("),
            (TokenKind::Ident, "a"),
            (TokenKind::Colon, ":"),
            (TokenKind::Ident, "int"),
            (TokenKind::Comma, ","),
            (TokenKind::Ident, "b"),
            (TokenKind::Colon, ":"),
            (TokenKind::Ident, "int"),
            (TokenKind::RParen, ")"),
            (TokenKind::LBrace, "{"),
            (TokenKind::Return, "return"),
            (TokenKind::Ident, "a"),
            (TokenKind::Plus, "+"),
            (TokenKind::Ident, "b"),
            (TokenKind::Semicolon, ";"),
            (TokenKind::RBrace, "}"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
#[should_panic(expected = "Unexpected token '!'")]
fn bare_bang_panics() {
    lex("!");
}

#[test]
#[should_panic(expected = "Could not convert")]
fn unknown_character_panics() {
    lex("@");
}

#[test]
#[should_panic(expected = "Could not convert")]
fn identifier_cannot_start_with_underscore() {
    lex("_hidden");
}

#[test]
#[should_panic(expected = "Could not convert")]
fn non_ascii_identifier_start_panics() {
    lex("äpfel");
}
