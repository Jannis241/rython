use rython_to_ir::ast::*;
use rython_to_ir::lexer::TokenKind;
use rython_to_ir::parser::ParseError;

use super::common::{parse_items, parse_items_no_panic};

fn first_function(source: &str) -> Function {
    let items = parse_items(source).expect("parse failed");
    match items.into_iter().next().expect("missing item") {
        Item::Function(function) => function,
        other => panic!("expected function, got {other:#?}"),
    }
}

fn single_return_expr(source: &str) -> Expr {
    let function = first_function(source);
    match function.body.statements.as_slice() {
        [
            Stmt::Return(Return {
                return_value: Some(expr),
            }),
        ] => expr.clone(),
        other => panic!("expected single return expr, got {other:#?}"),
    }
}

#[test]
fn parses_function_signature_parameters_return_type_and_body() {
    let function = first_function("fn add(x: int, y: int) int { return x + y; }");

    assert_eq!(function.name, "add");
    assert_eq!(function.params.len(), 2);
    assert_eq!(function.params[0].name, "x");
    assert!(matches!(&function.params[0].param_type, Type::Named(name) if name == "int"));
    assert_eq!(function.params[1].name, "y");
    assert!(matches!(&function.return_type, Some(Type::Named(name)) if name == "int"));
    assert_eq!(function.body.statements.len(), 1);
}

#[test]
fn parses_operator_precedence_and_left_associativity_in_ast() {
    let expr = single_return_expr("fn main() int { return 1 + 2 * 3 - 4 / 2; }");

    match expr {
        Expr::BinaryOp {
            binary_op: BinaryOp::Sub,
            lhs,
            rhs,
        } => {
            assert!(matches!(
                *lhs,
                Expr::BinaryOp {
                    binary_op: BinaryOp::Add,
                    ..
                }
            ));
            assert!(matches!(
                *rhs,
                Expr::BinaryOp {
                    binary_op: BinaryOp::Div,
                    ..
                }
            ));
        }
        other => panic!("expected top-level subtraction, got {other:#?}"),
    }
}

#[test]
fn parses_assignment_as_right_associative_and_only_for_lvalues() {
    let expr = single_return_expr("fn main() int { return a = b = 1; }");
    match expr {
        Expr::Assign { target, value } => {
            assert!(matches!(*target, Expr::Variable(ref name) if name == "a"));
            assert!(matches!(*value, Expr::Assign { .. }));
        }
        other => panic!("expected assignment, got {other:#?}"),
    }

    let err = parse_items("fn main() { 1 = 2; }").unwrap_err();
    assert!(matches!(err, ParseError::InvalidAssignmentTarget { .. }));
}

#[test]
fn parses_grouping_calls_fields_index_and_postfix_chains() {
    let expr = single_return_expr("fn main() int { return (make()).field(1, 2)[0]++; }");
    match expr {
        Expr::PostFix {
            Op: PostFixOp::PlusPlus,
            value,
        } => match *value {
            Expr::PostFix {
                Op: PostFixOp::Brackets(index),
                value,
            } => {
                assert!(matches!(*index, Expr::IntLiteral(ref value) if value == "0"));
                match *value {
                    Expr::Call {
                        callee,
                        arguments,
                        type_args,
                    } => {
                        assert!(type_args.is_empty());
                        assert_eq!(arguments.len(), 2);
                        assert!(
                            matches!(arguments[0], Expr::IntLiteral(ref value) if value == "1")
                        );
                        assert!(
                            matches!(arguments[1], Expr::IntLiteral(ref value) if value == "2")
                        );
                        match *callee {
                            Expr::FieldAccess { object, field_name } => {
                                assert_eq!(field_name, "field");
                                assert!(matches!(*object, Expr::Grouping(_)));
                            }
                            other => panic!("expected field access callee, got {other:#?}"),
                        }
                    }
                    other => panic!("expected call before index, got {other:#?}"),
                }
            }
            other => panic!("expected index before ++, got {other:#?}"),
        },
        other => panic!("expected postfix chain, got {other:#?}"),
    }
}

