use crate::ast::{Expr, Function, Item, Stmt, Type};

#[derive(Debug, Clone)]
pub struct IrModule {
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
        temp_id: TempId,
        ty: IrType,
        value: ConstValue,
    },
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
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Ret(Option<TempId>), // entweder zb ret %tmp0 bei Some(id) oder ret void bei None
}

#[derive(Debug, Clone)]
pub enum IrType {
    I64,
    Bool,
    Void,
    F64,
    Pointer(Box<IrType>),
    Named(String),
}

#[derive(Debug, Clone, Copy)]
pub struct IrGenerator {
    temp_counter: usize,
}

impl IrGenerator {
    fn new() -> Self {
        IrGenerator { temp_counter: 0 }
    }

    fn gen_func_struct(&mut self, function: &Function) -> IrFunction {
        self.temp_counter = 0;

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
            self.gen_stmt(stmt, &mut entry_block); // jedes statement aus der function handeln
            // entry block wird direkt als mutatable refenrences reingepackt, damit die instructions
            // oder der terminator bei bedarf direkt in den weiter folgenden funktionen geändert
            // werden kann ohne immer etwas returnen zu müssen
        }
        IrFunction {
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
        }
    }
    fn gen_stmt(&mut self, stmt: &Stmt, block: &mut IrBlock) {
        match stmt {
            Stmt::Return(ret) => {
                let return_value = ret.return_value.as_ref();
                // --> option <expr> entweder returnt es void oder eine expr

                match return_value {
                    Some(value) => {
                        let temp_id = self.gen_expr(value, block); // Expr handeln -> macht sein eigenes Ding und
                        // editiert die instructions des blocks. Return gibt nicht das ergebnis der
                        // expr selber zurück sondern nur die variable also brauchen wir die temp id
                        block.terminator = Terminator::Ret(Some(temp_id));
                    }
                    // Eigentlich unnötig, da block.terminator by default schon None ist aber egal
                    None => {
                        block.terminator = Terminator::Ret(None);
                    }
                }
            }
            _ => {
                panic!()
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

    fn gen_expr(&mut self, expr: &Expr, block: &mut IrBlock) -> TempId {
        // Expr handeln: Instructions in dem Block je nach expression verändern und die temp id
        // zurück geben wo das ergebnis der expr genau gespeichert wird, damit aufrufende methoden
        // das nutzen können (wie zb return)
        match expr {
            Expr::IntLiteral(value) => {
                let temp_id = self.next_temp_id();

                let new_const_instruction = IrInstruction::Const {
                    temp_id: temp_id,
                    ty: IrType::I64,
                    value: ConstValue::Int(value.parse().expect("Invalid int literal")),
                };

                block.instructions.push(new_const_instruction);

                temp_id
            }
            Expr::FloatLiteral(value) => {
                let temp_id = self.next_temp_id();

                let new_const_instruction = IrInstruction::Const {
                    temp_id: temp_id,
                    ty: IrType::F64,
                    value: ConstValue::Float(value.parse().expect("Invalid float literal")),
                };

                block.instructions.push(new_const_instruction);

                temp_id
            }
            Expr::BoolLiteral(value) => {
                let temp_id = self.next_temp_id();

                let new_const_instruction = IrInstruction::Const {
                    temp_id: temp_id,
                    ty: IrType::Bool,
                    value: ConstValue::Bool(*value),
                };

                block.instructions.push(new_const_instruction);

                temp_id
            }
            Expr::StringLiteral(value) => {
                let temp_id = self.next_temp_id();

                let new_const_instruction = IrInstruction::Const {
                    temp_id: temp_id,
                    ty: IrType::Named("string".to_string()),
                    value: ConstValue::String(value.clone()),
                };

                block.instructions.push(new_const_instruction);

                temp_id
            }
            _ => {
                panic!("not implemented expr in code gen")
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

    fn convert_const_expr(expr: &Expr) -> ConstValue {
        match expr {
            Expr::IntLiteral(value) => ConstValue::Int(value.parse().expect("Invalid int literal")),
            Expr::FloatLiteral(value) => {
                ConstValue::Float(value.parse().expect("Invalid float literal"))
            }
            Expr::BoolLiteral(value) => ConstValue::Bool(*value),
            Expr::CharLiteral(value) => ConstValue::Char(*value),
            Expr::StringLiteral(value) => ConstValue::String(value.clone()),
            Expr::NullLiteral => ConstValue::Null,
            other => panic!("global and const values must be literals, got {other:?}"),
        }
    }
}

pub fn generate_code(items: &[Item]) -> IrModule {
    let mut generator = IrGenerator::new();
    let mut module = IrModule::new();

    for item in items {
        match item {
            Item::Function(f) => {
                let func = generator.gen_func_struct(f);
                module.functions.push(func);
            }
            Item::GlobalVar(global) => {
                module.globals.push(IrGlobal {
                    name: global.var_name.clone(),
                    ty: IrGenerator::convert_to_ir_type(&global.var_type),
                    value: IrGenerator::convert_const_expr(&global.value),
                });
            }
            Item::ConstVar(constant) => {
                module.constants.push(IrConstant {
                    name: constant.var_name.clone(),
                    ty: IrGenerator::convert_to_ir_type(&constant.var_type),
                    value: IrGenerator::convert_const_expr(&constant.value),
                });
            }
            _ => panic!("Item {:?} not implemented yet.", item),
        };
    }
    return module;
}
