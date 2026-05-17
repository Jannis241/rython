use crate::ast::*;
use crate::lexer::Token;
use crate::lexer::TokenKind;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        expected: TokenKind,
        found: TokenKind,
        token_idx: usize,
    },
    UnexpectedTopLevel {
        found: TokenKind,
        token_idx: usize,
    },
    UnexpectedEof {
        token_idx: usize,
    },
    ExpectedStatement {
        token_idx: usize,
    },
    InvalidAssignmentTarget {
        token_idx: usize,
    },
    UnexpectedExprStart {
        found: TokenKind,
        token_idx: usize,
    },
    InvalidOperatorName {
        name: String,
        token_idx: usize,
    },
    EmptyOperatorName {
        token_idx: usize,
    },
    InvalidPattern {
        found: TokenKind,
        token_idx: usize,
    },
    InvalidCharLiteral {
        token_idx: usize,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    ast: Vec<Item>,
    pub current_idx: usize,
    allow_struct_literal: bool,
    // Name des gerade geparsten Structs/Impls für this nötig
    current_self_type: Option<String>,
}

impl Parser {
    pub fn new(input: Vec<Token>) -> Parser {
        Parser {
            tokens: input,
            ast: vec![],
            current_idx: 0,
            allow_struct_literal: true,
            current_self_type: None,
        }
    }

    pub fn advance(&mut self) -> Result<(), ParseError> {
        self.current_idx += 1;
        if self.current_idx >= self.tokens.len() {
            return Err(ParseError::UnexpectedEof {
                token_idx: self.current_idx,
            });
        }
        Ok(())
    }

    pub fn current(&self) -> Result<Token, ParseError> {
        self.tokens
            .get(self.current_idx)
            .cloned()
            .ok_or(ParseError::UnexpectedEof {
                token_idx: self.current_idx,
            })
    }

    pub fn peek(&self) -> Result<Token, ParseError> {
        self.tokens
            .get(self.current_idx + 1)
            .cloned()
            .ok_or(ParseError::UnexpectedEof {
                token_idx: self.current_idx + 1,
            })
    }

