## Zusammenfassung

Stand: 2026-05-23, nach erneutem Lauf der aktiven Cargo-Tests.

Bewerteter Compilerpfad:

```text
Rython source -> Lexer -> Parser/AST -> IR
```

Der Assembly-/Link-/Run-Pfad ist weiterhin nicht Teil dieser Bewertung.

Aktuell sichtbar:

- Der `rython_to_ir`-Test-Build bricht bereits beim Kompilieren ab, weil `lexer_semantics_tests::lexes_supported_keywords` die Token-Varianten `TokenKind::And`, `TokenKind::Or` und `TokenKind::Not` erwartet, diese Varianten in `lexer.rs` aber auskommentiert sind. Dadurch laufen die fachlichen Rython->IR-Regressionstests aktuell nicht bis zur Ausfuehrung.
- Field-Access auf Struct-Rvalues und Call-Ergebnissen funktioniert im Codegen nicht.
- `loop { return ... }` in nicht-void Funktionen erzeugt einen unerreichbaren, aber unterminierten `loop_end` Block.
- Prefixed Integer-Literale (`0x`, `0b`, `0o`) lexen korrekt, werden aber im Codegen/Const-Evaluator nicht geparst.
- Shadowing ist erlaubt, aber Namensauflösung bevorzugt aktuell `const`/`global` vor lokalen Bindings.
- `operator []` prueft den Index-Argumenttyp nicht und erzeugt inkonsistente IR.
- `a::;` panickt im Parser durch `unimplemented!()` statt einen `ParseError` zu liefern.
- Doppelte Parameter, Struct-Felder und Variant-Cases werden akzeptiert.
- Methoden akzeptieren `this` an falscher Parameterposition.
- Token-Spans und CLI-Token-Slices sind bei Unicode-Literalen falsch.
- `and` und `or` werden eager als Binary-IR erzeugt; gewuenscht ist Short-Circuit-Control-Flow.
- Parser-only Features wie `any` koennen im Codegen panicken statt sauber als unsupported/invalid zu fehlschlagen.
- Malformed Float-Exponents wie `1e` werden nicht als einzelner Lexer-Fehler erkannt.
- CLI `--emit-ir` schreibt aktuell nicht auf denselben Stream wie die anderen Emit-Ausgaben.

Nicht mehr als aktive Bug-Skripte gefuehrt:

- Alter binaerer Operator-Overload-Bug (`b + true` bei `rhs: int`) ist fuer normale binaere Operatoren gefixt.
- `null` wurde vollstaendig aus Rython entfernt. Das alte `null`-Bug-Skript wurde geloescht; `null` existiert nur noch in negativen Tests.

## Aktueller Teststatus

Verifiziert mit:

```text
cargo test -p rython_to_ir
cargo test -p manager
cargo test -p rython_cli
```

Ergebnis am 2026-05-23:

- `cargo test -p manager`: gruen, 14/14 Tests.
- `cargo test -p rython_cli`: rot, 11/13 Tests gruen.
- `cargo test -p rython_to_ir`: rot vor Testausfuehrung; Testcrate kompiliert nicht wegen fehlender `TokenKind::And`, `TokenKind::Or`, `TokenKind::Not`.

Die roten bzw. nicht kompilierenden Tests sind absichtlich fachliche Regressionstests. Sie wurden nicht an das aktuelle fehlerhafte Verhalten angepasst.

Aktuelle `rython_cli`-Fehler:

```text
rython_cli_tests::emit_tokens_ast_and_ir_print_stable_markers_to_stderr
rython_cli_tests::emit_tokens_handles_unicode_literal_contents_without_unicode_identifiers
```

Aktueller `rython_to_ir`-Buildfehler:

```text
error[E0599]: no variant or associated item named `And` found for enum `TokenKind`
error[E0599]: no variant or associated item named `Or` found for enum `TokenKind`
error[E0599]: no variant or associated item named `Not` found for enum `TokenKind`
```

Bis dieser Buildfehler behoben ist, ist die alte Zahl `56/73 Tests gruen` nicht mehr belastbar.

