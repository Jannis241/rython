#![allow(dead_code)]

use std::panic::{self, AssertUnwindSafe};

use rython_to_ir::ast::Item;
use rython_to_ir::codegen::{
    CodegenError, IrInstruction, IrModule, IrType, Terminator, generate_code,
};
use rython_to_ir::ir::{IrFunction, IrTypeDefinition, TempId};
use rython_to_ir::lexer::{Lexer, Token, TokenKind};
use rython_to_ir::parser::{ParseError, Parser};

pub fn lex(source: &str) -> Result<Vec<Token>, rython_to_ir::lexer::LexingError> {
    Lexer::create_tokens(source.to_string())
}

pub fn token_pairs(source: &str) -> Vec<(TokenKind, String)> {
    lex(source)
        .expect("lexing failed")
        .into_iter()
        .map(|token| (token.kind, token.value))
        .collect()
}

pub fn assert_tokens(source: &str, expected: &[(TokenKind, &str)]) {
    let actual = token_pairs(source);
    let expected: Vec<(TokenKind, String)> = expected
        .iter()
        .map(|(kind, value)| (kind.clone(), (*value).to_string()))
        .collect();
    assert_eq!(actual, expected, "source: {source:?}");
}

pub fn parse_items(source: &str) -> Result<Vec<Item>, ParseError> {
    let tokens = lex(source).expect("lexing failed");
    Parser::new(tokens).parse()
}

pub fn parse_items_no_panic(
    source: &str,
) -> Result<Result<Vec<Item>, ParseError>, Box<dyn std::any::Any + Send>> {
    panic::catch_unwind(AssertUnwindSafe(|| parse_items(source)))
}

pub fn compile_no_panic(
    source: &str,
) -> Result<Result<IrModule, CodegenError>, Box<dyn std::any::Any + Send>> {
    panic::catch_unwind(AssertUnwindSafe(|| compile(source)))
}

pub fn compile(source: &str) -> Result<IrModule, CodegenError> {
    let items = parse_items(source).expect("parsing failed");
    generate_code(&items)
}

pub fn compile_ok(source: &str) -> IrModule {
    match compile(source) {
        Ok(module) => module,
        Err(err) => panic!("expected IR generation to succeed, got {err:?}"),
    }
}

pub fn compile_verified(source: &str) -> IrModule {
    let module = compile_ok(source);
    assert_valid_ir(&module);
    module
}

pub fn compile_err(source: &str) -> CodegenError {
    match compile(source) {
        Ok(module) => panic!("expected IR generation to fail, got {module:#?}"),
        Err(err) => err,
    }
}

pub fn function<'a>(module: &'a IrModule, name: &str) -> &'a rython_to_ir::ir::IrFunction {
    module
        .functions
        .iter()
        .find(|function| function.name == name)
        .unwrap_or_else(|| {
            panic!(
                "missing function {name}; module functions: {:#?}",
                module.functions
            )
        })
}

pub fn all_instructions(module: &IrModule) -> Vec<&IrInstruction> {
    module
        .functions
        .iter()
        .flat_map(|function| function.blocks.iter())
        .flat_map(|block| block.instructions.iter())
        .collect()
}

pub fn assert_all_blocks_terminated(module: &IrModule) {
    for function in &module.functions {
        for block in &function.blocks {
            match &block.terminator {
                Terminator::Ret(_) | Terminator::Jump { .. } | Terminator::Branch { .. } => {}
            }
        }
    }
}

pub fn assert_branches_target_existing_blocks(module: &IrModule) {
    for function in &module.functions {
        let labels: Vec<&str> = function
            .blocks
            .iter()
            .map(|block| block.label.trim_end_matches(':'))
            .collect();

        for block in &function.blocks {
            match &block.terminator {
                Terminator::Jump { target } => {
                    assert!(
                        labels.contains(&target.as_str()),
                        "unknown jump target {target:?}"
                    );
                }
                Terminator::Branch {
                    then_block,
                    else_block,
                    ..
                } => {
                    assert!(
                        labels.contains(&then_block.as_str()),
                        "unknown then target {then_block:?}"
                    );
                    assert!(
                        labels.contains(&else_block.as_str()),
                        "unknown else target {else_block:?}"
                    );
                }
                Terminator::Ret(_) => {}
            }
        }
    }
}

pub fn assert_no_duplicate_names(names: &[String]) {
    let mut seen = std::collections::HashSet::new();
    for name in names {
        assert!(seen.insert(name), "duplicate name {name:?} in {names:?}");
    }
}

#[derive(Debug, Clone, PartialEq)]
enum TempKind {
    Value(IrType),
    Addr(IrType),
}

