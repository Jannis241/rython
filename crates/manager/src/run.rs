use crate::read_file;
use ir_to_assembly::codegen as asmcodegen;
use ir_to_assembly::codegen::AsmCodeGenErr;
use rython_to_ir::codegen::CodegenError;
use rython_to_ir::lexer::LexingError;
use rython_to_ir::parser::ParseError;
use rython_to_ir::{codegen, lexer, parser};

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct BuildOptions {
    pub keep_intermediates: bool,
    pub release: bool,
    pub run_after_build: bool,
    pub output_path: Option<PathBuf>,
    pub emit_tokens: bool,
    pub emit_ast: bool,
    pub emit_ir: bool,
    pub emit_asm: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        BuildOptions {
            keep_intermediates: false,
            release: false,
            run_after_build: true,
            output_path: None,
            emit_tokens: false,
            emit_ast: false,
            emit_ir: false,
            emit_asm: false,
        }
    }
}

#[derive(Debug)]
pub enum BuildError {
    InvalidExtension {
        path: PathBuf,
    },
    Read {
        path: PathBuf,
        source: io::Error,
    },
    CreateBuildDir {
        path: PathBuf,
        source: io::Error,
    },
    WriteAsm {
        path: PathBuf,
        source: io::Error,
    },
    Lex(LexingError),
    Parse(ParseError),
    IrCodegen(CodegenError),
    AsmCodegen(AsmCodeGenErr),
    NasmSpawn(io::Error),
    NasmFailed {
        stderr: String,
    },
    LdSpawn(io::Error),
    LdFailed {
        stderr: String,
    },
    BinarySpawn {
        path: PathBuf,
        source: io::Error,
    },
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::InvalidExtension { path } => {
                write!(f, "[input] {}: expected .ry file", path.display())
            }
            BuildError::Read { path, source } => {
                write!(f, "[io] {}: {source}", path.display())
            }
            BuildError::CreateBuildDir { path, source } => {
                write!(f, "[io] could not create {}: {source}", path.display())
            }
            BuildError::WriteAsm { path, source } => {
                write!(f, "[io] could not write {}: {source}", path.display())
            }
            BuildError::Lex(e) => write!(f, "[lexer] {e:?}"),
            BuildError::Parse(e) => write!(f, "[parser] {e:?}"),
            BuildError::IrCodegen(e) => write!(f, "[ir] {e:?}"),
            BuildError::AsmCodegen(e) => write!(f, "[asm] {e:?}"),
            BuildError::NasmSpawn(e) => write!(f, "[nasm] could not spawn nasm: {e}"),
            BuildError::NasmFailed { stderr } => write!(f, "[nasm] failed:\n{stderr}"),
            BuildError::LdSpawn(e) => write!(f, "[ld] could not spawn ld: {e}"),
            BuildError::LdFailed { stderr } => write!(f, "[ld] failed:\n{stderr}"),
            BuildError::BinarySpawn { path, source } => {
                write!(f, "[run] could not run {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for BuildError {}

/// Compile the given .ry file end-to-end and (optionally) run it.
///
/// Returns the resulting program's exit code, or 0 if the program was not run.
pub fn run(file_name: &str, options: &BuildOptions) -> Result<i32, BuildError> {
    let _ = options.release; // accepted, no optimisation pass yet

    let input_path = PathBuf::from(file_name);
    if input_path.extension().and_then(|s| s.to_str()) != Some("ry") {
        return Err(BuildError::InvalidExtension { path: input_path });
    }

    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("rython_program")
        .to_string();

    let content = read_file::read_file(file_name).map_err(|e| BuildError::Read {
        path: input_path.clone(),
        source: e,
    })?;

    let tokens = lexer::Lexer::create_tokens(content).map_err(BuildError::Lex)?;
    if options.emit_tokens {
        eprintln!("[tokens] {tokens:#?}");
    }

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse().map_err(BuildError::Parse)?;
    if options.emit_ast {
        eprintln!("[ast] {ast:#?}");
    }

    let module = codegen::generate_code(&ast).map_err(BuildError::IrCodegen)?;
    if options.emit_ir {
        eprintln!("[ir] {module:#?}");
    }

    let asm = asmcodegen::AsmCodeGen::gen_asm(module).map_err(BuildError::AsmCodegen)?;
    if options.emit_asm {
        eprintln!("[asm]\n{asm}");
    }

    let build_dir = PathBuf::from("target").join("rython");
    fs::create_dir_all(&build_dir).map_err(|e| BuildError::CreateBuildDir {
        path: build_dir.clone(),
        source: e,
    })?;

    let asm_path = build_dir.join(format!("{stem}.asm"));
    let obj_path = build_dir.join(format!("{stem}.o"));
    let bin_path = options
        .output_path
        .clone()
        .unwrap_or_else(|| build_dir.join(&stem));

    fs::write(&asm_path, &asm).map_err(|e| BuildError::WriteAsm {
        path: asm_path.clone(),
        source: e,
    })?;

    assemble(&asm_path, &obj_path)?;
    link(&obj_path, &bin_path)?;

    if !options.keep_intermediates {
        let _ = fs::remove_file(&asm_path);
        let _ = fs::remove_file(&obj_path);
    }

    if !options.run_after_build {
        return Ok(0);
    }

    execute(&bin_path)
}

fn assemble(asm_path: &Path, obj_path: &Path) -> Result<(), BuildError> {
    let nasm = Command::new("nasm")
        .args([
            "-felf64",
            asm_path.to_str().unwrap_or_default(),
            "-o",
            obj_path.to_str().unwrap_or_default(),
        ])
        .output()
        .map_err(BuildError::NasmSpawn)?;

    if !nasm.status.success() {
        return Err(BuildError::NasmFailed {
            stderr: String::from_utf8_lossy(&nasm.stderr).to_string(),
        });
    }
    Ok(())
}

fn link(obj_path: &Path, bin_path: &Path) -> Result<(), BuildError> {
    let ld = Command::new("ld")
        .args([
            obj_path.to_str().unwrap_or_default(),
            "-o",
            bin_path.to_str().unwrap_or_default(),
        ])
        .output()
        .map_err(BuildError::LdSpawn)?;

    if !ld.status.success() {
        return Err(BuildError::LdFailed {
            stderr: String::from_utf8_lossy(&ld.stderr).to_string(),
        });
    }
    Ok(())
}

fn execute(bin_path: &Path) -> Result<i32, BuildError> {
    let output = Command::new(bin_path)
        .output()
        .map_err(|e| BuildError::BinarySpawn {
            path: bin_path.to_path_buf(),
            source: e,
        })?;

    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));

    Ok(output.status.code().unwrap_or(1))
}
