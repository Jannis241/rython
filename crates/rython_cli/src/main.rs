use manager;
use rython_to_ir::ast::*;
use rython_to_ir::{
    ast::Item,
    codegen::{self, IrGenerator},
};
use std::env;

fn main() {
    // let args = env::args().collect::<Vec<String>>();
    //
    // if args.len() != 2 {
    //     println!("usage:\ncargo run your_program.ry");
    //     return;
    // }
    //
    // let filename = args[1].as_str();
    //
    // manager::run::run(filename);
    let items = vec![Item::Function(Function {
        name: "main".to_string(),
        generic_params: vec![],
        params: vec![],
        return_type: Some(Type::Named("i64".to_string())),
        body: Block {
            statements: vec![Stmt::Return(Return {
                return_value: Some(Expr::IntLiteral("42".to_string())),
            })],
        },
    })];
    let ir = codegen::generate_code(&items);
    println!("Generated IR Code: \n \n \n {}", ir);
}
