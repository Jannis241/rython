use manager;
use std::env;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        println!("usage:\ncargo run your_program.ry");
        return;
    }

    let filename = args[1].as_str();

    manager::run::run(filename);
}
