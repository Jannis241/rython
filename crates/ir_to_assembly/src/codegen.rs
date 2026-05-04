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
    fn gen_asm(input: IrModule) -> String {
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
    fn generate_function(&mut self, function: IrFunction) {}
    fn generate_globals(&mut self) {}
    fn generate_constants(&mut self) {}
}
