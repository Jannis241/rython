#[derive(Debug, Clone)]
pub enum Type {
    Named(String),
    AnyTrait(Vec<TraitBound>),
}

#[derive(Debug, Clone)]
pub struct TraitBound {
    pub trait_name: String,
    pub args: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<TraitBound>,
}

#[derive(Debug, Clone)]
pub enum Item {
    GlobalVar(GlobalVar),
    ConstVar(ConstVar),
    Function(Function),
    Trait(Trait),
    Struct(Struct),
    Variant(Variant),
    TraitImplementation(TraitImplementation),
    Import(Import),
    Asm(Asm),
}

//-----------------FUNCTION--------------------
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub body: Block,
    pub return_type: Option<Type>,
    pub operator: Option<String>,
}
#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Stmt>,
}
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub param_type: Type,
}
//-----------------FUNCTION--------------------

//-----------------Trait-----------------------
#[derive(Debug, Clone)]
pub struct Trait {
    pub trait_name: String,
    pub generic_params: Vec<GenericParam>,
    pub function_signatures: Vec<FunctionSignature>,
}
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub function_name: String,
    pub generic_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub operator: Option<String>,
}
//-----------------Trait-----------------------

//-----------------Struct----------------------
#[derive(Debug, Clone)]
pub struct Struct {
    pub struct_name: String,
    pub generic_params: Vec<GenericParam>,
    pub fields: Vec<StructField>,
    pub functions: Vec<Function>,
}
#[derive(Debug, Clone)]
pub struct StructField {
    pub field_name: String,
    pub field_type: Type,
}
//-----------------Struct----------------------

//-----------------Variant---------------------
#[derive(Debug, Clone)]
pub struct Variant {
    pub variant_name: String,
    pub cases: Vec<String>,
}
//-----------------Variant---------------------

//-----------------TraitImplementation---------
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct Import {
    pub import_name: String,
}
//-----------------Import----------------------

//-----------------GlobalVar-------------------
#[derive(Debug, Clone)]
pub struct GlobalVar {
    pub var_name: String,
    pub var_type: Type,
    pub value: Expr,
}
//-----------------GlobalVar-------------------

