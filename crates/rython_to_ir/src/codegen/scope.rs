use std::collections::HashMap;

use crate::ir::{IrType, TempId};

use super::error::CodegenError;
use super::generator::IrGenerator;

#[derive(Debug, Clone)]
pub struct Variable {
    pub(super) name: String,
    pub(super) ty: IrType,
    pub(super) addr: TempId,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub(super) symbols: HashMap<String, Variable>, // name, variable
}

impl IrGenerator {
    pub(super) fn enter_scope(&mut self) {
        self.scopes.push(Scope {
            symbols: HashMap::new(),
        });
    }

    pub(super) fn exit_scope(&mut self) {
        self.scopes.pop().expect("no active scope");
    }

    pub(super) fn insert_variable(
        &mut self,
        name: String,
        ty: IrType,
        addr: TempId,
    ) -> Result<(), CodegenError> {
        self.scopes
            .last_mut()
            .expect("No active scope")
            .symbols
            .insert(name.clone(), Variable { name, ty, addr });
        Ok(())
    }

    pub(super) fn lookup_variable(&self, name: &str) -> Option<&Variable> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.symbols.get(name))
    }
}
