use std::panic::{self, AssertUnwindSafe};

use rython_to_ir::ast::*;
use rython_to_ir::codegen::*;

fn named_type(name: &str) -> Type {
    Type::Named(name.to_string())
}

fn block(statements: Vec<Stmt>) -> Block {
    Block { statements }
}

fn return_stmt(return_value: Option<Expr>) -> Stmt {
    Stmt::Return(Return { return_value })
}

fn function(
    name: &str,
    params: Vec<Param>,
    return_type: Option<Type>,
    statements: Vec<Stmt>,
) -> Item {
    Item::Function(Function {
        name: name.to_string(),
        generic_params: Vec::new(),
        params,
        body: block(statements),
        return_type,
        operator: None,
    })
}

fn param(name: &str, param_type: Type) -> Param {
    Param {
        name: name.to_string(),
        param_type,
    }
}

fn global_var(name: &str, ty: Type, value: Expr) -> Item {
    Item::GlobalVar(GlobalVar {
        var_name: name.to_string(),
        var_type: ty,
        value,
    })
}

fn const_var(name: &str, ty: Type, value: Expr) -> Item {
    Item::ConstVar(ConstVar {
        var_name: name.to_string(),
        var_type: ty,
        value,
    })
}

fn temp_debug(temp_id: &TempId) -> String {
    format!("{temp_id:?}")
}

fn assert_ir_type(actual: &IrType, expected: &IrType) {
    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
}

fn assert_const_instruction(
    instruction: &IrInstruction,
    expected_temp: &str,
    expected_type: &IrType,
    expected_value: &ConstValue,
) {
    match instruction {
        IrInstruction::Const { temp_id, ty, value } => {
            assert_eq!(temp_debug(temp_id), expected_temp);
            assert_ir_type(ty, expected_type);
            assert_const_value(value, expected_value);
        }
        other => panic!("expected const instruction, got {other:?}"),
    }
}

fn assert_alloca_instruction(
    instruction: &IrInstruction,
    expected_temp: &str,
    expected_type: &IrType,
) {
    match instruction {
        IrInstruction::Alloca { temp_id, ty } => {
            assert_eq!(temp_debug(temp_id), expected_temp);
            assert_ir_type(ty, expected_type);
        }
        other => panic!("expected alloca instruction, got {other:?}"),
    }
}

fn assert_store_instruction(
    instruction: &IrInstruction,
    expected_type: &IrType,
    expected_value: &str,
    expected_addr: &str,
) {
    match instruction {
        IrInstruction::Store { ty, value, addr } => {
            assert_ir_type(ty, expected_type);
            assert_eq!(temp_debug(value), expected_value);
            assert_eq!(temp_debug(addr), expected_addr);
        }
        other => panic!("expected store instruction, got {other:?}"),
    }
}

fn assert_load_instruction(
    instruction: &IrInstruction,
    expected_temp: &str,
    expected_type: &IrType,
    expected_addr: &str,
) {
    match instruction {
        IrInstruction::Load { temp_id, ty, addr } => {
            assert_eq!(temp_debug(temp_id), expected_temp);
            assert_ir_type(ty, expected_type);
            assert_eq!(temp_debug(addr), expected_addr);
        }
        other => panic!("expected load instruction, got {other:?}"),
    }
}

fn assert_const_value(actual: &ConstValue, expected: &ConstValue) {
    match (actual, expected) {
        (ConstValue::Int(actual), ConstValue::Int(expected)) => assert_eq!(actual, expected),
        (ConstValue::Float(actual), ConstValue::Float(expected)) => assert_eq!(actual, expected),
        (ConstValue::Bool(actual), ConstValue::Bool(expected)) => assert_eq!(actual, expected),
        (ConstValue::String(actual), ConstValue::String(expected)) => assert_eq!(actual, expected),
        (ConstValue::Char(actual), ConstValue::Char(expected)) => assert_eq!(actual, expected),
        (ConstValue::Null, ConstValue::Null) => {}
        _ => panic!("expected {expected:?}, got {actual:?}"),
    }
}

