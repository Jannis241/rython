use std::collections::HashMap;

use crate::ir::{IrType, TempId};

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
        self.scopes.pop();
    }

    pub(super) fn insert_variable(&mut self, name: String, ty: IrType, addr: TempId) {
        self.scopes
            .last_mut()
            .expect("No active scope")
            .symbols
            .insert(name.clone(), Variable { name, ty, addr });
    }

    // brauchen wir glaube ich nicht, da constants und globals in module gespeichert werden und in
    // gen_var aufgerufen werden
    //
    // pub(super) fn insert_variable_global(&mut self, name: String, ty: IrType, addr: TempId) {
    //     self.scopes
    //         .first_mut()
    //         .expect("No active scope")
    //         .symbols
    //         .insert(name.clone(), Variable { name, ty, addr });
    // }

    pub(super) fn lookup_variable(&self, name: &str) -> Option<&Variable> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.symbols.get(name))
    }
}
