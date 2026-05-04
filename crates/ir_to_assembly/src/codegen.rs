use rython_to_ir::codegen::*;
use std::{collections::HashMap, fmt::Write};

#[derive(Debug)]
pub enum AsmCodeGenErr {
    TypeNotFound(String),
    MultipleTypesFound(String),
    UnsupportedConstValue(ConstValue),
    UnsupportedInstruction(String),
}

macro_rules! emit {
    ($self:expr, $($arg:tt)*) => {
        write!($self.out, $($arg)*).unwrap()
    };
}

pub struct AsmCodeGen {
    out: String,
    input: IrModule,
    alloca_offsets: HashMap<usize, usize>,
}

impl AsmCodeGen {
    pub fn gen_asm(input: IrModule) -> Result<String, AsmCodeGenErr> {
        let mut g = AsmCodeGen {
            out: String::new(),
            input,
            alloca_offsets: HashMap::new(),
        };

        g.generate_data_section()?;
        g.generate_top_level_asm()?;
        g.generate_functions()?;

        Ok(g.out)
    }

    fn generate_data_section(&mut self) -> Result<(), AsmCodeGenErr> {
        if self.input.globals.is_empty() && self.input.constants.is_empty() {
            return Ok(());
        }

        emit!(self, "section .data\n");

        for global in self.input.globals.clone() {
            emit!(self, "{}:\n", global.name);
            emit!(
                self,
                "    dq {}\n",
                self.const_value_into_string(global.value)?
            );
        }
        for constant in self.input.constants.clone() {
            emit!(self, "{}:\n", constant.name);
            emit!(
                self,
                "    dq {}\n",
                self.const_value_into_string(constant.value)?
            );
        }

        emit!(self, "\n");
        Ok(())
    }

    fn generate_top_level_asm(&mut self) -> Result<(), AsmCodeGenErr> {
        for code in self.input.inline_assembly.clone() {
            emit!(self, "    {}\n", code);
        }
        Ok(())
    }

    fn generate_functions(&mut self) -> Result<(), AsmCodeGenErr> {
        emit!(self, "section .text\n");
        emit!(self, "global _start\n\n");

        emit!(self, "_start:\n");
        emit!(self, "    call main\n");
        emit!(self, "    mov rdi, rax\n");
        emit!(self, "    mov rax, 60\n");
        emit!(self, "    syscall\n\n");

        for function in self.input.functions.clone() {
            self.generate_function(function)?;
        }

        Ok(())
    }

    fn generate_function(&mut self, function: IrFunction) -> Result<(), AsmCodeGenErr> {
        self.alloca_offsets.clear();

        let mut max_temp: i64 = -1;
        for block in &function.blocks {
            for instr in &block.instructions {
                if let Some(t) = result_temp(instr) {
                    if (t.0 as i64) > max_temp {
                        max_temp = t.0 as i64;
                    }
                }
            }
        }
        let n_temps = if max_temp < 0 {
            0
        } else {
            (max_temp + 1) as usize
        };
        let temp_area = n_temps * 8;

        let mut alloca_running = temp_area;
        for block in &function.blocks {
            for instr in &block.instructions {
                if let IrInstruction::Alloca { temp_id, ty } = instr {
                    let qds = self.get_type_size_in_qds(ty.clone())?;
                    let bytes = if qds == 0 { 8 } else { qds * 8 };
                    alloca_running += bytes;
                    self.alloca_offsets.insert(temp_id.0, alloca_running);
                }
            }
        }

        let mut frame_size = alloca_running;
        if frame_size % 16 != 0 {
            frame_size += 16 - (frame_size % 16);
        }

        emit!(self, "{}:\n", function.name);
        emit!(self, "    push rbp\n");
        emit!(self, "    mov rbp, rsp\n");
        if frame_size > 0 {
            emit!(self, "    sub rsp, {}\n", frame_size);
        }

        // emit!(self, "{}:\n", function.name);
        for block in function.blocks {
            self.generate_block(block)?;
        }

        emit!(self, "\n");
        Ok(())
    }

