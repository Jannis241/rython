use rython_to_ir::codegen::{
    CodegenError, IrBinaryOp, IrInstruction, IrType, PrimitiveValue, Terminator,
};
use rython_to_ir::ir::IrTypeDefinition;

use super::common::{
    all_instructions, assert_all_blocks_terminated, assert_branches_target_existing_blocks,
    compile_err, compile_ok, function,
};

#[test]
fn empty_program_and_void_function_generate_valid_empty_ir_shapes() {
    let module = compile_ok("");
    assert!(module.functions.is_empty());
    assert!(module.globals.is_empty());
    assert!(module.constants.is_empty());
    assert!(module.types.is_empty());

    let module = compile_ok("fn noop() {}");
    let noop = function(&module, "noop");
    assert_eq!(noop.return_type, IrType::Void);
    assert_eq!(noop.blocks.len(), 1);
    assert!(matches!(noop.blocks[0].terminator, Terminator::Ret(None)));
}

#[test]
fn primitive_returns_generate_typed_constants_and_return_them() {
    let module = compile_ok(
        r#"
        fn int_value() int { return 42; }
        fn float_value() float { return 1.5; }
        fn bool_value() bool { return true; }
        fn char_value() char { return 'x'; }
        "#,
    );

    assert!(all_instructions(&module).iter().any(|instruction| matches!(
        instruction,
        IrInstruction::PrimitiveConst {
            ty: IrType::I64,
            value: PrimitiveValue::Int(42),
            ..
        }
    )));
    assert!(all_instructions(&module).iter().any(|instruction| matches!(
        instruction,
        IrInstruction::PrimitiveConst {
            ty: IrType::F64,
            value: PrimitiveValue::Float(value),
            ..
        } if *value == 1.5
    )));
    assert!(all_instructions(&module).iter().any(|instruction| matches!(
        instruction,
        IrInstruction::PrimitiveConst {
            ty: IrType::Bool,
            value: PrimitiveValue::Bool(true),
            ..
        }
    )));
    assert!(all_instructions(&module).iter().any(|instruction| matches!(
        instruction,
        IrInstruction::PrimitiveConst {
            ty: IrType::Char,
            value: PrimitiveValue::Char('x'),
            ..
        }
    )));
}

#[test]
fn arithmetic_comparison_logic_and_bitwise_expressions_generate_typed_binary_ir() {
    let module = compile_ok(
        r#"
        fn main(a: int, b: int) int {
            let value: int = a + b * 2 - 1;
            let bits: int = (value << 1) ^ (b | 1);
            let ok: bool = value > 0 and bits != 0;
            if ok { return bits % 100; }
            return 0;
        }
        "#,
    );

    let ops: Vec<String> = all_instructions(&module)
        .into_iter()
        .filter_map(|instruction| match instruction {
            IrInstruction::Binary { op, .. } => Some(format!("{op:?}")),
            _ => None,
        })
        .collect();

    for expected in [
        IrBinaryOp::Add,
        IrBinaryOp::Mul,
        IrBinaryOp::Sub,
        IrBinaryOp::Shl,
        IrBinaryOp::BitXor,
        IrBinaryOp::BitOr,
        IrBinaryOp::Gt,
        IrBinaryOp::Ne,
        IrBinaryOp::And,
        IrBinaryOp::Mod,
    ] {
        let expected = format!("{expected:?}");
        assert!(ops.contains(&expected), "missing {expected} in {ops:?}");
    }
    assert_all_blocks_terminated(&module);
    assert_branches_target_existing_blocks(&module);
}

#[test]
fn globals_and_constants_are_materialized_and_readable_from_functions() {
    let module = compile_ok(
        r#"
        global bonus: int = 7;
        const max_score: int = 100;
        fn main() int { return max_score + bonus; }
        "#,
    );

    assert_eq!(module.globals.len(), 1);
    assert_eq!(module.globals[0].name, "bonus");
    assert_eq!(module.globals[0].ty, IrType::I64);
    assert!(matches!(module.globals[0].value, PrimitiveValue::Int(7)));
    assert_eq!(module.constants.len(), 1);
    assert_eq!(module.constants[0].name, "max_score");

    assert!(all_instructions(&module).iter().any(|instruction| matches!(
        instruction,
        IrInstruction::GlobalAddr { name, ty: IrType::I64, .. } if name == "bonus"
    )));
}

