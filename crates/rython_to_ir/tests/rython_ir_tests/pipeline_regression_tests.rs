use rython_to_ir::codegen::{CodegenError, IrInstruction, IrType, PrimitiveValue};
use rython_to_ir::lexer::TokenKind;

use super::common::{
    all_instructions, assert_all_blocks_terminated, assert_branches_target_existing_blocks,
    assert_no_duplicate_names, assert_valid_ir, compile, compile_err, compile_verified,
    parse_items, token_pairs,
};

#[test]
fn current_features_example_compiles_through_ir_without_backend_assumptions() {
    let module = compile_verified(include_str!("../../../../examples/current_features.ry"));

    assert!(
        module
            .functions
            .iter()
            .any(|function| function.name == "main")
    );
    assert!(
        module
            .functions
            .iter()
            .any(|function| function.name == "Vec2_add")
    );
    assert!(module.types.iter().any(|ty| matches!(
        ty,
        rython_to_ir::ir::IrTypeDefinition::Struct { name, .. } if name == "Vec2"
    )));
    assert!(module.types.iter().any(|ty| matches!(
        ty,
        rython_to_ir::ir::IrTypeDefinition::Variant { name, .. } if name == "Status"
    )));
    assert_all_blocks_terminated(&module);
    assert_branches_target_existing_blocks(&module);
}

#[test]
fn lexer_parser_codegen_pipeline_preserves_global_const_and_function_names() {
    let source = r#"
    const max: int = 10;
    global bonus: int = 2;
    fn add(x: int, y: int) int { return x + y; }
    fn main() int { return add(max, bonus); }
    "#;

    let tokens = token_pairs(source);
    assert!(tokens.contains(&(TokenKind::Const, "const".to_string())));
    assert!(tokens.contains(&(TokenKind::Global, "global".to_string())));

    let items = parse_items(source).expect("parse failed");
    assert_eq!(items.len(), 4);

    let module = compile(source).expect("compile failed");
    assert_valid_ir(&module);
    assert_eq!(module.constants[0].name, "max");
    assert_eq!(module.globals[0].name, "bonus");
    assert!(
        module
            .functions
            .iter()
            .any(|function| function.name == "add")
    );
    assert!(
        module
            .functions
            .iter()
            .any(|function| function.name == "main")
    );
}

#[test]
fn generated_ir_has_no_duplicate_public_names_for_supported_success_programs() {
    let module = compile_verified(
        r#"
        variant Status { Active, Done }
        struct Point {
            x: int,
            fn get(this) int { return this.x; }
        }
        const answer: int = 42;
        global counter: int = 0;
        fn main() int {
            let p: Point = Point { x: answer };
            counter = p.get();
            return counter;
        }
        "#,
    );

    assert_no_duplicate_names(
        &module
            .functions
            .iter()
            .map(|function| function.name.clone())
            .collect::<Vec<_>>(),
    );
    assert_no_duplicate_names(
        &module
            .constants
            .iter()
            .map(|constant| constant.name.clone())
            .collect::<Vec<_>>(),
    );
    assert_no_duplicate_names(
        &module
            .globals
            .iter()
            .map(|global| global.name.clone())
            .collect::<Vec<_>>(),
    );
}

#[test]
fn code_after_return_is_rejected_instead_of_silently_generating_unreachable_instructions() {
    assert!(matches!(
        compile_err("fn main() int { return 1; return 2; }"),
        CodegenError::CodeAfterTerminator
    ));
    assert!(matches!(
        compile_err("fn main() int { return 1; let x: int = 2; }"),
        CodegenError::CodeAfterTerminator
    ));
}

#[test]
fn invalid_programs_fail_at_the_earliest_relevant_compiler_phase() {
    assert!(rython_to_ir::lexer::Lexer::create_tokens("@".to_string()).is_err());
    assert!(parse_items("fn main() { let : int = 0; }").is_err());
    assert!(matches!(
        compile_err("fn main() int { return missing; }"),
        CodegenError::UnknownVariable(name) if name == "missing"
    ));
}

#[test]
fn bug_regression_operator_overload_mismatched_rhs_does_not_emit_bad_call_ir() {
    assert!(matches!(
        compile_err(
            r#"
            struct Box {
                value: int,
                fn operator + add(this, rhs: int) int { return this.value + rhs; }
            }
            fn main() int {
                let b: Box = Box { value: 1 };
                return b + false;
            }
            "#
        ),
        CodegenError::MismatchedTypes(IrType::I64, IrType::Bool)
    ));
}

#[test]
fn bug_regression_local_shadowing_must_resolve_to_the_local_binding_not_the_const() {
    let module = compile_verified(
        r#"
        const x: int = 1;
        fn main() int {
            let x: int = 2;
            return x;
        }
        "#,
    );
    let main = super::common::function(&module, "main");

    assert!(matches!(
        main.blocks[0].instructions.last(),
        Some(IrInstruction::Load { addr, .. }) if addr.0 == 0
    ));
}

#[test]
fn bug_regression_prefixed_integer_literals_reach_ir_as_values() {
    let module = compile_verified("fn main() int { return 0x10 + 0b10 + 0o7; }");
    let values: Vec<i64> = all_instructions(&module)
        .into_iter()
        .filter_map(|instruction| match instruction {
            IrInstruction::PrimitiveConst {
                value: PrimitiveValue::Int(value),
                ..
            } => Some(*value),
            _ => None,
        })
        .collect();

    assert!(
        values.contains(&16),
        "missing parsed hex value in {values:?}"
    );
    assert!(
        values.contains(&2),
        "missing parsed binary value in {values:?}"
    );
    assert!(
        values.contains(&7),
        "missing parsed octal value in {values:?}"
    );
}

#[test]
fn bug_regression_method_this_parameter_position_is_validated_before_codegen_call_mismatch() {
    assert!(parse_items("struct S { x: int, fn bad(x: int, this) int { return x; } }").is_err());
}
