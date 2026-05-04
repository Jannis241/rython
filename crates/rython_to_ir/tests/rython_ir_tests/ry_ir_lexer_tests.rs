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
fn empty_input_returns_eof() {
    assert_tokens("", &[(TokenKind::Eof, "EOF")]);
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
fn skips_slash_comments() {
    assert_tokens(
        "let x // comment\n// whole line\nreturn y // trailing comment",
        &[
            (TokenKind::Let, "let"),
            (TokenKind::Ident, "x"),
            (TokenKind::Return, "return"),
            (TokenKind::Ident, "y"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
#[should_panic(expected = "Could not convert")]
fn hash_comments_are_not_supported() {
    lex("# comment");
}

#[test]
fn lexes_all_keywords() {
    assert_tokens(
        "true false char null if else return loop while any let fn this in import struct trait impl for continue break variant and or operator asm",
        &[
            (TokenKind::True, "true"),
            (TokenKind::False, "false"),
            (TokenKind::Char, "char"),
            (TokenKind::Null, "null"),
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
        "True FALSE Char Null If ELSE Return ASM",
        &[
            (TokenKind::Ident, "True"),
            (TokenKind::Ident, "FALSE"),
            (TokenKind::Ident, "Char"),
            (TokenKind::Ident, "Null"),
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
        "truex xtrue null_value ifx xif let_value fn2 operator_overload",
        &[
            (TokenKind::Ident, "truex"),
            (TokenKind::Ident, "xtrue"),
            (TokenKind::Ident, "null_value"),
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
        "foo bar_1 CamelCase snake_case ABC xyz zed Zed",
        &[
            (TokenKind::Ident, "foo"),
            (TokenKind::Ident, "bar_1"),
            (TokenKind::Ident, "CamelCase"),
            (TokenKind::Ident, "snake_case"),
            (TokenKind::Ident, "ABC"),
            (TokenKind::Ident, "xyz"),
            (TokenKind::Ident, "zed"),
            (TokenKind::Ident, "Zed"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn lexes_integer_literals() {
    assert_tokens(
        "0 1 9 42 007 9876543210",
        &[
            (TokenKind::Int, "0"),
            (TokenKind::Int, "1"),
            (TokenKind::Int, "9"),
            (TokenKind::Int, "42"),
            (TokenKind::Int, "007"),
            (TokenKind::Int, "9876543210"),
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
    assert_tokens(
        "1.2.3",
        &[(TokenKind::Float, "1.2.3"), (TokenKind::Eof, "EOF")],
    );
}

#[test]
fn keeps_dot_sequences_inside_number_literal() {
    assert_tokens(
        "0..1 1.foo 1..",
        &[
            (TokenKind::Float, "0..1"),
            (TokenKind::Float, "1."),
            (TokenKind::Ident, "foo"),
            (TokenKind::Float, "1.."),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn dot_prefixed_decimal_starts_with_dot_token() {
    assert_tokens(
        ".5",
        &[
            (TokenKind::Dot, "."),
            (TokenKind::Int, "5"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
#[should_panic(expected = "Could not convert")]
fn number_literals_do_not_allow_underscores() {
    lex("123_456");
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
fn string_literal_handles_escape_sequences() {
    assert_tokens(
        r#""quote: \"" "slash: \\" "line\nnext" "tab\tend""#,
        &[
            (TokenKind::StringLiteral, "quote: \""),
            (TokenKind::StringLiteral, "slash: \\"),
            (TokenKind::StringLiteral, "line\nnext"),
            (TokenKind::StringLiteral, "tab\tend"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn string_literal_handles_carriage_return_escape() {
    assert_tokens(
        r#""line\rnext""#,
        &[
            (TokenKind::StringLiteral, "line\rnext"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn string_literal_keeps_unknown_escape_backslash() {
    assert_tokens(
        r#""unknown \x escape""#,
        &[
            (TokenKind::StringLiteral, "unknown \\x escape"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn string_literal_does_not_start_comment_inside_string() {
    assert_tokens(
        r#""a // not a comment" return"#,
        &[
            (TokenKind::StringLiteral, "a // not a comment"),
            (TokenKind::Return, "return"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn string_literal_keeps_single_quotes_as_text() {
    assert_tokens(
        r#""'a' and ''""#,
        &[
            (TokenKind::StringLiteral, "'a' and ''"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn string_literal_keeps_final_backslash_in_unterminated_string() {
    assert_tokens(
        r#""abc\"#,
        &[(TokenKind::StringLiteral, "abc\\"), (TokenKind::Eof, "EOF")],
    );
}

#[test]
fn lexes_char_literals() {
    assert_tokens(
        r#"'a' '\n' '\'' '\\'"#,
        &[
            (TokenKind::Char, "a"),
            (TokenKind::Char, "\n"),
            (TokenKind::Char, "'"),
            (TokenKind::Char, "\\"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn char_literal_keeps_unknown_escape_backslash() {
    assert_tokens(
        r#"'\q'"#,
        &[(TokenKind::Char, "\\q"), (TokenKind::Eof, "EOF")],
    );
}

#[test]
fn char_literal_keeps_actual_newline() {
    assert_tokens("'\n'", &[(TokenKind::Char, "\n"), (TokenKind::Eof, "EOF")]);
}

#[test]
fn char_literals_can_be_adjacent_without_whitespace() {
    assert_tokens(
        "'a''b'",
        &[
            (TokenKind::Char, "a"),
            (TokenKind::Char, "b"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn char_literal_handles_carriage_return_escape() {
    assert_tokens(
        r#"'\r'"#,
        &[(TokenKind::Char, "\r"), (TokenKind::Eof, "EOF")],
    );
}

#[test]
fn char_literal_accepts_empty_multiple_chars_and_unterminated_values() {
    assert_tokens(
        "'' 'ab' 'unterminated",
        &[
            (TokenKind::Char, ""),
            (TokenKind::Char, "ab"),
            (TokenKind::Char, "unterminated"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn string_literal_keeps_newlines_and_tabs_inside() {
    assert_tokens(
        "\"line 1\n\tline 2\"",
        &[
            (TokenKind::StringLiteral, "line 1\n\tline 2"),
            (TokenKind::Eof, "EOF"),
        ],
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
        "+ - * / % = ! < > & | ^ ~",
        &[
            (TokenKind::Plus, "+"),
            (TokenKind::Minus, "-"),
            (TokenKind::Star, "*"),
            (TokenKind::Slash, "/"),
            (TokenKind::Percent, "%"),
            (TokenKind::Eq, "="),
            (TokenKind::Bang, "!"),
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
fn compound_operators_use_longest_supported_prefix() {
    assert_tokens(
        "=== !== <<= >>=",
        &[
            (TokenKind::EqEq, "=="),
            (TokenKind::Eq, "="),
            (TokenKind::BangEq, "!="),
            (TokenKind::Eq, "="),
            (TokenKind::LtLt, "<<"),
            (TokenKind::Eq, "="),
            (TokenKind::GtGt, ">>"),
            (TokenKind::Eq, "="),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn lexes_adjacent_slash_operators_and_comments() {
    assert_tokens(
        "/ /= //// comment\n/",
        &[
            (TokenKind::Slash, "/"),
            (TokenKind::SlashEq, "/="),
            (TokenKind::Slash, "/"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn repeated_ampersands_and_pipes_are_separate_tokens() {
    assert_tokens(
        "&& ||",
        &[
            (TokenKind::Amp, "&"),
            (TokenKind::Amp, "&"),
            (TokenKind::Pipe, "|"),
            (TokenKind::Pipe, "|"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn percent_equal_is_percent_then_equal() {
    assert_tokens(
        "%=",
        &[
            (TokenKind::Percent, "%"),
            (TokenKind::Eq, "="),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn bang_before_other_character_is_separate_bang_token() {
    assert_tokens(
        "!< !true",
        &[
            (TokenKind::Bang, "!"),
            (TokenKind::Lt, "<"),
            (TokenKind::Bang, "!"),
            (TokenKind::True, "true"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn slash_comment_at_end_of_input_finishes_with_eof() {
    assert_tokens(
        "let x // comment at eof",
        &[
            (TokenKind::Let, "let"),
            (TokenKind::Ident, "x"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn block_comment_syntax_is_lexed_as_regular_tokens() {
    assert_tokens(
        "/* comment */",
        &[
            (TokenKind::Slash, "/"),
            (TokenKind::Star, "*"),
            (TokenKind::Ident, "comment"),
            (TokenKind::Star, "*"),
            (TokenKind::Slash, "/"),
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
fn bare_bang_is_tokenized() {
    assert_tokens("!", &[(TokenKind::Bang, "!"), (TokenKind::Eof, "EOF")]);
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

#[test]
#[should_panic(expected = "Could not convert")]
fn non_ascii_identifier_body_panics() {
    lex("aä");
}

#[test]
#[should_panic(expected = "Could not convert")]
fn unicode_whitespace_panics() {
    lex("let\u{00a0}x");
}
