use crate::ast::Stmt;
use crate::ir::{IrBlock, IrInstruction, IrType, Terminator};

use super::error::CodegenError;
use super::generator::IrGenerator;

impl IrGenerator {
    pub(super) fn gen_stmt(
        &mut self,
        stmt: &Stmt,
        block: &mut IrBlock,
    ) -> Result<(), CodegenError> {
        match stmt {
            Stmt::Let(l) => {
                let ir_type = Self::convert_to_ir_type(&l.var_type.clone()); // Todo: warum kann var type none sein???

                let id_for_alloc = self.next_temp_id();

                // id_for_alloc ist der name der temp variable, welche die freie Adresse hält
                block.instructions.push(IrInstruction::Alloca {
                    temp_id: id_for_alloc,
                    ty: ir_type.clone(),
                });

                // self.gen_expr returnt den name der temp variable, welcher zur Laufzeit das
                // Ergebnis halten wird
                // Bsp:
                // 1+1
                // %0 = const 1
                // %1 = const 2
                // %2 = add %0, %1
                // dann wäre hier expr_value = %2
                let (expr_value, expr_type) = self.gen_expr(&l.value, block)?;

                // gucken ob der ir_type welcher vom nutzer angegeben wurde, was die variable für
                // ein typ hat auch wirklich den selben typ hat wie das ergebnis der expression
                if (ir_type != expr_type) {
                    return Err(CodegenError::MismatchedTypes(ir_type, expr_type));
                }

                // Die temp variable welche das ergebnis der eval hält wird in die addr geladen
                // welche die variable id_for_alloc hält
                block.instructions.push(IrInstruction::Store {
                    ty: ir_type.clone(),
                    value: expr_value,
                    addr: id_for_alloc,
                });

                self.insert_variable(l.var_name.clone(), ir_type, id_for_alloc);

                Ok(())
            }
            Stmt::Return(ret) => {
                let return_value = ret.return_value.as_ref();
                // --> option <expr> entweder returnt es void oder eine expr

                match return_value {
                    Some(value) => {
                        let (temp_id, ret_t) = self.gen_expr(value, block)?; // Expr handeln -> macht sein eigenes Ding und
                                                                             // editiert die instructions des blocks. Return gibt nicht das ergebnis der
                                                                             // expr selber zurück sondern nur die variable also brauchen wir die temp id
                        if (ret_t != self.current_expected_return_type) {
                            return Err(CodegenError::InvalidReturnType(
                                self.current_expected_return_type.clone(),
                                ret_t,
                            ));
                        }
                        block.terminator = Terminator::Ret(Some(temp_id));

                        Ok(())
                    }
                    // Eigentlich unnötig, da block.terminator by default schon None ist aber egal
                    None => {
                        if (IrType::Void != self.current_expected_return_type) {
                            return Err(CodegenError::InvalidReturnType(
                                self.current_expected_return_type.clone(),
                                IrType::Void,
                            ));
                        }
                        block.terminator = Terminator::Ret(None);
                        Ok(())
                    }
                }
            }
            Stmt::Asm(asm) => {
                block.instructions.push(IrInstruction::Asm {
                    code: asm.asm_code.clone(),
                });
                Ok(())
            }
            _ => {
                return Err(CodegenError::InvalidStatement(stmt.clone()));
            }
        }
    }
}
