use std::any::Any;
use std::collections::HashMap;
use std::iter::OnceWith;
use std::ops::Deref;
use std::os::linux::net::SocketAddrExt;

use super::error::CodegenError;
use super::generator::IrGenerator;
use crate::ast::{BinaryOp, Expr, Type, UnaryOp};
use crate::ir::{
    IrBinaryOp, IrField, IrInstruction, IrType, IrTypeDefinition, IrUnaryOp, PrimitiveValue, TempId,
};

impl IrGenerator {
    pub(super) fn gen_expr(&mut self, expr: &Expr) -> Result<(TempId, IrType), CodegenError> {
        match expr {
            Expr::Variable(name) => self.gen_variable(name),
            Expr::IntLiteral(value) => self.gen_intliteral(value),
            Expr::FloatLiteral(value) => self.gen_floatliteral(value),
            Expr::BoolLiteral(value) => self.gen_boolliteral(*value),
            Expr::Unary { op, value } => self.gen_unary_op(op, value), // todo: Operator
            Expr::Assign { target, value } => self.gen_assign(target, value),
            Expr::BinaryOp {
                // todo: operator
                lhs,
                binary_op,
                rhs,
            } => self.gen_binary_op(lhs, binary_op, rhs),
            Expr::NullLiteral => unimplemented!(),
            Expr::CharLiteral(character) => self.gen_charliteral(*character),
            Expr::ListLiteral(inner) => unimplemented!(),
            Expr::Call {
                // todo: Mehr callee als variable, und generic types
                callee,
                type_args,
                arguments,
            } => self.gen_call(callee, type_args, arguments),
            Expr::PostFix { Op, value } => unimplemented!(), //operators
            Expr::StringLiteral(value) => self.gen_string_literal(value),
            Expr::StructLiteral {
                struct_name,
                arguments,
            } => self.gen_struct_literal(struct_name, arguments),
            Expr::Grouping(inner) => self.gen_grouping(inner),
            Expr::BinaryOpAssign {
                // todo: operators
                target,
                binary_op,
                value,
            } => self.gen_binary_op_assign(target, binary_op, value),
            Expr::FieldAccess { object, field_name } => unimplemented!(),
        }
    }

    fn gen_string_literal(&mut self, value: &str) -> Result<(TempId, IrType), CodegenError> {
        let (temp_id, _) = self.gen_struct_literal(
            &"string".to_string(),
            &vec![
                ("length".to_string(), Expr::IntLiteral("0".to_string())),
                ("start".to_string(), Expr::IntLiteral("0".to_string())),
            ],
        )?; //TODO names überall mangeln

        self.gen_call(Expr::Variable(""), type_args, arguments)
    }

    fn gen_struct_literal(
        &mut self,
        struct_name: &String,
        arguments: &Vec<(String, Expr)>,
    ) -> Result<(TempId, IrType), CodegenError> {
        let struct_pointer_base_temp_id = self.next_temp_id(); // ist die temp id wo der pointer zu
                                                               // dem struct gespeichert wird
        let struct_pointer_base_type =
            IrType::Pointer(Box::new(IrType::Named(struct_name.clone())));

        let mut first_temp_id = TempId(self.temp_counter + 1); // NICHT self.next_temp_id() callen , weil
                                                               // sonst es nicht ehrlich die id zum ersten
                                                               // field ist da in dem for loop unten schon
                                                               // beim ersten self.next_temp_id() gecallt
                                                               // wird

        self.block_handler
            .add_instruction_to_current_block(IrInstruction::PrimitiveConst {
                ty: struct_pointer_base_type.clone(),
                temp_id: struct_pointer_base_temp_id,
                value: PrimitiveValue::Pointer(first_temp_id),
            });

        let mut argument_temp_ids: HashMap<&String, TempId> = HashMap::new();

        //erst alle fields allocaten
        for (field_name, _) in arguments {
            let arg_temp_id = self.next_temp_id();
            self.block_handler
                .add_instruction_to_current_block(IrInstruction::Alloca {
                    temp_id: arg_temp_id,
                    ty: self.get_struct_field_typ(struct_name, field_name)?,
                });
            argument_temp_ids.insert(field_name, arg_temp_id);
        }

        //dann checken ob alle fields angegeben wurden
        if argument_temp_ids
            .keys()
            .map(|k| (*k).clone())
            .collect::<Vec<String>>()
            != self
                .get_struct_fields(struct_name)?
                .iter()
                .map(|irfld| irfld.name.clone())
                .collect::<Vec<String>>()
        {
            return Err(CodegenError::FieldsDontMatch);
        }

        //
        for (field_name, argument_value_expr) in arguments {
            let (this_field_temp_id, ty) = self.gen_expr(argument_value_expr)?;
            self.block_handler
                .add_instruction_to_current_block(IrInstruction::Store {
                    ty,
                    value: this_field_temp_id,
                    addr: argument_temp_ids
                        .get(field_name)
                        .unwrap_or_else(|| unreachable!())
                        .clone(),
                });
        }

        Ok((struct_pointer_base_temp_id, struct_pointer_base_type))
    }

