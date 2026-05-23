#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    True,
    False,
    Ident,
    Int,
    Float,
    StringLiteral,
    Char,

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
    Yield,
    Variant,
    // And,
    // Or,
    // Not,
    Operator,

    Underscore,

    Amp,
    Pipe,
    Caret,
    Tilde,
    LtLt,
    GtGt,
    AmpAmp,
    PipePipe,

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
    FatArrow,
    ColonColon,
    PlusPlus,
    MinusMinus,

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
pub struct Span {
    pub start_char_idx: usize,
    pub length: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub value: String,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, value: String, start_char_idx: usize, length: i32) -> Self {
        Token {
            kind,
            value,
            span: Span {
                start_char_idx: start_char_idx + 1,
                length: length,
            },
        }
    }
}

pub struct Lexer {
    current_idx: usize,
    current_char: Option<char>,
    chars: Vec<char>,
}

#[derive(Debug)]
pub enum LexingError {
    InvalidChar(char, Span),
    UnexpectedEof(Span),
    UnterminatedString(Span),
    UnterminatedChar(Span),
    InvalidCharLiteral(Span),
    UnterminatedBlockComment(Span),
    InvalidEscape(String, Span),
    InvalidNumber(String, Span),
    AsmMissingBrace(Span),
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
        self.skip_ignored()?;

        let token = match self.current_char {
            None => Token::new(TokenKind::Eof, "EOF".to_string(), self.current_idx, -1),

            Some('0'..='9') => self.handle_numbers()?,
            Some('"') => self.handle_strings()?,
            Some('\'') => self.handle_char()?,
            Some('a'..='z' | 'A'..='Z' | '_') => self.handle_idents()?,
            Some('+') => self.handle_plus(),
            Some('-') => self.handle_minus(),
            Some('*') => self.handle_star(),
            Some('/') => self.handle_slash(),
            Some('%') => Token::new(TokenKind::Percent, "%".to_string(), self.current_idx, -1),

            Some('=') => self.handle_eq(),
            Some('!') => self.handle_bang(),
            Some('<') => self.handle_lt(),
            Some('>') => self.handle_gt(),

            Some('&') => self.handle_amp(),
            Some('|') => self.handle_pipe(),
            Some('^') => Token::new(TokenKind::Caret, "^".to_string(), self.current_idx, -1),
            Some('~') => Token::new(TokenKind::Tilde, "~".to_string(), self.current_idx, -1),

            Some('(') => Token::new(TokenKind::LParen, "(".to_string(), self.current_idx, -1),
            Some(')') => Token::new(TokenKind::RParen, ")".to_string(), self.current_idx, -1),
            Some('{') => Token::new(TokenKind::LBrace, "{".to_string(), self.current_idx, -1),
            Some('}') => Token::new(TokenKind::RBrace, "}".to_string(), self.current_idx, -1),
            Some('[') => Token::new(TokenKind::LBracket, "[".to_string(), self.current_idx, -1),
            Some(']') => Token::new(TokenKind::RBracket, "]".to_string(), self.current_idx, -1),

            Some(',') => Token::new(TokenKind::Comma, ",".to_string(), self.current_idx, -1),
            Some(';') => Token::new(TokenKind::Semicolon, ";".to_string(), self.current_idx, -1),
            Some(':') => self.handle_colon(),
            Some('.') => Token::new(TokenKind::Dot, ".".to_string(), self.current_idx, -1),

            _ => {
                return Err(LexingError::InvalidChar(
                    self.current_char.unwrap(),
                    Span {
                        start_char_idx: self.current_idx,
                        length: -1,
                    },
                ));
            }
        };

        self.advance();

