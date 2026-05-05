use crate::read_file;
use ir_to_assembly::codegen as asmcodegen;
use rython_to_ir::{self, codegen};

use std::fs::write;
use std::process::Command;

pub fn run(file_name: &str) {
    // Todo: checken ob das file ein .ry file ist und die errors richtig handeln / unwrap weg machen
    let content = read_file::read_file(file_name).unwrap();
    let tokens = rython_to_ir::lexer::Lexer::create_tokens(content).unwrap();
    dbg!(&tokens);

    let mut parser = rython_to_ir::parser::Parser::new(tokens);
    let ast = parser.parse();
    dbg!(&ast);

    let module = codegen::generate_code(&ast.unwrap());
    dbg!(&module);

    let asm = asmcodegen::AsmCodeGen::gen_asm(module.unwrap()).unwrap();
    println!("{}", asm);

    compile_and_run(asm.as_str(), file_name).unwrap();
}

fn compile_and_run(asm: &str, file_name: &str) -> std::io::Result<()> {
    let base = file_name.strip_suffix(".ry").unwrap();

    let asm_file = format!("{base}.asm");
    let obj_file = format!("{base}.o");
    let out_file = format!("{base}out");

    std::fs::write(&asm_file, asm)?;

    let nasm = Command::new("nasm")
        .args(["-felf64", &asm_file, "-o", &obj_file])
        .output()?;

    if !nasm.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&nasm.stderr));
    }
    let ld_status = Command::new("ld")
        .args([&obj_file, "-o", &out_file])
        .status()?;

    if !ld_status.success() {
        panic!("ld failed");
    }

    let output = Command::new(format!("{out_file}")).output()?;

    println!(
        "Program output: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let exit_code = output.status.code();

    println!("Exit code: {:?}", exit_code);

    Ok(())
}