    fn generate_block(&mut self, block: IrBlock) -> Result<(), AsmCodeGenErr> {
        let label = strip_trailing_colon(&block.label);

        emit!(self, ".{}:\n", label);

        for instruction in block.instructions {
            self.generate_instruction(instruction)?;
        }

        match block.terminator {
            Terminator::Ret(Some(temp_id)) => {
                emit!(self, "    mov rax, {}\n", self.temp_loc(temp_id));
                emit!(self, "    mov rsp, rbp\n");
                emit!(self, "    pop rbp\n");
                emit!(self, "    ret\n");
            }
            Terminator::Ret(None) => {
                emit!(self, "    xor rax, rax\n");
                emit!(self, "    mov rsp, rbp\n");
                emit!(self, "    pop rbp\n");
                emit!(self, "    ret\n");
            }
            Terminator::Jump { target } => {
                let t = strip_trailing_colon(&target);
                emit!(self, "    jmp .{}\n", t);
            }
            Terminator::Branch {
                condition,
                then_block,
                else_block,
            } => {
                let then_lbl = strip_trailing_colon(&then_block);
                let else_lbl = strip_trailing_colon(&else_block);
                emit!(self, "    mov rax, {}\n", self.temp_loc(condition));
                emit!(self, "    cmp rax, 0\n");
                emit!(self, "    je .{}\n", else_lbl);
                emit!(self, "    jmp .{}\n", then_lbl);
            }
        }

        Ok(())
    }