## Vorgelagerter Test-Build-Blocker

### Build-Blocker 0: Keyword-Tests erwarten `and`/`or`/`not` als eigene Token

Prioritaet: Sehr hoch
Bereich: Lexer/Testvertrag

Aktueller Test:

```text
lexer_semantics_tests::lexes_supported_keywords
```

Tatsaechlich:

- `TokenKind::{And, Or, Not}` sind in `crates/rython_to_ir/src/lexer.rs` auskommentiert.
- Die Tests referenzieren diese Varianten direkt und verhindern dadurch den kompletten `rython_to_ir`-Testlauf.

Entscheidung noetig:

- Entweder `and`, `or` und `not` als Keyword-Token wieder einfuehren und Parser/Codegen darauf ausrichten.
- Oder den Testvertrag auf die aktuell verwendeten Operator-Token `&&`, `||`, `!` zuruecknehmen und die Sprachentscheidung zu ausgeschriebenen Bool-Operatoren entfernen.

## Gueltige Sprachentscheidungen fuer Tests

- Shadowing ist erlaubt. Innere Bindings muessen korrekt aufgeloest werden; nach Verlassen des inneren Scopes gilt wieder die aeussere Bindung.
- `and` und `or` muessen Short-Circuit-Semantik haben.
- `null` ist kein Rython-Feature mehr.
- Identifier sind absichtlich ASCII-only; Unicode ist in String-/Char-Literalen erlaubt.
- Der aktuelle Span-Kontrakt ist char-index-basiert und wird in Tests beibehalten.
- Methoden ohne `this` sind statische Methoden.
- Parser-Support und IR-Support fuer zukuenftige Features werden getrennt bewertet.
- `for`, imports, traits, impls, generics, `any` und top-level operator overloads sollen aktuell nicht als vollstaendig IR-unterstuetzt getestet werden; Codegen muss dafuer aber sauber fehlschlagen statt zu panicken.
- CLI ohne Argumente soll Exit Code 0 liefern.

## Aktive Bugs

### Bug 1: Field-Access funktioniert nur auf Lvalues

Prioritaet: Hoch
Bereich: AST/Semantik/Codegen
Beispiel: `examples/bugs/bug_nr_5_field_access_on_struct_rvalue_fails.ry`

Minimales Beispiel:

```source
struct Point { x: int }

fn make() Point {
    return Point { x: 41 };
}

fn main() int {
    return make().x + Point { x: 1 }.x;
}
```

Erwartet:

- Field-Access auf Struct-Rvalues und Call-Ergebnissen ist gueltig.
- Codegen berechnet die Feldadresse und laedt den Wert.

Tatsaechlich:

```text
InvalidExpr(Call { callee: Variable("make"), type_args: [], arguments: [] })
```

Aktiver Test:

```text
ir_codegen_semantics_tests::field_access_on_struct_rvalues_and_call_results_is_valid
```

Fix-Richtung:

- Lvalue-Adresslogik fuer Zuweisungsziele von lesendem Field-Access trennen.
- Fuer lesenden Zugriff zuerst `gen_expr(object)` ausfuehren und bei `Pointer(Named(...))` direkt diese Basisadresse verwenden.

### Bug 2: `loop { return ... }` erzeugt unterminierten unreachable Endblock

Prioritaet: Hoch
Bereich: Kontrollfluss/IR-Codegen
Beispiel: `examples/bugs/bug_nr_6_loop_return_creates_unterminated_unreachable_end_block.ry`

Minimales Beispiel:

```source
fn main() int {
    loop {
        return 1;
    }
}
```

Erwartet:

- Das Programm ist gueltig, weil jeder erreichbare Pfad returned.

Tatsaechlich:

```text
MissingTerminator("loop_end_1:")
```

Aktiver Test:

```text
ir_codegen_semantics_tests::loop_with_unconditional_return_is_valid_in_non_void_function
```

Fix-Richtung:

