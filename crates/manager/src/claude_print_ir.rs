use rython_to_ir::ir::{
    IrBinaryOp, IrBlock, IrConstant, IrField, IrFunction, IrGlobal, IrInstruction, IrModule,
    IrType, IrTypeDefinition, IrUnaryOp, PrimitiveValue, TempId, Terminator,
};

pub fn print_ir(module: &IrModule) {
    print!("{}", format_ir(module));
}

// Ziel: ausreichend ausfuehrlich, sodass aus dem Output dieselben Informationen
// rekonstruierbar sind wie aus {:?} fuer das jeweilige Struct/Enum.
// Variant-Tags (z.B. Pointer, Named, Int) und Feldnamen werden immer mit ausgegeben.
// Einzige Abkuerzung: TempId(n) -> %n, da bijektiv und ohne Informationsverlust.
pub fn format_ir(module: &IrModule) -> String {
    let mut out = String::new();
    out.push_str("==== IR Module ====\n\n");

    out.push_str("-- types --\n");
    if module.types.is_empty() {
        out.push_str("(none)\n");
    } else {
        for ty in &module.types {
            out.push_str(&format_type_def(ty));
            out.push('\n');
        }
    }
    out.push('\n');

    out.push_str("-- constants --\n");
    if module.constants.is_empty() {
        out.push_str("(none)\n");
    } else {
        for c in &module.constants {
            out.push_str(&format_constant(c));
            out.push('\n');
        }
    }
    out.push('\n');

    out.push_str("-- globals --\n");
    if module.globals.is_empty() {
        out.push_str("(none)\n");
    } else {
        for g in &module.globals {
            out.push_str(&format_global(g));
            out.push('\n');
        }
    }
    out.push('\n');

    out.push_str("-- module-level inline asm --\n");
    if module.inline_assembly.is_empty() {
        out.push_str("(none)\n");
    } else {
        for asm in &module.inline_assembly {
            out.push_str("Asm {\n");
            out.push_str("  code: \"\"\"\n");
            for line in asm.lines() {
                out.push_str("    ");
                out.push_str(line);
                out.push('\n');
            }
            out.push_str("  \"\"\"\n");
            out.push_str("}\n");
        }
    }
    out.push('\n');

    out.push_str("-- functions --\n");
    if module.functions.is_empty() {
        out.push_str("(none)\n");
    } else {
        for f in &module.functions {
            out.push_str(&format_function(f));
            out.push('\n');
        }
    }

    out
}

fn format_type_def(ty: &IrTypeDefinition) -> String {
    match ty {
        IrTypeDefinition::Struct { name, fields } => {
            let parts: Vec<String> = fields.iter().map(format_ir_field).collect();
            format!(
                "IrTypeDefinition::Struct {{ name: {:?}, fields: [{}] }}",
                name,
                parts.join(", ")
            )
        }
        IrTypeDefinition::Variant { name, cases } => {
            let parts: Vec<String> = cases.iter().map(|c| format!("{:?}", c)).collect();
            format!(
                "IrTypeDefinition::Variant {{ name: {:?}, cases: [{}] }}",
                name,
                parts.join(", ")
            )
        }
    }
}

fn format_ir_field(f: &IrField) -> String {
    format!(
        "IrField {{ name: {:?}, ty: {} }}",
        f.name,
        format_type(&f.ty)
    )
}

fn format_constant(c: &IrConstant) -> String {
    format!(
        "IrConstant {{ name: {:?}, ty: {}, value: {} }}",
        c.name,
        format_type(&c.ty),
        format_primitive(&c.value)
    )
}

fn format_global(g: &IrGlobal) -> String {
    format!(
        "IrGlobal {{ name: {:?}, ty: {}, value: {} }}",
        g.name,
        format_type(&g.ty),
        format_primitive(&g.value)
    )
}

fn format_function(f: &IrFunction) -> String {
    let mut s = String::new();
    let params: Vec<String> = f.parameter.iter().map(format_ir_field).collect();
    s.push_str(&format!(
        "IrFunction {{ name: {:?}, parameter: [{}], return_type: {}, blocks: [\n",
        f.name,
        params.join(", "),
        format_type(&f.return_type),
    ));
    for block in &f.blocks {
        s.push_str(&format_block(block));
    }
    s.push_str("]}\n");
    s
}

fn format_block(block: &IrBlock) -> String {
    let mut s = String::new();
    s.push_str(&format!("  IrBlock {{ label: {:?},\n", block.label));
    s.push_str("    instructions: [\n");
    for instr in &block.instructions {
        s.push_str("      ");
        s.push_str(&format_instr(instr));
        s.push_str(",\n");
    }
    s.push_str("    ],\n");
    s.push_str(&format!(
        "    terminator: {},\n",
        format_terminator(&block.terminator)
    ));
    s.push_str("  },\n");
    s
}