    fn generate_instruction(&mut self, instruction: IrInstruction) -> Result<(), AsmCodeGenErr> {
        match instruction {
            IrInstruction::Const {
                temp_id,
                ty: _,
                value,
            } => {
                let imm = match value {
                    ConstValue::Int(i) => i as u64,
                    ConstValue::Bool(b) => {
                        if b {
                            1
                        } else {
                            0
                        }
                    }
                    ConstValue::Null => 0,
                    ConstValue::Char(c) => c as u64,
                    ConstValue::Float(f) => f.to_bits(),
                    other => return Err(AsmCodeGenErr::UnsupportedConstValue(other)),
                };
                emit!(self, "    mov rax, {}\n", imm);
                emit!(self, "    mov {}, rax\n", self.temp_loc(temp_id));
            }
            IrInstruction::Alloca { temp_id, ty: _ } => {
                let off = *self
                    .alloca_offsets
                    .get(&temp_id.0)
                    .expect("alloca offset must exist");
                emit!(self, "    lea rax, [rbp - {}]\n", off);
                emit!(self, "    mov {}, rax\n", self.temp_loc(temp_id));
            }
            IrInstruction::Load {
                temp_id,
                ty: _,
                addr,
            } => {
                emit!(self, "    mov rax, {}\n", self.temp_loc(addr));
                emit!(self, "    mov rcx, [rax]\n");
                emit!(self, "    mov {}, rcx\n", self.temp_loc(temp_id));
            }
            IrInstruction::Store { ty: _, value, addr } => {
                emit!(self, "    mov rax, {}\n", self.temp_loc(addr));
                emit!(self, "    mov rcx, {}\n", self.temp_loc(value));
                emit!(self, "    mov [rax], rcx\n");
            }
            IrInstruction::Binary {
                temp_id,
                ty: _,
                op,
                lhs,
                rhs,
            } => {
                emit!(self, "    mov rax, {}\n", self.temp_loc(lhs));
                emit!(self, "    mov rcx, {}\n", self.temp_loc(rhs));
                match op {
                    IrBinaryOp::Add => emit!(self, "    add rax, rcx\n"),
                    IrBinaryOp::Sub => emit!(self, "    sub rax, rcx\n"),
                    IrBinaryOp::Mul => emit!(self, "    imul rax, rcx\n"),
                    IrBinaryOp::Div => {
                        emit!(self, "    cqo\n");
                        emit!(self, "    idiv rcx\n");
                    }
                    IrBinaryOp::Mod => {
                        emit!(self, "    cqo\n");
                        emit!(self, "    idiv rcx\n");
                        emit!(self, "    mov rax, rdx\n");
                    }
                    IrBinaryOp::Eq => {
                        emit!(self, "    cmp rax, rcx\n");
                        emit!(self, "    sete al\n");
                        emit!(self, "    movzx rax, al\n");
                    }
                    IrBinaryOp::Ne => {
                        emit!(self, "    cmp rax, rcx\n");
                        emit!(self, "    setne al\n");
                        emit!(self, "    movzx rax, al\n");
                    }
                    IrBinaryOp::Lt => {
                        emit!(self, "    cmp rax, rcx\n");
                        emit!(self, "    setl al\n");
                        emit!(self, "    movzx rax, al\n");
                    }
                    IrBinaryOp::Le => {
                        emit!(self, "    cmp rax, rcx\n");
                        emit!(self, "    setle al\n");
                        emit!(self, "    movzx rax, al\n");
                    }
                    IrBinaryOp::Gt => {
                        emit!(self, "    cmp rax, rcx\n");
                        emit!(self, "    setg al\n");
                        emit!(self, "    movzx rax, al\n");
                    }
                    IrBinaryOp::Ge => {
                        emit!(self, "    cmp rax, rcx\n");
                        emit!(self, "    setge al\n");
                        emit!(self, "    movzx rax, al\n");
                    }
                    IrBinaryOp::And | IrBinaryOp::BitAnd => emit!(self, "    and rax, rcx\n"),
                    IrBinaryOp::Or | IrBinaryOp::BitOr => emit!(self, "    or rax, rcx\n"),
                    IrBinaryOp::BitXor => emit!(self, "    xor rax, rcx\n"),
                    IrBinaryOp::Shl => emit!(self, "    shl rax, cl\n"),
                    IrBinaryOp::Shr => emit!(self, "    sar rax, cl\n"),
                }
                emit!(self, "    mov {}, rax\n", self.temp_loc(temp_id));
            }
            IrInstruction::Unary {
                temp_id,
                ty: _,
                op,
                value,
            } => {
                emit!(self, "    mov rax, {}\n", self.temp_loc(value));
                match op {
                    IrUnaryOp::Neg => emit!(self, "    neg rax\n"),
                    IrUnaryOp::Not => emit!(self, "    xor rax, 1\n"),
                    IrUnaryOp::BitNot => emit!(self, "    not rax\n"),
                }
                emit!(self, "    mov {}, rax\n", self.temp_loc(temp_id));
            }
            IrInstruction::Call {
                temp_id,
                function_name,
                args,
                return_type: _,
            } => {
                for arg in args.iter().rev() {
                    emit!(self, "    push qword {}\n", self.temp_loc(*arg));
                }
                emit!(self, "    call {}\n", function_name);
                if !args.is_empty() {
                    emit!(self, "    add rsp, {}\n", args.len() * 8);
                }
                if let Some(t) = temp_id {
                    emit!(self, "    mov {}, rax\n", self.temp_loc(t));
                }
            }
            IrInstruction::GlobalAddr {
                temp_id,
                name,
                ty: _,
            } => {
                emit!(self, "    mov rax, {}\n", name);
                emit!(self, "    mov {}, rax\n", self.temp_loc(temp_id));
            }
            IrInstruction::Asm { code } => {
                emit!(self, "    {}\n", code);
            }
            IrInstruction::InitArray { .. } => {
                return Err(AsmCodeGenErr::UnsupportedInstruction("InitArray".into()));
            }
            IrInstruction::InitStruct { .. } => {
                return Err(AsmCodeGenErr::UnsupportedInstruction("InitStruct".into()));
            }
            IrInstruction::InitVariant { .. } => {
                return Err(AsmCodeGenErr::UnsupportedInstruction("InitVariant".into()));
            }
            IrInstruction::GetFieldAddr { .. } => {
                return Err(AsmCodeGenErr::UnsupportedInstruction("GetFieldAddr".into()));
            }
            IrInstruction::GetElementAddr { .. } => {
                return Err(AsmCodeGenErr::UnsupportedInstruction(
                    "GetElementAddr".into(),
                ));
            }
        }

        Ok(())
    }