pub fn assert_valid_ir(module: &IrModule) {
    assert_no_duplicate_names(
        &module
            .functions
            .iter()
            .map(|function| function.name.clone())
            .collect::<Vec<_>>(),
    );
    assert_no_duplicate_names(
        &module
            .types
            .iter()
            .map(|ty| match ty {
                IrTypeDefinition::Struct { name, .. } | IrTypeDefinition::Variant { name, .. } => {
                    name.clone()
                }
            })
            .collect::<Vec<_>>(),
    );

    for function in &module.functions {
        verify_function(module, function);
    }
}

fn verify_function(module: &IrModule, function: &IrFunction) {
    assert!(
        !function.blocks.is_empty(),
        "function {} has no blocks",
        function.name
    );

    let mut labels = std::collections::HashSet::new();
    for block in &function.blocks {
        let label = normalize_label(&block.label);
        assert!(
            labels.insert(label.clone()),
            "duplicate block label {label:?} in function {}",
            function.name
        );
    }

    let mut temps: std::collections::HashMap<usize, TempKind> = std::collections::HashMap::new();

    for block in &function.blocks {
        for instruction in &block.instructions {
            match instruction {
                IrInstruction::PrimitiveConst { temp_id, ty, .. } => {
                    define_temp(&mut temps, *temp_id, TempKind::Value(ty.clone()), function);
                }
                IrInstruction::LoadParam { temp_id, ty, .. } => {
                    define_temp(&mut temps, *temp_id, TempKind::Value(ty.clone()), function);
                }
                IrInstruction::Alloca { temp_id, ty } => {
                    define_temp(&mut temps, *temp_id, TempKind::Addr(ty.clone()), function);
                }
                IrInstruction::GlobalAddr { temp_id, ty, .. } => {
                    define_temp(&mut temps, *temp_id, TempKind::Addr(ty.clone()), function);
                }
                IrInstruction::Load { temp_id, ty, addr } => {
                    assert_load_addr(&temps, *addr, ty, function);
                    define_temp(&mut temps, *temp_id, TempKind::Value(ty.clone()), function);
                }
                IrInstruction::Store { ty, value, addr } => {
                    assert_value_of_type(&temps, *value, ty, function);
                    assert_store_addr(&temps, *addr, ty, function);
                }
                IrInstruction::Binary {
                    temp_id,
                    ty_lr,
                    ty_res,
                    lhs,
                    rhs,
                    ..
                } => {
                    assert_value_of_type(&temps, *lhs, ty_lr, function);
                    assert_value_of_type(&temps, *rhs, ty_lr, function);
                    define_temp(
                        &mut temps,
                        *temp_id,
                        TempKind::Value(ty_res.clone()),
                        function,
                    );
                }
                IrInstruction::Unary {
                    temp_id, ty, value, ..
                } => {
                    assert_value_of_type(&temps, *value, ty, function);
                    define_temp(&mut temps, *temp_id, TempKind::Value(ty.clone()), function);
                }
                IrInstruction::Call {
                    temp_id,
                    args,
                    return_type,
                    ..
                } => {
                    for arg in args {
                        assert_defined(&temps, *arg, function);
                    }
                    define_temp(
                        &mut temps,
                        *temp_id,
                        TempKind::Value(return_type.clone()),
                        function,
                    );
                }
                IrInstruction::InitVariant { temp_id, ty, .. } => {
                    define_temp(&mut temps, *temp_id, TempKind::Value(ty.clone()), function);
                }
                IrInstruction::GetFieldAddr {
                    temp_id,
                    base_addr,
                    field_name,
                } => {
                    let field_ty = field_type_for_base(module, &temps, *base_addr, field_name)
                        .unwrap_or_else(|| {
                            panic!(
                                "unknown field {field_name:?} for base {:?} in function {}",
                                base_addr, function.name
                            )
                        });
                    define_temp(&mut temps, *temp_id, TempKind::Addr(field_ty), function);
                }
                IrInstruction::Asm { .. } => {}
            }
        }

        match &block.terminator {
            Terminator::Ret(Some(temp)) => {
                assert_value_of_type(&temps, *temp, &function.return_type, function);
            }
            Terminator::Ret(None) => {
                assert_eq!(
                    function.return_type,
                    IrType::Void,
                    "function {} returns void from non-void block {}",
                    function.name,
                    block.label
                );
            }
            Terminator::Jump { target } => {
                assert!(
                    labels.contains(target.as_str()),
                    "jump from {} in {} targets missing block {target:?}",
                    block.label,
                    function.name
                );
            }
            Terminator::Branch {
                condition,
                then_block,
                else_block,
            } => {
                assert_value_of_type(&temps, *condition, &IrType::Bool, function);
                assert!(
                    labels.contains(then_block.as_str()),
                    "branch from {} in {} targets missing then block {then_block:?}",
                    block.label,
                    function.name
                );
                assert!(
                    labels.contains(else_block.as_str()),
                    "branch from {} in {} targets missing else block {else_block:?}",
                    block.label,
                    function.name
                );
            }
        }
    }
}