#[test]
fn local_shadowing_of_params_globals_and_consts_is_rejected() {
    assert!(matches!(
        compile_err("const x: int = 1; fn main() int { let x: int = 2; return x; }"),
        CodegenError::DuplicateGlobal(_) | CodegenError::AmbigousVariable(_) | CodegenError::InvalidItem(_)
    ));
    assert!(matches!(
        compile_err("global x: int = 1; fn main() int { let x: int = 2; return x; }"),
        CodegenError::DuplicateGlobal(_) | CodegenError::AmbigousVariable(_) | CodegenError::InvalidItem(_)
    ));
    let _ = compile_err("fn main(x: int) int { let x: int = 2; return x; }");
}

#[test]
fn duplicate_local_names_in_the_same_scope_are_rejected() {
    let _ = compile_err(
        r#"
        fn main() int {
            let x: int = 1;
            let x: int = 2;
            return x;
        }
        "#,
    );
}

#[test]
fn function_calls_check_argument_count_and_argument_types() {
    let ok = compile_ok(
        r#"
        fn add(x: int, y: int) int { return x + y; }
        fn main() int { return add(1, 2); }
        "#,
    );
    assert!(all_instructions(&ok).iter().any(|instruction| matches!(
        instruction,
        IrInstruction::Call { function_name, return_type: IrType::I64, args, .. }
            if function_name == "add" && args.len() == 2
    )));

    assert!(matches!(
        compile_err("fn add(x: int, y: int) int { return x + y; } fn main() int { return add(1); }"),
        CodegenError::WrongArgumentCount(_, 2, 1)
    ));
    assert!(matches!(
        compile_err("fn add(x: int, y: int) int { return x + y; } fn main() int { return add(1, true); }"),
        CodegenError::MismatchedTypes(IrType::I64, IrType::Bool)
    ));
}

#[test]
fn struct_definitions_literals_fields_methods_and_assignments_generate_consistent_ir() {
    let module = compile_ok(
        r#"
        struct Point {
            x: int,
            y: int,
            fn move_by(this, dx: int, dy: int) Point {
                this.x += dx;
                this.y += dy;
                return this;
            }
        }
        fn main() int {
            let p: Point = Point { x: 1, y: 2 };
            let moved: Point = p.move_by(3, 4);
            return moved.x + moved.y;
        }
        "#,
    );

    assert!(module.types.iter().any(|ty| matches!(
        ty,
        IrTypeDefinition::Struct { name, fields }
            if name == "Point" && fields.len() == 2 && fields[0].name == "x" && fields[1].name == "y"
    )));
    assert!(module.functions.iter().any(|function| function.name == "Point_move_by"));
    assert!(all_instructions(&module).iter().any(|instruction| matches!(
        instruction,
        IrInstruction::GetFieldAddr { field_name, .. } if field_name == "x"
    )));
    assert!(all_instructions(&module).iter().any(|instruction| matches!(
        instruction,
        IrInstruction::Call { function_name, args, .. } if function_name == "Point_move_by" && args.len() == 3
    )));
}

#[test]
fn field_access_on_struct_rvalues_and_call_results_is_valid() {
    let module = compile_ok(
        r#"
        struct Point { x: int }
        fn make() Point { return Point { x: 41 }; }
        fn main() int { return make().x + Point { x: 1 }.x; }
        "#,
    );

    assert!(all_instructions(&module).iter().filter(|instruction| matches!(
        instruction,
        IrInstruction::GetFieldAddr { field_name, .. } if field_name == "x"
    )).count() >= 2);
}

#[test]
fn variants_generate_type_definitions_and_init_variant_instructions() {
    let module = compile_ok(
        r#"
        variant Status { Active, Done }
        fn main() Status { return Status::Done; }
        "#,
    );

    assert!(module.types.iter().any(|ty| matches!(
        ty,
        IrTypeDefinition::Variant { name, cases } if name == "Status" && cases == &vec!["Active".to_string(), "Done".to_string()]
    )));
    assert!(all_instructions(&module).iter().any(|instruction| matches!(
        instruction,
        IrInstruction::InitVariant { ty: IrType::Named(name), case_name, .. }
            if name == "Status" && case_name == "Done"
    )));
}