fn assert_ret(terminator: &Terminator, expected_temp: Option<&str>) {
    match (terminator, expected_temp) {
        (Terminator::Ret(Some(temp_id)), Some(expected_temp)) => {
            assert_eq!(temp_debug(temp_id), expected_temp);
        }
        (Terminator::Ret(None), None) => {}
        _ => panic!("expected ret {expected_temp:?}, got {terminator:?}"),
    }
}

fn assert_panics<F: FnOnce()>(f: F) {
    assert!(panic::catch_unwind(AssertUnwindSafe(f)).is_err());
}

fn unwrap_codegen(result: Result<IrModule, CodegenError>) -> IrModule {
    match result {
        Ok(module) => module,
        Err(_) => panic!("codegen failed"),
    }
}

fn assert_codegen_err(result: Result<IrModule, CodegenError>) {
    assert!(result.is_err());
}

#[test]
fn new_ir_module_starts_empty() {
    let module = IrModule::new();

    assert!(module.functions.is_empty());
    assert!(module.globals.is_empty());
    assert!(module.constants.is_empty());
    assert!(module.types.is_empty());
}

#[test]
fn generate_empty_item_list_returns_empty_module() {
    let module = unwrap_codegen(generate_code(&[]));

    assert!(module.functions.is_empty());
    assert!(module.globals.is_empty());
    assert!(module.constants.is_empty());
    assert!(module.types.is_empty());
}

#[test]
fn function_without_return_type_or_statements_generates_void_entry_function() {
    let module = unwrap_codegen(generate_code(&[function(
        "main",
        Vec::new(),
        None,
        Vec::new(),
    )]));

    assert_eq!(module.functions.len(), 1);
    assert!(module.globals.is_empty());
    assert!(module.constants.is_empty());
    assert!(module.types.is_empty());

    let function = &module.functions[0];
    assert_eq!(function.name, "main");
    assert!(function.parameter.is_empty());
    assert_ir_type(&function.return_type, &IrType::Void);
    assert_eq!(function.blocks.len(), 1);

    let entry = &function.blocks[0];
    assert_eq!(entry.label, "entry:");
    assert!(entry.instructions.is_empty());
    assert_ret(&entry.terminator, None);
}

#[test]
fn function_parameters_and_return_type_are_converted_to_ir_types() {
    let module = unwrap_codegen(generate_code(&[function(
        "typed",
        vec![
            param("i", named_type("int")),
            param("f", named_type("float")),
            param("b", named_type("bool")),
            param("v", named_type("void")),
            param("custom", named_type("UserType")),
        ],
        Some(named_type("UserResult")),
        Vec::new(),
    )]));

    let function = &module.functions[0];
    assert_eq!(function.parameter.len(), 5);

    assert_eq!(function.parameter[0].name, "i");
    assert_ir_type(&function.parameter[0].param_type, &IrType::I64);
    assert_eq!(function.parameter[1].name, "f");
    assert_ir_type(&function.parameter[1].param_type, &IrType::F64);
    assert_eq!(function.parameter[2].name, "b");
    assert_ir_type(&function.parameter[2].param_type, &IrType::Bool);
    assert_eq!(function.parameter[3].name, "v");
    assert_ir_type(&function.parameter[3].param_type, &IrType::Void);
    assert_eq!(function.parameter[4].name, "custom");
    assert_ir_type(
        &function.parameter[4].param_type,
        &IrType::Named("UserType".to_string()),
    );
    assert_ir_type(
        &function.return_type,
        &IrType::Named("UserResult".to_string()),
    );
}

#[test]
fn void_return_statement_keeps_empty_return_terminator() {
    let module = unwrap_codegen(generate_code(&[function(
        "main",
        Vec::new(),
        None,
        vec![return_stmt(None)],
    )]));

    let entry = &module.functions[0].blocks[0];
    assert!(entry.instructions.is_empty());
    assert_ret(&entry.terminator, None);
}

#[test]
fn int_return_generates_i64_const_and_returns_its_temp() {
    let module = unwrap_codegen(generate_code(&[function(
        "main",
        Vec::new(),
        Some(named_type("int")),
        vec![return_stmt(Some(Expr::IntLiteral("42".to_string())))],
    )]));

    let entry = &module.functions[0].blocks[0];
    assert_eq!(entry.instructions.len(), 1);
    assert_const_instruction(
        &entry.instructions[0],
        "TempId(0)",
        &IrType::I64,
        &ConstValue::Int(42),
    );
    assert_ret(&entry.terminator, Some("TempId(0)"));
}