fn format_instr(instr: &IrInstruction) -> String {
    match instr {
        IrInstruction::PrimitiveConst { temp_id, ty, value } => format!(
            "PrimitiveConst {{ temp_id: {}, ty: {}, value: {} }}",
            t(temp_id),
            format_type(ty),
            format_primitive(value)
        ),
        IrInstruction::LoadParam { temp_id, index, ty } => format!(
            "LoadParam {{ temp_id: {}, index: {}, ty: {} }}",
            t(temp_id),
            index,
            format_type(ty)
        ),
        IrInstruction::Alloca { temp_id, ty } => format!(
            "Alloca {{ temp_id: {}, ty: {} }}",
            t(temp_id),
            format_type(ty)
        ),
        IrInstruction::Load { temp_id, ty, addr } => format!(
            "Load {{ temp_id: {}, ty: {}, addr: {} }}",
            t(temp_id),
            format_type(ty),
            t(addr)
        ),
        IrInstruction::Store { ty, value, addr } => format!(
            "Store {{ ty: {}, value: {}, addr: {} }}",
            format_type(ty),
            t(value),
            t(addr)
        ),
        IrInstruction::Binary {
            temp_id,
            ty_lr,
            ty_res,
            op,
            lhs,
            rhs,
        } => format!(
            "Binary {{ temp_id: {}, ty_lr: {}, ty_res: {}, op: {}, lhs: {}, rhs: {} }}",
            t(temp_id),
            format_type(ty_lr),
            format_type(ty_res),
            format_binary_op(op),
            t(lhs),
            t(rhs)
        ),
        IrInstruction::Call {
            temp_id,
            function_name,
            args,
            return_type,
        } => {
            let args_s: Vec<String> = args.iter().map(t).collect();
            format!(
                "Call {{ temp_id: {}, function_name: {:?}, args: [{}], return_type: {} }}",
                t(temp_id),
                function_name,
                args_s.join(", "),
                format_type(return_type)
            )
        }
        IrInstruction::GlobalAddr { temp_id, name, ty } => format!(
            "GlobalAddr {{ temp_id: {}, name: {:?}, ty: {} }}",
            t(temp_id),
            name,
            format_type(ty)
        ),
        IrInstruction::Asm { code } => {
            // mehrzeiliger code wird ueber ein triple-quoted block dargestellt, damit
            // jede zeile sichtbar ist und keine zeichen verschluckt werden.
            let mut s = String::from("Asm { code: \"\"\"\n");
            for line in code.lines() {
                s.push_str("        ");
                s.push_str(line);
                s.push('\n');
            }
            s.push_str("      \"\"\" }");
            s
        }
        IrInstruction::Unary {
            temp_id,
            ty,
            op,
            value,
        } => format!(
            "Unary {{ temp_id: {}, ty: {}, op: {}, value: {} }}",
            t(temp_id),
            format_type(ty),
            format_unary_op(op),
            t(value)
        ),
        IrInstruction::InitVariant {
            temp_id,
            ty,
            case_name,
        } => format!(
            "InitVariant {{ temp_id: {}, ty: {}, case_name: {:?} }}",
            t(temp_id),
            format_type(ty),
            case_name
        ),
        IrInstruction::GetFieldAddr {
            temp_id,
            base_addr,
            field_name,
        } => format!(
            "GetFieldAddr {{ temp_id: {}, base_addr: {}, field_name: {:?} }}",
            t(temp_id),
            t(base_addr),
            field_name
        ),
    }
}

fn format_terminator(term: &Terminator) -> String {
    match term {
        Terminator::Ret(Some(id)) => format!("Ret(Some({}))", t(id)),
        Terminator::Ret(None) => "Ret(None)".to_string(),
        Terminator::Jump { target } => format!("Jump {{ target: {:?} }}", target),
        Terminator::Branch {
            condition,
            then_block,
            else_block,
        } => format!(
            "Branch {{ condition: {}, then_block: {:?}, else_block: {:?} }}",
            t(condition),
            then_block,
            else_block
        ),
    }
}

fn format_binary_op(op: &IrBinaryOp) -> &'static str {
    match op {
        IrBinaryOp::Add => "Add",
        IrBinaryOp::Sub => "Sub",
        IrBinaryOp::Mul => "Mul",
        IrBinaryOp::Div => "Div",
        IrBinaryOp::Mod => "Mod",
        IrBinaryOp::Eq => "Eq",
        IrBinaryOp::Ne => "Ne",
        IrBinaryOp::Lt => "Lt",
        IrBinaryOp::Le => "Le",
        IrBinaryOp::Gt => "Gt",
        IrBinaryOp::Ge => "Ge",
        IrBinaryOp::And => "And",
        IrBinaryOp::Or => "Or",
        IrBinaryOp::BitAnd => "BitAnd",
        IrBinaryOp::BitOr => "BitOr",
        IrBinaryOp::BitXor => "BitXor",
        IrBinaryOp::Shl => "Shl",
        IrBinaryOp::Shr => "Shr",
    }
}

fn format_unary_op(op: &IrUnaryOp) -> &'static str {
    match op {
        IrUnaryOp::Neg => "Neg",
        IrUnaryOp::Not => "Not",
        IrUnaryOp::BitNot => "BitNot",
    }
}

fn format_type(ty: &IrType) -> String {
    match ty {
        IrType::I64 => "I64".to_string(),
        IrType::Bool => "Bool".to_string(),
        IrType::Void => "Void".to_string(),
        IrType::F64 => "F64".to_string(),
        IrType::Char => "Char".to_string(),
        IrType::Named(name) => format!("Named({:?})", name),
        IrType::Pointer(inner) => format!("Pointer({})", format_type(inner)),
        IrType::Null => "Null".to_string(),
    }
}

fn format_primitive(value: &PrimitiveValue) -> String {
    match value {
        PrimitiveValue::Int(v) => format!("Int({})", v),
        PrimitiveValue::Float(v) => format!("Float({})", v),
        PrimitiveValue::Bool(v) => format!("Bool({})", v),
        PrimitiveValue::Char(v) => format!("Char({:?})", v),
        PrimitiveValue::Pointer(id) => format!("Pointer({})", t(id)),
        PrimitiveValue::Null => "Null".to_string(),
    }
}

fn t(id: &TempId) -> String {
    format!("%{}", id.0)
}