#[test]
fn operator_overloads_check_argument_types_like_normal_calls() {
    let ok = compile_ok(
        r#"
        struct Box {
            value: int,
            fn operator + add(this, rhs: int) int { return this.value + rhs; }
        }
        fn main() int {
            let b: Box = Box { value: 1 };
            return b + 2;
        }
        "#,
    );
    assert!(all_instructions(&ok).iter().any(|instruction| matches!(
        instruction,
        IrInstruction::Call { function_name, args, .. } if function_name == "Box_add" && args.len() == 2
    )));

    assert!(matches!(
        compile_err(
            r#"
            struct Box {
                value: int,
                fn operator + add(this, rhs: int) int { return this.value + rhs; }
            }
            fn main() int {
                let b: Box = Box { value: 1 };
                return b + true;
            }
            "#
        ),
        CodegenError::MismatchedTypes(IrType::I64, IrType::Bool)
    ));
}

#[test]
fn index_operator_overloads_check_index_argument_types() {
    let ok = compile_ok(
        r#"
        struct Box {
            value: int,
            fn operator [] get(this, index: int) int { return this.value + index; }
        }
        fn main() int {
            let b: Box = Box { value: 1 };
            return b[0];
        }
        "#,
    );
    assert!(all_instructions(&ok).iter().any(|instruction| matches!(
        instruction,
        IrInstruction::Call { function_name, args, return_type: IrType::I64, .. }
            if function_name == "Box_get" && args.len() == 2
    )));

    assert!(matches!(
        compile_err(
            r#"
            struct Box {
                value: int,
                fn operator [] get(this, index: int) int { return this.value + index; }
            }
            fn main() int {
                let b: Box = Box { value: 1 };
                return b[true];
            }
            "#
        ),
        CodegenError::MismatchedTypes(IrType::I64, IrType::Bool)
    ));
}

#[test]
fn inline_asm_substitutes_known_variables_and_rejects_unknown_variables() {
    let module = compile_ok(
        r#"
        fn main() int {
            let value: int = 42;
            asm { mov rax, %value };
            return value;
        }
        "#,
    );

    assert!(all_instructions(&module).iter().any(|instruction| matches!(
        instruction,
        IrInstruction::Asm { code } if code.contains("mov rax, %")
    )));

    assert!(matches!(
        compile_err("fn main() { asm { mov rax, %missing }; }"),
        CodegenError::UnknownVariable(name) if name == "missing"
    ));
}

#[test]
fn prefixed_integer_literals_work_in_globals_consts_and_expressions() {
    let module = compile_ok(
        r#"
        const ten: int = 0b1010;
        global mask: int = 0xFF;
        fn main() int { return ten + mask + 0o7; }
        "#,
    );

    assert!(matches!(module.constants[0].value, PrimitiveValue::Int(10)));
    assert!(matches!(module.globals[0].value, PrimitiveValue::Int(255)));
    assert!(all_instructions(&module).iter().any(|instruction| matches!(
        instruction,
        IrInstruction::PrimitiveConst { value: PrimitiveValue::Int(7), .. }
    )));
}

#[test]
fn control_flow_generates_terminated_blocks_with_existing_targets() {
    let module = compile_ok(
        r#"
        fn main(limit: int) int {
            let i: int = 0;
            let sum: int = 0;
            while i < limit {
                i += 1;
                if i == 3 { continue; }
                sum += i;
                if sum > 20 { break; }
            }
            if sum > 0 { return sum; } else { return 0; }
        }
        "#,
    );

    assert_all_blocks_terminated(&module);
    assert_branches_target_existing_blocks(&module);
    assert!(function(&module, "main").blocks.len() >= 5);
}

#[test]
fn loop_with_unconditional_return_is_valid_in_non_void_function() {
    let module = compile_ok("fn main() int { loop { return 1; } }");
    assert_all_blocks_terminated(&module);
}

#[test]
fn invalid_statements_and_type_errors_are_reported_as_codegen_errors() {
    assert!(matches!(
        compile_err("fn main() int { if 1 { return 1; } return 0; }"),
        CodegenError::MismatchedTypes(IrType::Bool, IrType::I64)
    ));
    assert!(matches!(
        compile_err("fn main() int { return true; }"),
        CodegenError::InvalidReturnType(IrType::I64, IrType::Bool)
    ));
    assert!(matches!(
        compile_err("fn main() { break; }"),
        CodegenError::BreakOutsideLoop
    ));
    assert!(matches!(
        compile_err("fn main() { continue; }"),
        CodegenError::ContinueOutsideLoop
    ));
}
