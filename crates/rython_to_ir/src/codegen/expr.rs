use std::iter::OnceWith;
use std::ops::Deref;

use super::error::CodegenError;
use super::generator::IrGenerator;
use crate::ast::{BinaryOp, Expr};
use crate::ir::{IrBinaryOp, IrInstruction, IrType, PrimitiveValue, TempId};

impl IrGenerator {
    pub(super) fn gen_expr(&mut self, expr: &Expr) -> Result<(TempId, IrType), CodegenError> {
        match expr {
            Expr::Variable(name) => self.gen_variable(name),
            Expr::IntLiteral(value) => self.gen_intliteral(value),
            Expr::FloatLiteral(value) => self.gen_floatliteral(value),
            Expr::BoolLiteral(value) => self.gen_boolliteral(*value),
            Expr::BinaryOp {
                lhs,
                binary_op,
                rhs,
            } => self.gen_binary_op(lhs, binary_op, rhs),
            other => return Err(CodegenError::InvalidExpr(expr.clone())),
        }
    }

    fn get_binary_expr_result_type(
        lhs: &IrType,
        op: &BinaryOp,
        rhs: &IrType,
    ) -> Result<IrType, CodegenError> {
        if lhs != rhs {
            return Err(CodegenError::InvalidBinaryOp(lhs.clone(), rhs.clone()));
        }
        match (lhs, op, lhs) {
            (IrType::I64, BinaryOp::Lt, IrType::I64) => return Ok(IrType::Bool),

            other => todo!(),
        }
    }

    fn gen_binary_op(
        &mut self,
        lhs: &Box<Expr>,
        binary_op: &BinaryOp,
        rhs: &Box<Expr>,
    ) -> Result<(TempId, IrType), CodegenError> {
        let (temp_id_1, ir_type_1) = self.gen_expr(lhs)?;
        let (temp_id_2, ir_type_2) = self.gen_expr(rhs)?;

        let result_type = Self::get_binary_expr_result_type(&ir_type_1, binary_op, &ir_type_2)?;

        let tmp_var = self.next_temp_id();
        let bin_op = Self::convert_to_ir_binary_op(&binary_op);

        let instruction = IrInstruction::Binary {
            temp_id: tmp_var,
            ty_res: result_type.clone(),
            ty_lr: ir_type_1,
            op: bin_op,
            lhs: temp_id_1,
            rhs: temp_id_2,
        };

        self.block_handler
            .add_instruction_to_current_block(instruction)?;

        return Ok((tmp_var, result_type));
    }

    fn convert_to_ir_binary_op(binary_op: &BinaryOp) -> IrBinaryOp {
        match binary_op {
            BinaryOp::Add => return IrBinaryOp::Add,
            BinaryOp::Lt => return IrBinaryOp::Lt,
            BinaryOp::Eq => return IrBinaryOp::Eq,
            BinaryOp::Ne => return IrBinaryOp::Ne,
            other => todo!()
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