    pub fn expect_current(&self, expected: TokenKind) -> Result<(), ParseError> {
        let found = self.current()?.kind;
        if found != expected {
            return Err(ParseError::UnexpectedToken {
                expected,
                found,
                token_idx: self.current_idx,
            });
        }
        Ok(())
    }

    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_assignment()
    }

    fn parse_expr_no_struct(&mut self) -> Result<Expr, ParseError> {
        let prev = self.allow_struct_literal;
        self.allow_struct_literal = false;
        let result = self.parse_expr();
        self.allow_struct_literal = prev;
        result
    }

    fn parse_assignment(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_or()?;

        let compound_op = match self.current()?.kind {
            TokenKind::Eq => {
                self.advance()?;
                let value = self.parse_assignment()?;
                if !is_valid_lvalue(&lhs) {
                    return Err(ParseError::InvalidAssignmentTarget {
                        token_idx: self.current_idx,
                    });
                }
                return Ok(Expr::Assign {
                    target: Box::new(lhs),
                    value: Box::new(value),
                });
            }
            TokenKind::PlusEq => BinaryOp::Add,
            TokenKind::MinusEq => BinaryOp::Sub,
            TokenKind::StarEq => BinaryOp::Mul,
            TokenKind::SlashEq => BinaryOp::Div,
            _ => return Ok(lhs),
        };

        self.advance()?;
        let value = self.parse_assignment()?;
        if !is_valid_lvalue(&lhs) {
            return Err(ParseError::InvalidAssignmentTarget {
                token_idx: self.current_idx,
            });
        }
        Ok(Expr::BinaryOpAssign {
            target: Box::new(lhs),
            binary_op: compound_op,
            value: Box::new(value),
        })
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_and()?;
        while matches!(self.current()?.kind, TokenKind::Or | TokenKind::PipePipe) {
            self.advance()?;
            let rhs = self.parse_and()?;
            lhs = Expr::BinaryOp {
                lhs: Box::new(lhs),
                binary_op: BinaryOp::Or,
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_equality()?;
        while matches!(self.current()?.kind, TokenKind::And | TokenKind::AmpAmp) {
            self.advance()?;
            let rhs = self.parse_equality()?;
            lhs = Expr::BinaryOp {
                lhs: Box::new(lhs),
                binary_op: BinaryOp::And,
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_comparison()?;
        loop {
            let op = match self.current()?.kind {
                TokenKind::EqEq => BinaryOp::Eq,
                TokenKind::BangEq => BinaryOp::Ne,
                _ => break,
            };
            self.advance()?;
            let rhs = self.parse_comparison()?;
            lhs = Expr::BinaryOp {
                lhs: Box::new(lhs),
                binary_op: op,
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_bitor()?;
        loop {
            let op = match self.current()?.kind {
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::LtEq => BinaryOp::Le,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::GtEq => BinaryOp::Ge,
                _ => break,
            };
            self.advance()?;
            let rhs = self.parse_bitor()?;
            lhs = Expr::BinaryOp {
                lhs: Box::new(lhs),
                binary_op: op,
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_bitor(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_bitxor()?;
        while self.current()?.kind == TokenKind::Pipe {
            self.advance()?;
            let rhs = self.parse_bitxor()?;
            lhs = Expr::BinaryOp {
                lhs: Box::new(lhs),
                binary_op: BinaryOp::BitOr,
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_bitxor(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_bitand()?;
        while self.current()?.kind == TokenKind::Caret {
            self.advance()?;
            let rhs = self.parse_bitand()?;
            lhs = Expr::BinaryOp {
                lhs: Box::new(lhs),
                binary_op: BinaryOp::BitXor,
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_bitand(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_shift()?;
        while self.current()?.kind == TokenKind::Amp {
            self.advance()?;
            let rhs = self.parse_shift()?;
            lhs = Expr::BinaryOp {
                lhs: Box::new(lhs),
                binary_op: BinaryOp::BitAnd,
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_shift(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_additive()?;
        loop {
            let op = match self.current()?.kind {
                TokenKind::LtLt => BinaryOp::Shl,
                TokenKind::GtGt => BinaryOp::Shr,
                _ => break,
            };
            self.advance()?;
            let rhs = self.parse_additive()?;
            lhs = Expr::BinaryOp {
                lhs: Box::new(lhs),
                binary_op: op,
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_multiplicative()?;
        loop {
            let op = match self.current()?.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance()?;
            let rhs = self.parse_multiplicative()?;
            lhs = Expr::BinaryOp {
                lhs: Box::new(lhs),
                binary_op: op,
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.current()?.kind {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                _ => break,
            };
            self.advance()?;
            let rhs = self.parse_unary()?;
            lhs = Expr::BinaryOp {
                lhs: Box::new(lhs),
                binary_op: op,
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.current()?.kind == TokenKind::Plus {
            // +-x parses as +(-x)
            self.advance()?;
            return self.parse_unary();
        }
        let op = match self.current()?.kind {
            TokenKind::Minus => UnaryOp::Neg,
            TokenKind::Tilde => UnaryOp::BitNot,
            TokenKind::Bang | TokenKind::Not => UnaryOp::Not,
            _ => return self.parse_postfix(),
        };
        self.advance()?;
        let value = self.parse_unary()?;
        Ok(Expr::Unary {
            op,
            value: Box::new(value),
        })
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.current()?.kind {
                TokenKind::LParen => {
                    expr = self.finish_call(expr, Vec::new())?;
                }
                TokenKind::ColonColon => {
                    // TODO kein turbofish ????j
                    // // turbofish: expr::<T, U, ...>(args)
                    // self.advance()?;
                    // self.expect_current(TokenKind::Lt)?;
                    // let type_args = self.parse_type_args()?;
                    // self.expect_current(TokenKind::LParen)?;
                    // expr = self.finish_call(expr, type_args)?;
                    unimplemented!()
                }
                TokenKind::Dot => {
                    self.advance()?;
                    self.expect_current(TokenKind::Ident)?;
                    let field_name = self.current()?.value;
                    self.advance()?;
                    expr = Expr::FieldAccess {
                        object: Box::new(expr),
                        field_name,
                    };
                }
                TokenKind::LBracket => {
                    self.advance()?;
                    let prev = self.allow_struct_literal;
                    self.allow_struct_literal = true;
                    let index = self.parse_expr()?;
                    self.allow_struct_literal = prev;
                    self.expect_current(TokenKind::RBracket)?;
                    self.advance()?;
                    expr = Expr::PostFix {
                        Op: PostFixOp::Brackets(Box::new(index)),
                        value: Box::new(expr),
                    };
                }
                TokenKind::PlusPlus => {
                    self.advance()?;
                    expr = Expr::PostFix {
                        Op: PostFixOp::PlusPlus,
                        value: Box::new(expr),
                    };
                }
                TokenKind::MinusMinus => {
                    self.advance()?;
                    expr = Expr::PostFix {
                        Op: PostFixOp::MinusMinus,
                        value: Box::new(expr),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr, type_args: Vec<Type>) -> Result<Expr, ParseError> {
        self.expect_current(TokenKind::LParen)?;
        self.advance()?;
        let mut args = vec![];
        if self.current()?.kind != TokenKind::RParen {
            args.push(self.parse_expr()?);
            while self.current()?.kind == TokenKind::Comma {
                self.advance()?;
                args.push(self.parse_expr()?);
            }
        }
        self.expect_current(TokenKind::RParen)?;
        self.advance()?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            type_args,
            arguments: args,
        })
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let curr = self.current()?;
        match curr.kind {
            TokenKind::While => self.parse_while(),
            TokenKind::Loop => self.parse_loop(),
            TokenKind::Int => {
                self.advance()?;
                Ok(Expr::IntLiteral(curr.value))
            }
            TokenKind::Float => {
                self.advance()?;
                Ok(Expr::FloatLiteral(curr.value))
            }
            TokenKind::True => {
                self.advance()?;
                Ok(Expr::BoolLiteral(true))
            }
            TokenKind::False => {
                self.advance()?;
                Ok(Expr::BoolLiteral(false))
            }
            TokenKind::Char => {
                let char_token_idx = self.current_idx;
                self.advance()?;
                let mut chars = curr.value.chars();
                let first = chars.next().ok_or(ParseError::InvalidCharLiteral {
                    token_idx: char_token_idx,
                })?;
                if chars.next().is_some() {
                    return Err(ParseError::InvalidCharLiteral {
                        token_idx: char_token_idx,
                    });
                }
                Ok(Expr::CharLiteral(first))
            }
            TokenKind::StringLiteral => {
                self.advance()?;
                Ok(Expr::StringLiteral(curr.value))
            }
            TokenKind::Ident => {
                self.advance()?;
                if self.allow_struct_literal && self.current()?.kind == TokenKind::LBrace {
                    self.parse_struct_literal(curr.value)
                } else if self.current()?.kind == TokenKind::ColonColon
                    && self.peek().is_ok()
                    && self.peek()?.kind == TokenKind::Ident
                {
                    // variant literal: MyVar::MyCase
                    self.advance()?; // konsumiert '::'
                    let case_token = self.current()?;
                    self.advance()?; // konsumiert den case ident
                    Ok(Expr::VariantLiteral {
                        variant_name: curr.value,
                        case_name: case_token.value,
                    })
                } else {
                    Ok(Expr::Variable(curr.value))
                }
            }
            TokenKind::This => {
                self.advance()?;
                Ok(Expr::Variable("this".to_string()))
            }
            TokenKind::LParen => {
                self.advance()?;
                let prev = self.allow_struct_literal;
                self.allow_struct_literal = true;
                let inner = self.parse_expr()?;
                self.allow_struct_literal = prev;
                self.expect_current(TokenKind::RParen)?;
                self.advance()?;
                Ok(Expr::Grouping(Box::new(inner)))
            }
            TokenKind::LBracket => {
                self.advance()?;
                let prev = self.allow_struct_literal;
                self.allow_struct_literal = true;
                let mut elements = vec![];
                if self.current()?.kind != TokenKind::RBracket {
                    elements.push(Box::new(self.parse_expr()?));
                    while self.current()?.kind == TokenKind::Comma {
                        self.advance()?;
                        elements.push(Box::new(self.parse_expr()?));
                    }
                }
                self.allow_struct_literal = prev;
                self.expect_current(TokenKind::RBracket)?;
                self.advance()?;
                Ok(Expr::ListLiteral(elements))
            }
            other => Err(ParseError::UnexpectedExprStart {
                found: other,
                token_idx: self.current_idx,
            }),
        }
    }

    fn parse_struct_literal(&mut self, struct_name: String) -> Result<Expr, ParseError> {
        self.expect_current(TokenKind::LBrace)?;
        self.advance()?;

        let prev = self.allow_struct_literal;
        self.allow_struct_literal = true;

        let mut arguments = vec![];

        loop {
            if self.current()?.kind == TokenKind::RBrace {
                self.advance()?;
                break;
            }

            self.expect_current(TokenKind::Ident)?;
            let field_name = self.current()?.value;
            self.advance()?;
            self.expect_current(TokenKind::Colon)?;
            self.advance()?;
            let value = self.parse_expr()?;
            arguments.push((field_name, value));

            if self.current()?.kind == TokenKind::RBrace {
                self.advance()?;
                break;
            }
            self.expect_current(TokenKind::Comma)?;
            self.advance()?;
        }

        self.allow_struct_literal = prev;

        Ok(Expr::StructLiteral {
            struct_name,
            arguments,
        })
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        if self.current()?.kind == TokenKind::Any {
            self.advance()?;

            let mut trait_bounds = vec![];

            loop {
                self.expect_current(TokenKind::Ident)?;
                let trait_name = self.current()?.value;
                let trait_bound = TraitBound {
                    args: vec![],
                    trait_name,
                };
                trait_bounds.push(trait_bound);

                self.advance()?;

                if self.current()?.kind == TokenKind::Plus {
                    self.advance()?;
                    continue;
                }
                break;
            }

            Ok(Type::AnyTrait(trait_bounds))
        } else {
            // primitive Typ-Keywords (aktuell nur `char`) sind eigene TokenKinds,
            // werden hier aber wie Idents als Type::Named uebernommen.
            let kind = self.current()?.kind;
            if kind != TokenKind::Ident && kind != TokenKind::Char {
                return Err(ParseError::UnexpectedToken {
                    expected: TokenKind::Ident,
                    found: kind,
                    token_idx: self.current_idx,
                });
            }
            let type_name = self.current()?.value;
            self.advance()?;
            Ok(Type::Named(type_name))
        }
    }

    fn parse_generic_params(&mut self) -> Result<Vec<GenericParam>, ParseError> {
        let mut generic_params = vec![];
        if self.current()?.kind != TokenKind::Lt {
            return Ok(generic_params);
        }
        self.advance()?;

        loop {
            if self.current()?.kind == TokenKind::Gt {
                self.advance()?;
                break;
            }

            self.expect_current(TokenKind::Ident)?;
            let param_name = self.current()?.value;
            self.advance()?;

            let bounds = if self.current()?.kind == TokenKind::Colon {
                self.advance()?;
                let mut bounds = vec![];
                loop {
                    // self.expect_current(TokenKind::Any)?; //TODO ????????? wollen wir das so
                    // haben?????ß
                    // self.advance()?;

                    self.expect_current(TokenKind::Ident)?;
                    let bound_name = self.current()?.value;
                    bounds.push(TraitBound {
                        trait_name: bound_name,
                        args: vec![],
                    });
                    self.advance()?;
                    if self.current()?.kind == TokenKind::Plus {
                        self.advance()?;
                        continue;
                    }
                    break;
                }
                bounds
            } else {
                vec![]
            };

            generic_params.push(GenericParam {
                name: param_name,
                bounds,
            });

            if self.current()?.kind == TokenKind::Gt {
                self.advance()?;
                break;
            }
            self.expect_current(TokenKind::Comma)?;
            self.advance()?;
        }

        Ok(generic_params)
    }

    fn parse_type_args(&mut self) -> Result<Vec<Type>, ParseError> {
        let mut args = vec![];
        if self.current()?.kind != TokenKind::Lt {
            return Ok(args);
        }
        self.advance()?;

        loop {
            if self.current()?.kind == TokenKind::Gt {
                self.advance()?;
                break;
            }
            args.push(self.parse_type()?);
            if self.current()?.kind == TokenKind::Gt {
                self.advance()?;
                break;
            }
            self.expect_current(TokenKind::Comma)?;
            self.advance()?;
        }

        Ok(args)
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        self.expect_current(TokenKind::LParen)?;
        self.advance()?;

        let mut params = vec![];

        loop {
            if self.current()?.kind == TokenKind::RParen {
                self.advance()?;
                break;
            }

            if self.current()?.kind == TokenKind::This {
                let struct_name =
                    self.current_self_type
                        .clone()
                        .ok_or(ParseError::UnexpectedToken {
                            expected: TokenKind::Ident,
                            found: TokenKind::This,
                            token_idx: self.current_idx,
                        })?;
                self.advance()?;
                params.push(Param {
                    name: "this".to_string(),
                    param_type: Type::Named(struct_name),
                });

                if self.current()?.kind == TokenKind::RParen {
                    self.advance()?;
                    break;
                }
                self.expect_current(TokenKind::Comma)?;
                self.advance()?;
                continue;
            }

            self.expect_current(TokenKind::Ident)?;
            let param_name = self.current()?.value;
            self.advance()?;
            self.expect_current(TokenKind::Colon)?;
            self.advance()?;

            params.push(Param {
                name: param_name,
                param_type: self.parse_type()?,
            });

            if self.current()?.kind == TokenKind::RParen {
                self.advance()?;
                break;
            }
            self.expect_current(TokenKind::Comma)?;
            self.advance()?;
        }

        Ok(params)
    }

    fn parse_operator_name(&mut self) -> Result<Option<String>, ParseError> {
        if self.current()?.kind != TokenKind::Operator {
            return Ok(None);
        }
        let operator_keyword_idx = self.current_idx;
        self.advance()?;

        let mut operator = String::new();
        loop {
            let kind = self.current()?.kind;
            if !is_operator_token(&kind) {
                break;
            }
            operator.push_str(self.current()?.value.as_str());
            self.advance()?;
        }

        if operator.is_empty() {
            return Err(ParseError::EmptyOperatorName {
                token_idx: operator_keyword_idx,
            });
        }
        if !is_known_operator(&operator) {
            return Err(ParseError::InvalidOperatorName {
                name: operator,
                token_idx: operator_keyword_idx,
            });
        }
        Ok(Some(operator))
    }

    fn parse_import(&mut self) -> Result<(), ParseError> {
        self.expect_current(TokenKind::Import)?;
        self.advance()?;
        let mut whole_path = String::new();

        loop {
            self.expect_current(TokenKind::Ident)?;
            let value = self.current()?.value;
            whole_path.push_str(value.as_str());

            if self.peek()?.kind == TokenKind::Dot {
                whole_path.push('.');
                self.advance()?;
                self.advance()?;
                continue;
            }

            self.advance()?;
            self.expect_current(TokenKind::Semicolon)?;
            self.advance()?;
            break;
        }

        self.ast.push(Item::Import(Import {
            import_name: whole_path,
        }));
        Ok(())
    }

    fn parse_global_var(&mut self) -> Result<(), ParseError> {
        self.expect_current(TokenKind::Global)?;
        self.advance()?;
        self.expect_current(TokenKind::Ident)?;
        let var_name = self.current()?.value;
        self.advance()?;
        self.expect_current(TokenKind::Colon)?;
        self.advance()?;
        let var_type = self.parse_type()?;
        self.expect_current(TokenKind::Eq)?;
        self.advance()?;
        let value = self.parse_expr()?;
        self.expect_current(TokenKind::Semicolon)?;
        self.advance()?;

        self.ast.push(Item::GlobalVar(GlobalVar {
            var_name,
            var_type,
            value,
        }));
        Ok(())
    }

    fn parse_const_var(&mut self) -> Result<(), ParseError> {
        self.expect_current(TokenKind::Const)?;
        self.advance()?;
        self.expect_current(TokenKind::Ident)?;
        let var_name = self.current()?.value;
        self.advance()?;
        self.expect_current(TokenKind::Colon)?;
        self.advance()?;
        let var_type = self.parse_type()?;
        self.expect_current(TokenKind::Eq)?;
        self.advance()?;
        let value = self.parse_expr()?;
        self.expect_current(TokenKind::Semicolon)?;
        self.advance()?;

        self.ast.push(Item::ConstVar(ConstVar {
            var_name,
            var_type,
            value,
        }));
        Ok(())
    }

    fn parse_variant(&mut self) -> Result<(), ParseError> {
        self.expect_current(TokenKind::Variant)?;
        self.advance()?;
        self.expect_current(TokenKind::Ident)?;
        let variant_name = self.current()?.value;
        self.advance()?;
        self.expect_current(TokenKind::LBrace)?;
        self.advance()?;

        let mut cases = vec![];

        loop {
            if self.current()?.kind == TokenKind::RBrace {
                self.advance()?;
                break;
            }

            self.expect_current(TokenKind::Ident)?;
            cases.push(self.current()?.value);
            self.advance()?;

            if self.current()?.kind == TokenKind::RBrace {
                self.advance()?;
                break;
            }

            self.expect_current(TokenKind::Comma)?;
            self.advance()?;
        }

        self.ast.push(Item::Variant(Variant {
            variant_name,
            cases,
        }));
        Ok(())
    }

    fn parse_trait(&mut self) -> Result<(), ParseError> {
        self.expect_current(TokenKind::Trait)?;
        self.advance()?;
        self.expect_current(TokenKind::Ident)?;
        let trait_name = self.current()?.value;
        self.advance()?;

        let generic_params = self.parse_generic_params()?;

        self.expect_current(TokenKind::LBrace)?;
        self.advance()?;

        let mut function_signatures = vec![];

        loop {
            if self.current()?.kind == TokenKind::RBrace {
                self.advance()?;
                break;
            }
            function_signatures.push(self.parse_function_signature()?);
        }

        self.ast.push(Item::Trait(Trait {
            trait_name,
            generic_params,
            function_signatures,
        }));
        Ok(())
    }

    fn parse_function_signature(&mut self) -> Result<FunctionSignature, ParseError> {
        self.expect_current(TokenKind::Fn)?;
        self.advance()?;

        let operator = self.parse_operator_name()?;

        self.expect_current(TokenKind::Ident)?;
        let function_name = self.current()?.value;
        self.advance()?;

        let generic_params = self.parse_generic_params()?;
        let params = self.parse_params()?;

        let return_type = if self.current()?.kind == TokenKind::Semicolon {
            None
        } else {
            Some(self.parse_type()?)
        };

        self.expect_current(TokenKind::Semicolon)?;
        self.advance()?;

        Ok(FunctionSignature {
            function_name,
            generic_params,
            params,
            return_type,
            operator,
        })
    }

    fn parse_struct(&mut self) -> Result<(), ParseError> {
        self.expect_current(TokenKind::Struct)?;
        self.advance()?;
        self.expect_current(TokenKind::Ident)?;
        let struct_name = self.current()?.value;
        self.advance()?;

        let generic_params = self.parse_generic_params()?;

        self.expect_current(TokenKind::LBrace)?;
        self.advance()?;

        let mut fields = vec![];
        let mut functions = vec![];

        let prev_self = self.current_self_type.take();
        self.current_self_type = Some(struct_name.clone());

        loop {
            if self.current()?.kind == TokenKind::RBrace {
                self.advance()?;
                break;
            }

            if self.current()?.kind == TokenKind::Fn {
                functions.push(self.parse_fn_def()?);
            } else {
                self.expect_current(TokenKind::Ident)?;
                let field_name = self.current()?.value;
                self.advance()?;
                self.expect_current(TokenKind::Colon)?;
                self.advance()?;
                let field_type = self.parse_type()?;
                fields.push(StructField {
                    field_name,
                    field_type,
                });

                if self.current()?.kind == TokenKind::RBrace {
                    self.advance()?;
                    break;
                }
                self.expect_current(TokenKind::Comma)?;
                self.advance()?;
            }
        }

        self.current_self_type = prev_self;

        self.ast.push(Item::Struct(Struct {
            struct_name,
            generic_params,
            fields,
            functions,
        }));
        Ok(())
    }

    fn parse_fn(&mut self) -> Result<(), ParseError> {
        let function = self.parse_fn_def()?;
        self.ast.push(Item::Function(function));
        Ok(())
    }

    fn parse_fn_def(&mut self) -> Result<Function, ParseError> {
        self.expect_current(TokenKind::Fn)?;
        self.advance()?;

        let operator = self.parse_operator_name()?;

        self.expect_current(TokenKind::Ident)?;
        let function_name = self.current()?.value;
        self.advance()?;

        let generic_params = self.parse_generic_params()?;
        let params = self.parse_params()?;

        let return_type = if self.current()?.kind == TokenKind::LBrace {
            None
        } else {
            Some(self.parse_type()?)
        };

        self.expect_current(TokenKind::LBrace)?;
        self.advance()?;

        let body = self.parse_block()?;

        self.expect_current(TokenKind::RBrace)?;
        self.advance()?;

        Ok(Function {
            name: function_name,
            generic_params,
            params,
            body,
            return_type,
            operator,
        })
    }

    fn parse_trait_implementation(&mut self) -> Result<(), ParseError> {
        self.expect_current(TokenKind::Impl)?;
        self.advance()?;

        let generic_params = self.parse_generic_params()?;

        self.expect_current(TokenKind::Ident)?;
        let trait_name = self.current()?.value;
        self.advance()?;

        let trait_args = self.parse_type_args()?;

        self.expect_current(TokenKind::For)?;
        self.advance()?;

        self.expect_current(TokenKind::Ident)?;
        let struct_name = self.current()?.value;
        self.advance()?;

        let struct_args = self.parse_type_args()?;

        self.expect_current(TokenKind::LBrace)?;
        self.advance()?;

        let mut functions = vec![];

        let prev_self = self.current_self_type.take();
        self.current_self_type = Some(struct_name.clone());

        loop {
            if self.current()?.kind == TokenKind::RBrace {
                self.advance()?;
                break;
            }
            functions.push(self.parse_fn_def()?);
        }

        self.current_self_type = prev_self;

        self.ast
            .push(Item::TraitImplementation(TraitImplementation {
                generic_params,
                trait_name,
                trait_args,
                struct_name,
                struct_args,
                functions,
            }));
        Ok(())
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let mut block = Block { statements: vec![] };

        loop {
            if self.current()?.kind == TokenKind::RBrace {
                break;
            }
            if is_statement_start(self.current()?.kind) {
                block.statements.push(self.parse_statement()?);
                continue;
            }
            // expression statement, possibly with implicit-return form
            let expr = self.parse_expr()?;
            match self.current()?.kind {
                TokenKind::Semicolon => {
                    self.advance()?;
                    block.statements.push(Stmt::Expr(expr));
                }
                TokenKind::RBrace => {
                    block.statements.push(Stmt::Return(Return {
                        return_value: Some(expr),
                    }));
                    break;
                }
                _ => {
                    self.expect_current(TokenKind::Semicolon)?;
                }
            }
        }

        Ok(block)
    }

    pub fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.current()?.kind {
            TokenKind::Let => self.parse_let(),
            TokenKind::If => self.parse_if(),
            // TokenKind::While => self.parse_while(),
            // TokenKind::Loop => self.parse_loop(),
            TokenKind::For => self.parse_for(),
            TokenKind::Yield => self.parse_yield(),
            TokenKind::Break => self.parse_break(),
            TokenKind::Continue => self.parse_continue(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Asm => self.parse_asm(),
            TokenKind::LBrace => self.parse_new_scope(),
            _ => self.parse_expr_statement(),
        }
    }

    fn parse_new_scope(&mut self) -> Result<Stmt, ParseError> {
        self.expect_current(TokenKind::LBrace)?;
        self.advance()?;
        let block = self.parse_block()?;
        self.expect_current(TokenKind::RBrace)?;
        self.advance()?;

        Ok(Stmt::Block(block))
    }

    fn parse_let(&mut self) -> Result<Stmt, ParseError> {
        self.expect_current(TokenKind::Let)?;
        self.advance()?;
        self.expect_current(TokenKind::Ident)?;
        let name = self.current()?.value;
        self.advance()?;
        self.expect_current(TokenKind::Colon)?;
        self.advance()?;
        let var_type = self.parse_type()?;
        self.expect_current(TokenKind::Eq)?;
        self.advance()?;
        let value = self.parse_expr()?;
        self.expect_current(TokenKind::Semicolon)?;
        self.advance()?;

        Ok(Stmt::Let(Let {
            var_name: name,
            var_type: var_type,
            value,
        }))
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        self.expect_current(TokenKind::If)?;
        self.advance()?;

        let condition = self.parse_expr_no_struct()?;
        self.expect_current(TokenKind::LBrace)?;
        self.advance()?;
        let if_code = self.parse_block()?;
        self.expect_current(TokenKind::RBrace)?;
        self.advance()?;

        let else_code = self.parse_else_branch()?;

        Ok(Stmt::If(If {
            condition,
            if_code,
            else_code,
        }))
    }

    fn parse_else_branch(&mut self) -> Result<Option<Box<Stmt>>, ParseError> {
        if self.current()?.kind != TokenKind::Else {
            return Ok(None);
        }
        self.advance()?;
        if self.current()?.kind == TokenKind::If {
            Ok(Some(Box::new(self.parse_if()?)))
        } else {
            self.expect_current(TokenKind::LBrace)?;
            self.advance()?;
            let block = self.parse_block()?;
            self.expect_current(TokenKind::RBrace)?;
            self.advance()?;
            Ok(Some(Box::new(Stmt::Block(block))))
        }
    }

    fn parse_while(&mut self) -> Result<Expr, ParseError> {
        self.expect_current(TokenKind::While)?;
        self.advance()?;
        let condition = self.parse_expr_no_struct()?;
        self.expect_current(TokenKind::LBrace)?;
        self.advance()?;
        let inner_code = self.parse_block()?;
        self.expect_current(TokenKind::RBrace)?;
        self.advance()?;
        Ok(Expr::WhileExpr(While {
            condition: Box::new(condition),
            inner_code,
        }))
    }

    fn parse_loop(&mut self) -> Result<Expr, ParseError> {
        self.expect_current(TokenKind::Loop)?;
        self.advance()?;
        self.expect_current(TokenKind::LBrace)?;
        self.advance()?;
        let inner_code = self.parse_block()?;
        self.expect_current(TokenKind::RBrace)?;
        self.advance()?;
        Ok(Expr::LoopExpr(Loop { inner_code }))
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        self.expect_current(TokenKind::For)?;
        self.advance()?;
        self.expect_current(TokenKind::Ident)?;
        let var_name = self.current()?.value;
        self.advance()?;
        self.expect_current(TokenKind::In)?;
        self.advance()?;
        let iterable = self.parse_expr_no_struct()?;
        self.expect_current(TokenKind::LBrace)?;
        self.advance()?;
        let inner_code = self.parse_block()?;
        self.expect_current(TokenKind::RBrace)?;
        self.advance()?;
        Ok(Stmt::For(For {
            var_name,
            iterable,
            inner_code,
        }))
    }

    fn parse_break(&mut self) -> Result<Stmt, ParseError> {
        self.expect_current(TokenKind::Break)?;
        self.advance()?;
        self.expect_current(TokenKind::Semicolon)?;
        self.advance()?;
        Ok(Stmt::Break)
    }

    fn parse_continue(&mut self) -> Result<Stmt, ParseError> {
        self.expect_current(TokenKind::Continue)?;
        self.advance()?;
        self.expect_current(TokenKind::Semicolon)?;
        self.advance()?;
        Ok(Stmt::Continue)
    }
    fn parse_yield(&mut self) -> Result<Stmt, ParseError> {
        // einfach mal return gecopied (funktioniert bestimmt, ka wie das hier sonst geht gurt yo)
        self.expect_current(TokenKind::Yield)?;
        self.advance()?;
        let return_value = match self.current()?.kind {
            TokenKind::Semicolon | TokenKind::RBrace => None,
            _ => Some(self.parse_expr()?),
        };
        if self.current()?.kind == TokenKind::Semicolon {
            self.advance()?;
        } else if self.current()?.kind != TokenKind::RBrace {
            self.expect_current(TokenKind::Semicolon)?;
        }
        Ok(Stmt::Yield(Yield {
            value: return_value,
        }))
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        self.expect_current(TokenKind::Return)?;
        self.advance()?;
        let return_value = match self.current()?.kind {
            TokenKind::Semicolon | TokenKind::RBrace => None,
            _ => Some(self.parse_expr()?),
        };
        if self.current()?.kind == TokenKind::Semicolon {
            self.advance()?;
        } else if self.current()?.kind != TokenKind::RBrace {
            self.expect_current(TokenKind::Semicolon)?;
        }
        Ok(Stmt::Return(Return { return_value }))
    }

    fn parse_asm(&mut self) -> Result<Stmt, ParseError> {
        self.expect_current(TokenKind::Asm)?;
        let asm_code = self.current()?.value;
        self.advance()?;
        self.expect_current(TokenKind::Semicolon)?;
        self.advance()?;
        Ok(Stmt::Asm(Asm { asm_code }))
    }

    fn parse_expr_statement(&mut self) -> Result<Stmt, ParseError> {
        let stmt = Stmt::Expr(self.parse_expr()?);
        self.expect_current(TokenKind::Semicolon)?;
        self.advance()?;
        Ok(stmt)
    }

    pub fn parse(&mut self) -> Result<Vec<Item>, ParseError> {
        loop {
            if self.current_idx >= self.tokens.len() {
                break;
            }

            match self.current()?.kind {
                TokenKind::Trait => self.parse_trait()?,
                TokenKind::Import => self.parse_import()?,
                TokenKind::Global => self.parse_global_var()?,
                TokenKind::Const => self.parse_const_var()?,
                TokenKind::Variant => self.parse_variant()?,
                TokenKind::Struct => self.parse_struct()?,
                TokenKind::Fn => self.parse_fn()?,
                TokenKind::Impl => self.parse_trait_implementation()?,
                TokenKind::Eof => break,
                TokenKind::Semicolon => self.advance()?,
                TokenKind::Asm => {
                    let asm_stmt = self.parse_asm()?;
                    let Stmt::Asm(asm) = asm_stmt else {
                        unreachable!()
                    };
                    self.ast.push(Item::Asm(asm));
                }
                other => {
                    return Err(ParseError::UnexpectedTopLevel {
                        found: other,
                        token_idx: self.current_idx,
                    });
                }
            }
        }

        Ok(self.ast.clone())
    }
}

fn is_valid_lvalue(expr: &Expr) -> bool {
    matches!(expr, Expr::Variable(_) | Expr::FieldAccess { .. })
}

fn is_statement_start(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Let
            | TokenKind::If
            | TokenKind::Yield
            // | TokenKind::While
            // | TokenKind::Loop
            | TokenKind::For
            | TokenKind::Break
            | TokenKind::Continue
            | TokenKind::Return
            | TokenKind::Asm
            | TokenKind::LBrace
    )
}

fn is_operator_token(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::EqEq
            | TokenKind::BangEq
            | TokenKind::Lt
            | TokenKind::LtEq
            | TokenKind::Gt
            | TokenKind::GtEq
            | TokenKind::Amp
            | TokenKind::Pipe
            | TokenKind::Caret
            | TokenKind::Tilde
            | TokenKind::LtLt
            | TokenKind::GtGt
            | TokenKind::Bang
            | TokenKind::LBracket
            | TokenKind::RBracket
    )
}

fn is_known_operator(name: &str) -> bool {
    matches!(
        name,
        "+" | "-"
            | "*"
            | "/"
            | "%"
            | "=="
            | "!="
            | "<"
            | "<="
            | ">"
            | ">="
            | "&"
            | "|"
            | "^"
            | "~"
            | "<<"
            | ">>"
            | "!"
            | "[]"
    )
}
