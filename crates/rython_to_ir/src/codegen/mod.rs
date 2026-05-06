mod error;
mod expr;
mod generator;
mod scope;
mod stmt;

pub use crate::ir::*;
pub use error::CodegenError;
pub use generator::{generate_code, IrGenerator};
pub use scope::{Scope, Variable};
