use crate::read_file;
use rython_to_ir::{self, codegen};

pub fn run(file_name: &str) {
    let content = read_file::read_file(file_name).unwrap();
    let tokens = rython_to_ir::lexer::Lexer::create_tokens(content);
    dbg!(&tokens);
    let mut parser = rython_to_ir::parser::Parser::new(tokens);
    let ast = parser.parse();
    let _ = dbg!(&ast);
    match ast {
        Ok(tree) => {
            let module = codegen::generate_code(&tree);

            println!("Generated IR Module: {:?}", module);

        }
        Err(e) => {
            eprintln!("Parsing Error: {:?}", e);
            return;
        }
    }
}
