use rython_to_ir::lexer::{Lexer, LexingError, TokenKind};

use super::common::{assert_tokens, lex};

#[test]
fn empty_and_whitespace_only_sources_produce_one_eof() {
    for source in ["", " \n\t\r", "// trailing comment", "/* block */"] {
        assert_tokens(source, &[(TokenKind::Eof, "EOF")]);
    }
}

#[test]
fn lexes_supported_keywords_except_removed_null() {
    assert_tokens(
        "true false char if else return loop while any let fn this in import struct trait global const impl for continue break variant and or not operator",
        &[
            (TokenKind::True, "true"),
            (TokenKind::False, "false"),
            (TokenKind::Char, "char"),
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
            (TokenKind::Not, "not"),
            (TokenKind::Operator, "operator"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn identifiers_are_ascii_and_keywords_are_exact_case_sensitive_matches() {
    assert_tokens(
        "_ value value2 let_value True FALSE operator2",
        &[
            (TokenKind::Underscore, "_"),
            (TokenKind::Ident, "value"),
            (TokenKind::Ident, "value2"),
            (TokenKind::Ident, "let_value"),
            (TokenKind::Ident, "True"),
            (TokenKind::Ident, "FALSE"),
            (TokenKind::Ident, "operator2"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn lexes_numeric_literals_without_interpreting_their_semantic_value() {
    assert_tokens(
        "0 42 1_000 0xFF 0xdead_beef 0b1010 0o77 1.25 42. 1e10 2.5e-3",
        &[
            (TokenKind::Int, "0"),
            (TokenKind::Int, "42"),
            (TokenKind::Int, "1000"),
            (TokenKind::Int, "0xFF"),
            (TokenKind::Int, "0xdeadbeef"),
            (TokenKind::Int, "0b1010"),
            (TokenKind::Int, "0o77"),
            (TokenKind::Float, "1.25"),
            (TokenKind::Float, "42."),
            (TokenKind::Float, "1e10"),
            (TokenKind::Float, "2.5e-3"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn malformed_prefixed_numbers_report_number_errors() {
    for source in ["0x", "0b", "0o"] {
        assert!(matches!(
            Lexer::create_tokens(source.to_string()),
            Err(LexingError::InvalidNumber(_, _))
        ));
    }
}

#[test]
fn lexes_string_and_char_literals_with_escapes_and_unicode_values() {
    assert_tokens(
        r#""hello" "line\nnext" "Grüße" 'x' '\n' 'ä'"#,
        &[
            (TokenKind::StringLiteral, "hello"),
            (TokenKind::StringLiteral, "line\nnext"),
            (TokenKind::StringLiteral, "Grüße"),
            (TokenKind::Char, "x"),
            (TokenKind::Char, "\n"),
            (TokenKind::Char, "ä"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn malformed_string_and_char_literals_report_specific_errors() {
    assert!(matches!(
        Lexer::create_tokens("\"unterminated".to_string()),
        Err(LexingError::UnterminatedString(_))
    ));
    assert!(matches!(
        Lexer::create_tokens("'".to_string()),
        Err(LexingError::UnterminatedChar(_))
    ));
    assert!(matches!(
        Lexer::create_tokens("''".to_string()),
        Err(LexingError::InvalidCharLiteral(_))
    ));
    assert!(matches!(
        Lexer::create_tokens("'ab'".to_string()),
        Err(LexingError::InvalidCharLiteral(_))
    ));
}

#[test]
fn lexes_all_operator_and_delimiter_tokens_by_longest_prefix() {
    assert_tokens(
        "+ += ++ - -= -- * *= / /= % = == => ! != < <= << > >= >> & && | || ^ ~ ( ) { } [ ] , ; : :: .",
        &[
            (TokenKind::Plus, "+"),
            (TokenKind::PlusEq, "+="),
            (TokenKind::PlusPlus, "++"),
            (TokenKind::Minus, "-"),
            (TokenKind::MinusEq, "-="),
            (TokenKind::MinusMinus, "--"),
            (TokenKind::Star, "*"),
            (TokenKind::StarEq, "*="),
            (TokenKind::Slash, "/"),
            (TokenKind::SlashEq, "/="),
            (TokenKind::Percent, "%"),
            (TokenKind::Eq, "="),
            (TokenKind::EqEq, "=="),
            (TokenKind::FatArrow, "=>"),
            (TokenKind::Bang, "!"),
            (TokenKind::BangEq, "!="),
            (TokenKind::Lt, "<"),
            (TokenKind::LtEq, "<="),
            (TokenKind::LtLt, "<<"),
            (TokenKind::Gt, ">"),
            (TokenKind::GtEq, ">="),
            (TokenKind::GtGt, ">>"),
            (TokenKind::Amp, "&"),
            (TokenKind::AmpAmp, "&&"),
            (TokenKind::Pipe, "|"),
            (TokenKind::PipePipe, "||"),
            (TokenKind::Caret, "^"),
            (TokenKind::Tilde, "~"),
            (TokenKind::LParen, "("),
            (TokenKind::RParen, ")"),
            (TokenKind::LBrace, "{"),
            (TokenKind::RBrace, "}"),
            (TokenKind::LBracket, "["),
            (TokenKind::RBracket, "]"),
            (TokenKind::Comma, ","),
            (TokenKind::Semicolon, ";"),
            (TokenKind::Colon, ":"),
            (TokenKind::ColonColon, "::"),
            (TokenKind::Dot, "."),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn comments_and_whitespace_never_create_tokens() {
    assert_tokens(
        "let /* ignored */ x // rest of line\n= 1;",
        &[
            (TokenKind::Let, "let"),
            (TokenKind::Ident, "x"),
            (TokenKind::Eq, "="),
            (TokenKind::Int, "1"),
            (TokenKind::Semicolon, ";"),
            (TokenKind::Eof, "EOF"),
        ],
    );
}

#[test]
fn invalid_characters_and_unterminated_block_comments_are_errors_not_panics() {
    assert!(matches!(
        Lexer::create_tokens("@".to_string()),
        Err(LexingError::InvalidChar('@', _))
    ));
    assert!(matches!(
        Lexer::create_tokens("/* never closed".to_string()),
        Err(LexingError::UnterminatedBlockComment(_))
    ));
}

#[test]
fn token_spans_reconstruct_unicode_literal_source_slices() {
    let source = "fn main() { let c: char = 'ä'; let s: string = \"Grüße\"; }";
    let tokens = lex(source).expect("lexing failed");

    let char_token = tokens
        .iter()
        .find(|token| token.kind == TokenKind::Char)
        .expect("missing char token");
    let string_token = tokens
        .iter()
        .find(|token| token.kind == TokenKind::StringLiteral)
        .expect("missing string token");

    let char_start = char_token.span.start_char_idx as i32;
    let char_end = char_start + char_token.span.length;
    let string_start = string_token.span.start_char_idx as i32;
    let string_end = string_start + string_token.span.length;

    assert_eq!(&source[char_end as usize..char_start as usize], "'ä'");
    assert_eq!(&source[string_end as usize..string_start as usize], "\"Grüße\"");
}
