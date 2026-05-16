## Zusammenfassung

Stand: aktuelles Testset nach Entfernung von `null`.

Der relevante Compilerpfad ist `Rython source -> Lexer -> Parser/AST -> IR`. Der Assembly-/Link-/Run-Pfad ist weiterhin nicht Bestandteil der Bewertung.

Aktuell sichtbar rote Bugs im aktiven Rython->IR-Pfad:

- Field-Access auf Struct-Rvalues und Call-Ergebnissen funktioniert im Codegen nicht.
- `loop { return ... }` in nicht-void Funktionen erzeugt einen unerreichbaren, aber unterminierten `loop_end` Block.
- Prefixed Integer-Literale (`0x`, `0b`, `0o`) lexen korrekt, werden aber im Codegen/Const-Evaluator nicht geparst.
- Lokale Namen duerfen laut aktueller Testsemantik `const`/`global`/Parameter/gleiche lokale Namen nicht shadowen, werden aber weiterhin akzeptiert oder falsch aufgeloest.
- `operator []` prueft den Index-Argumenttyp nicht und erzeugt inkonsistente IR.
- `a::;` panickt im Parser durch `unimplemented!()` statt einen `ParseError` zu liefern.
- Doppelte Parameter, Struct-Felder und Variant-Cases werden akzeptiert.
- Methoden akzeptieren `this` an falscher Parameterposition.
- Token-Spans und CLI-Token-Slices sind bei Unicode-Literalen falsch.
- CLI `--emit-ir` schreibt aktuell nicht auf denselben Stream wie die anderen Emit-Ausgaben.

`null` ist kein Bug-Thema mehr. Die Tests enthalten keine aktiven `null`-Faelle mehr.

## Aktueller Teststatus

Verifiziert mit:

```text
cargo test -p rython_to_ir
cargo test -p manager
cargo test -p rython_cli
rg "Null|NullLiteral|null" crates/rython_to_ir/tests crates/manager/tests crates/rython_cli/tests -n
```

Ergebnis:

- `cargo test -p manager`: gruen, 10/10 Tests.
- `cargo test -p rython_cli`: rot, 8/9 Tests gruen.
- `cargo test -p rython_to_ir`: rot, 37/50 Tests gruen.
- `rg "Null|NullLiteral|null" ...`: keine Treffer in aktiven Testdateien.

Die roten Tests sind absichtlich fachliche Regressionstests. Sie wurden nicht an das aktuelle fehlerhafte Verhalten angepasst.

## Unterstützte Annahmen

Als aktuell erwartete Spracheigenschaften getestet:

- Lexer fuer Keywords, Identifier, Zahlen, Strings, Chars, Kommentare, Operatoren, Delimiter und `asm { ... }`.
- Parser fuer Funktionen, Structs, Variants, Globals, Consts, Let/Return/If/While/Loop/For/Break/Continue, Blocks, Calls, Field-Access, Struct-Literale, Variant-Literale mit `::`, Zuweisungen, Postfix `++/--`, Index-Syntax, Operator-Prioritaeten, Imports, Traits, Impl-Blöcke, Generics und `any`-Trait-Typen.
- IR-Codegen fuer primitive Werte, lokale Variablen, Parameter, Funktionen, normale Calls, Structs, Struct-Literale, Field-Access auf Lvalues, Methoden, Operator-Overloads an Struct-Methoden, Varianten, globals/consts, If/While/Loop/Break/Continue und Inline-ASM.
- Type-Checking fuer normale Funktionsaufrufe, primitive Operatoren, Return-Typen, Let-Initialisierung und Struct-Felder.
- Shadowing ist verboten: lokale Namen duerfen nicht mit Parametern, anderen lokalen Namen im selben Scope, `const` oder `global` kollidieren.
- `null` existiert in der Testspezifikation nicht mehr.

Nicht als implementiert/erwartet im erfolgreichen IR-Codegen angenommen:

- Vollstaendiger Assembly-Backend/Link/Run-Pfad.
- Erfolgreicher Codegen fuer Imports, Traits, Trait-Impls, Generics, `any`, `for`, Turbofish.
- Vollstaendige Runtime-/Builtin-Semantik fuer String- und List-Literale.

## Gefundene Bugs

### Bug 1: Field-Access funktioniert nur auf Lvalues

Priorität: Hoch  
Bereich: AST/Semantik/Codegen

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

