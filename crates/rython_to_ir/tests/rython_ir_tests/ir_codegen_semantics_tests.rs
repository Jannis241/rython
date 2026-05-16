use rython_to_ir::codegen::{
    CodegenError, IrBinaryOp, IrInstruction, IrType, PrimitiveValue, TempId, Terminator,
};
use rython_to_ir::ir::IrTypeDefinition;

use super::common::{
    all_instructions, assert_all_blocks_terminated, assert_branches_target_existing_blocks,
    compile_err, compile_no_panic, compile_verified, function,
};

#[test]
fn empty_program_and_void_function_generate_valid_empty_ir_shapes() {
    let module = compile_verified("");
    assert!(module.functions.is_empty());
    assert!(module.globals.is_empty());
    assert!(module.constants.is_empty());
    assert!(module.types.is_empty());

    let module = compile_verified("fn noop() {}");
    let noop = function(&module, "noop");
    assert_eq!(noop.return_type, IrType::Void);
    assert_eq!(noop.blocks.len(), 1);
    assert!(matches!(noop.blocks[0].terminator, Terminator::Ret(None)));
}

#[test]
fn primitive_returns_generate_typed_constants_and_return_them() {
    let module = compile_verified(
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
fn exact_ir_for_return_expression_preserves_operand_order_and_return_value() {
    let module = compile_verified("fn main() int { return 1 + 2 * 3; }");
    let main = function(&module, "main");
    let entry = &main.blocks[0];

    assert_eq!(entry.instructions.len(), 5);
    assert!(matches!(
        entry.instructions[0],
        IrInstruction::PrimitiveConst {
            temp_id: TempId(0),
            ty: IrType::I64,
            value: PrimitiveValue::Int(1),
        }
    ));
    assert!(matches!(
        entry.instructions[1],
        IrInstruction::PrimitiveConst {
            temp_id: TempId(1),
            ty: IrType::I64,
            value: PrimitiveValue::Int(2),
        }
    ));
    assert!(matches!(
        entry.instructions[2],
        IrInstruction::PrimitiveConst {
            temp_id: TempId(2),
            ty: IrType::I64,
            value: PrimitiveValue::Int(3),
        }
    ));
    assert!(matches!(
        entry.instructions[3],
        IrInstruction::Binary {
            temp_id: TempId(3),
            op: IrBinaryOp::Mul,
            lhs: TempId(1),
            rhs: TempId(2),
            ty_lr: IrType::I64,
            ty_res: IrType::I64,
        }
    ));
    assert!(matches!(
        entry.instructions[4],
        IrInstruction::Binary {
            temp_id: TempId(4),
            op: IrBinaryOp::Add,
            lhs: TempId(0),
            rhs: TempId(3),
            ty_lr: IrType::I64,
            ty_res: IrType::I64,
        }
    ));
    assert!(matches!(entry.terminator, Terminator::Ret(Some(TempId(4)))));
}

#[test]
fn exact_ir_for_local_declaration_load_and_return_uses_the_allocated_slot() {
    let module = compile_verified("fn main() int { let x: int = 41; return x; }");
    let entry = &function(&module, "main").blocks[0];

    assert_eq!(entry.instructions.len(), 4);
    assert!(matches!(
        entry.instructions[0],
        IrInstruction::Alloca {
            temp_id: TempId(0),
            ty: IrType::I64,
        }
    ));
    assert!(matches!(
        entry.instructions[1],
        IrInstruction::PrimitiveConst {
            temp_id: TempId(1),
            ty: IrType::I64,
            value: PrimitiveValue::Int(41),
        }
    ));
    assert!(matches!(
        entry.instructions[2],
        IrInstruction::Store {
            ty: IrType::I64,
            value: TempId(1),
            addr: TempId(0),
        }
    ));
    assert!(matches!(
        entry.instructions[3],
        IrInstruction::Load {
            temp_id: TempId(2),
            ty: IrType::I64,
            addr: TempId(0),
        }
    ));
    assert!(matches!(entry.terminator, Terminator::Ret(Some(TempId(2)))));
}

#[test]
fn exact_ir_for_assignment_stores_new_value_before_the_final_load() {
    let module = compile_verified(
        r#"
        fn main() int {
            let x: int = 1;
            x = 2;
            return x;
        }
        "#,
    );
    let entry = &function(&module, "main").blocks[0];

    assert!(matches!(
        entry.instructions[0],
        IrInstruction::Alloca {
            temp_id: TempId(0),
            ty: IrType::I64,
        }
    ));
    assert!(matches!(
        entry.instructions[2],
        IrInstruction::Store {
            value: TempId(1),
            addr: TempId(0),
            ..
        }
    ));
    assert!(matches!(
        entry.instructions[3],
        IrInstruction::PrimitiveConst {
            temp_id: TempId(2),
            value: PrimitiveValue::Int(2),
            ..
        }
    ));
    assert!(matches!(
        entry.instructions[4],
        IrInstruction::Store {
            ty: IrType::I64,
            value: TempId(2),
            addr: TempId(0),
        }
    ));
    assert!(matches!(
        entry.instructions[5],
        IrInstruction::Load {
            temp_id: TempId(3),
            ty: IrType::I64,
            addr: TempId(0),
        }
    ));
    assert!(matches!(entry.terminator, Terminator::Ret(Some(TempId(3)))));
}

#[test]
fn arithmetic_comparison_logic_and_bitwise_expressions_generate_typed_binary_ir() {
    let module = compile_verified(
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
    let module = compile_verified(
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
fn local_shadowing_of_global_const_reads_the_local_binding() {
    let module = compile_verified("const x: int = 1; fn main() int { let x: int = 2; return x; }");
    let entry = &function(&module, "main").blocks[0];

    assert!(entry.instructions.iter().any(|instruction| matches!(
        instruction,
        IrInstruction::PrimitiveConst {
            temp_id: TempId(1),
            value: PrimitiveValue::Int(2),
            ..
        }
    )));
    assert!(matches!(
        entry.instructions.last(),
        Some(IrInstruction::Load {
            temp_id: TempId(2),
            addr: TempId(0),
            ..
        })
    ));
    assert!(matches!(entry.terminator, Terminator::Ret(Some(TempId(2)))));
}

#[test]
fn inner_block_shadowing_is_allowed_and_outer_binding_is_restored_after_block() {
    let module = compile_verified(
        r#"
        fn main() int {
            let x: int = 1;
            {
                let x: int = 2;
            }
            return x;
        }
        "#,
    );
    let entry = &function(&module, "main").blocks[0];

    assert!(matches!(
        entry.instructions.last(),
        Some(IrInstruction::Load {
            addr: TempId(0),
            ..
        })
    ));
}

#[test]
fn inner_block_shadowing_of_parameter_resolves_to_inner_binding_inside_block() {
    let module = compile_verified(
        r#"
        fn main(x: int) int {
            {
                let x: int = 2;
                return x;
            }
        }
        "#,
    );
    let entry = &function(&module, "main").blocks[0];

    assert!(entry.instructions.iter().any(|instruction| matches!(
        instruction,
        IrInstruction::PrimitiveConst {
            temp_id: TempId(3),
            value: PrimitiveValue::Int(2),
            ..
        }
    )));
    assert!(matches!(entry.terminator, Terminator::Ret(Some(TempId(4)))));
}

#[test]
fn same_local_names_in_different_functions_are_independent() {
    let module = compile_verified(
        r#"
        fn one() int { let x: int = 1; return x; }
        fn two() int { let x: int = 2; return x; }
        "#,
    );

    assert!(
        module
            .functions
            .iter()
            .any(|function| function.name == "one")
    );
    assert!(
        module
            .functions
            .iter()
            .any(|function| function.name == "two")
    );
}

#[test]
fn variable_from_inner_scope_is_not_visible_after_the_block() {
    assert!(matches!(
        compile_err("fn main() int { { let x: int = 1; } return x; }"),
        CodegenError::UnknownVariable(name) if name == "x"
    ));
}

#[test]
fn function_calls_check_argument_count_and_argument_types() {
    let ok = compile_verified(
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
        compile_err(
            "fn add(x: int, y: int) int { return x + y; } fn main() int { return add(1); }"
        ),
        CodegenError::WrongArgumentCount(_, 2, 1)
    ));
    assert!(matches!(
        compile_err(
            "fn add(x: int, y: int) int { return x + y; } fn main() int { return add(1, true); }"
        ),
        CodegenError::MismatchedTypes(IrType::I64, IrType::Bool)
    ));
}

#[test]
fn struct_definitions_literals_fields_methods_and_assignments_generate_consistent_ir() {
    let module = compile_verified(
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
    assert!(
        module
            .functions
            .iter()
            .any(|function| function.name == "Point_move_by")
    );
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
    let module = compile_verified(
        r#"
        struct Point { x: int }
        fn make() Point { return Point { x: 41 }; }
        fn main() int { return make().x + Point { x: 1 }.x; }
        "#,
    );

    assert!(
        all_instructions(&module)
            .iter()
            .filter(|instruction| matches!(
                instruction,
                IrInstruction::GetFieldAddr { field_name, .. } if field_name == "x"
            ))
            .count()
            >= 2
    );
}

#[test]
fn variants_generate_type_definitions_and_init_variant_instructions() {
    let module = compile_verified(
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
    let ok = compile_verified(
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
    let ok = compile_verified(
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
    let module = compile_verified(
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
    let module = compile_verified(
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
        IrInstruction::PrimitiveConst {
            value: PrimitiveValue::Int(7),
            ..
        }
    )));
}

#[test]
fn control_flow_generates_terminated_blocks_with_existing_targets() {
    let module = compile_verified(
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
    let module = compile_verified("fn main() int { loop { return 1; } }");
    assert_all_blocks_terminated(&module);
}

#[test]
fn if_else_codegen_connects_branch_targets_to_returning_blocks() {
    let module = compile_verified(
        r#"
        fn main(flag: bool) int {
            if flag { return 1; } else { return 2; }
        }
        "#,
    );
    let main = function(&module, "main");
    let entry = &main.blocks[0];

    assert!(matches!(
        entry.terminator,
        Terminator::Branch {
            condition: TempId(2),
            ref then_block,
            ref else_block,
        } if then_block == "if_then_0" && else_block == "if_else_2"
    ));
    assert!(main.blocks.iter().any(|block| {
        block.label == "if_then_0:"
            && matches!(block.terminator, Terminator::Ret(Some(_)))
            && block.instructions.iter().any(|instruction| {
                matches!(
                    instruction,
                    IrInstruction::PrimitiveConst {
                        value: PrimitiveValue::Int(1),
                        ..
                    }
                )
            })
    }));
    assert!(main.blocks.iter().any(|block| {
        block.label == "if_else_2:"
            && matches!(block.terminator, Terminator::Ret(Some(_)))
            && block.instructions.iter().any(|instruction| {
                matches!(
                    instruction,
                    IrInstruction::PrimitiveConst {
                        value: PrimitiveValue::Int(2),
                        ..
                    }
                )
            })
    }));
}

#[test]
fn while_codegen_branches_from_condition_to_body_and_end_then_body_jumps_back() {
    let module = compile_verified(
        r#"
        fn main(limit: int) int {
            let i: int = 0;
            while i < limit {
                i += 1;
            }
            return i;
        }
        "#,
    );
    let main = function(&module, "main");

    assert!(matches!(
        main.blocks[0].terminator,
        Terminator::Jump { ref target } if target == "while_cond_0"
    ));
    let cond = main
        .blocks
        .iter()
        .find(|block| block.label == "while_cond_0:")
        .expect("missing while condition block");
    assert!(matches!(
        cond.terminator,
        Terminator::Branch {
            ref then_block,
            ref else_block,
            ..
        } if then_block == "while_body_1" && else_block == "while_end_2"
    ));
    let body = main
        .blocks
        .iter()
        .find(|block| block.label == "while_body_1:")
        .expect("missing while body block");
    assert!(matches!(
        body.terminator,
        Terminator::Jump { ref target } if target == "while_cond_0"
    ));
    let end = main
        .blocks
        .iter()
        .find(|block| block.label == "while_end_2:")
        .expect("missing while end block");
    assert!(matches!(end.terminator, Terminator::Ret(Some(_))));
}

#[test]
fn normal_function_call_passes_arguments_in_source_order_and_returns_call_temp() {
    let module = compile_verified(
        r#"
        fn sub(left: int, right: int) int { return left - right; }
        fn main() int { return sub(10, 3); }
        "#,
    );
    let entry = &function(&module, "main").blocks[0];

    assert!(matches!(
        entry.instructions[0],
        IrInstruction::PrimitiveConst {
            temp_id: TempId(0),
            value: PrimitiveValue::Int(10),
            ..
        }
    ));
    assert!(matches!(
        entry.instructions[1],
        IrInstruction::PrimitiveConst {
            temp_id: TempId(1),
            value: PrimitiveValue::Int(3),
            ..
        }
    ));
    assert!(matches!(
        entry.instructions[2],
        IrInstruction::Call {
            temp_id: TempId(2),
            ref function_name,
            ref args,
            return_type: IrType::I64,
        } if function_name == "sub" && args.iter().map(|arg| arg.0).collect::<Vec<_>>() == vec![0, 1]
    ));
    assert!(matches!(entry.terminator, Terminator::Ret(Some(TempId(2)))));
}

#[test]
fn method_without_this_is_static_and_is_not_an_instance_method() {
    let module = compile_verified(
        r#"
        struct S {
            fn static_value() int { return 7; }
        }
        "#,
    );

    let static_fn = function(&module, "S_static_value");
    assert!(static_fn.parameter.is_empty());

    assert!(matches!(
        compile_err(
            r#"
            struct S {
                value: int,
                fn static_value() int { return 7; }
            }
            fn main() int {
                let s: S = S { value: 1 };
                return s.static_value();
            }
            "#
        ),
        CodegenError::WrongArgumentCount(name, 0, 1) if name == "S_static_value"
    ));
}

#[test]
fn short_circuit_and_is_lowered_to_control_flow_not_eager_binary_and() {
    let module = compile_verified(
        r#"
        fn rhs() bool { return true; }
        fn main(left: bool) bool { return left and rhs(); }
        "#,
    );
    let main = function(&module, "main");

    assert!(
        main.blocks
            .iter()
            .any(|block| matches!(block.terminator, Terminator::Branch { .. })),
        "`and` must branch so the right hand side can be skipped"
    );
    assert!(
        !main
            .blocks
            .iter()
            .flat_map(|block| block.instructions.iter())
            .any(|instruction| matches!(
                instruction,
                IrInstruction::Binary {
                    op: IrBinaryOp::And,
                    ..
                }
            )),
        "`and` must not be lowered to an eager binary op"
    );
}

#[test]
fn short_circuit_or_is_lowered_to_control_flow_not_eager_binary_or() {
    let module = compile_verified(
        r#"
        fn rhs() bool { return false; }
        fn main(left: bool) bool { return left or rhs(); }
        "#,
    );
    let main = function(&module, "main");

    assert!(
        main.blocks
            .iter()
            .any(|block| matches!(block.terminator, Terminator::Branch { .. })),
        "`or` must branch so the right hand side can be skipped"
    );
    assert!(
        !main
            .blocks
            .iter()
            .flat_map(|block| block.instructions.iter())
            .any(|instruction| matches!(
                instruction,
                IrInstruction::Binary {
                    op: IrBinaryOp::Or,
                    ..
                }
            )),
        "`or` must not be lowered to an eager binary op"
    );
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

#[test]
fn negative_codegen_reports_specific_name_type_and_struct_literal_errors() {
    assert!(matches!(
        compile_err("fn main() Missing { return 0; }"),
        CodegenError::UnknownType(name) if name == "Missing"
    ));
    assert!(matches!(
        compile_err("fn main() int { return missing(); }"),
        CodegenError::UnknownFunction(name) if name == "missing"
    ));
    assert!(matches!(
        compile_err("struct Point { x: int } fn main() int { let p: Point = Point { x: 1 }; return p.y; }"),
        CodegenError::UnknownField(name) if name == "y"
    ));
    assert!(matches!(
        compile_err("fn same() {} fn same() {}"),
        CodegenError::DuplicateFunction(name) if name == "same"
    ));
    assert!(matches!(
        compile_err("struct T { x: int } variant T { A }"),
        CodegenError::DuplicateType(name) if name == "T"
    ));
    assert!(matches!(
        compile_err("struct Point { x: int, y: int } fn main() { let p: Point = Point { x: 1 }; }"),
        CodegenError::FieldsDontMatch
    ));
    assert!(matches!(
        compile_err("struct Point { x: int } fn main() { let p: Point = Point { x: 1, y: 2 }; }"),
        CodegenError::FieldsDontMatch
    ));
    assert!(matches!(
        compile_err("struct Point { x: int } fn main() { let p: Point = Point { x: 1, x: 2 }; }"),
        CodegenError::FieldsDontMatch
    ));
    assert!(matches!(
        compile_err("const x: int = 1; fn main() { x = 2; }"),
        CodegenError::AssignToConst(name) if name == "x"
    ));
    assert!(matches!(
        compile_err("fn main() int { let x: int = 1; }"),
        CodegenError::MissingTerminator(_)
    ));
    assert!(matches!(
        compile_err("fn main() int { return null; }"),
        CodegenError::UnknownVariable(name) if name == "null"
    ));
}

#[test]
fn unsupported_parser_only_features_fail_cleanly_during_ir_codegen() {
    for source in [
        "import std.io;",
        "trait Display { fn show() int; }",
        "impl Display for Box { fn show() int { return 1; } }",
        "fn main<T>(value: T) T { return value; }",
        "fn main(value: any Display) {}",
        "fn operator + add(left: int, right: int) int { return left + right; }",
        "fn main() { for item in [1, 2, 3] { } }",
    ] {
        match compile_no_panic(source) {
            Ok(Ok(module)) => panic!("expected unsupported feature to fail, got {module:#?}"),
            Ok(Err(_)) => {}
            Err(_) => {
                panic!("unsupported feature panicked instead of returning CodegenError: {source}")
            }
        }
    }
}