        return Ok(token);
    }
    fn skip_ignored(&mut self) -> Result<(), LexingError> {
        loop {
            while self.current_char.is_some_and(|c| c.is_ascii_whitespace()) {
                self.advance();
            }

            if self.current_char == Some('/') && self.peek() == Some('/') {
                self.skip_until_newline();
                continue;
            }

            if self.current_char == Some('/') && self.peek() == Some('*') {
                self.skip_block_comment()?;
                continue;
            }

            break;
        }
        Ok(())
    }

    fn skip_until_newline(&mut self) {
        while self.current_char.is_some() && self.current_char != Some('\n') {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), LexingError> {
        let start_idx = self.current_idx;
        self.advance(); // '/'
        self.advance(); // '*'
        loop {
            match self.current_char {
                None => {
                    return Err(LexingError::UnterminatedBlockComment(Span {
                        start_char_idx: start_idx,
                        length: 2,
                    }));
                }
                Some('*') if self.peek() == Some('/') => {
                    self.advance(); // '*'
                    self.advance(); // '/'
                    return Ok(());
                }
                _ => self.advance(),
            }
        }
    }

    fn handle_amp(&mut self) -> Token {
        if self.peek() == Some('&') {
            self.advance();
            Token::new(TokenKind::AmpAmp, "&&".to_string(), self.current_idx, -2)
        } else {
            Token::new(TokenKind::Amp, "&".to_string(), self.current_idx, -1)
        }
    }

    fn handle_pipe(&mut self) -> Token {
        if self.peek() == Some('|') {
            self.advance();
            Token::new(TokenKind::PipePipe, "||".to_string(), self.current_idx, -2)
        } else {
            Token::new(TokenKind::Pipe, "|".to_string(), self.current_idx, -1)
        }
    }

    fn handle_plus(&mut self) -> Token {
        if self.peek() == Some('=') {
            self.advance();
            Token::new(TokenKind::PlusEq, "+=".to_string(), self.current_idx, -2)
        } else if self.peek() == Some('+') {
            self.advance();
            Token::new(TokenKind::PlusPlus, "++".to_string(), self.current_idx, -2)
        } else {
            Token::new(TokenKind::Plus, "+".to_string(), self.current_idx, -1)
        }
    }

    fn handle_minus(&mut self) -> Token {
        if self.peek() == Some('=') {
            self.advance();
            Token::new(TokenKind::MinusEq, "-=".to_string(), self.current_idx, -2)
        } else if self.peek() == Some('-') {
            self.advance();
            Token::new(
                TokenKind::MinusMinus,
                "--".to_string(),
                self.current_idx,
                -2,
            )
        } else {
            Token::new(TokenKind::Minus, "-".to_string(), self.current_idx, -1)
        }
    }

    fn handle_star(&mut self) -> Token {
        if self.peek() == Some('=') {
            self.advance();
            Token::new(TokenKind::StarEq, "*=".to_string(), self.current_idx, -2)
        } else {
            Token::new(TokenKind::Star, "*".to_string(), self.current_idx, -1)
        }
    }

    fn handle_slash(&mut self) -> Token {
        if self.peek() == Some('=') {
            self.advance();
            Token::new(TokenKind::SlashEq, "/=".to_string(), self.current_idx, -2)
        } else {
            Token::new(TokenKind::Slash, "/".to_string(), self.current_idx, -1)
        }
    }

    fn handle_eq(&mut self) -> Token {
        match self.peek() {
            Some('=') => {
                self.advance();
                Token::new(TokenKind::EqEq, "==".to_string(), self.current_idx, -2)
            }
            Some('>') => {
                self.advance();
                Token::new(TokenKind::FatArrow, "=>".to_string(), self.current_idx, -2)
            }
            _ => Token::new(TokenKind::Eq, "=".to_string(), self.current_idx, -1),
        }
    }

    fn handle_colon(&mut self) -> Token {
        if self.peek() == Some(':') {
            self.advance();
            Token::new(
                TokenKind::ColonColon,
                "::".to_string(),
                self.current_idx,
                -2,
            )
        } else {
            Token::new(TokenKind::Colon, ":".to_string(), self.current_idx, -1)
        }
    }

    fn handle_bang(&mut self) -> Token {
        if self.peek() == Some('=') {
            self.advance();
            Token::new(TokenKind::BangEq, "!=".to_string(), self.current_idx, -2)
        } else {
            Token::new(TokenKind::Bang, "!".to_string(), self.current_idx, -1)
        }
    }

    fn handle_lt(&mut self) -> Token {
        match self.peek() {
            Some('=') => {
                self.advance();
                Token::new(TokenKind::LtEq, "<=".to_string(), self.current_idx, -2)
            }
            Some('<') => {
                self.advance();
                Token::new(TokenKind::LtLt, "<<".to_string(), self.current_idx, -2)
            }
            _ => Token::new(TokenKind::Lt, "<".to_string(), self.current_idx, -1),
        }
    }

    fn handle_gt(&mut self) -> Token {
        match self.peek() {
            Some('=') => {
                self.advance();
                Token::new(TokenKind::GtEq, ">=".to_string(), self.current_idx, -2)
            }
            Some('>') => {
                self.advance();
                Token::new(TokenKind::GtGt, ">>".to_string(), self.current_idx, -2)
            }
            _ => Token::new(TokenKind::Gt, ">".to_string(), self.current_idx, -1),
        }
    }
    fn handle_idents(&mut self) -> Result<Token, LexingError> {
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

        let ci = self.current_idx;

        let ident2 = ident.clone();

        let token = match ident.as_str() {
            "yield" => Token::new(TokenKind::Yield, ident2, ci, -(ident.len() as i32)),
            "true" => Token::new(TokenKind::True, ident2, ci, -(ident.len() as i32)),
            "false" => Token::new(TokenKind::False, ident2, ci, -(ident.len() as i32)),
            "char" => Token::new(TokenKind::Char, ident2, ci, -(ident.len() as i32)),
            "if" => Token::new(TokenKind::If, ident2, ci, -(ident.len() as i32)),
            "else" => Token::new(TokenKind::Else, ident2, ci, -(ident.len() as i32)),
            "return" => Token::new(TokenKind::Return, ident2, ci, -(ident.len() as i32)),
            "global" => Token::new(TokenKind::Global, ident2, ci, -(ident.len() as i32)),
            "const" => Token::new(TokenKind::Const, ident2, ci, -(ident.len() as i32)),
            "loop" => Token::new(TokenKind::Loop, ident2, ci, -(ident.len() as i32)),
            "while" => Token::new(TokenKind::While, ident2, ci, -(ident.len() as i32)),
            "any" => Token::new(TokenKind::Any, ident2, ci, -(ident.len() as i32)),
            "let" => Token::new(TokenKind::Let, ident2, ci, -(ident.len() as i32)),
            "fn" => Token::new(TokenKind::Fn, ident2, ci, -(ident.len() as i32)),
            "this" => Token::new(TokenKind::This, ident2, ci, -(ident.len() as i32)),
            "in" => Token::new(TokenKind::In, ident2, ci, -(ident.len() as i32)),
            "import" => Token::new(TokenKind::Import, ident2, ci, -(ident.len() as i32)),
            "struct" => Token::new(TokenKind::Struct, ident2, ci, -(ident.len() as i32)),
            "trait" => Token::new(TokenKind::Trait, ident2, ci, -(ident.len() as i32)),
            "impl" => Token::new(TokenKind::Impl, ident2, ci, -(ident.len() as i32)),
            "for" => Token::new(TokenKind::For, ident2, ci, -(ident.len() as i32)),
            "continue" => Token::new(TokenKind::Continue, ident2, ci, -(ident.len() as i32)),
            "break" => Token::new(TokenKind::Break, ident2, ci, -(ident.len() as i32)),
            "variant" => Token::new(TokenKind::Variant, ident2, ci, -(ident.len() as i32)),
            "operator" => Token::new(TokenKind::Operator, ident2, ci, -(ident.len() as i32)),
            "_" => Token::new(TokenKind::Underscore, ident2, ci, -(ident.len() as i32)),
            "asm" => {
                // skip whitespace and comments between `asm` and `{`
                while self.peek().is_some_and(|c| c.is_ascii_whitespace()) {
                    self.advance();
                }
                if self.peek() != Some('{') {
                    return Err(LexingError::AsmMissingBrace(Span {
                        start_char_idx: ci,
                        length: -(ident.len() as i32),
                    }));
                }
                self.advance(); // current = '{'
                let body_start = self.current_idx + 1;
                let mut asm = String::new();
                let mut depth: u32 = 1;
                loop {
                    match self.peek() {
                        None => {
                            return Err(LexingError::UnexpectedEof(Span {
                                start_char_idx: self.current_idx,
                                length: 1,
                            }));
                        }
                        Some('{') => {
                            depth += 1;
                            asm.push('{');
                            self.advance();
                        }
                        Some('}') => {
                            depth -= 1;
                            if depth == 0 {
                                self.advance(); // current = '}'
                                break Token::new(
                                    TokenKind::Asm,
                                    asm.clone(),
                                    body_start,
                                    asm.len() as i32,
                                );
                            }
                            asm.push('}');
                            self.advance();
                        }
                        Some(c) => {
                            asm.push(c);
                            self.advance();
                        }
                    }
                }
            }
            _ => Token::new(TokenKind::Ident, ident.clone(), ci, -(ident.len() as i32)),
        };

        return Ok(token);
    }
    fn handle_strings(&mut self) -> Result<Token, LexingError> {
        let start_idx = self.current_idx;
        let mut str = String::new();
        self.advance(); // einmal advancen, damit man nicht mehr auf dem " ist.

        loop {
            match self.current_char {
                None => {
                    return Err(LexingError::UnterminatedString(Span {
                        start_char_idx: start_idx,
                        length: (self.current_idx - start_idx) as i32,
                    }));
                }
                Some('"') => break,
                Some(_) => {
                    str.push(self.handle_escaped_char()?);
                    self.advance();
                }
            }
        }
        let lexeme_char_len = (self.current_idx - start_idx + 1) as i32;
        Ok(Token::new(
            TokenKind::StringLiteral,
            str.clone(),
            self.current_idx,
            -lexeme_char_len,
        ))
    }

    fn handle_char(&mut self) -> Result<Token, LexingError> {
        let start_idx = self.current_idx;
        let mut char_literal = String::new();
        self.advance();

        loop {
            match self.current_char {
                None => {
                    return Err(LexingError::UnterminatedChar(Span {
                        start_char_idx: start_idx,
                        length: (self.current_idx - start_idx) as i32,
                    }));
                }
                Some('\'') => break,
                Some(_) => {
                    char_literal.push(self.handle_escaped_char()?);
                    self.advance();
                }
            }
        }

        if char_literal.chars().count() != 1 {
            return Err(LexingError::InvalidCharLiteral(Span {
                start_char_idx: start_idx,
                length: (self.current_idx - start_idx + 1) as i32,
            }));
        }

        let lexeme_char_len = (self.current_idx - start_idx + 1) as i32;
        Ok(Token::new(
            TokenKind::Char,
            char_literal.clone(),
            self.current_idx,
            -lexeme_char_len,
        ))
    }

    fn handle_escaped_char(&mut self) -> Result<char, LexingError> {
        if self.current_char != Some('\\') {
            return Ok(self.current_char.unwrap());
        }

        let escape_start = self.current_idx;
        match self.peek() {
            Some('n') => {
                self.advance();
                Ok('\n')
            }
            Some('t') => {
                self.advance();
                Ok('\t')
            }
            Some('r') => {
                self.advance();
                Ok('\r')
            }
            Some('0') => {
                self.advance();
                Ok('\0')
            }
            Some('"') => {
                self.advance();
                Ok('"')
            }
            Some('\'') => {
                self.advance();
                Ok('\'')
            }
            Some('\\') => {
                self.advance();
                Ok('\\')
            }
            Some('x') => {
                self.advance(); // current = 'x'
                let h1 = self
                    .peek()
                    .filter(|c| c.is_ascii_hexdigit())
                    .ok_or_else(|| {
                        LexingError::InvalidEscape(
                            "\\x".to_string(),
                            Span {
                                start_char_idx: escape_start,
                                length: 2,
                            },
                        )
                    })?;
                self.advance(); // current = h1
                let h2 = self
                    .peek()
                    .filter(|c| c.is_ascii_hexdigit())
                    .ok_or_else(|| {
                        LexingError::InvalidEscape(
                            format!("\\x{h1}"),
                            Span {
                                start_char_idx: escape_start,
                                length: 3,
                            },
                        )
                    })?;
                self.advance(); // current = h2
                let value =
                    u8::from_str_radix(&format!("{h1}{h2}"), 16).expect("validated hex digits");
                Ok(value as char)
            }
            Some('u') => {
                self.advance(); // current = 'u'
                if self.peek() != Some('{') {
                    return Err(LexingError::InvalidEscape(
                        "\\u".to_string(),
                        Span {
                            start_char_idx: escape_start,
                            length: 2,
                        },
                    ));
                }
                self.advance(); // current = '{'
                let mut hex = String::new();
                while self.peek().is_some_and(|c| c.is_ascii_hexdigit()) {
                    hex.push(self.peek().unwrap());
                    self.advance();
                }
                if hex.is_empty() || self.peek() != Some('}') {
                    return Err(LexingError::InvalidEscape(
                        format!("\\u{{{hex}"),
                        Span {
                            start_char_idx: escape_start,
                            length: (self.current_idx - escape_start + 1) as i32,
                        },
                    ));
                }
                self.advance(); // current = '}'
                let code = u32::from_str_radix(&hex, 16).map_err(|_| {
                    LexingError::InvalidEscape(
                        format!("\\u{{{hex}}}"),
                        Span {
                            start_char_idx: escape_start,
                            length: (hex.len() + 4) as i32,
                        },
                    )
                })?;
                char::from_u32(code).ok_or_else(|| {
                    LexingError::InvalidEscape(
                        format!("\\u{{{hex}}}"),
                        Span {
                            start_char_idx: escape_start,
                            length: (hex.len() + 4) as i32,
                        },
                    )
                })
            }
            _ => Ok('\\'),
        }
    }

    fn handle_numbers(&mut self) -> Result<Token, LexingError> {
        let start_idx = self.current_idx;
        let first = self.current_char.unwrap();

        if first == '0' {
            if let Some(prefix) = self.peek() {
                let radix = match prefix {
                    'x' | 'X' => Some((16u32, "0x")),
                    'b' | 'B' => Some((2u32, "0b")),
                    'o' | 'O' => Some((8u32, "0o")),
                    _ => None,
                };
                if let Some((radix, prefix_str)) = radix {
                    self.advance(); // consume '0', current = prefix char
                    let mut value = String::from(prefix_str);
                    let mut digit_count: usize = 0;
                    while let Some(c) = self.peek() {
                        if c == '_' {
                            self.advance();
                            continue;
                        }
                        if c.is_digit(radix) {
                            value.push(c);
                            digit_count += 1;
                            self.advance();
                            continue;
                        }
                        break;
                    }
                    if digit_count == 0 {
                        return Err(LexingError::InvalidNumber(
                            value,
                            Span {
                                start_char_idx: start_idx,
                                length: (self.current_idx - start_idx + 1) as i32,
                            },
                        ));
                    }
                    let len = value.len() as i32;
                    return Ok(Token::new(TokenKind::Int, value, self.current_idx, -len));
                }
            }
        }

        let mut number = String::new();
        let mut is_float = false;
        number.push(first);

        // integer-or-fraction digits and underscores
        loop {
            match self.peek() {
                Some(c) if c.is_ascii_digit() => {
                    number.push(c);
                    self.advance();
                }
                Some('_') if self.peek_2().is_some_and(|n| n.is_ascii_digit()) => {
                    self.advance(); // drop underscore
                }
                Some('.') if !is_float => {
                    is_float = true;
                    number.push('.');
                    self.advance();
                }
                _ => break,
            }
        }

        // optional exponent: e/E [+/-]? digits
        if matches!(self.peek(), Some('e' | 'E'))
            && (self
                .peek_2()
                .is_some_and(|c| c.is_ascii_digit() || c == '+' || c == '-'))
        {
            is_float = true;
            self.advance();
            number.push('e');
            if matches!(self.peek(), Some('+' | '-')) {
                let sign = self.peek().unwrap();
                number.push(sign);
                self.advance();
            }
            if !self.peek().is_some_and(|c| c.is_ascii_digit()) {
                return Err(LexingError::InvalidNumber(
                    number,
                    Span {
                        start_char_idx: start_idx,
                        length: (self.current_idx - start_idx + 1) as i32,
                    },
                ));
            }
            loop {
                match self.peek() {
                    Some(c) if c.is_ascii_digit() => {
                        number.push(c);
                        self.advance();
                    }
                    Some('_') if self.peek_2().is_some_and(|n| n.is_ascii_digit()) => {
                        self.advance();
                    }
                    _ => break,
                }
            }
        }

        let len = number.len() as i32;
        let kind = if is_float {
            TokenKind::Float
        } else {
            TokenKind::Int
        };
        Ok(Token::new(kind, number, self.current_idx, -len))
    }

    fn advance(&mut self) {
        self.current_idx += 1;
        self.current_char = self.chars.get(self.current_idx).copied();
    }

    fn peek(&self) -> Option<char> {
        return self.chars.get(self.current_idx + 1).copied();
    }

    fn peek_2(&self) -> Option<char> {
        return self.chars.get(self.current_idx + 2).copied();
    }
}
