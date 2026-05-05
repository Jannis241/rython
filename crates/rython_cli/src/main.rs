use std::path::PathBuf;
use std::process::ExitCode;

use manager::run::{BuildOptions, run};

const USAGE: &str = "\
usage: rython_cli [OPTIONS] <your_program.ry>

OPTIONS:
    -o <path>          Set output binary path
    --emit-tokens      Print lexer tokens to stderr
    --emit-ast         Print parser AST to stderr
    --emit-ir          Print IR module to stderr
    --emit-asm         Print generated assembly to stderr
    --keep             Keep intermediate .asm and .o files
    --release          Build with optimisations (no-op for now)
    --no-run           Build only, do not execute
    -h, --help         Print this help
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }

    let mut options = BuildOptions::default();
    let mut input_file: Option<String> = None;
    let mut iter = args.iter().skip(1);

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            "--emit-tokens" => options.emit_tokens = true,
            "--emit-ast" => options.emit_ast = true,
            "--emit-ir" => options.emit_ir = true,
            "--emit-asm" => options.emit_asm = true,
            "--keep" => options.keep_intermediates = true,
            "--release" => options.release = true,
            "--no-run" => options.run_after_build = false,
            "-o" => {
                let Some(path) = iter.next() else {
                    eprintln!("error: -o requires an argument");
                    return ExitCode::from(2);
                };
                options.output_path = Some(PathBuf::from(path));
            }
            other if other.starts_with('-') => {
                eprintln!("error: unknown option {other}");
                return ExitCode::from(2);
            }
            other => {
                if input_file.is_some() {
                    eprintln!("error: multiple input files are not supported");
                    return ExitCode::from(2);
                }
                input_file = Some(other.to_string());
            }
        }
    }

    let Some(input) = input_file else {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    };

    match run(&input, &options) {
        Ok(code) => {
            if (0..=255).contains(&code) {
                println!("exit code: {code}");
                ExitCode::from(code as u8)
            } else {
                println!("exit code: 1");
                ExitCode::from(1)
            }
        }
        Err(err) => {
            eprintln!("{err}");
            println!("exit code: 1");
            ExitCode::from(1)

        }
    }
}