//-----------------ConstVar--------------------
#[derive(Debug, Clone)]
pub struct ConstVar {
    pub var_name: String,
    pub var_type: Type,
    pub value: Expr,
}
//-----------------ConstVar--------------------

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Let {
    pub var_name: String,
    pub var_type: Type,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct If {
    pub condition: Expr,
    pub if_code: Block,
    pub else_code: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub inner_code: Block,
}

#[derive(Debug, Clone)]
pub struct While {
    pub condition: Expr,
    pub inner_code: Block,
}

#[derive(Debug, Clone)]
pub struct For {
    pub var_name: String,
    pub iterable: Expr,
    pub inner_code: Block,
}

#[derive(Debug, Clone)]
pub struct Return {
    pub return_value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct Asm {
    pub asm_code: String,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },

    BinaryOpAssign {
        target: Box<Expr>,
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
        type_args: Vec<Type>,
        arguments: Vec<Expr>,
    },

    Unary {
        op: UnaryOp,
        value: Box<Expr>,
    },

    PostFix {
        Op: PostFixOp,
        value: Box<Expr>,
    },

    FieldAccess {
        object: Box<Expr>,
        field_name: String,
    },

    Variable(String),

    IntLiteral(String),
    FloatLiteral(String),
    BoolLiteral(bool),
    StringLiteral(String),
    CharLiteral(char),
    NullLiteral,
    ListLiteral(Vec<Box<Expr>>),
    StructLiteral {
        struct_name: String,
        arguments: Vec<(String, Expr)>,
    },
    // VariantLiteral {
    //     struct_name: String,
    //     arguments: Vec<(String, Expr)>,
    // },
    Grouping(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum PostFixOp {
    Brackets(Box<Expr>),
    MinusMinus,
    PlusPlus,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
}

pub fn print_items(items: &[Item]) {
    for item in items {
        print_item(item, 0);
    }
}

fn indent(level: usize) -> String {
    "  ".repeat(level)
}

fn print_item(item: &Item, level: usize) {
    match item {
        Item::Function(f) => {
            println!("{}Function: {}", indent(level), f.name);

            if !f.generic_params.is_empty() {
                println!("{}Generics:", indent(level + 1));
                for g in &f.generic_params {
                    print_generic_param(g, level + 2);
                }
            }

            println!("{}Params:", indent(level + 1));
            for p in &f.params {
                println!(
                    "{}{}: {}",
                    indent(level + 2),
                    p.name,
                    format_type(&p.param_type)
                );
            }

            if let Some(ret) = &f.return_type {
                println!("{}Return: {}", indent(level + 1), format_type(ret));
            }

            println!("{}Body:", indent(level + 1));
            print_block(&f.body, level + 2);
        }

        Item::Struct(s) => {
            println!("{}Struct: {}", indent(level), s.struct_name);

            if !s.generic_params.is_empty() {
                println!("{}Generics:", indent(level + 1));
                for g in &s.generic_params {
                    print_generic_param(g, level + 2);
                }
            }

            println!("{}Fields:", indent(level + 1));
            for f in &s.fields {
                println!(
                    "{}{}: {}",
                    indent(level + 2),
                    f.field_name,
                    format_type(&f.field_type)
                );
            }

            if !s.functions.is_empty() {
                println!("{}Methods:", indent(level + 1));
                for func in &s.functions {
                    print_item(&Item::Function(func.clone()), level + 2);
                }
            }
        }

        Item::Trait(t) => {
            println!("{}Trait: {}", indent(level), t.trait_name);

            for sig in &t.function_signatures {
                println!("{}fn {}(...)", indent(level + 1), sig.function_name);
            }
        }

        Item::GlobalVar(v) => {
            println!(
                "{}Global {}: {}",
                indent(level),
                v.var_name,
                format_type(&v.var_type)
            );
        }

        Item::ConstVar(v) => {
            println!(
                "{}Const {}: {}",
                indent(level),
                v.var_name,
                format_type(&v.var_type)
            );
        }

        Item::Import(i) => {
            println!("{}Import: {}", indent(level), i.import_name);
        }

        Item::Variant(v) => {
            println!("{}Variant: {}", indent(level), v.variant_name);
            for case in &v.cases {
                println!("{}- {}", indent(level + 1), case);
            }
        }

        Item::TraitImplementation(ti) => {
            println!(
                "{}Impl {} for {}",
                indent(level),
                ti.trait_name,
                ti.struct_name
            );

            for f in &ti.functions {
                print_item(&Item::Function(f.clone()), level + 1);
            }
        }

        Item::Asm(asm) => {
            println!(
                "{}Asm {{...}} ({} chars)",
                indent(level),
                asm.asm_code.len()
            );
        }
    }
}

fn print_block(block: &Block, level: usize) {
    for stmt in &block.statements {
        print_stmt(stmt, level);
    }
}

fn print_stmt(stmt: &Stmt, level: usize) {
    match stmt {
        Stmt::Let(l) => {
            println!("{}let {} = ...", indent(level), l.var_name);
        }

        Stmt::Return(_) => {
            println!("{}return ...", indent(level));
        }

        Stmt::Expr(_) => {
            println!("{}expr", indent(level));
        }

        Stmt::If(i) => {
            println!("{}if (...)", indent(level));
            print_block(&i.if_code, level + 1);

            if let Some(e) = &i.else_code {
                println!("{}else", indent(level));
                print_stmt(e, level + 1);
            }
        }

        Stmt::While(w) => {
            println!("{}while (...)", indent(level));
            print_block(&w.inner_code, level + 1);
        }

        Stmt::Loop(l) => {
            println!("{}loop", indent(level));
            print_block(&l.inner_code, level + 1);
        }

        Stmt::For(f) => {
            println!("{}for {} in (...)", indent(level), f.var_name);
            print_block(&f.inner_code, level + 1);
        }

        Stmt::Block(b) => {
            print_block(b, level);
        }

        Stmt::Break => println!("{}break", indent(level)),
        Stmt::Continue => println!("{}continue", indent(level)),
        Stmt::Asm(_) => println!("{}asm {{...}}", indent(level)),
    }
}

fn format_type(t: &Type) -> String {
    match t {
        Type::Named(n) => n.clone(),
        Type::AnyTrait(bounds) => {
            let b: Vec<String> = bounds.iter().map(format_trait_bound).collect();
            format!("impl {}", b.join(" + "))
        }
    }
}

fn format_trait_bound(tb: &TraitBound) -> String {
    if tb.args.is_empty() {
        tb.trait_name.clone()
    } else {
        let args: Vec<String> = tb.args.iter().map(format_type).collect();
        format!("{}<{}>", tb.trait_name, args.join(", "))
    }
}

fn print_generic_param(g: &GenericParam, level: usize) {
    if g.bounds.is_empty() {
        println!("{}{}", indent(level), g.name);
    } else {
        let bounds: Vec<String> = g.bounds.iter().map(format_trait_bound).collect();
        println!("{}{}: {}", indent(level), g.name, bounds.join(" + "));
    }
}
