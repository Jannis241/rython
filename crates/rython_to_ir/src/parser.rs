use crate::ast::*;
use crate::lexer::Token;
use crate::lexer::TokenKind;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        expected: TokenKind,
        found: TokenKind,
    },
    UnexpectedTopLevel(TokenKind),
    UnexpectedEof,
}

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

    fn current(&self) -> Result<Token, ParseError> {
        self.tokens
            .get(self.current_idx)
            .cloned()
            .ok_or(ParseError::UnexpectedEof)
    }

    fn peek(&self) -> Result<Token, ParseError> {
        self.tokens
            .get(self.current_idx + 1)
            .cloned()
            .ok_or(ParseError::UnexpectedEof)
    }

    fn expect_current(&self, expected: TokenKind) -> Result<(), ParseError> {
        let found = self.current()?.kind;
        if found != expected {
            return Err(ParseError::UnexpectedToken { expected, found });
        }
        Ok(())
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        todo!()
    }

    fn parse_import(&mut self) -> Result<(), ParseError> {
        self.expect_current(TokenKind::Import)?;
        self.advance();
        let mut whole_path = String::new();

        loop {
            self.expect_current(TokenKind::Ident)?;
            let value = self.current()?.value;
            whole_path.push_str(value.as_str());

            if self.peek()?.kind == TokenKind::Dot {
                whole_path.push('.');
                self.advance();
                self.advance();
                continue;
            }

            self.advance();
            self.expect_current(TokenKind::Semicolon)?;
            self.advance();
            break;
        }

        self.ast.push(Item::Import(Import {
            import_name: whole_path,
        }));
        Ok(())
    }

    fn parse_trait(&mut self) -> Result<(), ParseError> {
        Ok(())
    }

    fn parse_variant(&mut self) -> Result<(), ParseError> {
        self.expect_current(TokenKind::Variant)?;
        self.advance();
        self.expect_current(TokenKind::Ident)?;
        let variant_name = self.current()?.value;
        self.advance();
        self.expect_current(TokenKind::LBrace)?;
        self.advance();

        let mut cases = vec![];

        loop {
            if self.current()?.kind == TokenKind::RBrace {
                self.advance();
                break;
            }

            self.expect_current(TokenKind::Ident)?;
            cases.push(self.current()?.value);
            self.advance();

            if self.current()?.kind == TokenKind::RBrace {
                self.advance();
                break;
            }

            self.expect_current(TokenKind::Comma)?;
            self.advance();
        }

        self.ast.push(Item::Variant(Variant {
            variant_name,
            cases,
        }));
        Ok(())
    }

    fn parse_struct(&mut self) -> Result<(), ParseError> {
        Ok(())
    }

    fn parse_fn(&mut self) -> Result<(), ParseError> {
        Ok(())
    }

    fn parse_trait_implementation(&mut self) -> Result<(), ParseError> {
        Ok(())
    }

    pub fn parse(&mut self) -> Result<&Vec<Item>, ParseError> {
        loop {
            if self.current_idx >= self.tokens.len() {
                break;
            }

            match self.current()?.kind {
                TokenKind::Trait => self.parse_trait()?,
                TokenKind::Import => self.parse_import()?,
                TokenKind::Variant => self.parse_variant()?,
                TokenKind::Struct => self.parse_struct()?,
                TokenKind::Fn => self.parse_fn()?,
                TokenKind::Impl => self.parse_trait_implementation()?,
                TokenKind::Eof => break,
                TokenKind::Semicolon => self.advance(),
                other => return Err(ParseError::UnexpectedTopLevel(other)),
            }
        }

        Ok(&self.ast)
    }
}
