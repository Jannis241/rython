## Compiler Bug Report

Stand: 2026-05-23

Bewerteter Pfad:

```text
Rython source -> Lexer -> Parser/AST -> IR -> CLI/Manager-Ausgabe
```

Der IR-to-Assembly-Pfad ist nicht Teil dieses Reports.

Geklaerte Sprachentscheidung:

- Gueltige Bool-Operator-Syntax ist `&&`, `||` und `!`.
- Die ausgeschriebenen Woerter `and`, `or` und `not` sind keine Rython-Operatoren.
- Tests, Beispiele oder Bugbeschreibungen, die `and`, `or` oder `not` als Operatoren erwarten, sind veraltet.

## Aktueller Teststatus

Verifiziert mit:

```text
cargo test
cargo test -p rython_to_ir --lib
cargo test -p manager
cargo test -p rython_cli
cargo test -p ir_to_assembly
```

Ergebnis:

- `cargo test`: rot. Der Workspace-Testlauf bricht beim Kompilieren der `rython_to_ir`-Tests ab.
- `cargo test -p rython_to_ir --lib`: gruen, aber es gibt dort aktuell 0 Unit-Tests.
- `cargo test -p manager`: gruen, 14/14 Tests.
- `cargo test -p rython_cli`: rot, 12/13 Tests gruen.
- `cargo test -p ir_to_assembly`: gruen, 2/2 Harness-Tests. Nicht Teil der Bewertung.

## Build-Blocker: veralteter `and/or/not`-Testvertrag

Prioritaet: Sehr hoch
Bereich: Tests/Beispiele/Lexer-Vertrag

Unerwartetes Verhalten:

- `crates/rython_to_ir/tests/rython_ir_tests/lexer_semantics_tests.rs` referenziert `TokenKind::And`, `TokenKind::Or` und `TokenKind::Not`.
- Diese Varianten existieren in `crates/rython_to_ir/src/lexer.rs` nicht; sie sind dort auskommentiert.
- Dadurch kompiliert das Testcrate `rython_ir_tests` nicht, und alle fachlichen Rython->IR-Tests sind blockiert.

Aktueller Fehler:

```text
error[E0599]: no variant or associated item named `And` found for enum `TokenKind`
error[E0599]: no variant or associated item named `Or` found for enum `TokenKind`
error[E0599]: no variant or associated item named `Not` found for enum `TokenKind`
```

Erwartet:

- Die Keyword-Tests entfernen `and`, `or`, `not` aus der Keyword-Liste.
- Operator-Tests verwenden `TokenKind::AmpAmp`, `TokenKind::PipePipe` und `TokenKind::Bang`.
- Quellen in Tests und Beispielen verwenden `&&`, `||` und `!`.

Nicht erwartet:

- `TokenKind::And`, `TokenKind::Or` oder `TokenKind::Not` wieder einzufuehren.

Zusatzbefund:

- `examples/current_features.ry`, `examples/all_features.ry` und mehrere IR-Codegen-Tests verwenden noch `and`/`or`.
- Mit der geklaerten Syntax sind diese Quellen veraltet und muessen auf `&&`/`||` umgestellt werden.

## Bug 1: `&&` und `||` werden eager statt short-circuit ausgewertet

Prioritaet: Hoch
Bereich: IR-Codegen/Kontrollfluss

Minimales Beispiel:

```source
fn rhs() bool { return true; }
fn main(left: bool) bool { return left && rhs(); }
```

Erwartet:

- `&&` wertet die rechte Seite nur aus, wenn die linke Seite `true` ist.
- `||` wertet die rechte Seite nur aus, wenn die linke Seite `false` ist.
- Das IR enthaelt Branch-/Merge-Blocks statt eines eager `Binary And` oder `Binary Or`.

Tatsaechlich:

- `gen_binary_op` ruft zuerst `gen_expr(lhs)` und danach immer `gen_expr(rhs)` auf.
- Danach wird fuer primitive Bool-Operatoren ein `IrInstruction::Binary { op: And/Or, ... }` erzeugt.
- Dadurch wuerden Seiteneffekte oder Fehler in der rechten Seite auftreten, obwohl sie durch Short-Circuiting uebersprungen werden muessten.

Fix-Richtung:

- `BinaryOp::And` und `BinaryOp::Or` vor der normalen Binary-Codegen-Logik gesondert behandeln.
- RHS-Block und Merge-Block erzeugen.
- Tests von `and/or` auf `&&/||` umstellen.