- `loop_end` nur erzeugen, wenn er erreichbar ist.
- Alternativ Reachability in `finish_blocks` beruecksichtigen.

### Bug 3: Prefixed Integer-Literale funktionieren nicht bis zum IR-Codegen

Prioritaet: Hoch
Bereich: Lexer/Codegen/Const-Eval
Beispiel: `examples/bugs/bug_nr_3_prefixed_integer_literal_rejected_by_codegen.ry`

Minimales Beispiel:

```source
const ten: int = 0b1010;
global mask: int = 0xFF;

fn main() int {
    return ten + mask + 0o7;
}
```

Erwartet:

- `0b1010 -> 10`
- `0xFF -> 255`
- `0o7 -> 7`

Tatsaechlich:

```text
InvalidIntLiteral("0b1010")
InvalidIntLiteral("0x10")
```

Aktive Tests:

```text
ir_codegen_semantics_tests::prefixed_integer_literals_work_in_globals_consts_and_expressions
pipeline_regression_tests::bug_regression_prefixed_integer_literals_reach_ir_as_values
```

Fix-Richtung:

- Gemeinsame Integer-Parser-Funktion fuer Dezimal, `0x`, `0b`, `0o`.
- In `gen_intliteral` und `eval_const_expr` verwenden.

### Bug 4: Shadowing-Aufloesung bevorzugt `const`/`global` vor lokalen Bindings

Prioritaet: Hoch
Bereich: Semantik/Symbol Resolution
Beispiel: `examples/bugs/bug_nr_2_local_variable_shadowed_by_const_on_read.ry`

Gueltige Testsemantik:

- Shadowing ist erlaubt.
- Ein lokales Binding muss bei Namensauflösung Vorrang vor aeusseren Bindings, Globals und Constants haben.

Minimales Beispiel:

```source
const x: int = 1;

fn main() int {
    let x: int = 2;
    return x;
}
```

Erwartet:

- `return x` liest das lokale `x` und returned `2`.

Tatsaechlich:

- Codegen liest aktuell die Konstante `x` und returned `1`.

Aktive Tests:

```text
ir_codegen_semantics_tests::local_shadowing_of_global_const_reads_the_local_binding
pipeline_regression_tests::bug_regression_local_shadowing_must_resolve_to_the_local_binding_not_the_const
```

Fix-Richtung:

- Lookup-Reihenfolge korrigieren: lokale Scopes zuerst, dann global/const.
- Gleiche Namen in inneren Scopes duerfen unterschiedliche Adressen/Temps behalten.

### Bug 5: `operator []` prueft Argumenttypen nicht

Prioritaet: Hoch
Bereich: Operator-Overloads/Type Checking

Minimales Beispiel:

```source
struct Box {
    value: int,
    fn operator [] get(this, index: int) int {
        return this.value + index;
    }
}

fn main() int {
    let b: Box = Box { value: 1 };
    return b[true];
}
```

Erwartet:

- Fehler, weil der Index `int` sein muss.

Tatsaechlich:

- Codegen erzeugt einen Call `Box_get(..., Bool)` und akzeptiert falsche IR.

Aktiver Test:

```text
ir_codegen_semantics_tests::index_operator_overloads_check_index_argument_types
```

Fix-Richtung:

- Im `PostFixOp::Brackets`-Pfad dieselbe Argumentanzahl- und Typpruefung wie bei normalen Calls und Method-Calls anwenden.

### Bug 6: Ungueltige `::`-Syntax panickt im Parser

Prioritaet: Hoch
Bereich: Parser/Fehlerbehandlung
Beispiel: `examples/bugs/bug_nr_7_invalid_coloncolon_syntax_panics_parser.ry`

Minimales Beispiel:

```source
fn main() {
    a::;
}
```

Erwartet:

- `ParseError`, keine Panic.

Tatsaechlich:

```text
not implemented
```

Aktive Tests:

```text
parser_ast_semantics_tests::malformed_double_colon_syntax_returns_parse_error_without_panic
parser_ast_semantics_tests::malformed_parser_inputs_return_errors_instead_of_panicking
```

