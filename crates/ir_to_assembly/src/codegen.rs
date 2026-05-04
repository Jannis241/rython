use rython_to_ir::codegen::*;
use std::fmt::Write;

macro_rules! emit {
    ($self:expr, $($arg:tt)*) => {
        write!($self.out, $($arg)*).unwrap();
    };
}

pub struct AsmCodeGen {
    out: String,
}

impl AsmCodeGen {
    fn gen_asm(input: IrModule) -> String {
        let mut asm_code_gen = AsmCodeGen { out: String::new() };

        asm_code_gen.generate_globals();
        asm_code_gen.generate_constants();
        asm_code_gen.generate_fucntions();

        asm_code_gen.out
    }

    fn generate_fucntions(&mut self) {
        emit!(self, "label:");
    }
    fn generate_globals(&mut self) {}
    fn generate_constants(&mut self) {}
}
