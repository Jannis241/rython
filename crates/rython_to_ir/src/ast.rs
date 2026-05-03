pub enum Type {
    Named(String),
    AnyTrait(Vec<TraitBound>),
}

pub struct TraitBound {
    pub trait_name: String,
    pub args: Vec<Type>,
}

pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TraitBound>,
}

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
pub struct Function {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub body: Block,
    pub return_type: Option<Type>,
}
pub struct Block {
    pub statements: Vec<Stmt>,
}
pub struct Param {
    pub name: String,
    pub param_type: Type,
}
//-----------------FUNCTION--------------------

//-----------------Trait-----------------------
pub struct Trait {
    pub trait_name: String,
    pub generic_params: Vec<GenericParam>,
    pub function_signatures: Vec<FunctionSignature>,
}
pub struct FunctionSignature {
    pub function_name: String,
    pub generic_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
}
//-----------------Trait-----------------------

//-----------------Struct----------------------
pub struct Struct {
    pub struct_name: String,
    pub generic_params: Vec<GenericParam>,
    pub fields: Vec<StructField>,
    pub functions: Vec<Function>,
}
pub struct StructField {
    pub field_name: String,
    pub field_type: Type,
}
//-----------------Struct----------------------

//-----------------Variant---------------------
// A sum type / ADT, e.g.:
//   variant Color { Red, Green, Blue }
//   variant Option<T> { Some(T), None }
//   variant Result<T, E> { Ok { value: T }, Err { error: E } }
pub struct Variant {
    pub variant_name: String,
    pub generic_params: Vec<GenericParam>,
    pub cases: Vec<VariantCase>,
}
pub enum VariantCase {
    Unit(String),
    Tuple(String, Vec<Type>),
    Record(String, Vec<StructField>),
}
//-----------------Variant---------------------

//-----------------TraitImplementation---------
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
pub struct Import {
    pub import_name: String,
}
//-----------------Import----------------------

//-----------------GlobalVar-------------------
pub struct GlobalVar {
    pub var_name: String,
    pub var_type: Type,
    pub value: Expr,
}
//-----------------GlobalVar-------------------

//-----------------ConstVar--------------------
pub struct ConstVar {
    pub var_name: String,
    pub var_type: Type,
    pub value: Expr,
}
//-----------------ConstVar--------------------

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

pub struct Let {
    pub var_name: String,
    pub var_type: Option<Type>,
    pub value: Expr,
}

pub struct If {
    pub condition: Expr,
    pub if_code: Block,
    pub else_code: Option<Box<Stmt>>,
}

pub struct Loop {
    pub inner_code: Block,
}

pub struct While {
    pub condition: Expr,
    pub inner_code: Block,
}

pub struct For {
    pub var_name: String,
    pub iterable: Expr,
    pub inner_code: Block,
}

pub struct Return {
    pub return_value: Option<Expr>,
}

pub struct Asm {
    pub asm_code: String,
}

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

    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    StringLiteral(String),
    ListLiteral(Vec<Box<Expr>>),
    StructLiteral {
        struct_name: String,
        arguments: Vec<(String, Expr)>,
    },

    Grouping(Box<Expr>),
}

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

pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
}