Fix-Richtung:

- Den `TokenKind::ColonColon`-Postfix-Arm bis Turbofish-Implementierung mit einem normalen `ParseError` ersetzen.

### Bug 7: Doppelte Parameter, Struct-Felder und Variant-Cases werden akzeptiert

Prioritaet: Mittel
Bereich: Parser/Semantik
Beispiel: `examples/bugs/bug_nr_8_duplicate_params_fields_and_variant_cases_accepted.ry`

Minimale Beispiele:

```source
fn pick(x: int, x: int) int { return x; }
struct Bad { x: int, x: bool }
variant V { A, A }
```

Erwartet:

- Duplicate-Fehler in derselben Parameterliste, demselben Struct und derselben Variant.

Tatsaechlich:

- Parser akzeptiert die Konstrukte.

Aktiver Test:

```text
parser_ast_semantics_tests::duplicate_parameters_fields_and_variant_cases_are_semantic_errors
```

Fix-Richtung:

- Beim Parsen oder in einem Semantik-Pass pro Namensraum ein `HashSet` fuehren.

### Bug 8: `this` wird an falscher Parameterposition akzeptiert

Prioritaet: Mittel
Bereich: Parser/Semantik/Method-Calls
Beispiel: `examples/bugs/bug_nr_9_this_parameter_position_in_method_is_inconsistent.ry`

Gueltige Testsemantik:

- Methoden ohne `this` sind statisch.
- Instanzmethoden duerfen `this` verwenden, aber nur als erster Parameter.

Minimales Beispiel:

```source
struct S {
    x: int,
    fn bad(value: int, this) int {
        return value;
    }
}
```

Erwartet:

- Parse-/Semantikfehler, weil `this` nicht an Position 0 steht.

Tatsaechlich:

- Parser akzeptiert `this` nach anderen Parametern.

Aktive Tests:

```text
parser_ast_semantics_tests::this_parameter_is_only_valid_as_first_method_parameter
pipeline_regression_tests::bug_regression_method_this_parameter_position_is_validated_before_codegen_call_mismatch
```

Fix-Richtung:

- Parser/Semantik muss `this` ausserhalb von Position 0 ablehnen.

### Bug 9: Unicode-Spans und CLI-Token-Slices sind falsch

Prioritaet: Mittel
Bereich: Lexer/Diagnostics/CLI
Beispiel: `examples/bugs/bug_nr_10_unicode_token_spans_are_wrong.ry`

Minimales Beispiel:

```source
fn main() {
    let c: char = 'ä';
    let s: string = "Grüße";
}
```

Erwartet:

- Unicode-Inhalte sind in Char-/String-Literalen erlaubt.
- Token-Spans rekonstruieren die korrekten Source-Slices unter dem aktuellen char-index-basierten Span-Kontrakt.
- `--emit-tokens` verrutscht oder panickt nicht.

Tatsaechlich:

- Der Span-Test rekonstruiert fuer das Char-Literal aktuell `"ä'"` statt `"'ä'"`.
- CLI-Token-Ausgabe kann bei Unicode-Inhalten fehlschlagen.

Aktive Tests:

```text
lexer_semantics_tests::token_spans_reconstruct_unicode_literal_source_slices
rython_cli_tests::emit_tokens_handles_unicode_literal_contents_without_unicode_identifiers
```

Fix-Richtung:

- Span-Laengen fuer Literale nach dem bestehenden Projektvertrag konsistent berechnen.
- CLI darf String-Slices nicht mit ungueltigen Char-/Byte-Index-Mischungen bilden.

### Bug 10: `and` und `or` haben keine Short-Circuit-IR

Prioritaet: Hoch
Bereich: Codegen/IR-Kontrollfluss
Beispiel: `examples/bugs/bug_nr_11_short_circuit_and_or_lowered_eagerly.ry`

Minimales Beispiel:

```source
fn rhs() bool { return true; }
fn main(left: bool) bool { return left and rhs(); }
```

Erwartet:

