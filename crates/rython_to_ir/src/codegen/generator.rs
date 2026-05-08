use std::collections::HashMap;
use std::fmt::format;

use crate::ast::{Asm, ConstVar, Expr, Function, GlobalVar, Item, Param, StructField, Type};
use crate::codegen::generator;
use crate::ir::{
    IrBlock, IrConstant, IrField, IrFunction, IrGlobal, IrInstruction, IrModule, IrType,
    IrTypeDefinition, PrimitiveValue, TempId, Terminator,
};

use super::error::CodegenError;
use super::scope::Scope;

#[derive(Debug, Clone)]
pub struct IrGenerator {
    pub(super) temp_counter: usize,
    pub(super) type_defs: Vec<IrTypeDefinition>,
    pub(super) current_expected_return_type: IrType,
    pub(super) scopes: Vec<Scope>,
    pub(super) module: IrModule,
    pub(super) block_handler: BlockHandler,
    pub(super) functions_return_type: HashMap<String, Option<IrType>>,
    pub(super) current_mangel_prefix: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BlockHandler {
    blocks: Vec<WorkingBlock>,
    current_block_index: usize,
}

#[derive(Debug, Clone)]
pub struct WorkingBlock {
    label: String,
    instructions: Vec<IrInstruction>,
    terminator: Option<Terminator>,
}

impl BlockHandler {
    pub fn init() -> Self {
        return BlockHandler {
            blocks: Vec::new(),
            current_block_index: 0,
        };
    }