- Der Parser erlaubt Field-Access auf beliebigen Ausdruecken.
- Codegen sollte fuer Struct-Rvalues und Call-Ergebnisse die Feldadresse berechnen und den Wert laden.

Tatsaechlich:

```text
InvalidExpr(Call { callee: Variable("make"), type_args: [], arguments: [] })
```

Aktiver Test:

```text
ir_codegen_semantics_tests::field_access_on_struct_rvalues_and_call_results_is_valid
```

Vermutete Ursache:

- Lesender Field-Access verwendet dieselbe Lvalue-Adresslogik wie Zuweisungsziele.
- `gen_field_addr` ruft `gen_left_value_addr`, das Struct-Literale und Call-Ergebnisse ablehnt.

Fix-Richtung:

- Field-Adresse fuer lesenden Zugriff und Lvalue-Zuweisung trennen.
- Fuer lesenden Zugriff zuerst `gen_expr(object)` ausfuehren und bei `Pointer(Named(...))` direkt diese Basisadresse verwenden.

### Bug 2: `loop { return ... }` erzeugt unterminierten unreachable Endblock

Priorität: Hoch  
Bereich: Kontrollfluss/IR-Codegen

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

Vermutete Ursache:

- `gen_loop` erzeugt immer einen `loop_end` Block, auch wenn der Body sicher terminiert und kein `break` existiert.

Fix-Richtung:

- `loop_end` nur erzeugen, wenn er erreichbar ist.
- Alternativ Reachability in `finish_blocks` beruecksichtigen und unreachable Blocks nicht terminatorpflichtig machen.

### Bug 3: Prefixed Integer-Literale funktionieren nicht bis zum IR-Codegen

Priorität: Hoch  
Bereich: Lexer/Codegen/Const-Eval

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

Vermutete Ursache:

- Lexer erzeugt korrekte `Int`-Tokens mit Prefix.
- Codegen/Const-Eval nutzen danach `value.parse::<i64>()`, was nur Dezimalstrings akzeptiert.

Fix-Richtung:

- Gemeinsame Integer-Parser-Funktion fuer Dezimal, `0x`, `0b`, `0o`.
- In `gen_intliteral` und `eval_const_expr` verwenden.

### Bug 4: Shadowing/Duplicate Names werden nicht sauber verboten

Priorität: Hoch  
Bereich: Semantik/Symbol Resolution

Festgelegte Testsemantik:

- Lokale Namen duerfen `const`/`global` nicht shadowen.
- Lokale Namen duerfen Parameter nicht shadowen.
- Doppelte lokale `let`-Namen im selben Scope sind verboten.

Minimale Beispiele:

```source
const x: int = 1;

fn main() int {
    let x: int = 2;
    return x;
}
```

```source
fn main() int {
    let x: int = 1;
    let x: int = 2;
    return x;
}
```

Erwartet:

- Sauberer Compilerfehler fuer Namenskonflikt.

Tatsaechlich:

- `const`-Fall kompiliert und `return x` liest die Konstante `1` statt die lokale Variable `2`.
- Doppelte lokale Namen im selben Scope werden akzeptiert und der zweite Eintrag ueberschreibt den ersten Scope-Eintrag.

Aktive Tests:

```text
ir_codegen_semantics_tests::local_shadowing_of_params_globals_and_consts_is_rejected
ir_codegen_semantics_tests::duplicate_local_names_in_the_same_scope_are_rejected
pipeline_regression_tests::bug_regression_local_names_must_not_be_resolved_to_globals_or_consts
```

Vermutete Ursache:

- `gen_variable` sucht zuerst in `module.constants`, dann `module.globals`, dann lokale Scopes.
- `insert_variable` nutzt `HashMap::insert` und ignoriert vorhandene Namen.
- `gen_let` prueft aktuell hoechstens einen Teil der globalen Konflikte.

Fix-Richtung:

- Zentrale Name-Conflict-Regel einfuehren.
- Beim Einfuegen lokaler Variablen Parameter und aktuelle Scope-Ebene pruefen.
- Bei `let` gegen sichtbare `const`/`global` pruefen.
- Lookup-Reihenfolge konsistent machen.

### Bug 5: `operator []` prueft Argumenttypen nicht

Priorität: Hoch  
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

- Fehler `MismatchedTypes(I64, Bool)` oder aequivalent, weil der Index `int` sein muss.

Tatsaechlich:

