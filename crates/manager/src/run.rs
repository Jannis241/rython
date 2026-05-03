use crate::read_file;
use rython_to_ir;

pub fn run(file_name: &str) {
    let content = read_file::read_file(file_name).unwrap();
    let tokens = rython_to_ir::lexer::Lexer::create_tokens(content);
    let mut parser = rython_to_ir::parser::Parser::new(tokens);
    let ast = parser.parse();
    dbg!(ast);
}