#[test]
fn int_literal_parser_accepts_i64_boundaries_and_negative_values() {
    let module = unwrap_codegen(generate_code(&[function(
        "bounds",
        Vec::new(),
        Some(named_type("int")),
        vec![
            return_stmt(Some(Expr::IntLiteral(i64::MAX.to_string()))),
            return_stmt(Some(Expr::IntLiteral(i64::MIN.to_string()))),
        ],
    )]));

    let entry = &module.functions[0].blocks[0];
    assert_eq!(entry.instructions.len(), 2);
    assert_const_instruction(
        &entry.instructions[0],
        "TempId(0)",
        &IrType::I64,
        &ConstValue::Int(i64::MAX),
    );
    assert_const_instruction(
        &entry.instructions[1],
        "TempId(1)",
        &IrType::I64,
        &ConstValue::Int(i64::MIN),
    );
    assert_ret(&entry.terminator, Some("TempId(1)"));
}

#[test]
fn float_return_generates_f64_const_and_returns_its_temp() {
    let module = unwrap_codegen(generate_code(&[function(
        "main",
        Vec::new(),
        Some(named_type("float")),
        vec![return_stmt(Some(Expr::FloatLiteral("-3.5".to_string())))],
    )]));

    let entry = &module.functions[0].blocks[0];
    assert_eq!(entry.instructions.len(), 1);
    assert_const_instruction(
        &entry.instructions[0],
        "TempId(0)",
        &IrType::F64,
        &ConstValue::Float(-3.5),
    );
    assert_ret(&entry.terminator, Some("TempId(0)"));
}

#[test]
fn float_literal_supports_infinity_and_nan_from_rust_parser() {
    let module = unwrap_codegen(generate_code(&[function(
        "float_edges",
        Vec::new(),
        Some(named_type("float")),
        vec![
            return_stmt(Some(Expr::FloatLiteral("inf".to_string()))),
            return_stmt(Some(Expr::FloatLiteral("NaN".to_string()))),
        ],
    )]));

    let entry = &module.functions[0].blocks[0];
    assert_eq!(entry.instructions.len(), 2);

    match &entry.instructions[0] {
        IrInstruction::Const {
            temp_id,
            ty,
            value: ConstValue::Float(value),
        } => {
            assert_eq!(temp_debug(temp_id), "TempId(0)");
            assert_ir_type(ty, &IrType::F64);
            assert!(value.is_infinite());
        }
        other => panic!("expected infinite float const, got {other:?}"),
    }

    match &entry.instructions[1] {
        IrInstruction::Const {
            temp_id,
            ty,
            value: ConstValue::Float(value),
        } => {
            assert_eq!(temp_debug(temp_id), "TempId(1)");
            assert_ir_type(ty, &IrType::F64);
            assert!(value.is_nan());
        }
        other => panic!("expected nan float const, got {other:?}"),
    }
    assert_ret(&entry.terminator, Some("TempId(1)"));
}

#[test]
fn bool_returns_generate_bool_consts_for_true_and_false() {
    let module = unwrap_codegen(generate_code(&[function(
        "bools",
        Vec::new(),
        Some(named_type("bool")),
        vec![
            return_stmt(Some(Expr::BoolLiteral(true))),
            return_stmt(Some(Expr::BoolLiteral(false))),
        ],
    )]));

    let entry = &module.functions[0].blocks[0];
    assert_eq!(entry.instructions.len(), 2);
    assert_const_instruction(
        &entry.instructions[0],
        "TempId(0)",
        &IrType::Bool,
        &ConstValue::Bool(true),
    );
    assert_const_instruction(
        &entry.instructions[1],
        "TempId(1)",
        &IrType::Bool,
        &ConstValue::Bool(false),
    );
    assert_ret(&entry.terminator, Some("TempId(1)"));
}

