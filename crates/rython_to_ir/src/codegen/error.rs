use crate::ast::{Expr, Item, Stmt};
use crate::ir::IrType;

#[derive(Debug, Clone)]
pub enum CodegenError {
    InvalidItem(Item),
    MismatchedTypes(IrType, IrType), // expected type, got

    UnknownVariable(String),

    InvalidIntLiteral(String),
    InvalidFloatLiteral(String),

    InvalidExpr(Expr),
    InvalidStatement(Stmt),
    InvalidReturnType(IrType, IrType), // expected, got
}
