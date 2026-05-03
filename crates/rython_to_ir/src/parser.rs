use crate::lexer::Token;

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

    fn expect_current(&self, expected: Token) {
        assert_eq!(self.current(), expected);
    }
    fn expect_next(&self, expected: Token) {
        assert_eq!(self.peek(), expected);
    }

    fn parse_expr(&mut self) -> () {}

    fn parse_import(&mut self) {}
    fn parse_trait(&mut self) {}
    fn parse_variant(&mut self) {}
    fn parse_struct(&mut self) {}
    fn parse_fn(&mut self) {}
    fn parse_trait_implementation(&mut self) {}

    pub fn parse(&mut self) -> &Vec<Item> {
        loop {
            if self.current_idx >= self.tokens.len() {
                break;
            }

            let curr_token = self.current();

            match curr_token {
                Token::Trait => self.parse_trait(),
                Token::Import => self.parse_import(),
                Token::Variant => self.parse_variant(),
                Token::Struct => self.parse_struct(),
                Token::Fn => self.parse_fn(),
                Token::Impl => self.parse_trait_implementation(),
                Token::Eof => break,
                other => panic!("unexpected Toke: {:?}", other),
            };
        }

        &self.ast
    }
}