#[test]
fn string_return_generates_named_string_const_without_losing_content() {
    let module = unwrap_codegen(generate_code(&[function(
        "main",
        Vec::new(),
        Some(named_type("string")),
        vec![return_stmt(Some(Expr::StringLiteral(
            "hello\nworld \"quoted\"".to_string(),
        )))],
    )]));

    let entry = &module.functions[0].blocks[0];
    assert_eq!(entry.instructions.len(), 1);
    assert_const_instruction(
        &entry.instructions[0],
        "TempId(0)",
        &IrType::Named("string".to_string()),
        &ConstValue::String("hello\nworld \"quoted\"".to_string()),
    );
    assert_ret(&entry.terminator, Some("TempId(0)"));
}

#[test]
fn temp_ids_increase_inside_a_function_and_terminator_uses_last_return_value() {
    let module = unwrap_codegen(generate_code(&[function(
        "main",
        Vec::new(),
        Some(named_type("int")),
        vec![
            return_stmt(Some(Expr::IntLiteral("1".to_string()))),
            return_stmt(Some(Expr::IntLiteral("2".to_string()))),
            return_stmt(Some(Expr::IntLiteral("3".to_string()))),
        ],
    )]));

    let entry = &module.functions[0].blocks[0];
    assert_eq!(entry.instructions.len(), 3);
    assert_const_instruction(
        &entry.instructions[0],
        "TempId(0)",
        &IrType::I64,
        &ConstValue::Int(1),
    );
    assert_const_instruction(
        &entry.instructions[1],
        "TempId(1)",
        &IrType::I64,
        &ConstValue::Int(2),
    );
    assert_const_instruction(
        &entry.instructions[2],
        "TempId(2)",
        &IrType::I64,
        &ConstValue::Int(3),
    );
    assert_ret(&entry.terminator, Some("TempId(2)"));
}

#[test]
fn temp_ids_reset_for_each_generated_function() {
    let module = unwrap_codegen(generate_code(&[
        function(
            "first",
            Vec::new(),
            Some(named_type("int")),
            vec![return_stmt(Some(Expr::IntLiteral("1".to_string())))],
        ),
        function(
            "second",
            Vec::new(),
            Some(named_type("int")),
            vec![return_stmt(Some(Expr::IntLiteral("2".to_string())))],
        ),
    ]));

    assert_eq!(module.functions.len(), 2);

    let first_entry = &module.functions[0].blocks[0];
    let second_entry = &module.functions[1].blocks[0];
    assert_const_instruction(
        &first_entry.instructions[0],
        "TempId(0)",
        &IrType::I64,
        &ConstValue::Int(1),
    );
    assert_ret(&first_entry.terminator, Some("TempId(0)"));
    assert_const_instruction(
        &second_entry.instructions[0],
        "TempId(0)",
        &IrType::I64,
        &ConstValue::Int(2),
    );
    assert_ret(&second_entry.terminator, Some("TempId(0)"));
}

#[test]
fn global_and_const_items_are_reported_as_unsupported_items() {
    assert_codegen_err(generate_code(&[
        global_var(
            "counter",
            named_type("int"),
            Expr::IntLiteral("0".to_string()),
        ),
        const_var(
            "pi",
            named_type("float"),
            Expr::FloatLiteral("3.5".to_string()),
        ),
        const_var("enabled", named_type("bool"), Expr::BoolLiteral(true)),
        global_var("letter", named_type("char"), Expr::CharLiteral('x')),
        const_var(
            "message",
            named_type("string"),
            Expr::StringLiteral("hello".to_string()),
        ),
        global_var("none", named_type("Option"), Expr::NullLiteral),
    ]));
}

#[test]
fn global_and_const_values_are_rejected_before_literal_validation() {
    assert_codegen_err(generate_code(&[global_var(
        "not_literal",
        named_type("int"),
        Expr::Variable("x".to_string()),
    )]));
    assert_codegen_err(generate_code(&[const_var(
        "also_not_literal",
        named_type("int"),
        Expr::BinaryOp {
            lhs: Box::new(Expr::IntLiteral("1".to_string())),
            binary_op: BinaryOp::Add,
            rhs: Box::new(Expr::IntLiteral("2".to_string())),
        },
    )]));
}

