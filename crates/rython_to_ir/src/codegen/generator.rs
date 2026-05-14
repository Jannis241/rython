use std::collections::{HashMap, HashSet};

use crate::ast::{Asm, Expr, Function, Item, Param, Type};
use crate::codegen::generator;
use crate::ir::{
    FunctionSignaturIr, IrBlock, IrConstant, IrField, IrFunction, IrGlobal, IrInstruction,
    IrModule, IrType, IrTypeDefinition, PrimitiveValue, TempId, Terminator,
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
    pub(super) function_signatures: HashMap<String, FunctionSignaturIr>,
    pub(super) current_mangel_prefix: Vec<String>,
    // (struct_name, operator_string) -> (mangled_function_name, return_type)
    pub(super) operator_functions: HashMap<(String, String), (String, IrType)>,
    // (struct_name, operator_string) -> (mangled_function_name, return_type)
    pub(super) unary_operator_functions: HashMap<(String, String), (String, IrType)>,
    pub(super) struct_names: HashSet<String>,
    pub(super) type_names: HashSet<String>,
    pub(super) block_label_counter: usize,
    // (continue_target, break_target)
    pub(super) loop_stack: Vec<(String, String)>,
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

    //Jannis slop fix!! yessirsky war ass (grr)
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

    //Jannis slop fix!! yessirsky war ass (grr)
    pub fn add_terminator(&mut self, terminator: Terminator) -> Result<(), CodegenError> {
        if self.blocks[self.current_block_index].terminator.is_some() {
            return Err(CodegenError::CodeAfterTerminator);
        }
        self.blocks[self.current_block_index].terminator = Some(terminator);
        Ok(())
    }

    pub fn is_current_terminated(&self) -> bool {
        self.blocks[self.current_block_index].terminator.is_some()
    }

    //Jannis slop fix!! yessirsky war ass (grr)
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
            type_names: HashSet::new(),
            type_defs: Vec::new(),
            block_handler: BlockHandler::init(),
            function_signatures: HashMap::new(),
            module: IrModule::new(),
            current_mangel_prefix: Vec::new(),
            operator_functions: HashMap::new(),
            unary_operator_functions: HashMap::new(),
            struct_names: HashSet::new(),
            block_label_counter: 0,
            loop_stack: Vec::new(),
        }
    }

    pub(super) fn fresh_block_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.block_label_counter);
        self.block_label_counter += 1;
        label
    }

    pub fn handle_parameters(&mut self, params: &Vec<Param>) -> Result<(), CodegenError> {
        for (index, parameter) in params.iter().enumerate() {
            let parameter_type = self.convert_to_ir_type(&parameter.param_type)?;

            // 1. platz für den Typ des Parameters allocaten und einen poointer zu der
            // Addr in temp_var_alloc_pointer speichern
            let temp_var_alloc_pointer = self.next_temp_id();
            let alloc_instruction = IrInstruction::Alloca {
                temp_id: temp_var_alloc_pointer,
                ty: parameter_type.clone(),
            };
            self.block_handler
                .add_instruction_to_current_block(alloc_instruction)?;

            // 2. den eigentlichen param in temp_var_value speichern (den wirklichen
            // wert
            let temp_var_value = self.next_temp_id();
            let load_param_instruction = IrInstruction::LoadParam {
                temp_id: temp_var_value,
                index,
                ty: parameter_type.clone(),
            };
            self.block_handler
                .add_instruction_to_current_block(load_param_instruction)?;

            // 3. damit man dem wert eine wirkliche Adrr zuweisen kann und nicht nur den
            // wirklichen wert hat (wie in Schritt 2) wird der wert einfach in dem vorher
            // allocateten platz gestored.
            // dann hat man die Adrr des werts in temp_var_alloc_pointer und könnte die value
            // mit load wieder bekommen
            let load_param_instruction = IrInstruction::Store {
                ty: parameter_type.clone(),
                value: temp_var_value,
                addr: temp_var_alloc_pointer,
            };
            self.block_handler
                .add_instruction_to_current_block(load_param_instruction)?;

            // Deswegen ist das Besser (hab ich von claude aber macht doch sind oder?)
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
            )?;
        }
        Ok(())
    }

    pub(super) fn gen_func_struct(
        &mut self,
        function: &Function,
    ) -> Result<IrFunction, CodegenError> {
        debug_assert!(
            self.scopes.is_empty(),
            "previous function didnt clear all scopes: {:?}",
            self.scopes
        );

        // Block handler initalizen, den entry block erstellen und zu ihm jumpen
        self.block_handler = BlockHandler::init();
        self.block_handler.create_new_block("entry");
        self.block_handler.jump_to_block("entry");

        // Neuer scope erstellen, da wir in einer neuen funktion sind
        self.enter_scope();

        // Temp variable counter reseten, da er für jede funktion wieder bei 0 anfängt
        self.temp_counter = 0;
        self.block_label_counter = 0;
        self.loop_stack.clear();

        // return type bekommen
        self.current_expected_return_type = self.convert_to_ir_type(
            &function
                .return_type
                .clone()
                .unwrap_or(Type::Named("void".to_string())),
        )?;

        // parameter handeln
        self.handle_parameters(&function.params)?;

        // alle statements in der function generieren
        for stmt in &function.body.statements {
            self.gen_stmt(stmt)?;
        }

        // man ist am ende also exited man den scope -> (maybe unnötig weil man eh immer alle scopes
        // am anfang cleart aber so ist klarer was passiert)
        self.exit_scope();

        // checkt einfach nur ob jeder Block einen Terminator hat
        let blocks = self
            .block_handler
            .finish_blocks(&self.current_expected_return_type)?;

        // Parameter zu IrField machen
        let params: Result<Vec<IrField>, CodegenError> = function
            .params
            .iter()
            .map(|param| {
                Ok(IrField {
                    name: param.name.clone(),
                    ty: self.convert_to_ir_type(&param.param_type)?,
                })
            })
            .collect();

        let params = params?;

        // function name mangeln
        let name = self.mangel(function.name.clone());

        let return_type = function
            .return_type
            .clone()
            .unwrap_or(Type::Named("void".to_string()));

        let return_type = self.convert_to_ir_type(&return_type)?;

        // Fertige funktion bauen und returnen
        Ok(IrFunction {
            name,
            parameter: params,
            return_type,
            blocks,
        })
    }

    pub(super) fn next_temp_id(&mut self) -> TempId {
        let id = TempId(self.temp_counter);
        self.temp_counter += 1;
        return id;
    }

    // Struct-Typen werden einheitlich als Pointer<Named> dargestellt.
    // collect_struct_names muss vorher gelaufen sein, damit struct_names befuellt ist.
    // unbekannte typen werden jetzt direkt als error gecatched
    pub(super) fn convert_to_ir_type(&self, ty: &Type) -> Result<IrType, CodegenError> {
        match ty {
            Type::Named(name) => match name.as_str() {
                "int" => Ok(IrType::I64),
                "float" => Ok(IrType::F64),
                "bool" => Ok(IrType::Bool),
                "void" => Ok(IrType::Void),
                "char" => Ok(IrType::Char),
                other => {
                    let inner = IrType::Named(other.to_string());
                    if self.struct_names.contains(other) {
                        Ok(IrType::Pointer(Box::new(inner)))
                    } else if self.type_names.contains(other) {
                        Ok(inner)
                    } else {
                        Err(CodegenError::UnknownType(other.to_string()))
                    }
                }
            },
            Type::AnyTrait(_) => {
                todo!()
            }
        }
    }

    pub(super) fn collect_type_names(&mut self, items: &[Item]) -> Result<(), CodegenError> {
        for item in items {
            let name = match item {
                Item::Struct(s) => {
                    self.struct_names.insert(s.struct_name.clone());
                    Some(&s.struct_name)
                }
                Item::Variant(v) => Some(&v.variant_name),
                _ => None,
            };

            if let Some(name) = name {
                if !self.type_names.insert(name.clone()) {
                    return Err(CodegenError::DuplicateType(name.clone()));
                }
            }
        }
        Ok(())
    }

    pub(super) fn preprocces_functions(&mut self, items: &[Item]) -> Result<(), CodegenError> {
        for item in items {
            match item {
                Item::Function(f) => {
                    let return_ty = if let Some(rt) = f.return_type.clone() {
                        Some(self.convert_to_ir_type(&rt)?)
                    } else {
                        None
                    };

                    let params = f
                        .params
                        .iter()
                        .map(|p| self.convert_to_ir_type(&p.param_type))
                        .collect::<Result<Vec<_>, _>>()?;
                    let func_sig = FunctionSignaturIr {
                        return_type: return_ty,
                        params,
                    };
                    if self
                        .function_signatures
                        .insert(f.name.clone(), func_sig)
                        .is_some()
                    {
                        return Err(CodegenError::DuplicateFunction(f.name.clone()));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    // registriert die return typen aller struct methoden unter ihrem mangled namen
    // damit method calls Struct_method(...) den richtigen typ finden
    pub(super) fn preprocess_method_return_types(
        &mut self,
        items: &[Item],
    ) -> Result<(), CodegenError> {
        for item in items {
            if let Item::Struct(s) = item {
                self.add_to_current_mangel_prefix(s.struct_name.clone());
                for f in s.functions.iter() {
                    let mangled = self.mangel(f.name.clone());
                    let return_ty = if let Some(rt) = f.return_type.clone() {
                        Some(self.convert_to_ir_type(&rt)?)
                    } else {
                        None
                    };

                    let params = f
                        .params
                        .iter()
                        .map(|p| self.convert_to_ir_type(&p.param_type))
                        .collect::<Result<Vec<_>, _>>()?;
                    let func_sig = FunctionSignaturIr {
                        return_type: return_ty,
                        params,
                    };
                    if self
                        .function_signatures
                        .insert(mangled.clone(), func_sig)
                        .is_some()
                    {
                        return Err(CodegenError::DuplicateFunction(mangled));
                    }
                }
                self.pop_last_mangel_prefix();
            }
        }
        Ok(())
    }

    pub fn add_to_current_mangel_prefix<T: ToString>(&mut self, add: T) {
        self.current_mangel_prefix.push(add.to_string());
    }
    pub fn pop_last_mangel_prefix(&mut self) {
        self.current_mangel_prefix.pop();
    }

    // mangling regel:
    // - kein prefix aktiv  -> Name bleibt lgeich ("main" -> "main").
    // - prefix aktiv       -> "<prefix1>_<prefix2>_..._<name>" ("Person" + "get_name" -> "Person_get_name").
    pub(super) fn mangel<T: ToString>(&self, name: T) -> String {
        if self.current_mangel_prefix.is_empty() {
            name.to_string()
        } else {
            format!(
                "{}_{}",
                self.current_mangel_prefix.join("_"),
                name.to_string()
            )
        }
    }

    pub fn preprocces_type_defs(&mut self, items: &[Item]) -> Result<(), CodegenError> {
        for item in items {
            match item {
                Item::Struct(structdef) => {
                    let mut ir_fields = vec![];

                    for parser_field in structdef.fields.iter() {
                        ir_fields.push(IrField {
                            name: parser_field.field_name.clone(),
                            ty: self.convert_to_ir_type(&parser_field.field_type)?,
                        });
                    }

                    let typedef = IrTypeDefinition::Struct {
                        name: structdef.struct_name.clone(),
                        fields: ir_fields,
                    };

                    self.type_defs.push(typedef.clone());
                    self.module.types.push(typedef);
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
        // muss als allererstes laufen: convert_to_ir_type braucht struct_names um
        // Struct-Namen einheitlich als Pointer<Named> auszugeben.
        self.collect_type_names(items)?;
        self.preprocces_functions(items)?;
        self.preprocess_method_return_types(items)?;
        self.preprocces_type_defs(items)?;
        self.preprocess_operators(items)?;
        self.process_consts_and_globals(items)?;

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
                Item::ConstVar(..) | Item::GlobalVar(..) => continue, // schon in process_consts_and_globals
                Item::Import(..) | Item::Trait(..) | Item::TraitImplementation(..) => {
                    return Err(CodegenError::InvalidItem(item.clone()));
                }
            };
        }
        return Ok(());
    }

    pub(super) fn preprocess_operators(&mut self, items: &[Item]) -> Result<(), CodegenError> {
        // erzeugt die gleichen mangled Namen wie gen_func_struct für struct Methods.
        // herausfinden ob binary op oder unary op
        for item in items {
            if let Item::Struct(s) = item {
                self.add_to_current_mangel_prefix(s.struct_name.clone());
                for f in s.functions.iter() {
                    if let Some(op) = &f.operator {
                        let mangled = self.mangel(f.name.clone());

                        let return_type = f
                            .return_type
                            .clone()
                            .unwrap_or(Type::Named("void".to_string()));
                        let return_type = self.convert_to_ir_type(&return_type)?;
                        let key = (s.struct_name.clone(), op.clone());
                        match f.params.len() {
                            1 => {
                                if self
                                    .unary_operator_functions
                                    .insert(key, (mangled.clone(), return_type))
                                    .is_some()
                                {
                                    return Err(CodegenError::AmbigousFunction(mangled));
                                }
                            }
                            2 => {
                                if self
                                    .operator_functions
                                    .insert(key, (mangled.clone(), return_type))
                                    .is_some()
                                {
                                    return Err(CodegenError::AmbigousFunction(mangled));
                                }
                            }
                            _ => return Err(CodegenError::InvalidItem(Item::Function(f.clone()))),
                        }
                    }
                }

                self.pop_last_mangel_prefix();
            }
        }
        Ok(())
    }

    pub(super) fn process_consts_and_globals(
        &mut self,
        items: &[Item],
    ) -> Result<(), CodegenError> {
        for item in items {
            match item {
                Item::ConstVar(const_var) => {
                    let name = &const_var.var_name;
                    if self.module.constants.iter().any(|c| &c.name == name)
                        || self.module.globals.iter().any(|g| &g.name == name)
                    {
                        return Err(CodegenError::DuplicateGlobal(name.clone()));
                    }
                    let declared_ty = self.convert_to_ir_type(&const_var.var_type)?;
                    let (val_ty, val) = self.eval_const_expr(&const_var.value)?;
                    if val_ty != declared_ty {
                        return Err(CodegenError::MismatchedTypes(declared_ty, val_ty));
                    }
                    self.module.constants.push(IrConstant {
                        name: name.clone(),
                        ty: declared_ty,
                        value: val,
                    });
                }
                Item::GlobalVar(global_var) => {
                    let name = &global_var.var_name;
                    if self.module.constants.iter().any(|c| &c.name == name)
                        || self.module.globals.iter().any(|g| &g.name == name)
                    {
                        return Err(CodegenError::DuplicateGlobal(name.clone()));
                    }
                    let declared_ty = self.convert_to_ir_type(&global_var.var_type)?;
                    let (val_ty, val) = self.eval_const_expr(&global_var.value)?;
                    if val_ty != declared_ty {
                        return Err(CodegenError::MismatchedTypes(declared_ty, val_ty));
                    }
                    self.module.globals.push(IrGlobal {
                        name: name.clone(),
                        ty: declared_ty,
                        value: val,
                    });
                }
                _ => continue,
            }
        }
        Ok(())
    }

    pub(super) fn eval_const_expr(
        &self,
        expr: &Expr,
    ) -> Result<(IrType, PrimitiveValue), CodegenError> {
        match expr {
            Expr::IntLiteral(s) => {
                let val: i64 = s
                    .parse()
                    .map_err(|_| CodegenError::InvalidIntLiteral(s.clone()))?;
                Ok((IrType::I64, PrimitiveValue::Int(val)))
            }
            Expr::FloatLiteral(s) => {
                let val: f64 = s
                    .parse()
                    .map_err(|_| CodegenError::InvalidFloatLiteral(s.clone()))?;
                Ok((IrType::F64, PrimitiveValue::Float(val)))
            }
            Expr::BoolLiteral(b) => Ok((IrType::Bool, PrimitiveValue::Bool(*b))),
            Expr::CharLiteral(c) => Ok((IrType::Char, PrimitiveValue::Char(*c))),
            Expr::NullLiteral => Ok((IrType::Null, PrimitiveValue::Null)),
            //Todo andere exprs machen die auch in ein const/ global gespeichert werden können
            other => Err(CodegenError::InvalidExpr(other.clone())),
        }
    }
}

pub fn generate_code(items: &[Item]) -> Result<IrModule, CodegenError> {
    let mut generator = IrGenerator::new();

    generator.generate_code_inner(items)?;

    return Ok(generator.module);
}
