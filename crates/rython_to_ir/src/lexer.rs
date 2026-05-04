#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    True,
    False,
    Ident,
    Int,
    Float,
    StringLiteral,
    Char,
    Null,

    If,
    Else,
    Return,
    Loop,
    While,
    Any,
    Let,
    Fn,
    This,
    In,
    Import,
    Struct,
    Trait,
    Global,
    Const,
    Impl,
    For,
    Continue,
    Break,
    Variant,
    And,
    Or,

    Operator,

    Amp,
    Pipe,
    Caret,
    Tilde,
    LtLt,
    GtGt,

    LParen,
    RParen,
    LBrace,
    RBrace,
    Bang,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Colon,
    Dot,

    Asm,

    Eq,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,

    EqEq,
    BangEq,
    Lt,
    LtEq,
    Gt,
    GtEq,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
}

impl Token {
    pub fn new(kind: TokenKind, value: String) -> Self {
        Token { kind, value }
    }
}

pub struct Lexer {
    current_idx: usize,
    current_char: Option<char>,
    chars: Vec<char>,
}

#[derive(Debug)]
pub enum LexingError {
    InvalidChar(char),
}

impl Lexer {
    pub fn create_tokens(input: String) -> Result<Vec<Token>, LexingError> {
        let chars: Vec<char> = input.chars().collect();

        let mut lexer = Lexer {
            current_idx: 0,
            current_char: chars.first().copied(),
            chars,
        };

        let mut tokens = Vec::new();

        while tokens
            .last()
            .is_none_or(|last: &Token| last.kind != TokenKind::Eof)
        {
            tokens.push(lexer.create_next_token()?);
        }

        return Ok(tokens);
    }
    fn create_next_token(&mut self) -> Result<Token, LexingError> {
        self.skip_ignored();

        let token = match self.current_char {
            None => Token::new(TokenKind::Eof, "EOF".to_string()),

            Some('0'..='9') => self.handle_numbers(),
            Some('"') => self.handle_strings(),
            Some('\'') => self.handle_char(),
            Some('a'..='z' | 'A'..='Z') => self.handle_idents(),
            Some('+') => self.handle_plus(),
            Some('-') => self.handle_minus(),
            Some('*') => self.handle_star(),
            Some('/') => self.handle_slash(),
            Some('%') => Token::new(TokenKind::Percent, "%".to_string()),

            Some('=') => self.handle_eq(),
            Some('!') => self.handle_bang(),
            Some('<') => self.handle_lt(),
            Some('>') => self.handle_gt(),

            Some('&') => Token::new(TokenKind::Amp, "&".to_string()),
            Some('|') => Token::new(TokenKind::Pipe, "|".to_string()),
            Some('^') => Token::new(TokenKind::Caret, "^".to_string()),
            Some('~') => Token::new(TokenKind::Tilde, "~".to_string()),

            Some('(') => Token::new(TokenKind::LParen, "(".to_string()),
            Some(')') => Token::new(TokenKind::RParen, ")".to_string()),
            Some('{') => Token::new(TokenKind::LBrace, "{".to_string()),
            Some('}') => Token::new(TokenKind::RBrace, "}".to_string()),
            Some('[') => Token::new(TokenKind::LBracket, "[".to_string()),
            Some(']') => Token::new(TokenKind::RBracket, "]".to_string()),

            Some(',') => Token::new(TokenKind::Comma, ",".to_string()),
            Some(';') => Token::new(TokenKind::Semicolon, ";".to_string()),
            Some(':') => Token::new(TokenKind::Colon, ":".to_string()),
            Some('.') => Token::new(TokenKind::Dot, ".".to_string()),

            _ => {

                return Err(LexingError::InvalidChar(self.current_char.unwrap()));
            }
        };

        self.advance();

        return Ok(token);
    }
    fn skip_ignored(&mut self) {
        loop {
            while self.current_char.is_some_and(|c| c.is_ascii_whitespace()) {
                self.advance();
            }

            if self.current_char == Some('/') && self.peek() == Some('/') {
                self.skip_until_newline();
                continue;
            }

            break;
        }
    }

    fn skip_until_newline(&mut self) {
        while self.current_char.is_some() && self.current_char != Some('\n') {
            self.advance();
        }
    }

    fn handle_plus(&mut self) -> Token {
        if self.peek() == Some('=') {
            self.advance();
            Token::new(TokenKind::PlusEq, "+=".to_string())
        } else {
            Token::new(TokenKind::Plus, "+".to_string())
        }
    }

    fn handle_minus(&mut self) -> Token {
        if self.peek() == Some('=') {
            self.advance();
            Token::new(TokenKind::MinusEq, "-=".to_string())
        } else {
            Token::new(TokenKind::Minus, "-".to_string())
        }
    }

    fn handle_star(&mut self) -> Token {
        if self.peek() == Some('=') {
            self.advance();
            Token::new(TokenKind::StarEq, "*=".to_string())
        } else {
            Token::new(TokenKind::Star, "*".to_string())
        }
    }

    fn handle_slash(&mut self) -> Token {
        if self.peek() == Some('=') {
            self.advance();
            Token::new(TokenKind::SlashEq, "/=".to_string())
        } else {
            Token::new(TokenKind::Slash, "/".to_string())
        }
    }

    fn handle_eq(&mut self) -> Token {
        if self.peek() == Some('=') {
            self.advance();
            Token::new(TokenKind::EqEq, "==".to_string())
        } else {
            Token::new(TokenKind::Eq, "=".to_string())
        }
    }

