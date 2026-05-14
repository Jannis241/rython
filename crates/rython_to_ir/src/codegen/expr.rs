use std::collections::HashMap;
use std::ops::Deref;

use super::error::CodegenError;
use super::generator::IrGenerator;
use crate::ast::{BinaryOp, Expr, PostFixOp, Type, UnaryOp};
use crate::codegen::generator;
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
            Expr::Unary { op, value } => self.gen_unary_op(op, value),
            Expr::Assign { target, value } => self.gen_assign(target, value),
            Expr::BinaryOp {
                lhs,
                binary_op,
                rhs,
            } => self.gen_binary_op(lhs, binary_op, rhs),
            Expr::NullLiteral => self.gen_null_literal(),
            Expr::CharLiteral(character) => self.gen_charliteral(*character),
            Expr::ListLiteral(inner) => self.gen_list_literal(inner),
            Expr::Call {
                callee,
                type_args,
                arguments,
            } => self.gen_call(callee, type_args, arguments),
            Expr::PostFix { Op, value } => self.gen_postfix(Op, value),
            Expr::StringLiteral(value) => self.gen_string_literal(value),
            Expr::StructLiteral {
                struct_name,
                arguments,
            } => self.gen_struct_literal(struct_name, arguments),
            Expr::Grouping(inner) => self.gen_grouping(inner),
            Expr::BinaryOpAssign {
                target,
                binary_op,
                value,
            } => self.gen_binary_op_assign(target, binary_op, value),
            Expr::FieldAccess { object, field_name } => self.gen_field_access(object, field_name),
            Expr::VariantLiteral {
                variant_name,
                case_name,
            } => self.gen_variant_literal(variant_name, case_name),
        }
    }

    fn gen_variant_literal(
        &mut self,
        variant_name: &String,
        case_name: &String,
    ) -> Result<(TempId, IrType), CodegenError> {
        let cases = self.get_variant_cases(variant_name)?;

        let case_idx = cases
            .iter()
            .position(|c| c == case_name)
            .ok_or(CodegenError::UnknownField(case_name.clone()))?;

        self.gen_intliteral(&case_idx.to_string())
    }

    fn get_variant_cases(&self, variant_name: &String) -> Result<Vec<String>, CodegenError> {
        let mut found_variant = false;

        let mut found_cases = None;

        for typedef in self.type_defs.iter() {
            let IrTypeDefinition::Variant { name, cases } = typedef else {
                continue;
            };

            if name == variant_name {
                if found_variant {
                    return Err(CodegenError::AmbigousType(variant_name.clone()));
                }
                found_variant = true;
                found_cases = Some(cases);
            }
        }

        if found_cases.is_none() {
            return Err(CodegenError::UnknownType(variant_name.clone()));
        }

        Ok(found_cases.unwrap().clone())
    }

    fn gen_string_literal(&mut self, value: &str) -> Result<(TempId, IrType), CodegenError> {
        let struct_literal = Expr::StructLiteral {
            struct_name: "string".into(),
            arguments: vec![
                ("start".to_string(), Expr::IntLiteral("0".to_string())),
                ("length".to_string(), Expr::IntLiteral("0".to_string())),
            ],
        };
        let (temp_id, ir_ty) = match &struct_literal {
            Expr::StructLiteral {
                struct_name,
                arguments,
            } => self.gen_struct_literal(&struct_name, &arguments)?,
            _ => unreachable!(),
        };

        self.gen_method_call_with_temp_id(
            temp_id,
            ir_ty.clone(),
            &"init_start".to_string(),
            &vec![],
        )?;

        for character in value.chars() {
            self.gen_method_call_with_temp_id(
                temp_id,
                ir_ty.clone(),
                &"push_char".to_string(),
                &vec![Expr::CharLiteral(character)],
            )?;
        }

        Ok((temp_id, ir_ty))
    }
    fn gen_list_literal(
        &mut self,
        values: &Vec<Box<Expr>>,
    ) -> Result<(TempId, IrType), CodegenError> {
        let struct_literal = Expr::StructLiteral {
            struct_name: "list".into(),
            arguments: vec![
                ("start".to_string(), Expr::IntLiteral("0".to_string())),
                ("length".to_string(), Expr::IntLiteral("0".to_string())),
            ],
        };
        let (temp_id, ir_ty) = match &struct_literal {
            Expr::StructLiteral {
                struct_name,
                arguments,
            } => self.gen_struct_literal(&struct_name, &arguments)?,
            _ => unreachable!(),
        };

        self.gen_method_call_with_temp_id(
            temp_id,
            ir_ty.clone(),
            &"init_start".to_string(),
            &vec![],
        )?;

        for expr in values {
            self.gen_method_call_with_temp_id(
                temp_id,
                ir_ty.clone(),
                &"push_element".to_string(),
                &vec![*expr.clone()], // achtung kann nicht values direkt sein, weil push_element
                                      // immer nur einzelnend ein element pusht
            )?;
        }

        Ok((temp_id, ir_ty))
    }

    fn gen_field_access(
        &mut self,
        object: &Box<Expr>,
        field_name: &String,
    ) -> Result<(TempId, IrType), CodegenError> {
        // lädt den feldwert aus dem objekt über GetFieldAddr und Load
        let (field_addr, field_ty) = self.gen_field_addr(object, field_name)?;
        let value_temp = self.next_temp_id();
        self.block_handler
            .add_instruction_to_current_block(IrInstruction::Load {
                temp_id: value_temp,
                ty: field_ty.clone(),
                addr: field_addr,
            })?;
        Ok((value_temp, field_ty))
    }

    // liefert die adresse vom feld zurück, nicht den wert.
    // für GetFieldAddr brauchen wir die addr des structs (nicht seinen wert),
    // also gen_lvalue_addr statt gen_expr. bei Pointer<Named> ist der wert die addr,
    // dann muss einmal geladen werden.
    fn gen_field_addr(
        &mut self,
        object: &Box<Expr>,
        field_name: &String,
    ) -> Result<(TempId, IrType), CodegenError> {
        let (lv_addr, lv_ty) = self.gen_left_value_addr(object)?;
        let (base_addr, struct_ty) = match &lv_ty {
            IrType::Pointer(_) => {
                let loaded = self.next_temp_id();
                self.block_handler
                    .add_instruction_to_current_block(IrInstruction::Load {
                        temp_id: loaded,
                        ty: lv_ty.clone(),
                        addr: lv_addr,
                    })?;
                (loaded, lv_ty)
            }
            _ => (lv_addr, lv_ty),
        };

        let struct_name = Self::struct_name_from_ty(&struct_ty)
            .ok_or_else(|| CodegenError::UnknownType(format!("{struct_ty:?}")))?;
        let field_ty = self.get_struct_field_typ(&struct_name, field_name)?;
        let field_addr = self.next_temp_id();
        self.block_handler
            .add_instruction_to_current_block(IrInstruction::GetFieldAddr {
                temp_id: field_addr,
                base_addr,
                field_name: field_name.clone(),
            })?;
        Ok((field_addr, field_ty))
    }

    // zieht den struct namen aus einem typ
    // funktioniert für Named und Pointer<Named>
    fn struct_name_from_ty(ty: &IrType) -> Option<String> {
        match ty {
            IrType::Named(name) => Some(name.clone()),
            IrType::Pointer(inner) => match inner.as_ref() {
                IrType::Named(name) => Some(name.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    // hilfsfunktion für left-werte: liefert die adresse und den typ.
    // unterstützt variable (lokal/global), feld zugriff und gruppierung.
    // konsts haben keine zuweisbare adresse -> AssignToConst.
    fn gen_left_value_addr(&mut self, expr: &Box<Expr>) -> Result<(TempId, IrType), CodegenError> {
        match expr.deref() {
            Expr::Variable(name) => {
                if let Some(var) = self.lookup_variable(name) {
                    return Ok((var.addr, var.ty.clone()));
                }
                if self.module.constants.iter().any(|c| c.name == *name) {
                    return Err(CodegenError::AssignToConst(name.clone()));
                }
                if let Some(global) = self
                    .module
                    .globals
                    .iter()
                    .find(|g| g.name == *name)
                    .cloned()
                {
                    let addr_temp = self.next_temp_id();
                    self.block_handler.add_instruction_to_current_block(
                        IrInstruction::GlobalAddr {
                            temp_id: addr_temp,
                            name: global.name,
                            ty: global.ty.clone(),
                        },
                    )?;
                    return Ok((addr_temp, global.ty));
                }
                Err(CodegenError::UnknownVariable(name.clone()))
            }
            Expr::FieldAccess { object, field_name } => self.gen_field_addr(object, field_name),
            Expr::Grouping(inner) => self.gen_left_value_addr(inner),
            other => Err(CodegenError::InvalidExpr(other.clone())),
        }
    }

    fn gen_postfix(
        &mut self,
        op: &PostFixOp,
        value: &Box<Expr>,
    ) -> Result<(TempId, IrType), CodegenError> {
        match op {
            PostFixOp::PlusPlus | PostFixOp::MinusMinus => {
                // left-wert holen aus variable oder feld zugriff
                let (target_addr, target_ty) = self.gen_left_value_addr(value)?;

                if target_ty != IrType::I64 && target_ty != IrType::F64 {
                    return Err(CodegenError::InvalidUnaryOp(target_ty));
                }

                let old_value_id = self.next_temp_id();
                self.block_handler
                    .add_instruction_to_current_block(IrInstruction::Load {
                        temp_id: old_value_id,
                        ty: target_ty.clone(),
                        addr: target_addr,
                    })?;

                let one_id = self.next_temp_id();
                let one_value = match target_ty {
                    IrType::I64 => PrimitiveValue::Int(1),
                    IrType::F64 => PrimitiveValue::Float(1.0),
                    _ => unreachable!(),
                };
                self.block_handler.add_instruction_to_current_block(
                    IrInstruction::PrimitiveConst {
                        temp_id: one_id,
                        ty: target_ty.clone(),
                        value: one_value,
                    },
                )?;

                let bin_op = match op {
                    PostFixOp::PlusPlus => IrBinaryOp::Add,
                    PostFixOp::MinusMinus => IrBinaryOp::Sub,
                    _ => unreachable!(),
                };
                let new_value_id = self.next_temp_id();
                self.block_handler
                    .add_instruction_to_current_block(IrInstruction::Binary {
                        temp_id: new_value_id,
                        ty_lr: target_ty.clone(),
                        ty_res: target_ty.clone(),
                        op: bin_op,
                        lhs: old_value_id,
                        rhs: one_id,
                    })?;

                self.block_handler
                    .add_instruction_to_current_block(IrInstruction::Store {
                        ty: target_ty.clone(),
                        value: new_value_id,
                        addr: target_addr,
                    })?;

                Ok((old_value_id, target_ty))
            }
            PostFixOp::Brackets(idx_expr) => {
                // [] wird über die operator überladung auf Named typen aufgelöst
                // arr[idx] wird zu Struct_<[]>(arr idx)
                let (lhs_temp, lhs_ty) = self.gen_expr(value)?;
                let (idx_temp, _idx_ty) = self.gen_expr(idx_expr)?;

                let struct_name = Self::struct_name_from_ty(&lhs_ty)
                    .ok_or_else(|| CodegenError::UnknownType(format!("{lhs_ty:?}")))?;
                let key = (struct_name.clone(), "[]".to_string());
                let (mangled, return_ty) =
                    self.operator_functions.get(&key).cloned().ok_or_else(|| {
                        CodegenError::UnknownFunction(format!("{}_[]", struct_name))
                    })?;

                let temp_id = self.next_temp_id();
                self.block_handler
                    .add_instruction_to_current_block(IrInstruction::Call {
                        temp_id: temp_id,
                        function_name: mangled,
                        args: vec![lhs_temp, idx_temp],
                        return_type: return_ty.clone(),
                    })?;
                Ok((temp_id, return_ty))
            }
        }
    }

    pub(super) fn gen_struct_literal(
        &mut self,
        struct_name: &String,
        arguments: &Vec<(String, Expr)>,
    ) -> Result<(TempId, IrType), CodegenError> {
        // 1. fields prüfen
        let struct_fields = self.get_struct_fields(struct_name)?;
        if arguments.len() != struct_fields.len() {
            return Err(CodegenError::FieldsDontMatch);
        }
        let arg_names: Vec<String> = arguments.iter().map(|(n, _)| n.clone()).collect();
        for f in struct_fields.iter() {
            if !arg_names.contains(&f.name) {
                return Err(CodegenError::FieldsDontMatch);
            }
        }
        let mut seen: HashMap<String, ()> = HashMap::new();
        for n in arg_names.iter() {
            if seen.insert(n.clone(), ()).is_some() {
                return Err(CodegenError::FieldsDontMatch);
            }
        }

        // 2. alloc für die fields
        let struct_ty = IrType::Named(struct_name.clone());
        let struct_addr = self.next_temp_id();
        self.block_handler
            .add_instruction_to_current_block(IrInstruction::Alloca {
                temp_id: struct_addr,
                ty: struct_ty.clone(),
            })?;

        //3.für jedes field value storen
        for (field_name, value_expr) in arguments {
            let field_ty = self.get_struct_field_typ(struct_name, field_name)?;
            let (value_temp, value_ty) = self.gen_expr(value_expr)?;
            if field_ty != value_ty {
                return Err(CodegenError::MismatchedTypes(field_ty, value_ty));
            }

            let field_addr = self.next_temp_id();
            self.block_handler
                .add_instruction_to_current_block(IrInstruction::GetFieldAddr {
                    temp_id: field_addr,
                    base_addr: struct_addr,
                    field_name: field_name.clone(),
                })?;
            self.block_handler
                .add_instruction_to_current_block(IrInstruction::Store {
                    ty: field_ty,
                    value: value_temp,
                    addr: field_addr,
                })?;
        }

        let result_ty = IrType::Pointer(Box::new(struct_ty));
        Ok((struct_addr, result_ty))
    }

    fn get_struct_fields(&self, struct_name: &String) -> Result<Vec<IrField>, CodegenError> {
        let mut found_struct = false;

        let mut typ = None;

        for typedef in self.type_defs.iter() {
            if let IrTypeDefinition::Struct { name, fields } = typedef {
                if name == struct_name {
                    if found_struct {
                        return Err(CodegenError::AmbigousType(struct_name.clone()));
                    } else {
                        found_struct = true;
                        typ = Some(fields.clone());
                    }
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
                if name != struct_name {
                    continue;
                }
                if found_struct {
                    return Err(CodegenError::AmbigousType(struct_name.clone()));
                }
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
        // damit z.B. (f)(args) oder ((obj).method)(args) funktionieren
        if let Expr::Grouping(inner) = callee.deref() {
            return self.gen_call(inner, type_args, arguments);
        }

        // obj.method(args) wird zu Struct_method(obj args)
        if let Expr::FieldAccess { object, field_name } = callee.deref() {
            return self.gen_method_call(object, field_name, arguments);
        }

        // indirekte calls (z.B. getFn()(), arr[0]()) sind nicht supported:
        let function_name = match callee.deref() {
            Expr::Variable(name) => name.clone(),
            other => return Err(CodegenError::InvalidExpr(other.clone())),
        };

        let mut arg_temp_ids = vec![];
        for arg in arguments {
            arg_temp_ids.push(self.gen_expr(arg)?.0);
        }

        let return_type = self
            .functions_return_type
            .get(&function_name)
            .ok_or(CodegenError::UnknownFunction(function_name.clone()))?
            .clone()
            .unwrap_or(IrType::Void);

        let temp_id = self.next_temp_id();
        self.block_handler
            .add_instruction_to_current_block(IrInstruction::Call {
                temp_id: temp_id,
                function_name,
                args: arg_temp_ids,
                return_type: return_type.clone(),
            })?;

        Ok((temp_id, return_type))
    }

    fn gen_method_call_with_temp_id(
        &mut self,
        obj_temp: TempId,
        obj_ty: IrType,
        method_name: &String,
        arguments: &Vec<Expr>,
    ) -> Result<(TempId, IrType), CodegenError> {
        let struct_name = Self::struct_name_from_ty(&obj_ty)
            .ok_or_else(|| CodegenError::UnknownType(format!("{obj_ty:?}")))?;
        let mangled = format!("{}_{}", struct_name, method_name);

        let mut arg_temp_ids = vec![obj_temp];
        for arg in arguments {
            arg_temp_ids.push(self.gen_expr(arg)?.0);
        }

        let return_type = self
            .functions_return_type
            .get(&mangled)
            .ok_or_else(|| CodegenError::UnknownFunction(mangled.clone()))?
            .clone()
            .unwrap_or(IrType::Void);

        let temp_id = self.next_temp_id();
        self.block_handler
            .add_instruction_to_current_block(IrInstruction::Call {
                temp_id: temp_id,
                function_name: mangled,
                args: arg_temp_ids,
                return_type: return_type.clone(),
            })?;

        Ok((temp_id, return_type))
    }
    // ruft eine methode auf einem objekt auf
    // das objekt wird automatisch als erstes argument vor die user args gepackt

    fn gen_method_call(
        &mut self,
        object: &Box<Expr>,
        method_name: &String,
        arguments: &Vec<Expr>,
    ) -> Result<(TempId, IrType), CodegenError> {
        let (obj_temp, obj_ty) = self.gen_expr(object)?;
        let struct_name = Self::struct_name_from_ty(&obj_ty)
            .ok_or_else(|| CodegenError::UnknownType(format!("{obj_ty:?}")))?;
        let mangled = format!("{}_{}", struct_name, method_name);

        let mut arg_temp_ids = vec![obj_temp];
        for arg in arguments {
            arg_temp_ids.push(self.gen_expr(arg)?.0);
        }

        let return_type = self
            .functions_return_type
            .get(&mangled)
            .ok_or_else(|| CodegenError::UnknownFunction(mangled.clone()))?
            .clone()
            .unwrap_or(IrType::Void);

        let temp_id = self.next_temp_id();
        self.block_handler
            .add_instruction_to_current_block(IrInstruction::Call {
                temp_id: temp_id,
                function_name: mangled,
                args: arg_temp_ids,
                return_type: return_type.clone(),
            })?;

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
            })?;

        Ok((temp_id, IrType::Char))
    }

    fn gen_assign(
        &mut self,
        target: &Box<Expr>,
        value: &Box<Expr>,
    ) -> Result<(TempId, IrType), CodegenError> {
        let (addr, target_type) = self.gen_left_value_addr(target)?;

        let (temp_value_var, value_type) = self.gen_expr(value)?;

        if target_type != value_type {
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

        //mangle maige
        // -x wird zu Struct_-(x) !x zu Struct_!(x) ~x zu Struct_~(x)
        if let Some(struct_name) = Self::struct_name_from_ty(&ir_type) {
            let op_str = Self::unary_op_to_str(unary_op).to_string();
            let key = (struct_name, op_str);
            if let Some((mangled_name, return_ty)) =
                self.unary_operator_functions.get(&key).cloned()
            {
                let temp_id = self.next_temp_id();
                self.block_handler
                    .add_instruction_to_current_block(IrInstruction::Call {
                        temp_id,
                        function_name: mangled_name,
                        args: vec![tmp_id_1],
                        return_type: return_ty.clone(),
                    })?;
                return Ok((temp_id, return_ty));
            }
        }

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

    fn unary_op_to_str(op: &UnaryOp) -> &'static str {
        match op {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
            UnaryOp::BitNot => "~",
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

        //erst linke seit probieren dann rechte Seite
        let op_str = Self::binary_op_to_str(binary_op).to_string();

        if let Some(struct_name) = Self::struct_name_from_ty(&ir_type_1) {
            let key = (struct_name, op_str.clone());
            if let Some((mangled_name, return_ty)) = self.operator_functions.get(&key).cloned() {
                let temp_id = self.next_temp_id();
                self.block_handler
                    .add_instruction_to_current_block(IrInstruction::Call {
                        temp_id: temp_id,
                        function_name: mangled_name,
                        args: vec![temp_id_1, temp_id_2],
                        return_type: return_ty.clone(),
                    })?;
                return Ok((temp_id, return_ty));
            }
        }

        if let Some(struct_name) = Self::struct_name_from_ty(&ir_type_2) {
            let key = (struct_name, op_str);
            if let Some((mangled_name, return_ty)) = self.operator_functions.get(&key).cloned() {
                let temp_id = self.next_temp_id();
                self.block_handler
                    .add_instruction_to_current_block(IrInstruction::Call {
                        temp_id: temp_id,
                        function_name: mangled_name,
                        args: vec![temp_id_2, temp_id_1],
                        return_type: return_ty.clone(),
                    })?;
                return Ok((temp_id, return_ty));
            }
        }

        //ist nicht struct _> dh. primitive
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

    fn binary_op_to_str(op: &BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::Shl => "<<",
            BinaryOp::Shr => ">>",
        }
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
        let const_data = self
            .module
            .constants
            .iter()
            .find(|c| c.name == name)
            .map(|c| (c.ty.clone(), c.value.clone()));
        let global_data = self
            .module
            .globals
            .iter()
            .find(|g| g.name == name)
            .map(|g| (g.name.clone(), g.ty.clone()));
        let looked_up_var = self.lookup_variable(name);

        if let Some((ty, value)) = const_data {
            let temp_id = self.next_temp_id();
            self.block_handler
                .add_instruction_to_current_block(IrInstruction::PrimitiveConst {
                    temp_id,
                    ty: ty.clone(),
                    value,
                })?;
            return Ok((temp_id, ty));
        }

        if let Some((g_name, ty)) = global_data {
            let addr_temp = self.next_temp_id();
            self.block_handler
                .add_instruction_to_current_block(IrInstruction::GlobalAddr {
                    temp_id: addr_temp,
                    name: g_name,
                    ty: ty.clone(),
                })?;
            let val_temp = self.next_temp_id();
            self.block_handler
                .add_instruction_to_current_block(IrInstruction::Load {
                    temp_id: val_temp,
                    ty: ty.clone(),
                    addr: addr_temp,
                })?;
            return Ok((val_temp, ty));
        }

        if let Some(var) = looked_up_var {
            let ty = var.ty.clone();
            let addr = var.addr;
            let freie_temp_var = self.next_temp_id();
            self.block_handler
                .add_instruction_to_current_block(IrInstruction::Load {
                    temp_id: freie_temp_var,
                    ty: ty.clone(),
                    addr,
                })?;
            return Ok((freie_temp_var, ty));
        }

        Err(CodegenError::UnknownVariable(name.to_string()))
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

    fn gen_null_literal(&mut self) -> Result<(TempId, IrType), CodegenError> {
        let temp_id = self.next_temp_id();
        let ty = IrType::Pointer(Box::new(IrType::Void));

        let new_const_instruction = IrInstruction::PrimitiveConst {
            temp_id,
            ty: ty.clone(),
            value: PrimitiveValue::Null,
        };

        self.block_handler
            .add_instruction_to_current_block(new_const_instruction)?;

        Ok((temp_id, ty))
    }
}
