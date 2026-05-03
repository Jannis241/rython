#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // keywords
    Fn,
    Let,
    If,
    Else,
    Return,
    While,
    For,
    In,
    True,
    False,
    Import,
    Struct,
    Pass,
    Break,
    Continue,

    // literals
    Ident(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),

    // maths
    Plus,
    Minus,
    Star,
    StarStar,
    Slash,
    Percent,

    // assignment
    Eq,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,

    // comparison
    EqEq,
    BangEq,
    Lt,
    LtEq,
    Gt,
    GtEq,

    // bit aal
    Amp,
    Pipe,
    Caret,
    Tilde,
    LtLt,
    GtGt,

    // delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    Dot,
    Arrow,

    Newline,
    Eof,
}

struct Lexer {
    current_idx: usize,
    input: Vec<char>,
    tokens: Vec<Token>,
}

impl Lexer {
    fn new(input: String) -> Self {
        Lexer {
            current_idx: 0,
            input: input.chars().collect(),
            tokens: vec![],
        }
    }

    fn current(&self) -> Option<char> {
        self.input.get(self.current_idx).copied()
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.current_idx + 1).copied()
    }

    fn advance(&mut self) {
        self.current_idx += 1;
    }

    fn consume_while(&mut self, predicate: impl Fn(char) -> bool) -> String {
        let mut s = String::new();
        while let Some(c) = self.current() {
            if predicate(c) {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        s
    }

    fn lex_string(&mut self, quote: char) -> Token {
        self.advance();
        let mut s = String::new();
        loop {
            match self.current() {
                None => break,
                Some('\\') => {
                    self.advance();
                    match self.current() {
                        Some('n') => {
                            s.push('\n');
                            self.advance();
                        }
                        Some('t') => {
                            s.push('\t');
                            self.advance();
                        }
                        Some('\\') => {
                            s.push('\\');
                            self.advance();
                        }
                        Some(c) if c == quote => {
                            s.push(c);
                            self.advance();
                        }
                        Some(c) => {
                            s.push('\\');
                            s.push(c);
                            self.advance();
                        }
                        None => break,
                    }
                }
                Some(c) if c == quote => {
                    self.advance();
                    break;
                }
                Some(c) => {
                    s.push(c);
                    self.advance();
                }
            }
        }
        Token::StringLiteral(s)
    }

    fn lex_number(&mut self) -> Token {
        let int_part = self.consume_while(|c| c.is_ascii_digit());
        if self.current() == Some('.') && self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.advance(); // skip '.'
            let frac_part = self.consume_while(|c| c.is_ascii_digit());
            Token::FloatLiteral(format!("{}.{}", int_part, frac_part).parse().unwrap())
        } else {
            Token::IntLiteral(int_part.parse().unwrap())
        }
    }

    fn lex_ident_or_keyword(&mut self) -> Token {
        let s = self.consume_while(|c| c.is_alphanumeric() || c == '_');
        match s.as_str() {
            "fn" => Token::Fn,
            "let" => Token::Let,
            "if" => Token::If,
            "else" => Token::Else,
            "return" => Token::Return,
            "while" => Token::While,
            "for" => Token::For,
            "in" => Token::In,
            "true" => Token::True,
            "false" => Token::False,
            "import" => Token::Import,
            "struct" => Token::Struct,
            "pass" => Token::Pass,
            "break" => Token::Break,
            "continue" => Token::Continue,
            _ => Token::Ident(s),
        }
    }

    fn lex(&mut self) -> Vec<Token> {
        loop {
            match self.current() {
                None => {
                    self.tokens.push(Token::Eof);
                    break;
                }
                Some(' ') | Some('\t') | Some('\r') => self.advance(),
                Some('\n') => {
                    self.tokens.push(Token::Newline);
                    self.advance();
                }
                Some('#') => {
                    self.consume_while(|c| c != '\n');
                }
                Some(q @ '"') | Some(q @ '\'') => {
                    let tok = self.lex_string(q);
                    self.tokens.push(tok);
                }
                Some(c) if c.is_ascii_digit() => {
                    let tok = self.lex_number();
                    self.tokens.push(tok);
                }
                Some(c) if c.is_alphabetic() || c == '_' => {
                    let tok = self.lex_ident_or_keyword();
                    self.tokens.push(tok);
                }
                Some('+') => {
                    if self.peek() == Some('=') {
                        self.tokens.push(Token::PlusEq);
                        self.advance();
                    } else {
                        self.tokens.push(Token::Plus);
                    }
                    self.advance();
                }
                Some('-') => {
                    if self.peek() == Some('>') {
                        self.tokens.push(Token::Arrow);
                        self.advance();
                    } else if self.peek() == Some('=') {
                        self.tokens.push(Token::MinusEq);
                        self.advance();
                    } else {
                        self.tokens.push(Token::Minus);
                    }
                    self.advance();
                }
                Some('*') => {
                    if self.peek() == Some('*') {
                        self.tokens.push(Token::StarStar);
                        self.advance();
                    } else if self.peek() == Some('=') {
                        self.tokens.push(Token::StarEq);
                        self.advance();
                    } else {
                        self.tokens.push(Token::Star);
                    }
                    self.advance();
                }
                Some('/') => {
                    if self.peek() == Some('=') {
                        self.tokens.push(Token::SlashEq);
                        self.advance();
                    } else {
                        self.tokens.push(Token::Slash);
                    }
                    self.advance();
                }
                Some('%') => {
                    self.tokens.push(Token::Percent);
                    self.advance();
                }
                Some('=') => {
                    if self.peek() == Some('=') {
                        self.tokens.push(Token::EqEq);
                        self.advance();
                    } else {
                        self.tokens.push(Token::Eq);
                    }
                    self.advance();
                }
                Some('!') => {
                    if self.peek() == Some('=') {
                        self.tokens.push(Token::BangEq);
                        self.advance();
                        self.advance();
                    } else {
                        self.advance();
                    }
                }
                Some('<') => {
                    if self.peek() == Some('=') {
                        self.tokens.push(Token::LtEq);
                        self.advance();
                    } else if self.peek() == Some('<') {
                        self.tokens.push(Token::LtLt);
                        self.advance();
                    } else {
                        self.tokens.push(Token::Lt);
                    }
                    self.advance();
                }
                Some('>') => {
                    if self.peek() == Some('=') {
                        self.tokens.push(Token::GtEq);
                        self.advance();
                    } else if self.peek() == Some('>') {
                        self.tokens.push(Token::GtGt);
                        self.advance();
                    } else {
                        self.tokens.push(Token::Gt);
                    }
                    self.advance();
                }
                Some('&') => {
                    self.tokens.push(Token::Amp);
                    self.advance();
                }
                Some('|') => {
                    self.tokens.push(Token::Pipe);
                    self.advance();
                }
                Some('^') => {
                    self.tokens.push(Token::Caret);
                    self.advance();
                }
                Some('~') => {
                    self.tokens.push(Token::Tilde);
                    self.advance();
                }
                Some('(') => {
                    self.tokens.push(Token::LParen);
                    self.advance();
                }
                Some(')') => {
                    self.tokens.push(Token::RParen);
                    self.advance();
                }
                Some('{') => {
                    self.tokens.push(Token::LBrace);
                    self.advance();
                }
                Some('}') => {
                    self.tokens.push(Token::RBrace);
                    self.advance();
                }
                Some('[') => {
                    self.tokens.push(Token::LBracket);
                    self.advance();
                }
                Some(']') => {
                    self.tokens.push(Token::RBracket);
                    self.advance();
                }
                Some(',') => {
                    self.tokens.push(Token::Comma);
                    self.advance();
                }
                Some(';') => {
                    self.tokens.push(Token::Semicolon);
                    self.advance();
                }
                Some(':') => {
                    self.tokens.push(Token::Colon);
                    self.advance();
                }
                Some('.') => {
                    self.tokens.push(Token::Dot);
                    self.advance();
                }
                Some(_) => self.advance(),
            }
        }
        self.tokens.clone()
    }
}

pub fn lex(input: String) -> Vec<Token> {
    Lexer::new(input).lex()
}