- `and` und `or` werden als Control-Flow mit Branches lowering.
- Die rechte Seite wird nur ausgewertet, wenn die Sprache das verlangt.

Tatsaechlich:

- Codegen erzeugt eager `Binary And`/`Binary Or` und wertet beide Seiten aus.

Aktive Tests:

```text
ir_codegen_semantics_tests::short_circuit_and_is_lowered_to_control_flow_not_eager_binary_and
ir_codegen_semantics_tests::short_circuit_or_is_lowered_to_control_flow_not_eager_binary_or
```

Fix-Richtung:

- `and`/`or` im Expr-Codegen gesondert behandeln.
- Branch-Blocks fuer RHS-Auswertung und Merge erzeugen.

### Bug 11: Parser-only Feature `any` panickt im IR-Codegen

Prioritaet: Mittel
Bereich: Unsupported-Feature-Handling
Beispiel: `examples/bugs/bug_nr_12_any_trait_type_codegen_panics.ry`

Minimales Beispiel:

```source
fn main(value: any Display) {
    return;
}
```

Erwartet:

- Solange `any` nicht IR-unterstuetzt ist, muss Codegen sauber mit einem `CodegenError` fehlschlagen.

Tatsaechlich:

- `convert_to_ir_type(Type::AnyTrait(_))` erreicht `todo!()` und panickt.

Aktiver Test:

```text
ir_codegen_semantics_tests::unsupported_parser_only_features_fail_cleanly_during_ir_codegen
```

Fix-Richtung:

- Eigene Fehlerart fuer unsupported Features einfuehren oder vorhandene `InvalidItem`/`UnknownType` konsistent verwenden.

### Bug 12: Malformed Float-Exponents werden nicht als Lexer-Fehler erkannt

Prioritaet: Mittel
Bereich: Lexer/Fehlerqualitaet
Beispiel: `examples/bugs/bug_nr_13_malformed_float_exponent_is_not_lex_error.ry`

Minimales Beispiel:

```source
fn main() float {
    return 1e;
}
```

Erwartet:

- `1e`, `1e+`, `1e-` und aehnliche Formen werden als ein `InvalidNumber`-Lexerfehler erkannt.

Tatsaechlich:

- Lexer tokenisiert `1e` als `1` und Identifier `e`; der Fehler wird erst im Parser sichtbar.

Aktiver Test:

```text
lexer_semantics_tests::malformed_float_exponents_report_number_errors_instead_of_split_tokens
```

Fix-Richtung:

- Wenn nach einer Zahl `e`/`E` folgt, muss der Lexer exponentielle Notation entweder vollstaendig validieren oder als `InvalidNumber` melden.

### Bug 13: CLI `--emit-ir` schreibt auf stdout statt stderr

Prioritaet: Niedrig bis Mittel
Bereich: CLI/Manager-Ausgabe

CLI-Usage sagt:

```text
--emit-ir          Print IR module to stderr
```

Erwartet:

- Alle `--emit-*` Debug-Ausgaben gehen konsistent nach stderr.

Tatsaechlich:

- Tokens und AST gehen nach stderr.
- IR geht ueber `print_ir` nach stdout.

Aktiver Test:

```text
rython_cli_tests::emit_tokens_ast_and_ir_print_stable_markers_to_stderr
```

Fix-Richtung:

- `emit_ir` im Manager mit `eprint!`/`eprintln!` oder einem stream-parametrisierten Formatter ausgeben.

## Gefixte oder entfernte Bug-Skripte

### Entfernt: binaerer Operator-Overload-RHS-Typ

Ehemaliges Skript:

```text
examples/bugs/bug_nr_1_operator_overload_argument_type_not_checked.ry
```

Grund:

- Der urspruengliche Fall `b + true` bei `rhs: int` wird inzwischen korrekt als `MismatchedTypes(I64, Bool)` abgelehnt.
- Aktive Tests bleiben:

