use std::{collections::HashMap, process::Termination};

use crate::ast::{Asm, Expr, Function, Item, Stmt, Type};

#[derive(Debug, Clone)]
pub struct IrModule {
    pub inline_assembly: Vec<String>,
    pub functions: Vec<IrFunction>,
    pub globals: Vec<IrGlobal>,
    pub constants: Vec<IrConstant>,
    pub types: Vec<IrTypeDef>,
}

#[derive(Debug, Clone)]
pub struct IrGlobal {
    pub name: String,
    pub ty: IrType,
    pub value: ConstValue,
}
#[derive(Debug, Clone)]
pub struct IrConstant {
    pub name: String,
    pub ty: IrType,
    pub value: ConstValue,
}

#[derive(Debug, Clone)]
pub struct IrField {
    pub name: String,
    pub ty: IrType,
}

#[derive(Debug, Clone)]
pub enum IrTypeDef {
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
    pub parameter: Vec<IrParameter>,
    pub return_type: IrType,
    pub blocks: Vec<IrBlock>,
}

#[derive(Debug, Clone)]
pub struct IrParameter {
    pub name: String,
    pub param_type: IrType,
}

#[derive(Debug, Clone)]
pub struct IrBlock {
    pub label: String, // z.B entry:
    pub instructions: Vec<IrInstruction>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone)]
pub enum IrInstruction {
    Const {
        temp_id: TempId,   // Ergebnis-Temp, der diesen konstanten Wert bezeichnet
        ty: IrType,        // Typ des konstanten Werts
        value: ConstValue, // der konkrete konstante Wert
    },

    Alloca {
        temp_id: TempId, // Ergebnis-Temp, der die neu reservierte Adresse bezeichnet
        ty: IrType,      // Typ des Werts, der an dieser Adresse gespeichert werden darf
    },

    Load {
        temp_id: TempId, // Ergebnis-Temp, in dem der gelesene Wert landet
        ty: IrType,      // Typ des gelesenen Werts
        addr: TempId,    // Adresse, aus der gelesen wird
    },

    Store {
        ty: IrType,    // Typ des Werts, der geschrieben wird
        value: TempId, // Wert-Temp, der geschrieben wird
        addr: TempId,  // Adresse, in die geschrieben wird
    },

    Binary {
        temp_id: TempId, // Ergebnis-Temp der Binary-Operation
        ty: IrType,      // Typ des Ergebnisses
        op: IrBinaryOp,  // Operation, z.B. Add oder Eq
        lhs: TempId,     // linker Operand als Wert-Temp
        rhs: TempId,     // rechter Operand als Wert-Temp
    },

    // functions
    Call {
        temp_id: Option<TempId>, // Ergebnis-Temp fuer den Rueckgabewert; None bei void
        function_name: String,   // Name der aufgerufenen Funktion
        args: Vec<TempId>,       // Wert-Temps der bereits berechneten Argumente
        return_type: IrType,     // Rueckgabetyp der Funktion
    },
    GlobalAddr {
        temp_id: TempId,
        name: String,
        ty: IrType,
    },
    Asm {
        code: String,
    },

    InitVariant {
        temp_id: TempId,   // Ergebnis-Temp des erzeugten Variant-Werts
        ty: IrType,        // Typ der Variant, z.B. Named("Option")
        case_name: String, // ausgewaehlter Fall, z.B. Some oder None
    },

    Unary {
        temp_id: TempId, // Ergebnis-Temp der Unary-Operation
        ty: IrType,      // Typ des Ergebnisses
        op: IrUnaryOp,   // Operation, z.B. Neg oder Not
        value: TempId,   // Operand als Wert-Temp
    },
    InitArray {
        temp_id: TempId,       // Ergebnis-Temp des erzeugten Array-Werts
        element_type: IrType,  // Typ der Array-Elemente
        elements: Vec<TempId>, // Wert-Temps der Elemente
    },
    GetElementAddr {
        temp_id: TempId,   // Ergebnis-Temp, der die Adresse des Elements bezeichnet
        base_addr: TempId, // Adresse des Arrays
        index: TempId,     // Wert-Temp des Index
    },
    InitStruct {
        temp_id: TempId,               // Ergebnis-Temp des erzeugten Struct-Werts
        ty: IrType,                    // Typ des Structs, z.B. Named("Point")
        fields: Vec<(String, TempId)>, // Feldname und Wert-Temp des jeweiligen Feldwerts
    },
    GetFieldAddr {
        temp_id: TempId,    // Ergebnis-Temp, der die Adresse des Feldes bezeichnet
        base_addr: TempId,  // Adresse des ganzen Structs
        field_name: String, // Name des Feldes
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
pub enum ConstValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
    Null,

    Struct {
        type_name: String,
        fields: Vec<(String, ConstValue)>,
    },
    Variant {
        type_name: String,
        case_name: String,
    },
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
    Pointer(Box<IrType>),
    Named(String),
}
#[derive(Debug, Clone)]
pub struct Variable {
    name: String,
    ty: IrType,
    addr: TempId,
}
#[derive(Debug, Clone)]
pub struct Scope {
    symbols: HashMap<String, Variable>, // name, variable
}

#[derive(Debug, Clone)]
pub struct IrGenerator {
    temp_counter: usize,
    type_defs: HashMap<String, IrTypeDef>,
    current_expected_return_type: IrType,
    scopes: Vec<Scope>, // Scope ist einfach eine hashmap welche die variablen aus dem scope
                        // speichert
}

impl IrGenerator {
    fn new() -> Self {
        IrGenerator {
            temp_counter: 0,
            current_expected_return_type: IrType::Void,
            scopes: Vec::new(),
            type_defs: HashMap::new(),
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(Scope {
            symbols: HashMap::new(),
        });
    }
    fn exit_scope(&mut self) {
        self.scopes.pop();
    }
    fn insert_variable(&mut self, name: String, ty: IrType, addr: TempId) {
        self.scopes
            .last_mut()
            .expect("No active scope")
            .symbols
            .insert(name.clone(), Variable { name, ty, addr });
    }
    fn lookup_variable(&self, name: &str) -> Option<&Variable> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.symbols.get(name))
    }

