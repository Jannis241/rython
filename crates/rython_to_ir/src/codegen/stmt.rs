use crate::ast::{Asm, Let, Return, Stmt};
use crate::ir::{IrInstruction, IrType, Terminator};

use super::error::CodegenError;
use super::generator::IrGenerator;

impl IrGenerator {
    pub(super) fn gen_let(&mut self, l: &Let) -> Result<(), CodegenError> {
        let ir_type = Self::convert_to_ir_type(&l.var_type.clone()); // Todo: warum kann var type none sein???

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

        self.insert_variable(l.var_name.clone(), ir_type, id_for_alloc);

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
        self.block_handler
            .add_instruction_to_current_block(IrInstruction::Asm {
                code: asm.asm_code.clone(),
            })?;
        Ok(())
    }

    pub(super) fn gen_stmt(&mut self, stmt: &Stmt) -> Result<(), CodegenError> {
        match stmt {
            Stmt::Let(l) => self.gen_let(l),
            Stmt::Return(ret) => self.gen_return(ret),
            Stmt::Asm(asm) => self.gen_asm(asm),
            _ => {
                return Err(CodegenError::InvalidStatement(stmt.clone()));
            }
        }
    }
}