fn normalize_label(label: &str) -> String {
    label.trim_end_matches(':').to_string()
}

fn define_temp(
    temps: &mut std::collections::HashMap<usize, TempKind>,
    temp_id: TempId,
    kind: TempKind,
    function: &IrFunction,
) {
    assert!(
        temps.insert(temp_id.0, kind.clone()).is_none(),
        "temp {:?} defined more than once in function {}",
        temp_id,
        function.name
    );
}

fn assert_defined(
    temps: &std::collections::HashMap<usize, TempKind>,
    temp_id: TempId,
    function: &IrFunction,
) -> TempKind {
    temps.get(&temp_id.0).cloned().unwrap_or_else(|| {
        panic!(
            "temp {:?} used before definition in {}",
            temp_id, function.name
        )
    })
}

fn assert_value_of_type(
    temps: &std::collections::HashMap<usize, TempKind>,
    temp_id: TempId,
    expected: &IrType,
    function: &IrFunction,
) {
    let actual = assert_defined(temps, temp_id, function);
    assert!(
        temp_matches_expected_value(&actual, expected),
        "temp {:?} in {} has kind {:?}, expected value {:?}",
        temp_id,
        function.name,
        actual,
        expected
    );
}

fn assert_load_addr(
    temps: &std::collections::HashMap<usize, TempKind>,
    temp_id: TempId,
    expected_loaded_ty: &IrType,
    function: &IrFunction,
) {
    let actual = assert_defined(temps, temp_id, function);
    match actual {
        TempKind::Addr(addr_ty) => assert_eq!(
            &addr_ty, expected_loaded_ty,
            "load from {:?} in {} has address type {:?}, expected {:?}",
            temp_id, function.name, addr_ty, expected_loaded_ty
        ),
        TempKind::Value(IrType::Pointer(inner)) => assert_eq!(
            inner.as_ref(),
            expected_loaded_ty,
            "load through pointer {:?} in {} has pointee {:?}, expected {:?}",
            temp_id,
            function.name,
            inner,
            expected_loaded_ty
        ),
        other => panic!(
            "load address {:?} in {} is not an address or pointer: {:?}",
            temp_id, function.name, other
        ),
    }
}

fn assert_store_addr(
    temps: &std::collections::HashMap<usize, TempKind>,
    temp_id: TempId,
    expected_stored_ty: &IrType,
    function: &IrFunction,
) {
    let actual = assert_defined(temps, temp_id, function);
    match actual {
        TempKind::Addr(addr_ty) => assert_eq!(
            &addr_ty, expected_stored_ty,
            "store to {:?} in {} has address type {:?}, expected {:?}",
            temp_id, function.name, addr_ty, expected_stored_ty
        ),
        TempKind::Value(IrType::Pointer(inner)) => assert_eq!(
            inner.as_ref(),
            expected_stored_ty,
            "store through pointer {:?} in {} has pointee {:?}, expected {:?}",
            temp_id,
            function.name,
            inner,
            expected_stored_ty
        ),
        other => panic!(
            "store address {:?} in {} is not an address or pointer: {:?}",
            temp_id, function.name, other
        ),
    }
}

fn temp_matches_expected_value(actual: &TempKind, expected: &IrType) -> bool {
    match (actual, expected) {
        (TempKind::Value(actual), expected) if actual == expected => true,
        (TempKind::Addr(actual), IrType::Pointer(expected_inner)) => {
            actual == expected_inner.as_ref()
        }
        _ => false,
    }
}

fn field_type_for_base(
    module: &IrModule,
    temps: &std::collections::HashMap<usize, TempKind>,
    base_addr: TempId,
    field_name: &str,
) -> Option<IrType> {
    let base = temps.get(&base_addr.0)?;
    let struct_name = match base {
        TempKind::Addr(IrType::Named(name)) => name,
        TempKind::Value(IrType::Pointer(inner)) => match inner.as_ref() {
            IrType::Named(name) => name,
            _ => return None,
        },
        TempKind::Addr(IrType::Pointer(inner)) => match inner.as_ref() {
            IrType::Named(name) => name,
            _ => return None,
        },
        _ => return None,
    };

    module.types.iter().find_map(|ty| match ty {
        IrTypeDefinition::Struct { name, fields } if name == struct_name => fields
            .iter()
            .find(|field| field.name == field_name)
            .map(|field| field.ty.clone()),
        _ => None,
    })
}