    fn handle_bang(&mut self) -> Token {
        if self.peek() == Some('=') {
            self.advance();
            Token::new(TokenKind::BangEq, "!=".to_string())
        } else {
            Token::new(TokenKind::Bang, "!".to_string())
        }
    }

    fn handle_lt(&mut self) -> Token {
        match self.peek() {
            Some('=') => {
                self.advance();
                Token::new(TokenKind::LtEq, "<=".to_string())
            }
            Some('<') => {
                self.advance();
                Token::new(TokenKind::LtLt, "<<".to_string())
            }
            _ => Token::new(TokenKind::Lt, "<".to_string()),
        }
    }

    fn handle_gt(&mut self) -> Token {
        match self.peek() {
            Some('=') => {
                self.advance();
                Token::new(TokenKind::GtEq, ">=".to_string())
            }
            Some('>') => {
                self.advance();
                Token::new(TokenKind::GtGt, ">>".to_string())
            }
            _ => Token::new(TokenKind::Gt, ">".to_string()),
        }
    }
    fn handle_idents(&mut self) -> Token {
        let mut ident = String::new();
        ident.push(self.current_char.unwrap()); // der current_char kann nicht None sein da
        // handle_idents nur aufgerufen wird bei Some('a'..'z' | 'A'..'Z')

        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            ident.push(self.peek().unwrap()); // -> unwrap ist safe da vorher geguckt wurde
            // ob self.peek Some ist
            self.advance();
        }

        let token = match ident.as_str() {
            "true" => Token::new(TokenKind::True, ident),
            "false" => Token::new(TokenKind::False, ident),
            "char" => Token::new(TokenKind::Char, ident),
            "null" => Token::new(TokenKind::Null, ident),
            "if" => Token::new(TokenKind::If, ident),
            "else" => Token::new(TokenKind::Else, ident),
            "return" => Token::new(TokenKind::Return, ident),
            "global" => Token::new(TokenKind::Global, ident),
            "const" => Token::new(TokenKind::Const, ident),
            "loop" => Token::new(TokenKind::Loop, ident),
            "while" => Token::new(TokenKind::While, ident),
            "any" => Token::new(TokenKind::Any, ident),
            "let" => Token::new(TokenKind::Let, ident),
            "fn" => Token::new(TokenKind::Fn, ident),
            "this" => Token::new(TokenKind::This, ident),
            "in" => Token::new(TokenKind::In, ident),
            "import" => Token::new(TokenKind::Import, ident),
            "struct" => Token::new(TokenKind::Struct, ident),
            "trait" => Token::new(TokenKind::Trait, ident),
            "impl" => Token::new(TokenKind::Impl, ident),
            "for" => Token::new(TokenKind::For, ident),
            "continue" => Token::new(TokenKind::Continue, ident),
            "break" => Token::new(TokenKind::Break, ident),
            "variant" => Token::new(TokenKind::Variant, ident),
            "and" => Token::new(TokenKind::And, ident),
            "or" => Token::new(TokenKind::Or, ident),
            "operator" => Token::new(TokenKind::Operator, ident),
            "asm" => Token::new(TokenKind::Asm, ident),
            _ => Token::new(TokenKind::Ident, ident),
        };

        return token;
    }
    fn handle_strings(&mut self) -> Token {
        let mut str = String::new();
        self.advance(); // einmal advancen, damit man nicht mehr auf dem " ist.

        while self.current_char != Some('"') && self.current_char.is_some() {
            str.push(self.handle_escaped_char());
            self.advance();
        }
        Token::new(TokenKind::StringLiteral, str)
    }

    fn handle_char(&mut self) -> Token {
        let mut char_literal = String::new();
        self.advance();

        while self.current_char != Some('\'') && self.current_char.is_some() {
            char_literal.push(self.handle_escaped_char());
            self.advance();
        }

        Token::new(TokenKind::Char, char_literal)
    }

    fn handle_escaped_char(&mut self) -> char {
        if self.current_char != Some('\\') {
            return self.current_char.unwrap();
        }

        match self.peek() {
            Some('n') => {
                self.advance();
                '\n'
            }
            Some('t') => {
                self.advance();
                '\t'
            }
            Some('r') => {
                self.advance();
                '\r'
            }
            Some('"') => {
                self.advance();
                '"'
            }
            Some('\'') => {
                self.advance();
                '\''
            }
            Some('\\') => {
                self.advance();
                '\\'
            }
            _ => '\\',
        }
    }

    fn handle_numbers(&mut self) -> Token {
        let mut number = String::new();

        let mut is_float = false;

        number.push(self.current_char.unwrap()); // unwrap ist safe da die methode nur bei
        // Some() aufgerufen wird

        while self
            .peek()
            .is_some_and(|c| c.is_ascii_digit() || (c == '.' && !is_float))
        {
            if self.peek().unwrap() == '.' {
                is_float = true;
            }
            number.push(self.peek().unwrap()); // safe weil wurde gecheckt ob some ist
            self.advance();
        }
        if is_float {
            return Token::new(TokenKind::Float, number);
        }
        return Token::new(TokenKind::Int, number);
    }

    fn advance(&mut self) {
        self.current_idx += 1;
        self.current_char = self.chars.get(self.current_idx).copied();
    }

    fn peek(&self) -> Option<char> {
        return self.chars.get(self.current_idx + 1).copied();
    }
}
