use ir_to_assembly::codegen::{get_type_size_in_qds, AsmCodeGen};
use rython_to_ir::codegen::{ConstValue, IrConstant, IrGlobal, IrModule, IrType};

#[test]
fn empty_module_generates_empty_assembly() {
    let asm = AsmCodeGen::gen_asm(IrModule::new());

    assert_eq!(asm, "");
}

#[test]
fn globals_and_constants_generate_data_sections() {
    let mut module = IrModule::new();
    module.globals.push(IrGlobal {
        name: "counter".to_string(),
        ty: IrType::I64,
        value: ConstValue::Int(42),
    });
    module.globals.push(IrGlobal {
        name: "enabled".to_string(),
        ty: IrType::Bool,
        value: ConstValue::Bool(true),
    });
    module.constants.push(IrConstant {
        name: "nothing".to_string(),
        ty: IrType::I64,
        value: ConstValue::Null,
    });

    let asm = AsmCodeGen::gen_asm(module);

    assert_eq!(
        asm,
        "section .data\ncounter: dq 42\nsection .data\nenabled: dq 1\nsection .rodata\nnothing: dq 0\n"
    );
}

#[test]
fn primitive_type_sizes_are_stable() {
    assert_eq!(get_type_size_in_qds(IrType::I64), 1);
    assert_eq!(get_type_size_in_qds(IrType::F64), 1);
    assert_eq!(get_type_size_in_qds(IrType::Bool), 1);
    assert_eq!(get_type_size_in_qds(IrType::Void), 0);
}

#[test]
#[should_panic(expected = "not yet implemented")]
fn named_type_size_is_not_silently_guessed() {
    get_type_size_in_qds(IrType::Named("Widget".to_string()));
}

#[test]
#[should_panic(expected = "not yet implemented")]
fn unsupported_constant_values_are_not_silently_emitted() {
    let mut module = IrModule::new();
    module.constants.push(IrConstant {
        name: "pi".to_string(),
        ty: IrType::F64,
        value: ConstValue::Float(3.14),
    });

    AsmCodeGen::gen_asm(module);
}