    fn gen_func_struct(&mut self, function: &Function) -> Result<IrFunction, CodegenError> {
        self.temp_counter = 0;
        self.scopes.clear();
        self.enter_scope();
        self.current_expected_return_type = Self::convert_to_ir_type(
            &function
                .return_type
                .clone()
                .unwrap_or(Type::Named("void".to_string())),
        );
        // Entry block ist erstmal der main block der function und wird leer erstellt
        let mut entry_block = IrBlock {
            label: "entry:".to_string(),
            instructions: Vec::new(),
            terminator: Terminator::Ret(None), // jede function hat einen terminator wie return,
                                               // falls es kein return in der eigentlich function gibt ist es einfach ret(none) also
                                               // return void
        };
        // die eigentlichen statements aus der function in instructions für den entry block machen

        for stmt in &function.body.statements {
            self.gen_stmt(stmt, &mut entry_block)?; // jedes statement aus der function handeln
                                                    // entry block wird direkt als mutatable refenrences reingepackt, damit die instructions
                                                    // oder der terminator bei bedarf direkt in den weiter folgenden funktionen geändert
                                                    // werden kann ohne immer etwas returnen zu müssen
        }
        self.exit_scope();

        Ok(IrFunction {
            name: function.name.clone(),
            parameter: function
                .params
                .iter()
                .map(|param| IrParameter {
                    name: param.name.clone(),
                    param_type: Self::convert_to_ir_type(&param.param_type),
                })
                .collect(),
            return_type: function
                .return_type
                .as_ref()
                .map(Self::convert_to_ir_type)
                .unwrap_or(IrType::Void),
            blocks: vec![entry_block], // Todo: mehrere blöcke ??
        })
    }
    fn gen_stmt(&mut self, stmt: &Stmt, block: &mut IrBlock) -> Result<(), CodegenError> {
        match stmt {
            Stmt::Let(l) => {
                let ir_type = Self::convert_to_ir_type(&l.var_type.clone()); // Todo: warum kann var type none sein???

                let id_for_alloc = self.next_temp_id();

                // id_for_alloc ist der name der temp variable, welche die freie Adresse hält
                block.instructions.push(IrInstruction::Alloca {
                    temp_id: id_for_alloc,
                    ty: ir_type.clone(),
                });

                // self.gen_expr returnt den name der temp variable, welcher zur Laufzeit das
                // Ergebnis halten wird
                // Bsp:
                // 1+1
                // %0 = const 1
                // %1 = const 2
                // %2 = add %0, %1
                // dann wäre hier expr_value = %2
                let (expr_value, expr_type) = self.gen_expr(&l.value, block)?;

                // gucken ob der ir_type welcher vom nutzer angegeben wurde, was die variable für
                // ein typ hat auch wirklich den selben typ hat wie das ergebnis der expression
                if (ir_type != expr_type) {
                    return Err(CodegenError::MismatchedTypes);
                }

                // Die temp variable welche das ergebnis der eval hält wird in die addr geladen
                // welche die variable id_for_alloc hält
                block.instructions.push(IrInstruction::Store {
                    ty: ir_type.clone(),
                    value: expr_value,
                    addr: id_for_alloc,
                });

                self.insert_variable(l.var_name.clone(), ir_type, id_for_alloc);

                Ok(())
            }
            Stmt::Return(ret) => {
                let return_value = ret.return_value.as_ref();
                // --> option <expr> entweder returnt es void oder eine expr

                match return_value {
                    Some(value) => {
                        let (temp_id, ret_t) = self.gen_expr(value, block)?; // Expr handeln -> macht sein eigenes Ding und
                                                                             // editiert die instructions des blocks. Return gibt nicht das ergebnis der
                                                                             // expr selber zurück sondern nur die variable also brauchen wir die temp id
                        if (ret_t != self.current_expected_return_type) {
                            return Err(CodegenError::InvalidReturnType(
                                self.current_expected_return_type.clone(),
                                ret_t,
                            ));
                        }
                        block.terminator = Terminator::Ret(Some(temp_id));

                        Ok(())
                    }
                    // Eigentlich unnötig, da block.terminator by default schon None ist aber egal
                    None => {
                        if (IrType::Void != self.current_expected_return_type) {
                            return Err(CodegenError::InvalidReturnType(
                                self.current_expected_return_type.clone(),
                                IrType::Void,
                            ));
                        }
                        block.terminator = Terminator::Ret(None);
                        Ok(())
                    }
                }
            }
            Stmt::Asm(asm) => {
                block.instructions.push(IrInstruction::Asm {
                    code: asm.asm_code.clone(),
                });
                Ok(())
            }
            _ => {
                return Err(CodegenError::InvalidStatement(stmt.clone()));
            }
        }
    }

