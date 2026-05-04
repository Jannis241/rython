use std::env;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        println!("usage:\ncargo run --bin rython_cli <your_program.ry>");
        return;
    }

    manager::run::run(&args[1]);
}
