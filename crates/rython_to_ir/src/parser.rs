use std::thread::sleep_ms;

use crate::ast::*;
use crate::lexer::Token;
use crate::lexer::TokenKind;

pub struct Parser {
    tokens: Vec<Token>,
    ast: Vec<Item>,
    current_idx: usize,
}

impl Parser {
    pub fn new(input: Vec<Token>) -> Parser {
        Parser {
            tokens: input,
            ast: vec![],
            current_idx: 0,
        }
    }

    fn advance(&mut self) {
        self.current_idx += 1;
    }

    fn current(&self) -> Token {
        self.tokens[self.current_idx].clone()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current_idx + 1].clone()
    }

    fn expect_current(&self, expected: TokenKind) {
        assert_eq!(self.current().kind, expected);
    }

    fn expect_next(&self, expected: TokenKind) {
        assert_eq!(self.peek().kind, expected);
    }

    fn parse_expr(&mut self) -> Expr {
        todo!()
    }

    fn parse_import(&mut self) {
        self.expect_current(TokenKind::Import);
        self.advance();
        let mut whole_path = String::new();

        loop {
            self.expect_current(TokenKind::Ident);
            let value = self.current().value;
            whole_path.push_str(value.as_str());

            if self.peek().kind == TokenKind::Dot {
                self.advance();
                self.advance();
                continue;
            }

            self.advance();
            self.expect_current(TokenKind::Semicolon);
            self.advance();
            break;
        }

        self.ast.push(Item::Import(Import {
            import_name: whole_path,
        }));
    }
    fn parse_trait(&mut self) {}
    fn parse_variant(&mut self) {
        self.expect_current(TokenKind::Variant);
        self.advance();
        self.expect_current(TokenKind::Ident);
        let variant_name = self.current().value;
        self.advance();
        self.expect_current(TokenKind::LBrace);
        self.advance();

        let mut cases = vec![];

        loop {
            if self.current().kind == TokenKind::RBrace {
                self.advance();
                break;
            }

            self.expect_current(TokenKind::Ident);
            cases.push(self.current().value);
            self.advance();

            if self.current().kind == TokenKind::RBrace {
                self.advance();
                break;
            }

            self.expect_current(TokenKind::Comma);
            self.advance();
        }

        self.ast.push(Item::Variant(Variant {
            variant_name,
            cases,
        }));
    }
    fn parse_struct(&mut self) {}
    fn parse_fn(&mut self) {}
    fn parse_trait_implementation(&mut self) {}

    pub fn parse(&mut self) -> &Vec<Item> {
        loop {
            if self.current_idx >= self.tokens.len() {
                break;
            }

            let curr_token = self.current();

            match curr_token.kind {
                TokenKind::Trait => self.parse_trait(),
                TokenKind::Import => self.parse_import(),
                TokenKind::Variant => self.parse_variant(),
                TokenKind::Struct => self.parse_struct(),
                TokenKind::Fn => self.parse_fn(),
                TokenKind::Impl => self.parse_trait_implementation(),
                TokenKind::Eof => break,
                TokenKind::Semicolon => continue,
                other => panic!("unexpected Toke: {:?}", other),
            };
        }

        &self.ast
    }
}
