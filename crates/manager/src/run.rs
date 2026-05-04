use crate::read_file;
use rython_to_ir::{self, codegen};

pub fn run(file_name: &str) {
    // Todo: checken ob das file ein .ry file ist und die errors richtig handeln / unwrap weg machen
    let content = read_file::read_file(file_name).unwrap();
    let tokens = rython_to_ir::lexer::Lexer::create_tokens(content).unwrap();

    dbg!(&tokens);

    let mut parser = rython_to_ir::parser::Parser::new(tokens);
    let ast = parser.parse();
    dbg!(&ast);
    let module = codegen::generate_code(&ast.unwrap());

    dbg!(module);
}
