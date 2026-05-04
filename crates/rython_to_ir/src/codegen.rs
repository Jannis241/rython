use std::fmt::Write;

use crate::{ast::{Expr, Function, Item, Stmt, Type}, lexer::Token};

// Module hält alle ober Items also alles was in der oberen ebene im file stehen könnte
pub struct IrModule {
    pub functions: Vec<IrFunction>,
    // Todo: weitere dinge wie globale variablen, imports, traits etc
}
impl IrModule {
    pub fn new() -> Self {
        IrModule { functions: Vec::new() }
    }
}

pub struct IrFunction {
    pub name: String,
    pub parameter: Vec<IrParameter>,
    pub return_type: IrType,
    pub blocks: Vec<IrBlock>
}

pub struct IrParameter {
    pub name: String,
    pub param_type: IrType,
}

pub struct IrBlock {
    pub label: String, // z.B entry:
    pub instructions: Vec<IrInstruction>,
    pub terminator: Terminator,
}

pub enum IrInstruction {
    Const {
        temp_id: TempId,
        ty: IrType,
        value: ConstValue,
    }
}

pub struct TempId(usize);

pub enum ConstValue {
      Int(i64),
      Float(f64),
      Bool(bool),
      Char(char),
      String(String),
      Null,
}


pub enum Terminator {
    ret(Option<TempId>) // entweder zb ret %tmp0 bei Some(id) oder ret void bei None
}

pub enum IrType {
    I64,
    Bool,
    Void,
    F64,
    Pointer(Box<IrType>),
    Named(String),
}


pub struct IrGenerator {
    temp_counter: usize,

}



impl IrGenerator {
    fn new() -> Self {
        IrGenerator {temp_counter: 0}
    }


    fn gen_func_struct(&mut self, function: &Function) -> IrFunction {
        self.temp_counter = 0;

        // Entry block ist erstmal der main block der function und wird leer erstellt
        let mut entry_block = IrBlock {
            label: "entry:".to_string(),
            instructions: Vec::new(),
            terminator: Terminator::ret(None), // jede function hat einen terminator wie return,
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
            parameter: function.params.iter().map(f)
        }
        panic!()
    }
    fn gen_stmt(&mut self, stmt: &Stmt, block: &mut IrBlock) {

    }


    fn convert_to_ir_type(ty: &Type) -> IrType {
        match ty {
            Type::Named(name) => {
                match name.as_str() {
                    "int" => IrType::I64,
                    "float" => IrType::F64,
                    "bool" => IrType::Bool,
                    "void" => IrType::Void,
                    other => IrType::Named(other.to_string()),
                }
            }
            Type::AnyTrait(_) => {panic!("traits not implemented in code gen")}
        }
    }

}

pub fn generate_code(items: &[Item]) -> IrModule {
    let mut generator = IrGenerator::new();
    let mut module = IrModule::new();

    for item in items {
        let code = match item {
            Item::Function(f) => {
                let func = generator.gen_func_struct(f);
                module.functions.push(func);
            }
            _ => panic!("Item {:?} not implemented yet.", item)
        };

    }
    return module;
}