#[test]
fn unsupported_top_level_item_returns_error() {
    assert_codegen_err(generate_code(&[Item::Import(Import {
        import_name: "std".to_string(),
    })]));
}

#[test]
fn all_non_function_top_level_items_return_errors() {
    let items = vec![
        Item::Trait(Trait {
            trait_name: "Display".to_string(),
            generic_params: Vec::new(),
            function_signatures: Vec::new(),
        }),
        Item::Struct(Struct {
            struct_name: "Point".to_string(),
            generic_params: Vec::new(),
            fields: Vec::new(),
            functions: Vec::new(),
        }),
        Item::Variant(Variant {
            variant_name: "Option".to_string(),
            cases: vec!["Some".to_string(), "None".to_string()],
        }),
        Item::TraitImplementation(TraitImplementation {
            generic_params: Vec::new(),
            trait_name: "Display".to_string(),
            trait_args: Vec::new(),
            struct_name: "Point".to_string(),
            struct_args: Vec::new(),
            functions: Vec::new(),
        }),
        Item::Import(Import {
            import_name: "std".to_string(),
        }),
    ];

    for item in items {
        assert_codegen_err(generate_code(&[item]));
    }
}

#[test]
fn function_order_is_preserved() {
    let module = unwrap_codegen(generate_code(&[
        function("first", Vec::new(), None, Vec::new()),
        function("second", Vec::new(), None, Vec::new()),
        function("third", Vec::new(), None, Vec::new()),
    ]));

    assert_eq!(module.functions.len(), 3);
    assert_eq!(module.functions[0].name, "first");
    assert_eq!(module.functions[1].name, "second");
    assert_eq!(module.functions[2].name, "third");
}

#[test]
fn function_generic_params_and_operator_metadata_do_not_change_generated_function_shape() {
    let item = Item::Function(Function {
        name: "add".to_string(),
        generic_params: vec![GenericParam {
            name: "T".to_string(),
            bounds: vec![TraitBound {
                trait_name: "Add".to_string(),
                args: Vec::new(),
            }],
        }],
        params: Vec::new(),
        body: block(vec![return_stmt(None)]),
        return_type: None,
        operator: Some("+".to_string()),
    });

    let module = unwrap_codegen(generate_code(&[item]));
    let function = &module.functions[0];
    assert_eq!(function.name, "add");
    assert!(function.parameter.is_empty());
    assert_ir_type(&function.return_type, &IrType::Void);
    assert_ret(&function.blocks[0].terminator, None);
}

#[test]
fn unsupported_statement_returns_error() {
    assert_codegen_err(generate_code(&[function(
        "main",
        Vec::new(),
        None,
        vec![Stmt::Expr(Expr::IntLiteral("1".to_string()))],
    )]));
}

#[test]
fn let_statement_allocates_initializes_and_stores_variable() {
    let module = unwrap_codegen(generate_code(&[function(
        "main",
        Vec::new(),
        None,
        vec![Stmt::Let(Let {
            var_name: "x".to_string(),
            var_type: named_type("int"),
            value: Expr::IntLiteral("5".to_string()),
        })],
    )]));

    let entry = &module.functions[0].blocks[0];
    assert_eq!(entry.instructions.len(), 3);
    assert_alloca_instruction(&entry.instructions[0], "TempId(0)", &IrType::I64);
    assert_const_instruction(
        &entry.instructions[1],
        "TempId(1)",
        &IrType::I64,
        &ConstValue::Int(5),
    );
    assert_store_instruction(
        &entry.instructions[2],
        &IrType::I64,
        "TempId(1)",
        "TempId(0)",
    );
    assert_ret(&entry.terminator, None);
}

#[test]
fn let_statement_rejects_initializer_with_wrong_type() {
    assert_codegen_err(generate_code(&[function(
        "main",
        Vec::new(),
        None,
        vec![Stmt::Let(Let {
            var_name: "x".to_string(),
            var_type: named_type("int"),
            value: Expr::BoolLiteral(true),
        })],
    )]));
}

