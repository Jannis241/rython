use crate::ast::Expr;
use crate::ir::{IrBlock, IrInstruction, IrType, PrimitiveValue, TempId};

use super::error::CodegenError;
use super::generator::IrGenerator;

impl IrGenerator {
    pub(super) fn gen_expr(
        &mut self,
        expr: &Expr,
        block: &mut IrBlock,
    ) -> Result<(TempId, IrType), CodegenError> {
        // Expr handeln: Instructions in dem Block je nach expression verändern und die temp id
        // zurück geben wo das ergebnis der expr genau gespeichert wird, damit aufrufende methoden
        // das nutzen können (wie zb return)
        match expr {
            Expr::Variable(name) => {
                let freie_temp_var = self.next_temp_id();

                let var = self
                    .lookup_variable(name)
                    .ok_or_else(|| CodegenError::UnknownVariable(name.clone()))?;

                block.instructions.push(IrInstruction::Load {
                    temp_id: freie_temp_var,
                    ty: var.ty.clone(),
                    addr: var.addr,
                });

                Ok((freie_temp_var, var.ty.clone()))
            }
            Expr::IntLiteral(value) => {
                let temp_id = self.next_temp_id();

                let val = value
                    .parse()
                    .map_err(|e| CodegenError::InvalidIntLiteral(value.clone()))?;
                let new_const_instruction = IrInstruction::PrimitiveConst {
                    temp_id: temp_id,
                    ty: IrType::I64,
                    value: PrimitiveValue::Int(val),
                };

                block.instructions.push(new_const_instruction);
                return Ok((temp_id, IrType::I64));
            }
            Expr::FloatLiteral(value) => {
                let temp_id = self.next_temp_id();

                let val = value
                    .parse()
                    .map_err(|e| CodegenError::InvalidFloatLiteral(value.clone()))?;

                let new_const_instruction = IrInstruction::PrimitiveConst {
                    temp_id: temp_id,
                    ty: IrType::F64,
                    value: PrimitiveValue::Float(val),
                };

                block.instructions.push(new_const_instruction);

                return Ok((temp_id, IrType::F64));
            }
            Expr::BoolLiteral(value) => {
                let temp_id = self.next_temp_id();

                let new_const_instruction = IrInstruction::PrimitiveConst {
                    temp_id: temp_id,
                    ty: IrType::Bool,
                    value: PrimitiveValue::Bool(*value),
                };

                block.instructions.push(new_const_instruction);

                return Ok((temp_id, IrType::Bool));
            }
            Expr::StringLiteral(value) => {
                let temp_id = self.next_temp_id();

                // todo: String muss zu einem String struct gemacht werden
                // struct init callen
                // Fields: length, start
                // length = value.len
                // start = 0,

                // jesko yapping
                // block.instructions.push(value);
                // init_start()
                // push_char(char) -> value chars

                return Ok((temp_id, IrType::Named("string".to_string())));
            }
            other => {
                return Err(CodegenError::InvalidExpr(other.clone()));
            }
        }
    }
}