    pub fn create_new_block(&mut self, name: &str) {
        self.blocks.push(WorkingBlock {
            label: format!("{name}:"),
            instructions: Vec::new(),
            terminator: None,
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

    pub fn add_instruction_to_current_block(
        &mut self,
        instruction: IrInstruction,
    ) -> Result<(), CodegenError> {
        // checken ob der block zu dem man instructions adden will schon terminated wurde,
        // dann darf man nichts mehr hinzufügen
        if self.blocks[self.current_block_index].terminator.is_some() {
            return Err(CodegenError::CodeAfterTerminator);
        }

        self.blocks[self.current_block_index]
            .instructions
            .push(instruction);
        Ok(())
    }

    pub fn add_terminator(&mut self, terminator: Terminator) -> Result<(), CodegenError> {
        if self.blocks[self.current_block_index].terminator.is_some() {
            return Err(CodegenError::CodeAfterTerminator);
        }
        self.blocks[self.current_block_index].terminator = Some(terminator);
        Ok(())
    }

    pub fn finish_blocks(&self, return_type: &IrType) -> Result<Vec<IrBlock>, CodegenError> {
        let mut final_ir_blocks = Vec::new();
        for block in &self.blocks {
            let block_terminator = match block.terminator.clone() {
                Some(t) => Ok(t),
                None => {
                    if return_type == &IrType::Void {
                        Ok(Terminator::Ret(None))
                    } else {
                        Err(CodegenError::MissingTerminator(block.label.clone()))
                    }
                }
            }?;
            final_ir_blocks.push(IrBlock {
                label: block.label.clone(),
                instructions: block.instructions.clone(),
                terminator: block_terminator,
            });
        }
        Ok(final_ir_blocks)
    }
}

impl IrGenerator {
    fn new() -> Self {
        IrGenerator {
            temp_counter: 0,
            current_expected_return_type: IrType::Void,
            scopes: Vec::new(),
            type_defs: Vec::new(),
            block_handler: BlockHandler::init(),
            functions_return_type: HashMap::new(),
            module: IrModule::new(),
            current_mangel_prefix: vec!["mangeld_".to_string()],
        }
    }

    pub fn handle_parameters(&mut self, params: &Vec<Param>) -> Result<(), CodegenError> {
        for (index, parameter) in params.iter().enumerate() {
            let parameter_type = Self::convert_to_ir_type(&parameter.param_type);

            // Schritt 1: Platz für den Typ des Parameters allocaten und einen Pointer zu der
            // Adresse in temp_var_alloc_pointer speichern
            let temp_var_alloc_pointer = self.next_temp_id();
            let alloc_instruction = IrInstruction::Alloca {
                temp_id: temp_var_alloc_pointer,
                ty: parameter_type.clone(),
            };
            self.block_handler
                .add_instruction_to_current_block(alloc_instruction)?;

            // Schritt 2: Den eigentlichen parameter in temp_var_value speichern (den wirklichen
            // Wert)
            let temp_var_value = self.next_temp_id();
            let load_param_instruction = IrInstruction::LoadParam {
                temp_id: temp_var_value,
                index,
                ty: parameter_type.clone(),
            };
            self.block_handler
                .add_instruction_to_current_block(load_param_instruction)?;

            // Schritt 3: Damit man dem Wert eine wirkliche Adresse zuweisen kann und nicht nur den
            // wirklichen wert hat (wie in Schritt 2) wird der Wert einfach in dem vorher
            // allocateten Space gestored.
            // Jetzt hat man die Adresse des Wertes in temp_var_alloc_pointer und könnte die Value
            // mit load wieder bekommen
            let load_param_instruction = IrInstruction::Store {
                ty: parameter_type.clone(),
                value: temp_var_value,
                addr: temp_var_alloc_pointer,
            };
            self.block_handler
                .add_instruction_to_current_block(load_param_instruction)?;

            // Man könnte theoretisch auch direkt nur mit der value weiter machen welche man aus
            // load param bekommt ohne alloc und store.
            // Parameter sollen nur wie Variablen behandelt werden und das aktuelle Variablen modell
            // speichert den Name, Typ und die Adresse der Variable, nicht direkt die Value, daher
            // ist das so nötig.
            // Außerdem ist mutatation von parametern dafür später einfacher, bsp:
            //
            // fn test(a: int) int {
            //     a = 10;
            //     return a;
            // }
            //
            // So kann man einfach den Wert in der adresse von A überschreiben, mit dem direkten
            // Wert wäre das etwas schwerer.
            // Dazu ist es einheitlicher, da Variablen genau so gehandelt werden, dann kann
            // Expr::Variable immer sagen: variable finden -> adresse nehmen -> load
            // egal ob es ein parameter oder eine normale variable ist

            self.insert_variable(
                parameter.name.clone(),
                parameter_type,
                temp_var_alloc_pointer,
            );
        }
        Ok(())
    }

    pub(super) fn gen_func_struct(
        &mut self,
        function: &Function,
    ) -> Result<IrFunction, CodegenError> {
        self.add_to_current_mangel_prefix(function.name.clone());
        self.block_handler = BlockHandler::init(); // Block handler am anfang jeder funktion reseten
        self.block_handler
            .create_new_block(self.mangel("entry").as_str());
        self.block_handler
            .jump_to_block(self.mangel("entry").as_str());

        self.temp_counter = 0; // ------> @Jesko!!! Codex sagt das soll man so machen du handelst
                               // das schon richtig in asm sonst nicht so sigma von dir

        // Für die neue Funktion alle vorherigen Scopes clearen
        self.scopes.clear();
        self.enter_scope();

        self.current_expected_return_type = Self::convert_to_ir_type(
            &function
                .return_type
                .clone()
                .unwrap_or(Type::Named("void".to_string())),
        );

        self.handle_parameters(&function.params)?;

        for stmt in &function.body.statements {
            self.gen_stmt(stmt)?;
        }

        self.exit_scope();

        // checkt einfach nur ob jeder Block einen Terminator hat
        let blocks = self
            .block_handler
            .finish_blocks(&self.current_expected_return_type)?;

        let params = function
            .params
            .iter()
            .map(|param| IrField {
                name: param.name.clone(),
                ty: Self::convert_to_ir_type(&param.param_type),
            })
            .collect();

        let name = self.mangel(function.name.clone());

        self.pop_last_mangel_prefix();

        Ok(IrFunction {
            name,
            parameter: params,
            return_type: function
                .return_type
                .as_ref()
                .map(Self::convert_to_ir_type)
                .unwrap_or(IrType::Void),
            blocks,
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
                todo!("traits not implemented in code gen")
            }
        }
    }

    pub(super) fn preprocces_function_return_types(&mut self, items: &[Item]) {
        for item in items {
            match item {
                Item::Function(f) => {
                    self.functions_return_type.insert(
                        //TODO: generic Params
                        f.name.clone(),
                        if let Some(rt) = f.return_type.clone() {
                            Some(IrGenerator::convert_to_ir_type(&rt))
                        } else {
                            None
                        },
                    );
                }
                _ => {}
            }
        }
    }

    pub fn add_to_current_mangel_prefix<T: ToString>(&mut self, add: T) {
        self.current_mangel_prefix.push(add.to_string());
    }
    pub fn pop_last_mangel_prefix(&mut self) {
        self.current_mangel_prefix.pop();
    }
    pub fn current_mangel(&self) -> String {
        self.current_mangel_prefix.join("_")
    }
    pub(super) fn mangel<T: ToString>(&self, add: T) -> String {
        self.current_mangel() + &add.to_string().as_str()
    }

    pub fn preprocces_type_defs(&mut self, items: &[Item]) -> Result<(), CodegenError> {
        for item in items {
            match item {
                Item::Struct(structdef) => {
                    self.add_to_current_mangel_prefix(structdef.struct_name.clone());
                    let mut ir_fields = vec![];

                    for parser_field in structdef.fields.iter() {
                        ir_fields.push(IrField {
                            name: self.mangel(parser_field.field_name.clone()),
                            ty: IrGenerator::convert_to_ir_type(&parser_field.field_type),
                        });
                    }

                    let typedef = IrTypeDefinition::Struct {
                        name: structdef.struct_name.clone(),
                        fields: ir_fields,
                    };

                    self.type_defs.push(typedef.clone());
                    self.module.types.push(typedef);
                    self.pop_last_mangel_prefix();
                }
                Item::Variant(variantdef) => {
                    let typedef = IrTypeDefinition::Variant {
                        name: variantdef.variant_name.clone(),
                        cases: variantdef.cases.clone(),
                    };

                    self.type_defs.push(typedef.clone());
                    self.module.types.push(typedef);
                }
                _ => continue, //wird im zweiten pass gemacht,
            };
        }

        Ok(())
    }
    pub fn generate_code_inner(&mut self, items: &[Item]) -> Result<(), CodegenError> {
        self.preprocces_function_return_types(items);
        self.preprocces_type_defs(items);

        for item in items {
            match item {
                Item::Function(f) => {
                    let func = self.gen_func_struct(f)?;
                    self.module.functions.push(func);
                }
                Item::Asm(asm) => {
                    let Asm { asm_code } = asm;
                    self.module.inline_assembly.push(asm_code.clone());
                }
                Item::Struct(struct_def) => {
                    self.add_to_current_mangel_prefix(struct_def.struct_name.clone());

                    for f in struct_def.functions.iter() {
                        let func = self.gen_func_struct(f)?;
                        self.module.functions.push(func);
                    }

                    self.pop_last_mangel_prefix();
                }
                Item::Variant(..) => continue, //wurde im ersten pass gemacht
                Item::Import(..) => unimplemented!(),
                Item::ConstVar(const_var) => unimplemented!(),
                Item::GlobalVar(global_var) => unimplemented!(),
                Item::Trait(..) => unimplemented!(),
                Item::TraitImplementation(..) => unimplemented!(),
            };
        }
        return Ok(());
    }

    pub fn generate_code(items: &[Item]) -> Result<IrModule, CodegenError> {
        let mut generator = IrGenerator::new();

        generator.generate_code_inner(items)?;

        return Ok(generator.module);
    }
}
