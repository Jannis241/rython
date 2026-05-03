#[derive(Debug)]
pub enum Type {
    Named(String),
    AnyTrait(Vec<TraitBound>),
}

#[derive(Debug)]
pub struct TraitBound {
    pub trait_name: String,
    pub args: Vec<Type>,
}

#[derive(Debug)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TraitBound>,
}

#[derive(Debug)]
pub enum Item {
    GlobalVar(GlobalVar),
    ConstVar(ConstVar),
    Function(Function),
    Trait(Trait),
    Struct(Struct),
    Variant(Variant),
    TraitImplementation(TraitImplementation),
    Import(Import),
}

//-----------------FUNCTION--------------------
#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub body: Block,
    pub return_type: Option<Type>,
    pub operator: Option<String>,
}
#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Stmt>,
}
#[derive(Debug)]
pub struct Param {
    pub name: String,
    pub param_type: Type,
}
//-----------------FUNCTION--------------------

//-----------------Trait-----------------------
#[derive(Debug)]
pub struct Trait {
    pub trait_name: String,
    pub generic_params: Vec<GenericParam>,
    pub function_signatures: Vec<FunctionSignature>,
}
#[derive(Debug)]
pub struct FunctionSignature {
    pub function_name: String,
    pub generic_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub operator: Option<String>,
}
//-----------------Trait-----------------------

//-----------------Struct----------------------
#[derive(Debug)]
pub struct Struct {
    pub struct_name: String,
    pub generic_params: Vec<GenericParam>,
    pub fields: Vec<StructField>,
    pub functions: Vec<Function>,
}
#[derive(Debug)]
pub struct StructField {
    pub field_name: String,
    pub field_type: Type,
}
//-----------------Struct----------------------

//-----------------Variant---------------------
#[derive(Debug)]
pub struct Variant {
    pub variant_name: String,
    pub cases: Vec<String>,
}
//-----------------Variant---------------------

//-----------------TraitImplementation---------
#[derive(Debug)]
pub struct TraitImplementation {
    pub generic_params: Vec<GenericParam>, // <T: Bound>

    pub trait_name: String,
    pub trait_args: Vec<Type>, // Trait<T>

    pub struct_name: String,
    pub struct_args: Vec<Type>, // Struct<T>

    pub functions: Vec<Function>,
}
//-----------------TraitImplementation---------

//-----------------Import----------------------
#[derive(Debug)]
pub struct Import {
    pub import_name: String,
}
//-----------------Import----------------------

//-----------------GlobalVar-------------------
#[derive(Debug)]
pub struct GlobalVar {
    pub var_name: String,
    pub var_type: Type,
    pub value: Expr,
}
//-----------------GlobalVar-------------------

//-----------------ConstVar--------------------
#[derive(Debug)]
pub struct ConstVar {
    pub var_name: String,
    pub var_type: Type,
    pub value: Expr,
}
//-----------------ConstVar--------------------

#[derive(Debug)]
pub enum Stmt {
    Let(Let),
    If(If),
    Loop(Loop),
    While(While),
    For(For),
    Return(Return),
    Asm(Asm),
    Block(Block),
    Break,
    Continue,
    Expr(Expr),
}

#[derive(Debug)]
pub struct Let {
    pub var_name: String,
    pub var_type: Option<Type>,
    pub value: Expr,
}

#[derive(Debug)]
pub struct If {
    pub condition: Expr,
    pub if_code: Block,
    pub else_code: Option<Box<Stmt>>,
}

#[derive(Debug)]
pub struct Loop {
    pub inner_code: Block,
}

#[derive(Debug)]
pub struct While {
    pub condition: Expr,
    pub inner_code: Block,
}

#[derive(Debug)]
pub struct For {
    pub var_name: String,
    pub iterable: Expr,
    pub inner_code: Block,
}

#[derive(Debug)]
pub struct Return {
    pub return_value: Option<Expr>,
}

#[derive(Debug)]
pub struct Asm {
    pub asm_code: String,
}

#[derive(Debug)]
pub enum Expr {
    Assign {
        target_name: String,
        value: Box<Expr>,
    },

    BinaryOpAssign {
        target_name: String,
        binary_op: BinaryOp,
        value: Box<Expr>,
    },

    BinaryOp {
        lhs: Box<Expr>,
        binary_op: BinaryOp,
        rhs: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },

    Unary {
        op: UnaryOp,
        value: Box<Expr>,
    },

    Variable(String),

    IntLiteral(String),
    FloatLiteral(String),
    BoolLiteral(bool),
    StringLiteral(String),
    ListLiteral(Vec<Box<Expr>>),
    StructLiteral {
        struct_name: String,
        arguments: Vec<(String, Expr)>,
    },

    Grouping(Box<Expr>),
}

#[derive(Debug)]
pub enum BinaryOp {
    // arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // logical
    And,
    Or,

    // bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
}
