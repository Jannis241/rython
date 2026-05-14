use crate::ast::{Expr, Item, Stmt};
use crate::ir::IrType;

#[derive(Debug, Clone)]
pub enum CodegenError {
    InvalidItem(Item),
    MismatchedTypes(IrType, IrType), // expected type, got

    WrongArgumentCount(String, usize, usize), // Func name, expected args, got
    //
    DuplicateType(String),
    DuplicateFunction(String),

    ExpectedPrimitiveValue,

    InvalidBinaryOp(IrType, IrType), // zb 1 + true
    InvalidUnaryOp(IrType),

    CodeAfterTerminator,       // wenn man zb nach return 1; noch etwas schreibt
    MissingTerminator(String), // Kein Terminator, String ist der name des labels

    UnknownVariable(String),
    UnknownFunction(String),
    UnknownField(String),
    UnknownType(String),

    AssignToConst(String),
    DuplicateGlobal(String),

    FieldsDontMatch,

    AmbigousVariable(String),
    AmbigousFunction(String),
    AmbigousField(String),
    AmbigousType(String),

    InvalidIntLiteral(String),
    InvalidFloatLiteral(String),

    InvalidExpr(Expr),
    InvalidStatement(Stmt),
    InvalidReturnType(IrType, IrType), // expected, got

    BreakOutsideLoop,
    ContinueOutsideLoop,
}
