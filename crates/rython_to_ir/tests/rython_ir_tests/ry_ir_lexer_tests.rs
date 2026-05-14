use rython_to_ir::lexer::{Lexer, Token, TokenKind};

fn lex(input: &str) -> Vec<Token> {
    Lexer::create_tokens(input.to_string()).expect("lexing failed")
}

fn assert_tokens(input: &str, expected: &[(TokenKind, &str)]) {
    let actual = lex(input);
    let actual_pairs: Vec<(TokenKind, String)> = actual
        .iter()
        .map(|t| (t.kind.clone(), t.value.clone()))
        .collect();
    let expected_pairs: Vec<(TokenKind, String)> = expected
        .iter()
        .map(|(k, v)| (k.clone(), (*v).to_string()))
        .collect();

    assert_eq!(actual_pairs, expected_pairs, "input: {input:?}");
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
fn hash_comments_are_not_supported() {
    assert!(Lexer::create_tokens("# comment".to_string()).is_err());
}

#[test]
fn lexes_all_keywords() {
    assert_tokens(
        "true false char null if else return loop while any let fn this in import struct trait global const impl for continue break variant and or operator asm {a}",
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
            (TokenKind::Global, "global"),
            (TokenKind::Const, "const"),
            (TokenKind::Impl, "impl"),
            (TokenKind::For, "for"),
            (TokenKind::Continue, "continue"),
            (TokenKind::Break, "break"),
            (TokenKind::Variant, "variant"),
            (TokenKind::And, "and"),
            (TokenKind::Or, "or"),
            (TokenKind::Operator, "operator"),
            (TokenKind::Asm, "a"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn keywords_are_case_sensitive() {
    assert_tokens(
        "True FALSE Char Null If ELSE Return Global CONST ASM",
        &[
            (TokenKind::Ident, "True"),
            (TokenKind::Ident, "FALSE"),
            (TokenKind::Ident, "Char"),
            (TokenKind::Ident, "Null"),
            (TokenKind::Ident, "If"),
            (TokenKind::Ident, "ELSE"),
            (TokenKind::Ident, "Return"),
            (TokenKind::Ident, "Global"),
            (TokenKind::Ident, "CONST"),
            (TokenKind::Ident, "ASM"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn keyword_prefixes_and_suffixes_are_identifiers() {
    assert_tokens(
        "truex xtrue null_value ifx xif let_value fn2 global_value const2 xglobal xconst operator_overload",
        &[
            (TokenKind::Ident, "truex"),
            (TokenKind::Ident, "xtrue"),
            (TokenKind::Ident, "null_value"),
            (TokenKind::Ident, "ifx"),
            (TokenKind::Ident, "xif"),
            (TokenKind::Ident, "let_value"),
            (TokenKind::Ident, "fn2"),
            (TokenKind::Ident, "global_value"),
            (TokenKind::Ident, "const2"),
            (TokenKind::Ident, "xglobal"),
            (TokenKind::Ident, "xconst"),
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
fn stops_float_literal_before_second_dot() {
    assert_tokens(
        "1.2.3",
        &[
            (TokenKind::Float, "1.2"),
            (TokenKind::Dot, "."),
            (TokenKind::Int, "3"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn stops_float_literal_after_first_dot_in_dot_sequences() {
    assert_tokens(
        "0..1 1.foo 1..",
        &[
            (TokenKind::Float, "0."),
            (TokenKind::Dot, "."),
            (TokenKind::Int, "1"),
            (TokenKind::Float, "1."),
            (TokenKind::Ident, "foo"),
            (TokenKind::Float, "1."),
            (TokenKind::Dot, "."),
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
fn number_literals_allow_underscores_as_digit_separators() {
    assert_tokens(
        "123_456 1_000_000",
        &[
            (TokenKind::Int, "123456"),
            (TokenKind::Int, "1000000"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn underscore_not_followed_by_digit_ends_number() {
    assert_tokens(
        "1_abc",
        &[
            (TokenKind::Int, "1"),
            (TokenKind::Ident, "_abc"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn lexes_hex_binary_octal_literals() {
    assert_tokens(
        "0xFF 0xdead_beef 0b1010 0b1_0 0o17 0o7_7",
        &[
            (TokenKind::Int, "0xFF"),
            (TokenKind::Int, "0xdeadbeef"),
            (TokenKind::Int, "0b1010"),
            (TokenKind::Int, "0b10"),
            (TokenKind::Int, "0o17"),
            (TokenKind::Int, "0o77"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn lexes_scientific_float_literals() {
    assert_tokens(
        "1e10 1.5e3 2.5e-3 4.0E+2",
        &[
            (TokenKind::Float, "1e10"),
            (TokenKind::Float, "1.5e3"),
            (TokenKind::Float, "2.5e-3"),
            (TokenKind::Float, "4.0e+2"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn hex_prefix_with_no_digits_errors() {
    assert!(Lexer::create_tokens("0x".to_string()).is_err());
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
    // \q is not a recognised escape; the backslash stays in the literal.
    assert_tokens(
        r#""unknown \q escape""#,
        &[
            (TokenKind::StringLiteral, "unknown \\q escape"),
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
fn unterminated_string_with_trailing_backslash_errors() {
    assert!(Lexer::create_tokens(r#""abc\"#.to_string()).is_err());
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
fn char_literal_with_unknown_escape_errors() {
    // unknown escape keeps the backslash, which turns the literal into two
    // characters and rejects it.
    assert!(Lexer::create_tokens(r#"'\q'"#.to_string()).is_err());
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
fn empty_char_literal_errors() {
    assert!(Lexer::create_tokens("''".to_string()).is_err());
}

#[test]
fn multi_char_literal_errors() {
    assert!(Lexer::create_tokens("'ab'".to_string()).is_err());
}

#[test]
fn unterminated_char_literal_errors() {
    assert!(Lexer::create_tokens("'unterminated".to_string()).is_err());
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
fn unterminated_string_errors() {
    assert!(Lexer::create_tokens("\"unterminated".to_string()).is_err());
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
fn lexes_null_keyword() {
    assert_tokens(
        "null nullx xnull null_value",
        &[
            (TokenKind::Null, "null"),
            (TokenKind::Ident, "nullx"),
            (TokenKind::Ident, "xnull"),
            (TokenKind::Ident, "null_value"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn lexes_not_keyword() {
    assert_tokens(
        "not notx xnot",
        &[
            (TokenKind::Not, "not"),
            (TokenKind::Ident, "notx"),
            (TokenKind::Ident, "xnot"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn double_ampersand_and_double_pipe_lex_as_logical_operators() {
    assert_tokens(
        "&& ||",
        &[
            (TokenKind::AmpAmp, "&&"),
            (TokenKind::PipePipe, "||"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn triple_ampersand_lexes_as_double_then_single() {
    assert_tokens(
        "&&&",
        &[
            (TokenKind::AmpAmp, "&&"),
            (TokenKind::Amp, "&"),
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
fn skips_block_comments() {
    assert_tokens(
        "let /* comment */ x",
        &[
            (TokenKind::Let, "let"),
            (TokenKind::Ident, "x"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn block_comment_can_span_multiple_lines() {
    assert_tokens(
        "let /* multi\n line\n comment */ x",
        &[
            (TokenKind::Let, "let"),
            (TokenKind::Ident, "x"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn unterminated_block_comment_errors() {
    assert!(Lexer::create_tokens("/* never closed".to_string()).is_err());
}

#[test]
fn lexes_escape_sequences_null_hex_unicode() {
    assert_tokens(
        r#""null:\0 hex:\x41 uni:\u{1F600}""#,
        &[
            (TokenKind::StringLiteral, "null:\0 hex:A uni:\u{1F600}"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn invalid_hex_escape_errors() {
    assert!(Lexer::create_tokens(r#""\xZZ""#.to_string()).is_err());
}

#[test]
fn invalid_unicode_escape_errors() {
    assert!(Lexer::create_tokens(r#""\u{GGG}""#.to_string()).is_err());
}

#[test]
fn asm_body_with_inner_braces_tracks_balance() {
    let tokens = lex("asm { mov %x, { 1 + 2 } };");
    assert_eq!(tokens[0].kind, TokenKind::Asm);
    assert_eq!(tokens[0].value, " mov %x, { 1 + 2 } ");
    assert_eq!(tokens[1].kind, TokenKind::Semicolon);
}

#[test]
fn asm_unterminated_errors() {
    assert!(Lexer::create_tokens("asm { mov rax, 1".to_string()).is_err());
}

#[test]
fn asm_without_brace_errors() {
    assert!(Lexer::create_tokens("asm hello".to_string()).is_err());
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
fn unknown_character_panics() {
    assert!(Lexer::create_tokens("@".to_string()).is_err());
}

#[test]
fn identifier_can_start_with_underscore() {
    assert_tokens(
        "_hidden",
        &[(TokenKind::Ident, "_hidden"), (TokenKind::Eof, "EOF")],
    );
}

#[test]
fn bare_underscore_lexes_as_wildcard_token() {
    assert_tokens(
        "_",
        &[(TokenKind::Underscore, "_"), (TokenKind::Eof, "EOF")],
    );
}

#[test]
fn non_ascii_identifier_start_panics() {
    assert!(Lexer::create_tokens("äpfel".to_string()).is_err());
}

#[test]
fn non_ascii_identifier_body_panics() {
    assert!(Lexer::create_tokens("aä".to_string()).is_err());
}

#[test]
fn unicode_whitespace_panics() {
    assert!(Lexer::create_tokens("let\u{00a0}x".to_string()).is_err());
}
