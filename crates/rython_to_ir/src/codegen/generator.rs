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
    pub(super) scopes: Vec<Scope>,
    pub(super) block_handler: BlockHandler,
}

#[derive(Debug, Clone)]
pub struct BlockHandler {
    blocks: Vec<IrBlock>,
    current_block_index: usize,
}

impl BlockHandler {
    pub fn init() -> Self {
        return BlockHandler {
            blocks: Vec::new(),
            current_block_index: 0,
        };
    }

    pub fn create_new_block(&mut self, name: &str) {
        self.blocks.push(IrBlock {
            label: format!("{name}:"),
            instructions: Vec::new(),
            terminator: Terminator::Ret(None),
        });
    }

    pub fn jump_to_block(&mut self, label: &str) {
        let label = format!("{label}:");
        self.current_block_index = self
            .blocks
            .iter()
            .position(|block| block.label == label)
            .expect("block does not exist");
    }

    pub fn add_instruction_to_current_block(&mut self, instruction: IrInstruction) {
        self.blocks[self.current_block_index]
            .instructions
            .push(instruction);
    }

    pub fn add_terminator(&mut self, terminator: Terminator) {
        self.blocks[self.current_block_index].terminator = terminator;
    }
}

impl IrGenerator {
    fn new() -> Self {
        IrGenerator {
            temp_counter: 0,
            current_expected_return_type: IrType::Void,
            scopes: Vec::new(),
            type_defs: HashMap::new(),
            block_handler: BlockHandler::init(),
        }
    }

    pub(super) fn gen_func_struct(
        &mut self,
        function: &Function,
    ) -> Result<IrFunction, CodegenError> {

        self.temp_counter = 0;

        self.current_expected_return_type = Self::convert_to_ir_type(
            &function
                .return_type
                .clone()
                .unwrap_or(Type::Named("void".to_string())),
        );

        self.block_handler.create_new_block("entry");
        self.block_handler.jump_to_block("entry");

        // todo: Parameter handeln
        // todo: scopes

        for stmt in &function.body.statements {
            self.gen_stmt(stmt)?;
        }

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
            blocks: self.block_handler.blocks.clone(),
        })
    }

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