    fn get_struct_fields(&self, struct_name: &String) -> Result<Vec<IrField>, CodegenError> {
        let mut found_struct = false;

        let mut typ = None;

        for typedef in self.type_defs.iter() {
            if let IrTypeDefinition::Struct { name, fields } = typedef {
                if found_struct {
                    return Err(CodegenError::AmbigousType(struct_name.clone()));
                } else {
                    found_struct = true;
                    typ = Some(fields.clone());
                }
            }
        }
        if !found_struct {
            return Err(CodegenError::UnknownType(struct_name.clone()));
        }
        Ok(typ.unwrap_or_else(|| unreachable!()))
    }

    fn get_struct_field_typ(
        &self,
        struct_name: &String,
        field_name: &String,
    ) -> Result<IrType, CodegenError> {
        let mut found_struct = false;
        let mut found_field = false;

        let mut typ = None;

        for typedef in self.type_defs.iter() {
            if let IrTypeDefinition::Struct { name, fields } = typedef {
                if found_struct {
                    return Err(CodegenError::AmbigousType(struct_name.clone()));
                } else {
                    found_struct = true;

                    for field in fields {
                        if &field.name == field_name {
                            if found_field {
                                return Err(CodegenError::AmbigousField(field_name.clone()));
                            } else {
                                found_field = true;
                                typ = Some(field.ty.clone());
                            }
                        }
                    }
                }
            }
        }

        if !found_struct {
            return Err(CodegenError::UnknownType(struct_name.clone()));
        }
        if !found_field {
            return Err(CodegenError::UnknownField(field_name.clone()));
        }
        Ok(typ.unwrap_or_else(|| unreachable!()))
    }

    fn gen_call(
        &mut self,
        callee: &Box<Expr>,
        type_args: &Vec<Type>,
        arguments: &Vec<Expr>,
    ) -> Result<(TempId, IrType), CodegenError> {
        let mut arg_temp_ids = vec![];

        for arg in arguments {
            arg_temp_ids.push(self.gen_expr(arg)?.0);
        }

        let function_name = match *callee.clone() {
            Expr::Variable(name) => name,
            _ => unimplemented!(),
        };
        let return_type = self
            .functions_return_type
            .get(&function_name)
            .ok_or(CodegenError::UnknownFunction(function_name.clone()))?
            .clone()
            .unwrap_or(IrType::Void);

        let temp_id = self.next_temp_id();
        self.block_handler
            .add_instruction_to_current_block(IrInstruction::Call {
                temp_id: Some(temp_id),
                function_name,
                args: arg_temp_ids,
                return_type: return_type.clone(),
            });

        Ok((temp_id, return_type))
    }

    fn gen_binary_op_assign(
        &mut self,
        target: &Box<Expr>,
        binary_op: &BinaryOp,
        value: &Box<Expr>,
    ) -> Result<(TempId, IrType), CodegenError> {
        let expanded_expr = Expr::BinaryOp {
            lhs: target.clone(),
            binary_op: binary_op.clone(),
            rhs: value.clone(),
        };
        self.gen_assign(target, &Box::new(expanded_expr))
    }

    fn gen_grouping(&mut self, inner: &Box<Expr>) -> Result<(TempId, IrType), CodegenError> {
        self.gen_expr(&inner)
    }

    fn gen_charliteral(&mut self, character: char) -> Result<(TempId, IrType), CodegenError> {
        let temp_id = self.next_temp_id();
        self.block_handler
            .add_instruction_to_current_block(IrInstruction::PrimitiveConst {
                temp_id,
                ty: IrType::Char,
                value: PrimitiveValue::Char(character),
            });

        Ok((temp_id, IrType::Char))
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
            Expr::PostFix { Op, value } => unimplemented!(),
            Expr::FieldAccess { object, field_name } => unimplemented!(),
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

    fn mangel(&self, input: String) -> String {
        self.scopes
    }
}