    // Methode wird aufgerufen falls man eine temp id belegen will -> aktuell freie temp id struct
    // wird erstellt und returnt und der counter wird um 1 erhöht damit dieser wieder bei der nächst
    // freien temp id ist
    fn next_temp_id(&mut self) -> TempId {
        let id = TempId(self.temp_counter);
        self.temp_counter += 1;
        return id;
    }

    fn gen_expr(
        &mut self,
        expr: &Expr,
        block: &mut IrBlock,
    ) -> Result<(TempId, IrType), CodegenError> {
        // Expr handeln: Instructions in dem Block je nach expression verändern und die temp id
        // zurück geben wo das ergebnis der expr genau gespeichert wird, damit aufrufende methoden
        // das nutzen können (wie zb return)
        match expr {
            Expr::Variable(name) => {
                let freie_temp_var = self.next_temp_id();

                let var = self
                    .lookup_variable(name)
                    .ok_or_else(|| CodegenError::UnknwonVariable(name.clone()))?;

                block.instructions.push(IrInstruction::Load {
                    temp_id: freie_temp_var,
                    ty: var.ty.clone(),
                    addr: var.addr,
                });

                Ok((freie_temp_var, var.ty.clone()))
            }
            Expr::IntLiteral(value) => {
                let temp_id = self.next_temp_id();

                let val = value
                    .parse()
                    .map_err(|e| CodegenError::InvalidIntLiteral(value.clone()))?;
                let new_const_instruction = IrInstruction::Const {
                    temp_id: temp_id,
                    ty: IrType::I64,
                    value: ConstValue::Int(val),
                };

                block.instructions.push(new_const_instruction);
                return Ok((temp_id, IrType::I64));
            }
            Expr::FloatLiteral(value) => {
                let temp_id = self.next_temp_id();

                let val = value
                    .parse()
                    .map_err(|e| CodegenError::InvalidFloatLiteral(value.clone()))?;

                let new_const_instruction = IrInstruction::Const {
                    temp_id: temp_id,
                    ty: IrType::F64,
                    value: ConstValue::Float(val),
                };

                block.instructions.push(new_const_instruction);

                return Ok((temp_id, IrType::F64));
            }
            Expr::BoolLiteral(value) => {
                let temp_id = self.next_temp_id();

                let new_const_instruction = IrInstruction::Const {
                    temp_id: temp_id,
                    ty: IrType::Bool,
                    value: ConstValue::Bool(*value),
                };

                block.instructions.push(new_const_instruction);

                return Ok((temp_id, IrType::Bool));
            }
            Expr::StringLiteral(value) => {
                let temp_id = self.next_temp_id();

                let new_const_instruction = IrInstruction::Const {
                    temp_id: temp_id,
                    ty: IrType::Named("string".to_string()),
                    value: ConstValue::String(value.clone()),
                };

                block.instructions.push(new_const_instruction);

                return Ok((temp_id, IrType::Named("string".to_string())));
            }
            other => {
                return Err(CodegenError::InvalidExpr(other.clone()));
            }
        }
    }

    fn convert_to_ir_type(ty: &Type) -> IrType {
        match ty {
            Type::Named(name) => match name.as_str() {
                "int" => IrType::I64,
                "float" => IrType::F64,
                "bool" => IrType::Bool,
                "void" => IrType::Void,
                other => IrType::Named(other.to_string()),
            },
            Type::AnyTrait(_) => {
                panic!("traits not implemented in code gen")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum CodegenError {
    InvalidItem(Item),
    MismatchedTypes,

    UnknwonVariable(String),

    InvalidIntLiteral(String),
    InvalidFloatLiteral(String),

    InvalidExpr(Expr),
    InvalidStatement(Stmt),
    InvalidReturnType(IrType, IrType), // expected, got
}

pub fn generate_code(items: &[Item]) -> Result<IrModule, CodegenError> {
    let mut generator = IrGenerator::new();
    let mut module = IrModule::new();

    for item in items {
        match item {
            Item::Function(f) => {
                let func = generator.gen_func_struct(f)?;
                module.functions.push(func);
            }
            Item::Asm(asm) => {
                let Asm { asm_code } = asm;
                module.inline_assembly.push(asm_code.clone());
            }
            _ => return Err(CodegenError::InvalidItem(item.clone())),
        };
    }
    return Ok(module);
}
