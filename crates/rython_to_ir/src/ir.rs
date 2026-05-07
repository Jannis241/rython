#[derive(Debug, Clone)]
pub struct IrModule {
    pub inline_assembly: Vec<String>,
    pub functions: Vec<IrFunction>,
    pub globals: Vec<IrGlobal>,
    pub constants: Vec<IrConstant>,
    pub types: Vec<IrTypeDefinition>,
}

#[derive(Debug, Clone)]
pub struct IrGlobal {
    pub name: String,
    pub ty: IrType,
    pub value: PrimitiveValue,
}

#[derive(Debug, Clone)]
pub struct IrConstant {
    pub name: String,
    pub ty: IrType,
    pub value: PrimitiveValue,
}

#[derive(Debug, Clone)]
pub struct IrField {
    pub name: String,
    pub ty: IrType,
}

#[derive(Debug, Clone)]
pub enum IrTypeDefinition {
    Struct { name: String, fields: Vec<IrField> },
    Variant { name: String, cases: Vec<String> },
}

impl IrModule {
    pub fn new() -> Self {
        IrModule {
            inline_assembly: Vec::new(),
            functions: Vec::new(),
            constants: Vec::new(),
            globals: Vec::new(),
            types: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IrFunction {
    pub name: String,
    pub parameter: Vec<IrField>,
    pub return_type: IrType,
    pub blocks: Vec<IrBlock>,
}

#[derive(Debug, Clone)]
pub struct IrBlock {
    pub label: String, // z.B entry:
    pub instructions: Vec<IrInstruction>,
    pub terminator: Terminator,
}
#[derive(Debug, Clone)]
pub enum IrInstruction {
    PrimitiveConst {
        temp_id: TempId,       // Wert-Temp: enthaelt danach den konstanten primitiven Wert.
        ty: IrType,            // Typ des konstanten Werts.
        value: PrimitiveValue, // Konkreter Wert, der in temp_id materialisiert wird.
    },

    // Liest den N-ten Argument-Wert beim Funktionseintritt aus dem Aufrufer-Frame.
    // Wird vom IR-Generator am Funktionsanfang fuer jeden Parameter eingefuegt.
    LoadParam {
        temp_id: TempId, // Wert-Temp: enthaelt danach den uebergebenen Parameterwert.
        index: usize,    // 0-basierter Parameterindex: 0 ist der erste Parameter.
        ty: IrType,      // Typ des geladenen Parameterwerts.
    },

    Alloca {
        temp_id: TempId, // Adress-Temp: enthaelt danach die Basisadresse des reservierten Speicherplatzes.
        ty: IrType,      // Typ des Werts, der an dieser Adresse gespeichert werden darf.
    },

    Load {
        temp_id: TempId, // Wert-Temp: enthaelt danach den Wert, der aus addr gelesen wurde.
        ty: IrType,      // Typ des gelesenen Werts.
        addr: TempId,    // Adress-Temp: Speicheradresse, aus der gelesen wird.
    },

    Store {
        ty: IrType,    // Typ des Werts, der geschrieben wird.
        value: TempId, // Wert-Temp: Wert, der nach addr geschrieben wird.
        addr: TempId,  // Adress-Temp: Speicheradresse, in die geschrieben wird.
    },

    Binary {
        temp_id: TempId, // Wert-Temp: enthaelt danach das Ergebnis der binaeren Operation.
        ty_lr: IrType,   // Typ von lhs, rhs und Ergebnis.
        ty_res: IrType,
        op: IrBinaryOp, // Operation, z.B. Add oder Eq.
        lhs: TempId,    // Wert-Temp: linker Operand.
        rhs: TempId,    // Wert-Temp: rechter Operand.
    },

    // functions
    Call {
        temp_id: Option<TempId>, // Wert-Temp fuer den Rueckgabewert; None, wenn die Funktion void liefert.
        function_name: String,   // Name der aufgerufenen Funktion.
        args: Vec<TempId>,       // Wert-Temps der bereits berechneten Argumente.
        return_type: IrType,     // Rueckgabetyp der Funktion.
    },
    GlobalAddr {
        temp_id: TempId, // Adress-Temp: enthaelt danach die Adresse des globalen Symbols.
        name: String,    // Name der globalen Variable oder Konstante.
        ty: IrType,      // Typ des Werts, der an dieser globalen Adresse liegt.
    },
    Asm {
        code: String, // Roher Assembly-Code, der unveraendert in den Output geschrieben wird.
    },

    Unary {
        temp_id: TempId, // Wert-Temp: enthaelt danach das Ergebnis der unaeren Operation.
        ty: IrType,      // Typ von value und Ergebnis.
        op: IrUnaryOp,   // Operation, z.B. Neg oder Not.
        value: TempId,   // Wert-Temp: Operand.
    },

    // -------------- variant -----------------------------------
    InitVariant {
        temp_id: TempId, // Wert- oder Adress-Temp des erzeugten Variant-Werts, je nach spaeterem Lowering.
        ty: IrType,      // Typ der Variant, z.B. Named("Option").
        case_name: String, // Ausgewaehlter Fall, z.B. Some oder None.
    },

    // -------------- structs -----------------------------------
    GetFieldAddr {
        temp_id: TempId, // Adress-Temp: enthaelt danach die Adresse des ausgewaehlten Feldes.
        base_addr: TempId, // Adress-Temp: Basisadresse des ganzen Struct-Speicherbereichs.
        field_name: String, // Name des Feldes, dessen Offset berechnet wird.
    },
}

#[derive(Debug, Clone)]
pub enum IrBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    And,
    Or,

    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

#[derive(Debug, Clone)]
pub enum IrUnaryOp {
    Neg,
    Not,
    BitNot,
}

#[derive(Clone, Debug, Copy)]
pub struct TempId(pub usize);

#[derive(Debug, Clone)]
pub enum PrimitiveValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    Pointer(TempId),
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Ret(Option<TempId>), // entweder zb ret %tmp0 bei Some(id) oder ret void bei None
    Jump {
        target: String,
    },
    Branch {
        condition: TempId,
        then_block: String,
        else_block: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrType {
    I64,
    Bool,
    Void,
    F64,
    Char,
    Named(String),
    Pointer(Box<IrType>),
}