#[test]
fn parses_structs_methods_variants_globals_consts_and_asm_items() {
    let items = parse_items(
        r#"
        global counter: int = 0;
        const answer: int = 42;
        variant Status { Active, Done, }
        struct Point {
            x: int,
            y: int,
            fn move_by(this, dx: int, dy: int) Point { return this; }
        }
        asm { mov rax, 0 };
        "#,
    )
    .expect("parse failed");

    assert_eq!(items.len(), 5);
    assert!(matches!(items[0], Item::GlobalVar(_)));
    assert!(matches!(items[1], Item::ConstVar(_)));
    assert!(matches!(items[2], Item::Variant(_)));
    match &items[3] {
        Item::Struct(point) => {
            assert_eq!(point.struct_name, "Point");
            assert_eq!(point.fields.len(), 2);
            assert_eq!(point.functions.len(), 1);
            assert_eq!(point.functions[0].params[0].name, "this");
        }
        other => panic!("expected struct, got {other:#?}"),
    }
    assert!(matches!(items[4], Item::Asm(_)));
}

#[test]
fn parses_control_flow_blocks_and_else_if_shape() {
    let function = first_function(
        r#"
        fn main(limit: int) int {
            let x: int = 0;
            if limit < 0 {
                return -1;
            } else if limit == 0 {
                return 0;
            } else {
                while x < limit {
                    x += 1;
                    if x == 3 { continue; }
                    if x == 4 { break; }
                }
            }
            loop { break; }
            return x;
        }
        "#,
    );

    assert!(matches!(function.body.statements[0], Stmt::Let(_)));
    assert!(matches!(function.body.statements[1], Stmt::If(_)));
    assert!(matches!(function.body.statements[2], Stmt::Loop(_)));
    assert!(matches!(function.body.statements[3], Stmt::Return(_)));
}

#[test]
fn parses_for_loop_list_literal_and_block_scope_syntax() {
    let function = first_function(
        r#"
        fn main() int {
            let x: int = 0;
            {
                let y: int = 1;
            }
            for item in [1, 2, 3] {
                x += item;
            }
            return x;
        }
        "#,
    );

    assert!(matches!(function.body.statements[1], Stmt::Block(_)));
    match &function.body.statements[2] {
        Stmt::For(for_stmt) => {
            assert_eq!(for_stmt.var_name, "item");
            assert!(matches!(for_stmt.iterable, Expr::ListLiteral(_)));
            assert_eq!(for_stmt.inner_code.statements.len(), 1);
        }
        other => panic!("expected for loop, got {other:#?}"),
    }
}

#[test]
fn parses_imports_traits_impls_generics_and_any_trait_types() {
    let items = parse_items(
        r#"
        import std.io;
        trait Display<T: Clone + Debug> {
            fn show(value: any Printable + Debug) string;
        }
        struct Box<T> {
            value: T,
            fn unwrap(this) T { return this.value; }
        }
        impl<T: Clone> Display<int> for Box<T> {
            fn show(this) string { return "Box"; }
        }
        "#,
    )
    .expect("parse failed");

    assert!(matches!(&items[0], Item::Import(import) if import.import_name == "std.io"));
    match &items[1] {
        Item::Trait(trait_item) => {
            assert_eq!(trait_item.trait_name, "Display");
            assert_eq!(trait_item.generic_params.len(), 1);
            assert_eq!(trait_item.function_signatures.len(), 1);
            assert!(matches!(
                trait_item.function_signatures[0].params[0].param_type,
                Type::AnyTrait(_)
            ));
        }
        other => panic!("expected trait, got {other:#?}"),
    }
    assert!(matches!(&items[2], Item::Struct(struct_item) if struct_item.struct_name == "Box"));
    assert!(matches!(
        &items[3],
        Item::TraitImplementation(implementation)
            if implementation.trait_name == "Display" && implementation.struct_name == "Box"
    ));
}

#[test]
fn parses_variant_literal_with_double_colon() {
    let expr = single_return_expr(
        r#"
        fn main() Status {
            return Status::Done;
        }
        "#,
    );

    assert!(matches!(
        expr,
        Expr::VariantLiteral {
            ref variant_name,
            ref case_name
        } if variant_name == "Status" && case_name == "Done"
    ));
}

