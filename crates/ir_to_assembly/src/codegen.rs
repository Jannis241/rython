use rython_to_ir::codegen::*;
use std::{
    collections::HashMap,
    error::Error,
    fmt::{format, Write},
};

enum AsmCodeGenErr {
    TypeNotFound(String),
    MultipleTypesFound(String),
}

macro_rules! emit {
    ($self:expr, $($arg:tt)*) => {
        write!($self.out, $($arg)*).unwrap();
    };
}

pub struct AsmCodeGen {
    out: String,
    input: IrModule,
}

impl AsmCodeGen {
    pub fn gen_asm(input: IrModule) -> Result<String, AsmCodeGenErr> {
        let mut asm_code_gen = AsmCodeGen {
            out: String::new(),
            input,
        };

        asm_code_gen.generate_globals()?; //done
        asm_code_gen.generate_constants()?; //done
        asm_code_gen.generate_functions()?;

        Ok(asm_code_gen.out)
    }

    fn generate_functions(&mut self) -> Result<(), AsmCodeGenErr> {
        emit!(self, "section .text\n");
        emit!(self, "global _start\n");
        emit!(self, "\n");

        for function in self.input.functions.clone() {
            self.generate_function(function)?;
        }

        Ok(())
    }
    fn generate_function(&mut self, function: IrFunction) -> Result<(), AsmCodeGenErr> {
        if function.name == "main" {
            emit!(self, "_start:\n");
        }

        emit!(self, "{}:\n", function.name);

        let mut arg_locations = HashMap::new();

        let mut base = 16;

        for param in function.parameter {
            let size_dqs = self.get_type_size_in_qds(param.param_type)?;
            let name = param.name;
            for partition in 0..size_dqs {
                arg_locations.insert(format!("{}_{}", name, partition), format!("[rbp+{}]", base));

                base += 8;
            }
        }

        Ok(())
    }
    fn generate_globals(&mut self) -> Result<(), AsmCodeGenErr> {
        for global in self.input.globals.clone() {
            emit!(self, "section .data\n");
            emit!(self, "{}:\n", global.name);
            emit!(
                self,
                "   dq {}\n",
                self.const_value_into_string(global.value)?
            );
        }

        Ok(())
    }
    fn generate_constants(&mut self) -> Result<(), AsmCodeGenErr> {
        for constant in self.input.constants.clone() {
            emit!(self, "section .data\n");
            emit!(self, "{}:\n", constant.name);
            emit!(
                self,
                "   dq {}\n",
                self.const_value_into_string(constant.value)?
            );
        }

        //TODO: feheler   wenn man versuch ein const value zu schreiben

        Ok(())
    }

    pub fn const_value_into_string(&self, value: ConstValue) -> Result<String, AsmCodeGenErr> {
        match value {
            ConstValue::Int(i) => Ok(i.to_string()),
            ConstValue::Bool(val) => Ok(if val {
                "1".to_string()
            } else {
                "0".to_string()
            }),
            ConstValue::Null => Ok("0".to_string()),
            ConstValue::Char(c) => Ok(format!("'{}'", c)),
            ConstValue::Float(f) => Ok(f.to_string()),
            ConstValue::String(s) => Ok(s),
        }
    }

    pub fn get_type_size_in_qds(&self, typ: IrType) -> Result<usize, AsmCodeGenErr> {
        match typ {
            IrType::I64 => Ok(1),
            IrType::F64 => Ok(1),
            IrType::Bool => Ok(1),
            IrType::Void => Ok(0),
            IrType::Pointer(_) => todo!("pointer implementen"),
            IrType::Named(name) => self.get_named_size_in_qds(name),
        }
    }

    pub fn get_named_size_in_qds(&self, named_typ_name: String) -> Result<usize, AsmCodeGenErr> {
        let types: Vec<IrTypeDef> = self.input.types.clone();

        let mut matching_type = None;

        for typ in types {
            match typ {
                IrTypeDef::Struct { name, fields } => {
                    if named_typ_name == name {
                        if matching_type.is_none() {
                            matching_type = Some(IrTypeDef::Struct { name, fields });
                        } else {
                            return Err(AsmCodeGenErr::MultipleTypesFound(named_typ_name));
                        }
                    }
                }
                IrTypeDef::Variant { name, cases } => {
                    if matching_type.is_none() {
                        matching_type = Some(IrTypeDef::Variant { name, cases });
                    } else {
                        return Err(AsmCodeGenErr::MultipleTypesFound(named_typ_name));
                    }
                }
            }
        }

        let Some(matching_type_def) = matching_type else {
            return Err(AsmCodeGenErr::TypeNotFound(named_typ_name));
        };

        let qds = match matching_type_def {
            IrTypeDef::Struct { name, fields } => {
                let mut total_qds = 0;
                for field in fields {
                    total_qds += self.get_type_size_in_qds(field.ty)?;
                }
                total_qds
            }
            IrTypeDef::Variant { name, cases } => 1,
        };

        Ok(qds)
    }
}