- Codegen erzeugt einen Call `Box_get(..., Bool)` und akzeptiert die falsche IR.

Aktiver Test:

```text
ir_codegen_semantics_tests::index_operator_overloads_check_index_argument_types
```

Vermutete Ursache:

- `gen_postfix` fuer `PostFixOp::Brackets` nutzt die Operator-Signatur nur fuer Return-Typ/Name, aber nicht fuer Argumenttyppruefung.

Fix-Richtung:

- Dieselbe Argumentanzahl- und Typpruefung wie bei normalen Calls und Method-Calls anwenden.

### Bug 6: Ungueltige `::`-Syntax panickt im Parser

Priorität: Hoch  
Bereich: Parser/Fehlerbehandlung

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
thread ... panicked at crates/rython_to_ir/src/parser.rs:358:21:
not implemented
```

Aktiver Test:

```text
parser_ast_semantics_tests::malformed_double_colon_syntax_returns_parse_error_without_panic
```

Fix-Richtung:

- Den `TokenKind::ColonColon`-Postfix-Arm bis Turbofish-Implementierung mit einem normalen `ParseError` ersetzen.

### Bug 7: Doppelte Parameter, Struct-Felder und Variant-Cases werden akzeptiert

Priorität: Mittel  
Bereich: Parser/Semantik

Minimale Beispiele:

```source
fn pick(x: int, x: int) int {
    return x;
}
```

```source
struct Bad {
    x: int,
    x: bool
}
```

```source
variant V {
    A,
    A
}
```

Erwartet:

- Duplicate-Fehler in derselben Parameterliste, demselben Struct und derselben Variant.

Tatsaechlich:

- Parser akzeptiert die Konstrukte.
- Codegen kann mehrdeutige oder ueberschriebene Symbole erzeugen.

Aktiver Test:

```text
parser_ast_semantics_tests::duplicate_parameters_fields_and_variant_cases_are_semantic_errors
```

Fix-Richtung:

- Beim Parsen oder in einem Semantik-Pass pro Namensraum ein `HashSet` fuehren.
- Duplicate-Konflikte als eigene Fehlerart oder als vorhandene geeignete Fehlerart melden.

### Bug 8: `this` wird an falscher Parameterposition akzeptiert

Priorität: Mittel  
Bereich: Parser/Semantik/Method-Calls

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

- `this` ist, falls vorhanden, nur als erster Methodenparameter gueltig.

Tatsaechlich:

- Parser akzeptiert `this` nach anderen Parametern.
- `obj.method(...)` fuegt `this` aber immer als Argument 0 ein.

Aktive Tests:

```text
parser_ast_semantics_tests::this_parameter_is_only_valid_as_first_method_parameter
pipeline_regression_tests::bug_regression_method_this_parameter_position_is_validated_before_codegen_call_mismatch
```

Fix-Richtung:

- Parser/Semantik muss `this` ausserhalb von Position 0 ablehnen.
- Optional zusaetzlich entscheiden, ob Methoden ohne `this` statisch sind oder fuer `obj.method(...)` verboten werden.

### Bug 9: Unicode-Spans und Token-Slices sind falsch

Priorität: Mittel  
Bereich: Lexer/Diagnostics/CLI

Minimales Beispiel:

```source
fn main() {
    let c: char = 'ä';
    let s: string = "Grüße";
}
```

Erwartet:

- Token-Spans rekonstruieren die korrekten Source-Slices.
- `--emit-tokens` verrutscht bei Unicode nicht.

Tatsaechlich:

Aktiver Test rekonstruiert fuer das Char-Token `"char"` statt `"'ä'"`.

Aktiver Test:

```text
lexer_semantics_tests::token_spans_reconstruct_unicode_literal_source_slices
```

Vermutete Ursache:

- Mischung aus char-basierten Indizes und byte-basierten String-Laengen.
- `manager::run` sliced Rust-Strings mit diesen Werten als Byte-Indizes.

Fix-Richtung:

- Span-Modell vereinheitlichen, am besten byte-basierte `[start, end)` Spans.
- CLI-Token-Ausgabe nur mit validen Byte-Spans slicen.

### Bug 10: CLI `--emit-ir` schreibt auf stdout statt stderr

Priorität: Niedrig bis Mittel  
Bereich: CLI/Manager-Ausgabe

Aktueller CLI-Test erwartet fuer `--emit-tokens --emit-ast --emit-ir`, dass alle Emit-Diagnosen/Debug-Ausgaben auf stderr liegen.

Tatsaechlich:

- Tokens und AST erscheinen auf stderr.
- IR wird ueber `print_ir` auf stdout geschrieben.

Aktiver Test:

```text
rython_cli_tests::emit_tokens_ast_and_ir_print_stable_markers_to_stderr
```

Fix-Richtung:

- Entweder `print_ir`/`run` fuer Emit-IR auf stderr umstellen.
- Oder CLI-Hilfe/Tests explizit auf stdout fuer IR umdefinieren. Konsistenter waere stderr fuer alle `--emit-*` Debug-Ausgaben.

## Bereits gefixter oder nicht mehr relevanter Punkt

### Struct-Binary-Operator-Overloads

Der fruehere Bug, dass `b + true` bei einem `operator +` mit `rhs: int` akzeptiert wurde, ist in der aktuellen Codebasis fuer binaere Operatoren offenbar behoben.

Aktive Tests:

```text
ir_codegen_semantics_tests::operator_overloads_check_argument_types_like_normal_calls
pipeline_regression_tests::bug_regression_operator_overload_mismatched_rhs_does_not_emit_bad_call_ir
```

Beide sind gruen. Der analoge Fehler besteht aber weiterhin fuer `operator []`.

### `null`

`null` wurde aus Compiler und Tests entfernt und wird nicht mehr als Bug oder Feature bewertet. Rython soll das laut aktueller Zielrichtung ueber Enums wie `Some`/`None` modellieren.

## Nicht als Bug gezählt

- Assembly-Backend, Linken und Programmausfuehrung sind nicht Teil des aktuellen Testziels.
- `-o` erzeugt aktuell kein Binary; das ist fuer Source->IR nicht als Fehler bewertet.
- Programm-Exit-Codes werden nicht propagiert, solange der Backend-/Run-Pfad deaktiviert ist.
- Imports, Traits, Trait-Implementations, Generics, `any`, `for` und Turbofish werden hoechstens geparst oder als sauberer Fehler erwartet; erfolgreicher IR-Codegen dafuer ist noch nicht getestet.
- String-/List-Literal-Codegen ohne Runtime-Definitionen ist nicht als Bug bewertet.

## Features ohne vollstaendige Tests

Noch nicht oder nur teilweise getestet:

- Erfolgreicher IR-Codegen fuer `for`.
- Erfolgreicher IR-Codegen fuer Imports, Traits, Trait-Implementations, Generics und `any`.
- Turbofish/Call-Type-Args.
- Top-Level-Operator-Overloads als Semantikfeature.
- Unary-Operator-Overloads auf Structs mit falschen Argumenttypen.
- String- und List-Literal-Codegen mit vollstaendiger Runtime-Semantik.
- Short-circuit-Semantik von `and`/`or`; aktuell wird nur IR-Operator-Erzeugung getestet.
- Integer-Grenzwerte inklusive `i64::MIN` als geparstes `-9223372036854775808`.
- Umfassende Parser-Fuzz-/No-Panic-Tests fuer viele kleine Tokenfolgen.
- Detaillierte Source-Span-Tests fuer alle Tokenarten, nicht nur Unicode String/Char.
- Mehrere Dateien/Imports/Module-System.
- Vollstaendige CLI-Verifikation fuer stdout/stderr-Policy, sobald diese bewusst festgelegt ist.

## Testdateien

Aktive Tests fuer den Rython->IR-Pfad:

- `crates/rython_to_ir/tests/rython_ir_tests/common.rs`
- `crates/rython_to_ir/tests/rython_ir_tests/lexer_semantics_tests.rs`
- `crates/rython_to_ir/tests/rython_ir_tests/parser_ast_semantics_tests.rs`
- `crates/rython_to_ir/tests/rython_ir_tests/ir_codegen_semantics_tests.rs`
- `crates/rython_to_ir/tests/rython_ir_tests/pipeline_regression_tests.rs`

Manager/CLI:

- `crates/manager/tests/manager_tests.rs`
- `crates/rython_cli/tests/rython_cli_tests.rs`

Alte ersetzte Tests wurden geloescht:

- `ry_ir_lexer_tests.rs`
- `ry_ir_parser_tests.rs`
- `ry_ir_codegen_tests.rs`
- `ry_ir_pipeline_tests.rs`