## Bug 2: Lokales Shadowing von `const`/`global` funktioniert nicht korrekt

Prioritaet: Hoch
Bereich: Namensaufloesung/Scopes

Minimales Beispiel:

```source
const x: int = 1;

fn main() int {
    let x: int = 2;
    return x;
}
```

Erwartet:

- Shadowing ist erlaubt.
- `return x` liest das lokale Binding und returned `2`.
- Nach Verlassen eines inneren Blocks gilt wieder das aeussere Binding.

Tatsaechlich:

- `gen_let` lehnt lokale Variablen ab, wenn bereits ein `const` oder `global` mit demselben Namen existiert.
- Zusaetzlich sucht `gen_variable` aktuell zuerst in `module.constants`, danach in `module.globals` und erst danach in lokalen Scopes.
- Selbst wenn `gen_let` Shadowing zulassen wuerde, wuerde ein lokales Binding beim Lesen hinter `const`/`global` verlieren.

Fix-Richtung:

- Lokale Scopes zuerst aufloesen, danach `const` und `global`.
- Shadowing innerhalb innerer Scopes erlauben.
- Nur doppelte Namen im selben Scope verbieten.

## Bug 3: `while` und `loop` sind als normale Statements nicht stabil

Prioritaet: Hoch
Bereich: Parser/IR-Codegen/Kontrollfluss

Minimales Beispiel:

```source
fn main(limit: int) int {
    let x: int = 0;
    while x < limit {
        x += 1;
    }
    return x;
}
```

Erwartet:

- `while condition { ... }` und `loop { ... }` koennen wie normale Kontrollfluss-Statements ohne Semikolon verwendet werden.
- Break/continue funktionieren innerhalb der Schleife.
- Der IR-Codegen erzeugt terminierte Condition-, Body- und End-Blocks.

Tatsaechlich:

- `While` und `Loop` sind aus `is_statement_start` auskommentiert.
- Der Parser behandelt sie als Expression-Statements und erwartet danach ein Semikolon.
- Beispiele wie `examples/current_features.ry` schlagen deshalb bereits im Parser fehl.
- `gen_loop` und `gen_while` in `codegen/expr.rs` enthalten `todo!()` und wuerden bei erreichter Codegen-Stelle panicken.

Fix-Richtung:

- Entscheiden und dann konsistent umsetzen: Schleifen als Statements oder echte Expressions mit sauberem Blockwert.
- Fuer den aktuellen Test-/Beispielvertrag: `while` und `loop` wieder als Statement-Starts behandeln.
- `gen_while` und `gen_loop` ohne Panic implementieren.

## Bug 4: `loop { return ... }` in nicht-void Funktionen ist nicht sauber abgedeckt

Prioritaet: Hoch
Bereich: Kontrollfluss/Terminatoren

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
- Es wird kein erreichbarer, unterminierter Endblock erzeugt.

Tatsaechlich:

- Der Parser behandelt `loop` aktuell als Expression-Statement und verlangt nach dem Block ein Semikolon, obwohl die Beispiele und Tests `loop { ... }` als normales Statement verwenden.
- Wenn der Codegen fuer `loop` erreicht wird, landet er in `gen_loop` bei `todo!()` und panickt.
- Der aktive Testvertrag erwartet, dass dieser Fall gueltiges, voll terminiertes IR erzeugt.

Fix-Richtung:

- `loop_end` nur erzeugen, wenn er erreichbar ist.
- Alternativ Reachability in `finish_blocks` korrekt berechnen, bevor `MissingTerminator` gemeldet wird.

## Bug 5: Doppelte Namen werden nicht frueh und konsistent abgelehnt

Prioritaet: Mittel
Bereich: Parser/Semantik

Minimale Beispiele:

```source
fn pick(x: int, x: int) int { return x; }
struct Bad { x: int, x: bool }
variant V { A, A }
```

Erwartet:

- Doppelte Parameter in derselben Parameterliste werden abgelehnt.
- Doppelte Felder im selben Struct werden abgelehnt.
- Doppelte Cases in derselben Variant werden abgelehnt.
- Der Fehler entsteht deterministisch in Parser oder Semantik-Pass, bevor spaeterer Codegen Seiteneffekte erzeugt.

Tatsaechlich:

