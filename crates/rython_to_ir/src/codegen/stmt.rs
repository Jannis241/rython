use std::collections::HashMap;

use crate::ast::{Asm, Block, If, Let, Loop, Return, Stmt, While};
use crate::ir::{IrInstruction, IrType, TempId, Terminator};

use super::error::CodegenError;
use super::generator::IrGenerator;

impl IrGenerator {
    pub(super) fn gen_let(&mut self, l: &Let) -> Result<(), CodegenError> {
        let ir_type = self.convert_to_ir_type(&l.var_type.clone())?;

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
                if ret_t == IrType::Void {
                    self.block_handler.add_terminator(Terminator::Ret(None))?;
                    // Fix, dass wenn man zb macht:
                    // fn foo() {}
                    // fn main() {
                    //      return foo();
                    // }
                    // der return type nicht Some(<ergebnis von foo>) ist sondern None, da es void
                    // ist
                    // => void sollte kein wert sein den man wirklich returnt also sozusagen nicht
                    // Terminator::Ret(Some(void))
                } else {
                    self.block_handler
                        .add_terminator(Terminator::Ret(Some(temp_id)))?;
                }

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
            Stmt::Block(b) => self.gen_block(b),
            Stmt::If(if_stmt) => self.gen_if(if_stmt),
            Stmt::While(w) => self.gen_while(w),
            Stmt::Loop(l) => self.gen_loop(l),
            Stmt::Break => self.gen_break(),
            Stmt::Continue => self.gen_continue(),
            _ => {
                return Err(CodegenError::InvalidStatement(stmt.clone()));
            }
        }
    }

    // führt einen block aus mit eigenem scope.
    pub(super) fn gen_block(&mut self, block: &Block) -> Result<(), CodegenError> {
        self.enter_scope();
        for stmt in &block.statements {
            self.gen_stmt(stmt)?;
        }
        self.exit_scope();
        Ok(())
    }

    pub(super) fn gen_if(&mut self, if_stmt: &If) -> Result<(), CodegenError> {
        let (cond_temp, cond_ty) = self.gen_expr(&if_stmt.condition)?;
        if cond_ty != IrType::Bool {
            return Err(CodegenError::MismatchedTypes(IrType::Bool, cond_ty));
        }

        let then_label = self.fresh_block_label("if_then");
        let merge_label = self.fresh_block_label("if_merge");
        let else_label = if if_stmt.else_code.is_some() {
            self.fresh_block_label("if_else")
        } else {
            merge_label.clone()
        };

        self.block_handler.add_terminator(Terminator::Branch {
            condition: cond_temp,
            then_block: then_label.clone(),
            else_block: else_label.clone(),
        })?;

        // then arm
        self.block_handler.create_new_block(&then_label);
        self.block_handler.jump_to_block(&then_label);
        self.gen_block(&if_stmt.if_code)?;
        let then_terminated = self.block_handler.is_current_terminated();
        if !then_terminated {
            self.block_handler.add_terminator(Terminator::Jump {
                target: merge_label.clone(),
            })?;
        }

        // else arm (falls vorhanden)
        let mut else_terminated = false;
        let has_else = if_stmt.else_code.is_some();
        if let Some(else_stmt) = &if_stmt.else_code {
            self.block_handler.create_new_block(&else_label);
            self.block_handler.jump_to_block(&else_label);
            self.gen_stmt(else_stmt)?;
            else_terminated = self.block_handler.is_current_terminated();
            if !else_terminated {
                self.block_handler.add_terminator(Terminator::Jump {
                    target: merge_label.clone(),
                })?;
            }
        }

        // merge block nur erzeugen, wenn er erreichbar ist:
        // kein else -> branch springt im false-fall direkt zu merge
        // hat else, aber mindestens ein arm fällt durch -> merge erreichbar
        let merge_reachable = !has_else || !then_terminated || !else_terminated;
        if merge_reachable {
            self.block_handler.create_new_block(&merge_label);
            self.block_handler.jump_to_block(&merge_label);
        }

        Ok(())
    }

    pub(super) fn gen_while(&mut self, w: &While) -> Result<(), CodegenError> {
        let cond_label = self.fresh_block_label("while_cond");
        let body_label = self.fresh_block_label("while_body");
        let end_label = self.fresh_block_label("while_end");

        // aktuellen block in cond springen lassen
        self.block_handler.add_terminator(Terminator::Jump {
            target: cond_label.clone(),
        })?;

        // cond
        self.block_handler.create_new_block(&cond_label);
        self.block_handler.jump_to_block(&cond_label);
        let (cond_temp, cond_ty) = self.gen_expr(&w.condition)?;
        if cond_ty != IrType::Bool {
            return Err(CodegenError::MismatchedTypes(IrType::Bool, cond_ty));
        }
        self.block_handler.add_terminator(Terminator::Branch {
            condition: cond_temp,
            then_block: body_label.clone(),
            else_block: end_label.clone(),
        })?;

        // body
        self.block_handler.create_new_block(&body_label);
        self.block_handler.jump_to_block(&body_label);
        self.loop_stack
            .push((cond_label.clone(), end_label.clone()));
        self.gen_block(&w.inner_code)?;
        self.loop_stack.pop();
        if !self.block_handler.is_current_terminated() {
            self.block_handler.add_terminator(Terminator::Jump {
                target: cond_label.clone(),
            })?;
        }

        // end (block nach der loop)
        self.block_handler.create_new_block(&end_label);
        self.block_handler.jump_to_block(&end_label);
        Ok(())
    }

    pub(super) fn gen_loop(&mut self, l: &Loop) -> Result<(), CodegenError> {
        let body_label = self.fresh_block_label("loop_body");
        let end_label = self.fresh_block_label("loop_end");

        self.block_handler.add_terminator(Terminator::Jump {
            target: body_label.clone(),
        })?;

        self.block_handler.create_new_block(&body_label);
        self.block_handler.jump_to_block(&body_label);
        // continue in loop springt zurück zum body start
        self.loop_stack
            .push((body_label.clone(), end_label.clone()));
        self.gen_block(&l.inner_code)?;
        self.loop_stack.pop();
        if !self.block_handler.is_current_terminated() {
            self.block_handler.add_terminator(Terminator::Jump {
                target: body_label.clone(),
            })?;
        }

        self.block_handler.create_new_block(&end_label);
        self.block_handler.jump_to_block(&end_label);
        Ok(())
    }

    pub(super) fn gen_break(&mut self) -> Result<(), CodegenError> {
        let (_cont, brk) = self
            .loop_stack
            .last()
            .ok_or(CodegenError::BreakOutsideLoop)?
            .clone();
        self.block_handler
            .add_terminator(Terminator::Jump { target: brk })?;
        Ok(())
    }

    pub(super) fn gen_continue(&mut self) -> Result<(), CodegenError> {
        let (cont, _brk) = self
            .loop_stack
            .last()
            .ok_or(CodegenError::ContinueOutsideLoop)?
            .clone();
        self.block_handler
            .add_terminator(Terminator::Jump { target: cont })?;
        Ok(())
    }
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_ascii_alphabetic()
}

fn is_ident_cont(c: char) -> bool {
    c == '_' || c.is_ascii_alphanumeric()
}