#[test]
fn parses_struct_literal_field_names_and_values_in_source_order() {
    let expr = single_return_expr(
        r#"
        fn main() Point {
            return Point { x: 1, y: 2 + 3 };
        }
        "#,
    );

    match expr {
        Expr::StructLiteral {
            struct_name,
            arguments,
        } => {
            assert_eq!(struct_name, "Point");
            assert_eq!(arguments.len(), 2);
            assert_eq!(arguments[0].0, "x");
            assert!(matches!(arguments[0].1, Expr::IntLiteral(ref value) if value == "1"));
            assert_eq!(arguments[1].0, "y");
            assert!(matches!(
                arguments[1].1,
                Expr::BinaryOp {
                    binary_op: BinaryOp::Add,
                    ..
                }
            ));
        }
        other => panic!("expected struct literal, got {other:#?}"),
    }
}

#[test]
fn parses_grouping_as_distinct_from_struct_literal_in_control_flow_conditions() {
    let function = first_function(
        r#"
        fn main(flag: bool) int {
            if (flag) { return 1; }
            return 0;
        }
        "#,
    );

    match &function.body.statements[0] {
        Stmt::If(if_stmt) => {
            assert!(matches!(if_stmt.condition, Expr::Grouping(_)));
            assert_eq!(if_stmt.if_code.statements.len(), 1);
        }
        other => panic!("expected if statement, got {other:#?}"),
    }
}

#[test]
fn method_without_this_parses_as_static_method_with_no_implicit_receiver_param() {
    let items = parse_items(
        r#"
        struct S {
            fn static_value() int { return 1; }
        }
        "#,
    )
    .expect("parse failed");

    match &items[0] {
        Item::Struct(struct_item) => {
            assert_eq!(struct_item.functions.len(), 1);
            assert_eq!(struct_item.functions[0].name, "static_value");
            assert!(struct_item.functions[0].params.is_empty());
        }
        other => panic!("expected struct, got {other:#?}"),
    }
}

#[test]
fn malformed_double_colon_syntax_returns_parse_error_without_panic() {
    let result = parse_items_no_panic("fn main() { a::; }");

    match result {
        Ok(Err(_)) => {}
        Ok(Ok(items)) => panic!("expected parse error, got {items:#?}"),
        Err(_) => panic!("parser panicked on malformed :: syntax"),
    }
}

#[test]
fn malformed_parser_inputs_return_errors_instead_of_panicking() {
    let malformed_sources = [
        "fn main() { ::; }",
        "fn main() { a::; }",
        "fn broken<T {}",
        "fn main( { return; }",
        "fn main() { foo(1,); }",
        "fn main() { { let x: int = 1; }",
        "fn main() int { return;",
        "fn main() { Point { x: 1, }; }",
        "fn operator add() {}",
        "fn main() int { return 1 + ; }",
        "fn main() { if true }",
        "fn main() { if true { } else }",
        "fn main() { while true }",
        "fn main(value int) {}",
        "import std.;",
        "trait T { fn ; }",
        "impl T for { }",
    ];

    for source in malformed_sources {
        match parse_items_no_panic(source) {
            Ok(Err(_)) => {}
            Ok(Ok(items)) => panic!("expected parse error for {source:?}, got {items:#?}"),
            Err(_) => panic!("parser panicked for malformed source {source:?}"),
        }
    }
}

#[test]
fn parser_reports_missing_delimiters_and_unexpected_top_level_tokens() {
    assert!(matches!(
        parse_items("let x: int = 1;").unwrap_err(),
        ParseError::UnexpectedTopLevel {
            found: TokenKind::Let,
            ..
        }
    ));

    assert!(matches!(
        parse_items("fn main( { return; }").unwrap_err(),
        ParseError::UnexpectedToken { .. } | ParseError::UnexpectedExprStart { .. }
    ));
}

#[test]
fn duplicate_parameters_fields_and_variant_cases_are_semantic_errors() {
    assert!(
        parse_items("fn pick(x: int, x: int) int { return x; }").is_err(),
        "duplicate parameters in one function signature must be rejected"
    );
    assert!(
        parse_items("struct Bad { x: int, x: bool }").is_err(),
        "duplicate fields in one struct must be rejected"
    );
    assert!(
        parse_items("variant V { A, A }").is_err(),
        "duplicate variant cases must be rejected"
    );
}

#[test]
fn this_parameter_is_only_valid_as_first_method_parameter() {
    assert!(
        parse_items("struct S { x: int, fn ok(this, value: int) int { return value; } }").is_ok()
    );
    assert!(
        parse_items("struct S { x: int, fn bad(value: int, this) int { return value; } }").is_err(),
        "`this` after user parameters cannot match obj.method(...) calling convention"
    );
}