    fn temp_loc(&self, temp_id: TempId) -> String {
        format!("[rbp - {}]", (temp_id.0 + 1) * 8)
    }

    pub fn const_value_into_string(&self, value: ConstValue) -> Result<String, AsmCodeGenErr> {
        match value {
            ConstValue::Int(i) => Ok(i.to_string()),
            ConstValue::Bool(val) => Ok(if val {
                "1".to_string()
            } else {
                "0".to_string()
            }),
            ConstValue::Null => Ok("0".to_string()),
            ConstValue::Char(c) => Ok((c as u32).to_string()),
            ConstValue::Float(f) => Ok(f.to_bits().to_string()),
            ConstValue::String(_) | ConstValue::Struct { .. } | ConstValue::Variant { .. } => {
                Err(AsmCodeGenErr::UnsupportedConstValue(value))
            }
        }
    }

    pub fn get_type_size_in_qds(&self, typ: IrType) -> Result<usize, AsmCodeGenErr> {
        match typ {
            IrType::I64 => Ok(1),
            IrType::F64 => Ok(1),
            IrType::Bool => Ok(1),
            IrType::Void => Ok(0),
            IrType::Pointer(_) => Ok(1),
            IrType::Named(name) => self.get_named_size_in_qds(name),
        }
    }

    pub fn get_named_size_in_qds(&self, named_typ_name: String) -> Result<usize, AsmCodeGenErr> {
        let types: Vec<IrTypeDef> = self.input.types.clone();

        let mut matching_type = None;

        for typ in types {
            match typ {
                IrTypeDef::Struct { name, fields } => {
                    if named_typ_name == name {
                        if matching_type.is_none() {
                            matching_type = Some(IrTypeDef::Struct { name, fields });
                        } else {
                            return Err(AsmCodeGenErr::MultipleTypesFound(named_typ_name));
                        }
                    }
                }
                IrTypeDef::Variant { name, cases } => {
                    if named_typ_name == name {
                        if matching_type.is_none() {
                            matching_type = Some(IrTypeDef::Variant { name, cases });
                        } else {
                            return Err(AsmCodeGenErr::MultipleTypesFound(named_typ_name));
                        }
                    }
                }
            }
        }

        let Some(matching_type_def) = matching_type else {
            return Err(AsmCodeGenErr::TypeNotFound(named_typ_name));
        };

        let qds = match matching_type_def {
            IrTypeDef::Struct { name: _, fields } => {
                let mut total_qds = 0;
                for field in fields {
                    total_qds += self.get_type_size_in_qds(field.ty)?;
                }
                total_qds
            }
            IrTypeDef::Variant { .. } => 1,
        };

        Ok(qds)
    }
}

fn strip_trailing_colon(s: &str) -> String {
    s.strip_suffix(':').unwrap_or(s).to_string()
}

fn result_temp(instr: &IrInstruction) -> Option<TempId> {
    match instr {
        IrInstruction::Const { temp_id, .. }
        | IrInstruction::Alloca { temp_id, .. }
        | IrInstruction::Load { temp_id, .. }
        | IrInstruction::Binary { temp_id, .. }
        | IrInstruction::Unary { temp_id, .. }
        | IrInstruction::GlobalAddr { temp_id, .. }
        | IrInstruction::InitArray { temp_id, .. }
        | IrInstruction::InitStruct { temp_id, .. }
        | IrInstruction::InitVariant { temp_id, .. }
        | IrInstruction::GetFieldAddr { temp_id, .. }
        | IrInstruction::GetElementAddr { temp_id, .. } => Some(*temp_id),
        IrInstruction::Call { temp_id, .. } => *temp_id,
        IrInstruction::Store { .. } | IrInstruction::Asm { .. } => None,
    }
}
