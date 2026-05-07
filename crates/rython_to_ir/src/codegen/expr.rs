use super::error::CodegenError;
use super::generator::IrGenerator;
use crate::ast::Expr;
use crate::ir::{IrInstruction, IrType, PrimitiveValue, TempId};

impl IrGenerator {
    pub(super) fn gen_expr(&mut self, expr: &Expr) -> Result<(TempId, IrType), CodegenError> {
        match expr {
            Expr::Variable(name) => self.gen_variable(name),
            Expr::IntLiteral(value) => self.gen_intliteral(value),
            Expr::FloatLiteral(value) => self.gen_floatliteral(value),
            Expr::BoolLiteral(value) => self.gen_boolliteral(*value),
            other => return Err(CodegenError::InvalidExpr(expr.clone())),
        }
    }

    fn gen_variable(&mut self, name: &str) -> Result<(TempId, IrType), CodegenError> {
        let freie_temp_var = self.next_temp_id();

        let var = self
            .lookup_variable(name)
            .ok_or_else(|| CodegenError::UnknownVariable(name.to_string()))?;
        let ty = var.ty.clone();
        let addr = var.addr;

        self.block_handler
            .add_instruction_to_current_block(IrInstruction::Load {
                temp_id: freie_temp_var,
                ty: ty.clone(),
                addr,
            })?;

        Ok((freie_temp_var, ty))
    }

    fn gen_intliteral(&mut self, value: &str) -> Result<(TempId, IrType), CodegenError> {
        let temp_id = self.next_temp_id();

        let val = value
            .parse()
            .map_err(|_| CodegenError::InvalidIntLiteral(value.to_string()))?;
        let new_const_instruction = IrInstruction::PrimitiveConst {
            temp_id,
            ty: IrType::I64,
            value: PrimitiveValue::Int(val),
        };

        self.block_handler
            .add_instruction_to_current_block(new_const_instruction)?;
        Ok((temp_id, IrType::I64))
    }

    fn gen_floatliteral(&mut self, value: &str) -> Result<(TempId, IrType), CodegenError> {
        let temp_id = self.next_temp_id();

        let val = value
            .parse()
            .map_err(|_| CodegenError::InvalidFloatLiteral(value.to_string()))?;

        let new_const_instruction = IrInstruction::PrimitiveConst {
            temp_id,
            ty: IrType::F64,
            value: PrimitiveValue::Float(val),
        };

        self.block_handler
            .add_instruction_to_current_block(new_const_instruction)?;

        Ok((temp_id, IrType::F64))
    }

    fn gen_boolliteral(&mut self, value: bool) -> Result<(TempId, IrType), CodegenError> {
        let temp_id = self.next_temp_id();

        let new_const_instruction = IrInstruction::PrimitiveConst {
            temp_id,
            ty: IrType::Bool,
            value: PrimitiveValue::Bool(value),
        };

        self.block_handler
            .add_instruction_to_current_block(new_const_instruction)?;

        Ok((temp_id, IrType::Bool))
    }
}