#[test]
fn declared_variable_expression_loads_from_variable_address() {
    let module = unwrap_codegen(generate_code(&[function(
        "main",
        Vec::new(),
        Some(named_type("int")),
        vec![
            Stmt::Let(Let {
                var_name: "x".to_string(),
                var_type: named_type("int"),
                value: Expr::IntLiteral("5".to_string()),
            }),
            return_stmt(Some(Expr::Variable("x".to_string()))),
        ],
    )]));

    let entry = &module.functions[0].blocks[0];
    assert_eq!(entry.instructions.len(), 4);
    assert_alloca_instruction(&entry.instructions[0], "TempId(0)", &IrType::I64);
    assert_const_instruction(
        &entry.instructions[1],
        "TempId(1)",
        &IrType::I64,
        &ConstValue::Int(5),
    );
    assert_store_instruction(
        &entry.instructions[2],
        &IrType::I64,
        "TempId(1)",
        "TempId(0)",
    );
    assert_load_instruction(
        &entry.instructions[3],
        "TempId(2)",
        &IrType::I64,
        "TempId(0)",
    );
    assert_ret(&entry.terminator, Some("TempId(2)"));
}

#[test]
fn unknown_variable_expression_returns_error() {
    assert_codegen_err(generate_code(&[function(
        "main",
        Vec::new(),
        Some(named_type("int")),
        vec![return_stmt(Some(Expr::Variable("x".to_string())))],
    )]));
}

#[test]
fn all_non_literal_return_expressions_return_errors() {
    let expressions = vec![
        Expr::Assign {
            target_name: "x".to_string(),
            value: Box::new(Expr::IntLiteral("1".to_string())),
        },
        Expr::BinaryOpAssign {
            target_name: "x".to_string(),
            binary_op: BinaryOp::Add,
            value: Box::new(Expr::IntLiteral("1".to_string())),
        },
        Expr::BinaryOp {
            lhs: Box::new(Expr::IntLiteral("1".to_string())),
            binary_op: BinaryOp::Add,
            rhs: Box::new(Expr::IntLiteral("2".to_string())),
        },
        Expr::Call {
            callee: Box::new(Expr::Variable("f".to_string())),
            arguments: Vec::new(),
        },
        Expr::Unary {
            op: UnaryOp::Neg,
            value: Box::new(Expr::IntLiteral("1".to_string())),
        },
        Expr::Unary {
            op: UnaryOp::Not,
            value: Box::new(Expr::BoolLiteral(true)),
        },
        Expr::ListLiteral(vec![Box::new(Expr::IntLiteral("1".to_string()))]),
        Expr::StructLiteral {
            struct_name: "Point".to_string(),
            arguments: vec![("x".to_string(), Expr::IntLiteral("1".to_string()))],
        },
        Expr::Grouping(Box::new(Expr::IntLiteral("1".to_string()))),
    ];

    for expr in expressions {
        assert_codegen_err(generate_code(&[function(
            "main",
            Vec::new(),
            Some(named_type("int")),
            vec![return_stmt(Some(expr))],
        )]));
    }
}

#[test]
fn any_trait_parameter_type_panics() {
    assert_panics(|| {
        let _ = generate_code(&[function(
            "main",
            vec![param(
                "x",
                Type::AnyTrait(vec![TraitBound {
                    trait_name: "Display".to_string(),
                    args: Vec::new(),
                }]),
            )],
            None,
            Vec::new(),
        )]);
    });
}

#[test]
fn any_trait_return_type_panics() {
    assert_panics(|| {
        let _ = generate_code(&[function(
            "main",
            Vec::new(),
            Some(Type::AnyTrait(vec![TraitBound {
                trait_name: "Display".to_string(),
                args: Vec::new(),
            }])),
            Vec::new(),
        )]);
    });
}

#[test]
fn invalid_int_literal_returns_error() {
    assert_codegen_err(generate_code(&[function(
        "main",
        Vec::new(),
        Some(named_type("int")),
        vec![return_stmt(Some(Expr::IntLiteral(
            "9223372036854775808".to_string(),
        )))],
    )]));
}

#[test]
fn invalid_float_literal_returns_error() {
    assert_codegen_err(generate_code(&[function(
        "main",
        Vec::new(),
        Some(named_type("float")),
        vec![return_stmt(Some(Expr::FloatLiteral(
            "not_a_float".to_string(),
        )))],
    )]));
}