- Der Parser akzeptiert diese Konstrukte.
- Einige Duplikate werden spaeter im Codegen entdeckt, aber nicht konsistent frueh.
- Der Parser-Testvertrag erwartet aktuell `parse_items(...).is_err()`.

Fix-Richtung:

- Pro Namensraum beim Parsen oder in einem expliziten Semantik-Pass ein `HashSet` fuehren.
- Fehlerart fuer Duplicate-Parameter, Duplicate-Fields und Duplicate-Variant-Cases klar trennen.

## Bug 6: Parser-only Feature `any` panickt im IR-Codegen

Prioritaet: Mittel
Bereich: Unsupported-Feature-Handling

Minimales Beispiel:

```source
fn main(value: any Display) {
    return;
}
```

Erwartet:

- Solange `any` nicht IR-unterstuetzt ist, muss Codegen sauber mit `CodegenError` fehlschlagen.
- Unsupported Parser-Features duerfen nicht panicken.

Tatsaechlich:

- `convert_to_ir_type(Type::AnyTrait(_))` erreicht `todo!()`.
- Dadurch panickt der Codegen statt einen Fehlerwert zurueckzugeben.

Fix-Richtung:

- `CodegenError::UnsupportedFeature(...)` oder eine passende vorhandene Fehlerart verwenden.
- Dieselbe Regel fuer Generics, Traits, Impl und andere Parser-only Features anwenden.

## Bug 7: Malformed Float-Exponents werden als getrennte Tokens lexed

Prioritaet: Mittel
Bereich: Lexer/Fehlerqualitaet

Minimale Beispiele:

```source
1e
1e+
1e-
2.5e+
```

Erwartet:

- Diese Formen ergeben jeweils einen `LexingError::InvalidNumber`.
- Der Lexer behandelt sie als fehlerhafte Zahl, nicht als mehrere gueltige Tokens.

Tatsaechlich:

- `1e` wird als `Int("1")` plus `Ident("e")` tokenisiert.
- Der eigentliche Fehler erscheint dadurch erst spaeter im Parser und ist weniger praezise.

Fix-Richtung:

- Wenn direkt nach einer Zahl `e` oder `E` folgt, muss der Exponent komplett validiert oder als `InvalidNumber` gemeldet werden.

## Bug 8: `--emit-ir` schreibt nicht auf denselben Stream wie andere Emit-Ausgaben

Prioritaet: Niedrig bis Mittel
Bereich: CLI/Manager

Erwartet laut CLI-Hilfe:

```text
--emit-ir          Print IR module to stderr
```

Tatsaechlich:

- Tokens und AST gehen nach stderr.
- IR wird ueber `claude_print_ir::print_ir(&module)` nach stdout geschrieben.
- Der Test `emit_tokens_ast_and_ir_print_stable_markers_to_stderr` schlaegt fehl, weil stderr den Marker `==== IR Module ====` nicht enthaelt.

Fix-Richtung:

- IR-Formatierung als String verwenden und mit `eprint!`/`eprintln!` ausgeben.
- Alternativ `print_ir` stream-parametrisieren.

## Gefixte oder entfernte Alt-Bugs

Nicht mehr als aktive Bugs fuehren:

- Field-Access auf Struct-Rvalues und Call-Ergebnissen funktioniert fuer `make().x` und `Point { x: 1 }.x`.
- Prefixed Integer-Literale `0x`, `0b`, `0o` werden in Expr-Codegen und Const-Eval verarbeitet.
- `operator []` prueft den Index-Typ gegen die Operator-Signatur.
- Malformed `a::;` liefert Parserfehler statt Panic.
- `this` nach anderen Methodenparametern wird vom Parser abgelehnt.
- `null` ist kein Rython-Feature mehr.
- CLI-Token-Ausgabe fuer Unicode-Literale wie `'ä'` und `"Grüße"` ist stabil.

## Zu verifizieren, sobald der Test-Build-Blocker weg ist

- Unicode-Span-Test fuer Char-/String-Literale im `rython_to_ir`-Testcrate erneut ausfuehren. Die CLI-Ausgabe ist bereits gruen, der eigentliche Lexer-Test ist aktuell durch den Build-Blocker verdeckt.
- Alle IR-Codegen-Tests nach Umstellung von `and/or` auf `&&/||` erneut ausfuehren; erst danach ist die genaue Zahl der roten Runtime-Semantiktests belastbar.
