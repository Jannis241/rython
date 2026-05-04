use rython_to_ir::codegen::*;
use std::fmt::Write;

macro_rules! emit {
    ($self:expr, $($arg:tt)*) => {
        write!($self.out, $($arg)*).unwrap();
    };
}

pub struct AsmCodeGen {
    out: String,
    input: IrModule,
}

impl AsmCodeGen {
    pub fn gen_asm(input: IrModule) -> String {
        let mut asm_code_gen = AsmCodeGen {
            out: String::new(),
            input,
        };

        asm_code_gen.generate_globals();
        asm_code_gen.generate_constants();
        asm_code_gen.generate_functions();

        asm_code_gen.out
    }

    fn generate_functions(&mut self) {
        for function in self.input.functions.clone() {
            self.generate_function(function);
        }
    }
    fn generate_function(&mut self, _function: IrFunction) {}
    fn generate_globals(&mut self) {
        for global in self.input.globals.clone() {
            emit!(self, "section .data\n");
            emit!(self, "{}: dq {}\n", global.name, const_value_to_asm(global.value));
        }
    }
    fn generate_constants(&mut self) {
        for constant in self.input.constants.clone() {
            emit!(self, "section .rodata\n");
            emit!(self, "{}: dq {}\n", constant.name, const_value_to_asm(constant.value));
        }
    }
}

pub fn get_type_size_in_qds(typ: IrType) -> usize {
    match typ {
        IrType::I64 => 1,
        IrType::F64 => 1,
        IrType::Bool => 1,
        IrType::Void => 0,
        _ => todo!(),
    }
}

fn const_value_to_asm(value: ConstValue) -> String {
    match value {
        ConstValue::Int(value) => value.to_string(),
        ConstValue::Bool(value) => usize::from(value).to_string(),
        ConstValue::Null => "0".to_string(),
        other => todo!("constant value {other:?} is not supported by asm codegen"),
    }
}
