use std::any::Any;
use std::iter::OnceWith;
use std::ops::Deref;

use super::error::CodegenError;
use super::generator::IrGenerator;
use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::ir::{IrBinaryOp, IrInstruction, IrType, IrUnaryOp, PrimitiveValue, TempId};

impl IrGenerator {
    pub(super) fn gen_expr(&mut self, expr: &Expr) -> Result<(TempId, IrType), CodegenError> {
        match expr {
            Expr::Variable(name) => self.gen_variable(name),
            Expr::IntLiteral(value) => self.gen_intliteral(value),
            Expr::FloatLiteral(value) => self.gen_floatliteral(value),
            Expr::BoolLiteral(value) => self.gen_boolliteral(*value),
            Expr::Unary { op, value } => self.gen_unary_op(op, value), // todo: Operator
            Expr::Assign { target, value } => self.gen_assign(target, value),
            Expr::BinaryOp { // todo: operator
                lhs,
                binary_op,
                rhs,
            } => self.gen_binary_op(lhs, binary_op, rhs),
            other => return Err(CodegenError::InvalidExpr(expr.clone())),
        }
    }

    fn gen_assign(
        &mut self,
        target: &Box<Expr>,
        value: &Box<Expr>,
    ) -> Result<(TempId, IrType), CodegenError> {
        let t = target.deref();

        let (addr, target_type) = match t {
            Expr::Variable(name) => {
                let variable = self
                    .lookup_variable(name)
                    .ok_or_else(|| CodegenError::UnknownVariable(name.clone()))?;

                (variable.addr, variable.ty.clone())
            }
            other => return Err(CodegenError::InvalidExpr(other.clone())),
        };

        let (temp_value_var, value_type) = self.gen_expr(value)?;

        if (target_type != value_type) {
            return Err(CodegenError::MismatchedTypes(
                target_type.clone(),
                value_type,
            ));
        }

        self.block_handler
            .add_instruction_to_current_block(IrInstruction::Store {
                ty: target_type.clone(),
                value: temp_value_var,
                addr,
            })?;

        Ok((temp_value_var, target_type))
    }

    fn convert_to_ir_unary_op(unary_op: &UnaryOp) -> IrUnaryOp {
        match unary_op {
            UnaryOp::Neg => IrUnaryOp::Neg,
            UnaryOp::Not => IrUnaryOp::Not,
            UnaryOp::BitNot => IrUnaryOp::BitNot,
        }
    }
    fn check_unary_op_type(op: &UnaryOp, value_type: &IrType) -> Result<(), CodegenError> {
        match op {
            UnaryOp::Neg => match value_type {
                IrType::I64 | IrType::F64 => Ok(()),
                _ => Err(CodegenError::InvalidUnaryOp(value_type.clone())),
            },

            UnaryOp::Not => match value_type {
                IrType::Bool => Ok(()),
                _ => Err(CodegenError::InvalidUnaryOp(value_type.clone())),
            },

            UnaryOp::BitNot => match value_type {
                IrType::I64 => Ok(()),
                _ => Err(CodegenError::InvalidUnaryOp(value_type.clone())),
            },
        }
    }

    fn gen_unary_op(
        &mut self,
        unary_op: &UnaryOp,
        value: &Box<Expr>,
    ) -> Result<(TempId, IrType), CodegenError> {
        let (tmp_id_1, ir_type) = self.gen_expr(value)?;

        Self::check_unary_op_type(unary_op, &ir_type)?;
        let ir_unary_op = Self::convert_to_ir_unary_op(unary_op);

        let temp_var = self.next_temp_id();
        let instruction = IrInstruction::Unary {
            temp_id: temp_var,
            ty: ir_type.clone(),
            op: ir_unary_op,
            value: tmp_id_1,
        };
        self.block_handler
            .add_instruction_to_current_block(instruction)?;
        return Ok((temp_var, ir_type));
    }

    fn get_binary_expr_result_type(
        lhs: &IrType,
        op: &BinaryOp,
        rhs: &IrType,
    ) -> Result<IrType, CodegenError> {
        if lhs != rhs {
            return Err(CodegenError::InvalidBinaryOp(lhs.clone(), rhs.clone()));
        }
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => match lhs {
                IrType::I64 | IrType::F64 => Ok(lhs.clone()),
                _ => Err(CodegenError::InvalidBinaryOp(lhs.clone(), rhs.clone())),
            },
            BinaryOp::Mod => match lhs {
                IrType::I64 => Ok(IrType::I64),
                _ => Err(CodegenError::InvalidBinaryOp(lhs.clone(), rhs.clone())),
            },
            BinaryOp::Eq | BinaryOp::Ne => match lhs {
                IrType::Void => Err(CodegenError::InvalidBinaryOp(lhs.clone(), rhs.clone())),
                _ => Ok(IrType::Bool),
            },
            BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => match lhs {
                IrType::I64 | IrType::F64 => Ok(IrType::Bool),
                _ => Err(CodegenError::InvalidBinaryOp(lhs.clone(), rhs.clone())),
            },
            BinaryOp::And | BinaryOp::Or => match lhs {
                IrType::Bool => Ok(IrType::Bool),
                _ => Err(CodegenError::InvalidBinaryOp(lhs.clone(), rhs.clone())),
            },
            BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor
            | BinaryOp::Shl
            | BinaryOp::Shr => match lhs {
                IrType::I64 => Ok(IrType::I64),
                _ => Err(CodegenError::InvalidBinaryOp(lhs.clone(), rhs.clone())),
            },
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
            BinaryOp::Sub => return IrBinaryOp::Sub,
            BinaryOp::Mul => return IrBinaryOp::Mul,
            BinaryOp::Div => return IrBinaryOp::Div,
            BinaryOp::Mod => return IrBinaryOp::Mod,
            BinaryOp::Eq => return IrBinaryOp::Eq,
            BinaryOp::Ne => return IrBinaryOp::Ne,
            BinaryOp::Lt => return IrBinaryOp::Lt,
            BinaryOp::Le => return IrBinaryOp::Le,
            BinaryOp::Gt => return IrBinaryOp::Gt,
            BinaryOp::Ge => return IrBinaryOp::Ge,
            BinaryOp::And => return IrBinaryOp::And,
            BinaryOp::Or => return IrBinaryOp::Or,
            BinaryOp::BitAnd => return IrBinaryOp::BitAnd,
            BinaryOp::BitOr => return IrBinaryOp::BitOr,
            BinaryOp::BitXor => return IrBinaryOp::BitXor,
            BinaryOp::Shl => return IrBinaryOp::Shl,
            BinaryOp::Shr => return IrBinaryOp::Shr,
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