```text
ir_codegen_semantics_tests::operator_overloads_check_argument_types_like_normal_calls
pipeline_regression_tests::bug_regression_operator_overload_mismatched_rhs_does_not_emit_bad_call_ir
```

Der verwandte `operator []`-Bug bleibt aktiv, weil dort der Index-Typ noch nicht geprueft wird.

### Entfernt: `null`

Ehemaliges Skript:

```text
examples/bugs/bug_nr_4_null_not_assignable_to_struct_pointer_type.ry
```

Grund:

- `null` ist kein Rython-Feature mehr.
- `null` wird nur noch negativ getestet:

```text
ir_codegen_semantics_tests::negative_codegen_reports_specific_name_type_and_struct_literal_errors
manager_tests::run_rejects_removed_null_expression_during_ir_codegen
rython_cli_tests::removed_null_expression_is_reported_as_ir_error
```

## Aktive Bug-Skripte

```text
examples/bugs/bug_nr_2_local_variable_shadowed_by_const_on_read.ry
examples/bugs/bug_nr_3_prefixed_integer_literal_rejected_by_codegen.ry
examples/bugs/bug_nr_5_field_access_on_struct_rvalue_fails.ry
examples/bugs/bug_nr_6_loop_return_creates_unterminated_unreachable_end_block.ry
examples/bugs/bug_nr_7_invalid_coloncolon_syntax_panics_parser.ry
examples/bugs/bug_nr_8_duplicate_params_fields_and_variant_cases_accepted.ry
examples/bugs/bug_nr_9_this_parameter_position_in_method_is_inconsistent.ry
examples/bugs/bug_nr_10_unicode_token_spans_are_wrong.ry
examples/bugs/bug_nr_11_short_circuit_and_or_lowered_eagerly.ry
examples/bugs/bug_nr_12_any_trait_type_codegen_panics.ry
examples/bugs/bug_nr_13_malformed_float_exponent_is_not_lex_error.ry
```

Die Nummern behalten historische Luecken, damit alte Referenzen nicht still umgedeutet werden.

Hinweis Stand 2026-05-23: Im Arbeitsbaum existiert aktuell kein `examples/bugs/`-Verzeichnis. Die obige Liste beschreibt die historischen/gewollten Bug-Skripte, nicht vorhandene Dateien.

## Nicht als Bug gezaehlt

- Assembly-Backend, Linken und Programmausfuehrung sind nicht Teil des aktuellen Testziels.
- `-o` erzeugt aktuell kein Binary; das ist fuer Source->IR nicht als Fehler bewertet.
- Programm-Exit-Codes werden nicht propagiert, solange der Backend-/Run-Pfad deaktiviert ist.
- Imports, Traits, Trait-Implementations, Generics, `any`, `for` und top-level operator overloads sind aktuell hoechstens Parser-Features. Erfolgreicher IR-Codegen dafuer ist noch nicht erwartet; sauberes Fehlschlagen ist aber erwartet.
- Vollstaendige Runtime-/Builtin-Semantik fuer String- und List-Literale ist noch nicht Teil der Bewertung.

## Naechste sinnvolle Fix-Reihenfolge

1. Testvertrag fuer `and`/`or`/`not` klaeren und `rython_to_ir` wieder kompilierbar machen.
2. Parser-Panic bei `::` entfernen, weil das ein klarer Stabilitaetsfehler ist.
3. Integer-Prefix-Parsing zentralisieren, weil Lexer und Sprachsyntax das Feature bereits akzeptieren.
4. Lokale Scope-Lookup-Reihenfolge korrigieren, damit Shadowing semantisch stimmt.
5. `operator []` an normale Call-Typpruefung angleichen.
6. Short-Circuit-IR fuer `and`/`or` implementieren, falls ausgeschriebene Bool-Operatoren Teil der Sprache bleiben.
7. `any` und andere unsupported Parser-Features in saubere `CodegenError`s umwandeln.
8. Unicode-Span-Berechnung und CLI-Token-Slicing stabilisieren.
9. `--emit-ir` auf denselben Debug-Ausgabestream wie Tokens/AST bringen.
