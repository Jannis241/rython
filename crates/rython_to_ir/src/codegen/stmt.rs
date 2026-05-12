use std::collections::HashMap;

use crate::ast::{Asm, Expr, Let, Return, Stmt};
use crate::ir::{IrInstruction, IrType, TempId, Terminator};

use super::error::CodegenError;
use super::generator::IrGenerator;

impl IrGenerator {
    pub(super) fn gen_let(&mut self, l: &Let) -> Result<(), CodegenError> {
        let ir_type = self.convert_to_ir_type(&l.var_type.clone()); // Todo: warum kann var type none sein??? -> für type inference später vllt zb let x = 10; -> jz grade muss man immer let x: iint = 19;

        let id_for_alloc = self.next_temp_id();

        self.block_handler
            .add_instruction_to_current_block(IrInstruction::Alloca {
                temp_id: id_for_alloc,
                ty: ir_type.clone(),
            })?;

        let (expr_value, expr_type) = self.gen_expr(&l.value)?;

        if ir_type != expr_type {
            return Err(CodegenError::MismatchedTypes(ir_type, expr_type));
        }

        self.block_handler
            .add_instruction_to_current_block(IrInstruction::Store {
                ty: ir_type.clone(),
                value: expr_value,
                addr: id_for_alloc,
            })?;

        self.insert_variable(l.var_name.clone(), ir_type, id_for_alloc)?;

        Ok(())
    }

    pub(super) fn gen_return(&mut self, ret: &Return) -> Result<(), CodegenError> {
        match &ret.return_value {
            Some(value) => {
                let (temp_id, ret_t) = self.gen_expr(value)?;

                if ret_t != self.current_expected_return_type {
                    return Err(CodegenError::InvalidReturnType(
                        self.current_expected_return_type.clone(),
                        ret_t,
                    ));
                }
                self.block_handler
                    .add_terminator(Terminator::Ret(Some(temp_id)))?;

                Ok(())
            }
            None => {
                if IrType::Void != self.current_expected_return_type {
                    return Err(CodegenError::InvalidReturnType(
                        self.current_expected_return_type.clone(),
                        IrType::Void,
                    ));
                }
                self.block_handler.add_terminator(Terminator::Ret(None))?;
                Ok(())
            }
        }
    }

    pub(super) fn gen_asm(&mut self, asm: &Asm) -> Result<(), CodegenError> {
        let code = self.substitute_asm_variables(&asm.asm_code)?;
        self.block_handler
            .add_instruction_to_current_block(IrInstruction::Asm { code })?;
        Ok(())
    }

    fn substitute_asm_variables(&mut self, asm_code: &str) -> Result<String, CodegenError> {
        let bytes = asm_code.as_bytes();
        let mut out = String::with_capacity(asm_code.len());
        let mut loaded: HashMap<String, TempId> = HashMap::new();
        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i] as char;
            if c != '%' {
                out.push(c);
                i += 1;
                continue;
            }

            let start = i + 1;
            if start >= bytes.len() || !is_ident_start(bytes[start] as char) {
                out.push('%');
                i += 1;
                continue;
            }

            let mut end = start + 1;
            while end < bytes.len() && is_ident_cont(bytes[end] as char) {
                end += 1;
            }
            let name = &asm_code[start..end];

            let value_temp = if let Some(t) = loaded.get(name) {
                *t
            } else {
                let (var_ty, var_addr) = {
                    let var = self
                        .lookup_variable(name)
                        .ok_or_else(|| CodegenError::UnknownVariable(name.to_string()))?;
                    (var.ty.clone(), var.addr)
                };
                let temp = self.next_temp_id();
                self.block_handler
                    .add_instruction_to_current_block(IrInstruction::Load {
                        temp_id: temp,
                        ty: var_ty,
                        addr: var_addr,
                    })?;
                loaded.insert(name.to_string(), temp);
                temp
            };

            out.push('%');
            out.push_str(&value_temp.0.to_string());
            i = end;
        }
        Ok(out)
    }

    pub(super) fn gen_stmt(&mut self, stmt: &Stmt) -> Result<(), CodegenError> {
        match stmt {
            Stmt::Let(l) => self.gen_let(l),
            Stmt::Return(ret) => self.gen_return(ret),
            Stmt::Asm(asm) => self.gen_asm(asm),
            Stmt::Expr(expr) => {
                self.gen_expr(expr)?;
                Ok(())
            }
            _ => {
                return Err(CodegenError::InvalidStatement(stmt.clone()));
            }
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_ascii_alphabetic()
}

fn is_ident_cont(c: char) -> bool {
    c == '_' || c.is_ascii_alphanumeric()
}
