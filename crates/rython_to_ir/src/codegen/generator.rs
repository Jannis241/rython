use std::collections::HashMap;

use crate::ast::{Asm, Function, Item, Type};
use crate::ir::{
    IrBlock, IrField, IrFunction, IrInstruction, IrModule, IrType, IrTypeDefinition, TempId,
    Terminator,
};

use super::error::CodegenError;
use super::scope::Scope;

#[derive(Debug, Clone)]
pub struct IrGenerator {
    pub(super) temp_counter: usize,
    pub(super) type_defs: HashMap<String, IrTypeDefinition>,
    pub(super) current_expected_return_type: IrType,
    pub(super) scopes: Vec<Scope>, // Scope ist einfach eine hashmap welche die variablen aus dem scope
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

    pub(super) fn gen_func_struct(
        &mut self,
        function: &Function,
    ) -> Result<IrFunction, CodegenError> {
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

        // Parameter werden wie normale lokale Variablen behandelt: erst eine Stack-Slot
        // per Alloca holen, dann den eingehenden Argument-Wert per LoadParam einlesen,
        // dann in den Slot schreiben und unter dem Parameter-Namen ins Scope eintragen.
        for (index, param) in function.params.iter().enumerate() {
            let param_ty = Self::convert_to_ir_type(&param.param_type);

            let addr_temp = self.next_temp_id();
            entry_block.instructions.push(IrInstruction::Alloca {
                temp_id: addr_temp,
                ty: param_ty.clone(),
            });

            let value_temp = self.next_temp_id();
            entry_block.instructions.push(IrInstruction::LoadParam {
                temp_id: value_temp,
                index,
                ty: param_ty.clone(),
            });

            entry_block.instructions.push(IrInstruction::Store {
                ty: param_ty.clone(),
                value: value_temp,
                addr: addr_temp,
            });

            self.insert_variable(param.name.clone(), param_ty, addr_temp);
        }

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
                .map(|param| IrField {
                    name: param.name.clone(),
                    ty: Self::convert_to_ir_type(&param.param_type),
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

    // Methode wird aufgerufen falls man eine temp id belegen will -> aktuell freie temp id struct
    // wird erstellt und returnt und der counter wird um 1 erhöht damit dieser wieder bei der nächst
    // freien temp id ist
    pub(super) fn next_temp_id(&mut self) -> TempId {
        let id = TempId(self.temp_counter);
        self.temp_counter += 1;
        return id;
    }

    pub(super) fn convert_to_ir_type(ty: &Type) -> IrType {
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
