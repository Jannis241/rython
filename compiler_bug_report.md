## Zusammenfassung

Die wichtigsten echten Bugs liegen im bereits aktiven Lexer/Parser/IR-Codegen-Pfad:

- Operator-Overloads erzeugen Calls ohne Argument-Typprüfung und können dadurch ungültige IR akzeptieren.
- Lokale Variablen und Parameter werden beim Lesen von gleichnamigen `const`/`global` Symbolen verdeckt, obwohl sie im Scope existieren.
- Der Lexer unterstützt prefixed Integer-Literale (`0x`, `0b`, `0o`), aber Codegen/Const-Evaluator parsen sie später als Dezimalzahlen.
- `null` ist im Codegen typinkonsistent und praktisch nicht einem Struct-/Pointer-Typ zuweisbar.
- Field-Access funktioniert nur auf Lvalues, obwohl Parser und AST Field-Access auf beliebigen Ausdrücken erlauben.
- `loop { return ... }` in nicht-void Funktionen wird wegen eines unerreichbaren, aber unterminierten `loop_end` Blocks abgelehnt.
- Bestimmte normale Benutzereingaben mit `::` crashen den Compiler durch `unimplemented!()`.
- Doppelte Parameter, Struct-Felder und Variant-Cases werden akzeptiert.
- Token-Spans und `--emit-tokens` sind bei Unicode in Strings/Chars falsch.

Verifiziert mit:

```text
cargo test
cargo test -p rython_to_ir
cargo run -q -p rython_cli -- --emit-ir --no-run <minimal>.ry
```

`cargo test` ist aktuell nicht gruen. Ein Teil der Fehlschlaege kommt von offenbar veralteten Tests, die inzwischen implementierte Features noch als unsupported erwarten, sowie vom auskommentierten Assembly/Run-Pfad. Diese Punkte habe ich nicht als Compiler-Bugs gezaehlt.

## Unterstützte Annahmen

Als bereits implementiert/unterstuetzt angenommen:

- Lexer fuer Keywords, Identifier, `int`/`float`, Strings, Chars, `null`, Kommentare, Operatoren und `asm { ... }`.
- Parser fuer Funktionen, Structs, Variants, globale/konstante Variablen, Let/Return/If/While/Loop/Break/Continue, Blocks, Calls, Field-Access, Struct-Literale, Variant-Literale mit `::`, Zuweisungen, Postfix `++/--`, Index-Syntax, Operator-Prioritaeten.
- IR-Codegen fuer primitive Werte, lokale Variablen, Parameter, Funktionen, Calls, Struct-Definitionen und Struct-Literale, Field-Access, Methoden, Operator-Overloads an Struct-Methoden, Varianten, globals/consts, If/While/Loop/Break/Continue.
- Type-Checking im IR-Codegen fuer normale Funktionsaufrufe, primitive Operatoren, Return-Typen, Let-Initialisierung und Struct-Felder.

Nicht als implementiert angenommen:

- Vollstaendiger Assembly-Backend/Link/Run-Pfad.
- Imports, Trait-/Impl-Codegen, Generics, `any` Trait-Objekte, `for` Codegen, Turbofish, vollstaendige Runtime fuer `string`/`list`.

## Gefundene Bugs

### Bug 1: Operator-Overloads pruefen Argumenttypen nicht

Priorität: Kritisch  
Bereich: Semantik/Codegen  
Betroffene Dateien/Funktionen: `crates/rython_to_ir/src/codegen/generator.rs:524` (`preprocess_operators`), `crates/rython_to_ir/src/codegen/expr.rs:844` (`gen_binary_op`), `crates/rython_to_ir/src/codegen/expr.rs:281` (`gen_postfix`)

Beschreibung:  
Normale Funktions- und Methodenaufrufe pruefen die Argumenttypen gegen `function_signatures`. Operator-Overloads tun das nicht. `preprocess_operators` speichert fuer `(struct_name, op)` nur `(mangled_name, return_type)`. `gen_binary_op` und `gen_postfix` emittieren danach direkt `IrInstruction::Call`, ohne die Signaturparameter zu vergleichen.

Minimales Beispiel:

```source
struct Box {
    value: int,

    fn operator + add(this, rhs: int) int {
        return this.value + rhs;
    }
}

fn main() int {
    let b: Box = Box { value: 1 };
    return b + true;
}
```

Erwartetes Verhalten:  
Codegen muss `b + true` ablehnen, weil `Box_add` als zweiten Parameter `int` erwartet, aber `bool` uebergeben wird.

Tatsächliches Verhalten:  
Codegen ist erfolgreich und erzeugt IR mit einem falsch typisierten Call:

```text
Call { function_name: "Box_add", args: [%4, %5], return_type: I64 }
```

Dabei ist `%5` ein `Bool`, obwohl `Box_add` `rhs: I64` erwartet.

Warum das ein echter Bug ist:  
Operator-Overloads sind implementiert und werden in `current_features`/Parser-Tests benutzt. Normale Calls haben bereits Typpruefung; Operator-Calls umgehen diese Semantik und erzeugen inkonsistente IR.

Vermutete Ursache:  
`operator_functions` und `unary_operator_functions` speichern keine vollstaendige `FunctionSignaturIr`. `gen_binary_op` nimmt den gefundenen Operator als passend an, sobald der Struct-Name und Operator-String passen.

Fix-Vorschlag:  
In den Operator-Maps die vollstaendige Signatur speichern oder den `mangled_name` gegen `function_signatures` nachschlagen. Vor dem Emittieren des Calls dieselbe Argumentanzahl- und Typpruefung wie in `gen_call`/`gen_method_call` ausfuehren.

Test-Vorschlag:

```source
struct Box {
    value: int,
    fn operator + add(this, rhs: int) int { return this.value + rhs; }
}
fn main() int {
    let b: Box = Box { value: 1 };
    return b + true;
}
```

Der Test muss `CodegenError::MismatchedTypes(I64, Bool)` oder einen aequivalenten Fehler erwarten.

### Bug 2: Lokale Variablen und Parameter werden von gleichnamigen const/global Symbolen beim Lesen ueberdeckt

Priorität: Kritisch  
Bereich: Semantik/Symbol Resolution/Codegen  
Betroffene Dateien/Funktionen: `crates/rython_to_ir/src/codegen/expr.rs:953` (`gen_variable`), `crates/rython_to_ir/src/codegen/expr.rs:247` (`gen_left_value_addr`)

Beschreibung:  
`gen_variable` sucht zuerst in `module.constants`, dann in `module.globals`, und erst danach im lokalen Scope. `gen_left_value_addr` sucht dagegen zuerst lokal. Dadurch kann derselbe Name beim Lesen und Schreiben auf verschiedene Speicherorte zeigen.

Minimales Beispiel:

```source
const x: int = 1;

fn main() int {
    let x: int = 2;
    return x;
}
```

Erwartetes Verhalten:  
Entweder ist Shadowing verboten und der Compiler meldet eine Duplicate-/Shadowing-Fehlermeldung, oder `return x` liest die lokale Variable und gibt `2` zurueck.

Tatsächliches Verhalten:  
Der erzeugte IR-Code legt die lokale Variable mit Wert `2` an, ignoriert sie beim Lesen aber und returnt die Konstante `1`:

```text
PrimitiveConst { value: Int(2) }
Store { ... addr: %0 }
PrimitiveConst { value: Int(1) }
Ret(Some(%2))
```

Warum das ein echter Bug ist:  
Lokale Scopes und Globals/Consts sind implementiert. Unterschiedliche Lookup-Reihenfolgen fuer Lesen und Schreiben sind inkonsistent und koennen falschen Code erzeugen.

Vermutete Ursache:  
`gen_variable` berechnet `const_data`/`global_data` vor `looked_up_var` und returnt frueh fuer globale Symbole.

Fix-Vorschlag:  
Lookup-Reihenfolge vereinheitlichen: zuerst lokale Scopes, dann `const`, dann `global`. Falls Shadowing von globals/consts nicht erlaubt sein soll, muss `insert_variable`/`gen_let` oder eine Semantik-Pass-Validierung den Konflikt ablehnen.

Test-Vorschlag:

```source
global x: int = 1;
fn main() int {
    let x: int = 2;
    x = 3;
    return x;
}
```

Der Test muss entweder einen sauberen Shadowing-Fehler erwarten oder IR, das aus der lokalen Adresse laedt und `3` returnt.

### Bug 3: Prefixed Integer-Literale werden lexikalisch unterstuetzt, aber im Codegen abgelehnt

Priorität: Hoch  
Bereich: Lexer/Semantik/Codegen  
Betroffene Dateien/Funktionen: `crates/rython_to_ir/src/lexer.rs:680` (`handle_numbers`), `crates/rython_to_ir/src/codegen/expr.rs:1013` (`gen_intliteral`), `crates/rython_to_ir/src/codegen/generator.rs:623` (`eval_const_expr`)

Beschreibung:  
Der Lexer erkennt `0x`, `0b` und `0o` explizit und es gibt Tests fuer diese Tokens. Der IR-Codegen ruft spaeter aber nur `value.parse::<i64>()` auf. Rusts Dezimalparser akzeptiert Strings wie `0x10` nicht.

Minimales Beispiel:

```source
fn main() int {
    return 0x10;
}
```

Erwartetes Verhalten:  
Das Programm erzeugt eine `I64`-Konstante mit Wert `16`.

Tatsächliches Verhalten:  
Codegen bricht ab:

```text
[ir] InvalidIntLiteral("0x10")
```

Warum das ein echter Bug ist:  
Base-prefixed Integer sind nicht nur halbfertig angedeutet, sondern im Lexer implementiert und getestet. Der spaetere Compilerpfad verarbeitet dasselbe Literal falsch.

Vermutete Ursache:  
Die Normalisierung im Lexer entfernt `_`, laesst aber den Prefix im Tokenwert. Codegen und Const-Evaluator behandeln den Tokenwert als reines Dezimalformat.

Fix-Vorschlag:  
Eine gemeinsame Integer-Parsing-Funktion einfuehren, die Prefixe erkennt:

- `0x`/`0X` mit Radix 16
- `0b`/`0B` mit Radix 2
- `0o`/`0O` mit Radix 8
- sonst Radix 10

Diese Funktion in `gen_intliteral` und `eval_const_expr` verwenden.

Test-Vorschlag:

```source
const a: int = 0b1010;
fn main() int { return 0x10 + 0o7 + a; }
```

Der Test sollte erfolgreich IR fuer `16 + 7 + 10` erzeugen.

Zusatz-Edge-Case:  
`-9223372036854775808` wird als Unary-Minus plus positives Literal `9223372036854775808` geparst. Dadurch laeuft der positive Teil vor der Negation ueber. Falls `int` wirklich `i64` ist, sollte `i64::MIN` als Spezialfall akzeptiert oder mit einer praezisen Range-Fehlermeldung behandelt werden.

### Bug 4: `null` ist typinkonsistent und nicht sinnvoll Struct-/Pointer-Typen zuweisbar

Priorität: Hoch  
Bereich: Semantik/Type Checking/Codegen  
Betroffene Dateien/Funktionen: `crates/rython_to_ir/src/codegen/expr.rs:1064` (`gen_null_literal`), `crates/rython_to_ir/src/codegen/generator.rs:637` (`eval_const_expr`), `crates/rython_to_ir/src/codegen/stmt.rs:10` (`gen_let`)

Beschreibung:  
Im normalen Ausdruckscodegen bekommt `null` den Typ `Pointer(Void)`. Im Const-Evaluator bekommt `null` den Typ `IrType::Null`. Deklarierte Struct-Typen werden als `Pointer(Named("Struct"))` modelliert. Es gibt keine Kompatibilitaetsregel, die `null` einem Struct-/Pointer-Typ zuweist.

Minimales Beispiel:

```source
struct Node {
    value: int
}

fn main() {
    let n: Node = null;
}
```

Erwartetes Verhalten:  
Wenn Struct-Werte als Pointer modelliert sind, sollte `null` entweder jedem passenden Pointer-/Struct-Typ zuweisbar sein oder die Sprache sollte `null` fuer solche Typen klar verbieten. Da `null` als Literal implementiert ist, ist die naheliegende Semantik: `let n: Node = null;` ist gueltig.

Tatsächliches Verhalten:  
Codegen bricht ab:

```text
[ir] MismatchedTypes(Pointer(Named("Node")), Pointer(Void))
```

Warum das ein echter Bug ist:  
`null` ist im Lexer, Parser, AST und Codegen implementiert. Die beiden Codepfade verwenden verschiedene Typen (`Pointer(Void)` vs `Null`) und die Typvergleichslogik macht das Literal fuer deklarierte Pointer-/Struct-Typen unbrauchbar.

Vermutete Ursache:  
Es fehlt eine zentrale Assignability-/Compatibility-Funktion. Der Code vergleicht Typen meist mit `==`.

Fix-Vorschlag:  
Eine Funktion wie `is_assignable(expected, actual)` einfuehren. `actual == Null` oder `actual == Pointer(Void)` sollte fuer Pointer-/Struct-Typen erlaubt sein, falls `null` so gedacht ist. `gen_null_literal` und `eval_const_expr` sollten denselben Null-Typ verwenden.

Test-Vorschlag:

```source
struct Node { value: int }
global root: Node = null;
fn main() {
    let local: Node = null;
}
```

Beide Initialisierungen sollten konsistent akzeptiert oder konsistent mit derselben Fehlermeldung abgelehnt werden.

### Bug 5: Field-Access funktioniert nur auf Lvalues, nicht auf Struct-Rvalues oder Call-Ergebnissen

Priorität: Hoch  
Bereich: AST/Semantik/Codegen  
Betroffene Dateien/Funktionen: `crates/rython_to_ir/src/parser.rs:360` (`parse_postfix` FieldAccess), `crates/rython_to_ir/src/codegen/expr.rs:177` (`gen_field_access`), `crates/rython_to_ir/src/codegen/expr.rs:198` (`gen_field_addr`)

Beschreibung:  
Der Parser erlaubt Field-Access als Postfix auf beliebigen Ausdruecken. Der Codegen fuer Field-Access ruft aber immer `gen_field_addr`, und `gen_field_addr` verlangt ueber `gen_left_value_addr`, dass das Objekt eine Variable, ein FieldAccess oder eine Grouping-Lvalue ist. Struct-Literale und Funktionsaufrufe sind dadurch als Field-Basis ungueltig.

Minimales Beispiel:

```source
struct Point {
    x: int
}

fn main() int {
    return Point { x: 1 }.x;
}
```

Erwartetes Verhalten:  
Das Struct-Literal wird erzeugt, die Adresse von `x` berechnet und der Wert `1` geladen.

Tatsächliches Verhalten:  
Codegen bricht ab:

```text
[ir] InvalidExpr(StructLiteral { struct_name: "Point", arguments: [("x", IntLiteral("1"))] })
```

Auch ein Call-Ergebnis ist betroffen:

```source
struct Point { x: int }
fn make() Point { return Point { x: 1 }; }
fn main() int { return make().x; }
```

Dieses Programm endet mit `InvalidExpr(Call { ... })`.

Warum das ein echter Bug ist:  
Struct-Literale, Funktionsrueckgaben und Field-Access sind implementiert. Die AST-Form `Expr::FieldAccess { object: Box<Expr>, ... }` beschraenkt `object` nicht auf Lvalues.

Vermutete Ursache:  
`gen_field_access` teilt sich die Adresslogik mit Zuweisungen. Fuer lesenden Field-Access muss aber auch ein bereits erzeugter Pointer-/Struct-Wert als Basis akzeptiert werden.

Fix-Vorschlag:  
Zwei Pfade trennen:

- `gen_field_addr_for_lvalue` fuer Zuweisungsziele.
- `gen_field_addr_from_value` fuer lesenden Zugriff, der zuerst `gen_expr(object)` ausfuehrt und bei `Pointer(Named(...))` direkt die gelieferte Basisadresse nutzt.

Test-Vorschlag:

```source
struct Point { x: int }
fn make() Point { return Point { x: 41 }; }
fn main() int {
    return make().x + Point { x: 1 }.x;
}
```

Der Test sollte IR fuer Feld-Loads aus beiden Rvalues erzeugen.

### Bug 6: `loop { return ... }` in nicht-void Funktionen wird wegen unerreichbarem `loop_end` Block abgelehnt

Priorität: Hoch  
Bereich: Codegen/Kontrollfluss  
Betroffene Dateien/Funktionen: `crates/rython_to_ir/src/codegen/stmt.rs:272` (`gen_loop`), `crates/rython_to_ir/src/codegen/generator.rs:103` (`finish_blocks`)

Beschreibung:  
`gen_loop` erzeugt immer einen `loop_end` Block und setzt ihn als aktuellen Block, auch wenn der Loop-Body bereits sicher terminiert und es keinen `break` gibt. In nicht-void Funktionen verlangt `finish_blocks` fuer jeden Block einen Terminator. Der unerreichbare `loop_end` Block hat keinen Terminator und verursacht einen Fehler.

Minimales Beispiel:

```source
fn main() int {
    loop {
        return 1;
    }
}
```

Erwartetes Verhalten:  
Das Programm ist gueltig: der einzige erreichbare Pfad returned `1`.

Tatsächliches Verhalten:  
Codegen bricht ab:

```text
[ir] MissingTerminator("loop_end_1:")
```

Warum das ein echter Bug ist:  
`loop`, `return` und nicht-void Return-Checking sind implementiert. Der Fehler entsteht nicht durch ein fehlendes Feature, sondern durch einen unreachable Block, den der Codegen selbst erzeugt.

Vermutete Ursache:  
`gen_loop` kann nicht unterscheiden, ob `end_label` wirklich erreichbar ist. Es erstellt den Endblock immer, selbst wenn kein `break` auf ihn springt und der Body terminiert.

Fix-Vorschlag:  
Tracken, ob ein `break` zum Loop-Ende erzeugt wurde, oder nach Body-Codegen nur dann `loop_end` erstellen, wenn er erreichbar ist. Alternativ unreachable Blocks in `finish_blocks` nicht terminatorpflichtig machen, falls Reachability vorhanden ist.

Test-Vorschlag:

```source
fn main() int {
    loop { return 1; }
}
```

Der Test sollte erfolgreich IR erzeugen mit `entry -> loop_body` und `loop_body -> Ret(Some(...))`, ohne unterminierten `loop_end`.

### Bug 7: Ungueltige `::`-Syntax kann den Parser crashen

Priorität: Hoch  
Bereich: Parser/Fehlerbehandlung  
Betroffene Dateien/Funktionen: `crates/rython_to_ir/src/parser.rs:343` (`parse_postfix`), besonders `crates/rython_to_ir/src/parser.rs:350`

Beschreibung:  
`Ident::Ident` wird in `parse_primary` als Variant-Literal behandelt. Wenn aber nach einem Ausdruck ein `::` steht, das nicht in diesen Sonderfall passt, landet der Parser in `parse_postfix` beim `TokenKind::ColonColon` Arm und ruft `unimplemented!()` auf.

Minimales Beispiel:

```source
fn main() {
    a::;
}
```

Erwartetes Verhalten:  
Ein normaler `ParseError`, z.B. `UnexpectedToken` oder `UnexpectedExprStart`.

Tatsächliches Verhalten:  
Die CLI beendet sich mit Rust-Panic:

```text
thread 'main' panicked at crates/rython_to_ir/src/parser.rs:358:21:
not implemented
```

Warum das ein echter Bug ist:  
Turbofish selbst ist offensichtlich nicht fertig und wurde nicht als fehlendes Feature gezaehlt. Der Bug hier ist die Fehlerbehandlung: ungueltige, tokenisierbare Benutzereingabe darf den Compiler nicht durch `panic!` beenden.

Vermutete Ursache:  
Der `ColonColon`-Postfix-Arm ist als Placeholder stehen geblieben und wird auch fuer malformed Syntax erreicht.

Fix-Vorschlag:  
Bis Turbofish implementiert ist, in diesem Arm einen `ParseError::UnexpectedToken` oder spezifischen `UnsupportedSyntax`/`UnexpectedToken` zurueckgeben. Den Variant-Literal-Fall weiter in `parse_primary` behandeln.

Test-Vorschlag:

```source
fn main() { a::; }
```

Der Test sollte `parser.parse().is_err()` pruefen und darf keine Panic ausloesen.

### Bug 8: Doppelte Parameter, Struct-Felder und Variant-Cases werden akzeptiert

Priorität: Mittel  
Bereich: Semantik/Symbol Resolution/Type Definitions  
Betroffene Dateien/Funktionen: `crates/rython_to_ir/src/parser.rs:686` (`parse_params`), `crates/rython_to_ir/src/parser.rs:948` (`parse_struct`), `crates/rython_to_ir/src/parser.rs:850` (`parse_variant`), `crates/rython_to_ir/src/codegen/scope.rs:31` (`insert_variable`), `crates/rython_to_ir/src/codegen/generator.rs:448` (`preprocces_type_defs`)

Beschreibung:  
Der Compiler akzeptiert mehrere Deklarationen mit demselben Namen in Kontexten, in denen das keine sinnvolle Shadowing-Semantik haben kann.

Minimales Beispiel 1:

```source
fn pick(x: int, x: int) int {
    return x;
}
```

Erwartetes Verhalten:  
Duplicate-Parameter-Fehler.

Tatsächliches Verhalten:  
Codegen ist erfolgreich. Die IR-Funktion hat zwei Parameter namens `x`; `insert_variable` ueberschreibt den ersten Scope-Eintrag, daher liest `return x` nur den zweiten Parameter.

Minimales Beispiel 2:

```source
struct Bad {
    x: int,
    x: bool
}

fn main() {}
```

Erwartetes Verhalten:  
Duplicate-Field-Fehler.

Tatsächliches Verhalten:  
Codegen erzeugt einen Struct-Type mit zwei Feldern namens `x`.

Minimales Beispiel 3:

```source
variant V {
    A,
    A
}

fn main() V {
    return V::A;
}
```

Erwartetes Verhalten:  
Duplicate-Case-Fehler.

Tatsächliches Verhalten:  
Codegen akzeptiert die Variant und erzeugt `cases: ["A", "A"]`.

Warum das ein echter Bug ist:  
Parameter, Structs und Variants sind implementierte Sprachkonstrukte. Doppelte Namen in derselben Parameterliste, demselben Struct oder derselben Variant erzeugen mehrdeutige oder unbrauchbare Semantik.

Vermutete Ursache:  
Es gibt keine Duplicate-Pruefung in Parser oder Semantik-Pass. Bei lokalen Symbolen verwendet `insert_variable` `HashMap::insert` und ignoriert, ob ein Eintrag ersetzt wurde.

Fix-Vorschlag:  
Beim Parsen oder vor Codegen pro Namensraum ein `HashSet` pflegen:

- Parameterliste: Duplikate verbieten.
- Struct-Felder: Duplikate verbieten.
- Variant-Cases: Duplikate verbieten.
- Lokale `let`-Shadowing-Regel explizit entscheiden; falls gleiche Scope-Ebene verboten ist, `insert_variable` entsprechend aendern.

Test-Vorschlag:  
Je ein Regressionstest fuer duplicate params, duplicate fields und duplicate cases, alle mit erwarteten Fehlern.

### Bug 9: Methoden akzeptieren `this` an beliebiger Parameterposition, aber Method-Calls setzen `this` immer an Position 0

Priorität: Mittel  
Bereich: Parser/Semantik/Codegen  
Betroffene Dateien/Funktionen: `crates/rython_to_ir/src/parser.rs:686` (`parse_params`), `crates/rython_to_ir/src/codegen/expr.rs:613` (`gen_method_call`)

Beschreibung:  
`parse_params` erlaubt `this` ueberall in der Parameterliste einer Struct-Methode. `gen_method_call` konstruiert den realen Call aber immer als `[object, user_arg_1, ...]`. Wenn `this` nicht der erste Parameter ist, ist die Methode syntaktisch gueltig, aber normal unaufrufbar.

Minimales Beispiel:

```source
struct S {
    x: int,

    fn bad(x: int, this) int {
        return x;
    }
}

fn main() int {
    let s: S = S { x: 0 };
    return s.bad(1);
}
```

Erwartetes Verhalten:  
Entweder der Parser/Semantik-Pass lehnt die Methodendefinition ab, weil `this` nicht an erster Position steht, oder der Method-Call respektiert die deklarierte Position.

Tatsächliches Verhalten:  
Die Definition wird akzeptiert. Beim Call entsteht ein Typfehler:

```text
[ir] MismatchedTypes(I64, Pointer(Named("S")))
```

Warum das ein echter Bug ist:  
Struct-Methoden und explizites `this` sind implementiert. Die akzeptierte Methodensignatur passt nicht zur implementierten Call-Konvention.

Vermutete Ursache:  
Parser validiert nur, dass `this` innerhalb eines Struct-/Impl-Kontexts steht. Codegen hat eine harte Konvention, die nicht in der Signatur validiert wird.

Fix-Vorschlag:  
Eine klare Regel einfuehren: `this` muss, falls vorhanden, der erste Parameter einer Methode sein. Methoden ohne `this` entweder als statische Methoden mit eigener Call-Syntax behandeln oder fuer `obj.method(...)` ausschliessen.

Test-Vorschlag:

```source
struct S { x: int, fn bad(x: int, this) int { return x; } }
```

Der Test sollte bereits beim Parser/Semantik-Pass einen Fehler erwarten.

### Bug 10: Token-Spans und `--emit-tokens` sind bei Unicode in Strings/Chars falsch

Priorität: Mittel  
Bereich: Lexer/Fehlerbehandlung/Diagnostics  
Betroffene Dateien/Funktionen: `crates/rython_to_ir/src/lexer.rs:104` (`Token::new`), `crates/rython_to_ir/src/lexer.rs:515` (`handle_strings`), `crates/rython_to_ir/src/lexer.rs:551` (`handle_char`), `crates/manager/src/run.rs:123` (`--emit-tokens` slicing)

Beschreibung:  
Der Lexer arbeitet mit `Vec<char>` und speichert char-basierte Indizes. Einige Token-Laengen werden aber mit `String::len()` berechnet, also in Bytes. `manager::run` verwendet diese Werte anschliessend als Byte-Indizes in `content[start..end]`. Bei Unicode in String-/Char-Literalen zeigen die ausgegebenen Token-Slices auf falsche Stellen.

Minimales Beispiel:

```source
fn main() {
    let c: char = 'ä';
}
```

Erwartetes Verhalten:  
`--emit-tokens` sollte fuer das Char-Token den Quelltextbereich `"'ä'"` oder mindestens konsistente Start-/Endpositionen ausgeben. Nachfolgende Tokens duerfen nicht verschoben sein.

Tatsächliches Verhalten:  
Die Ausgabe zeigt falsche Slices nach dem Unicode-Token. Reproduziert wurde u.a.:

```text
[token] Char      | ä | 29 | 27 | ä
[token] Semicolon | ; | 30 | 29 | '
[token] RBrace    | } | 32 | 31 |  
```

Der Semicolon-Token zeigt also einen Quote-Slice, der RBrace-Token ein Leerzeichen.

Warum das ein echter Bug ist:  
Strings und Chars erlauben Unicode-Zeichen. Diagnostics/Token-Emission sind implementiert und sollen Positionen/Slices zeigen. Die Positionen werden nach Unicode falsch.

Vermutete Ursache:  
Mischung aus char-Indizes und Byte-Laengen. Zusaetzlich sind Token-Spans generell als `start_char_idx + 1` plus haeufig negative `length` modelliert, was die Interpretation erschwert.

Fix-Vorschlag:  
Span-Definition vereinheitlichen:

- Entweder konsequent Byte-Spans `[start_byte, end_byte)` speichern.
- Oder char-Spans speichern, aber nie direkt als String-Byte-Slices verwenden.

Fuer Diagnostics ist byte-basierter Start/Ende im Lexer meist einfacher, weil Rust-Strings bytebasiert gesliced werden.

Test-Vorschlag:

```source
fn main() {
    let c: char = 'ä';
    let s: string = "Grüße";
}
```

Ein Test sollte pruefen, dass `--emit-tokens` nicht verrutscht und fuer jedes Token den korrekten Quelltextbereich rekonstruiert.

## Nicht als Bug gezählt

- `crates/ir_to_assembly/src/codegen.rs` ist leer und der Assembly/Link/Run-Pfad in `crates/manager/src/run.rs` ist auskommentiert. Die daraus folgenden CLI-Testfehlschlaege (`-o`, Exit-Code-Propagation, Binary-Erzeugung) wurden als unfertiger Backend-Pfad gewertet.
- Imports werden geparst, aber nicht im Codegen verarbeitet.
- Traits, Trait-Implementations, `any` Trait-Typen und Generics sind sichtbar unfertig bzw. werden im Codegen abgelehnt oder enthalten TODOs.
- `for` wird geparst, aber `Stmt::For` ist im Codegen nicht implementiert.
- Turbofish/Type-Arguments bei Calls sind nicht implementiert. Nur der Panic-Fall bei malformed `::` wurde als Fehlerbehandlungsbug gezaehlt.
- `string`/`list` Runtime ist nicht als Builtin vorhanden. String-/List-Literale scheinen auf userdefinierte `struct string`/`struct list` plus Methoden wie `init_start`/`push_char`/`push_element` ausgelegt zu sein; fehlende Runtime-Definitionen wurden daher nicht als Bug gemeldet.
- Alte oder widerspruechliche Tests, die inzwischen implementierte Features noch als unsupported erwarten, wurden nicht als Compiler-Bug gewertet.

## Empfohlene nächste Tests

Konkrete Regressionstests:

```source
// Operator-Overload muss Argumenttypen pruefen.
struct Box {
    value: int,
    fn operator + add(this, rhs: int) int { return this.value + rhs; }
}
fn main() int {
    let b: Box = Box { value: 1 };
    return b + true;
}
```

```source
// Lokales Shadowing muss konsistent sein oder sauber verboten werden.
const x: int = 1;
fn main() int {
    let x: int = 2;
    return x;
}
```

```source
// Prefixed Literale muessen bis Codegen funktionieren.
const a: int = 0b1010;
fn main() int {
    return 0x10 + 0o7 + a;
}
```

```source
// null muss mit Struct-/Pointer-Typen konsistent sein.
struct Node { value: int }
fn main() {
    let n: Node = null;
}
```

```source
// Rvalue Field-Access.
struct Point { x: int }
fn make() Point { return Point { x: 1 }; }
fn main() int {
    return make().x + Point { x: 2 }.x;
}
```

```source
// Loop mit sicherem Return darf keinen unreachable loop_end Fehler erzeugen.
fn main() int {
    loop { return 1; }
}
```

```source
// Parser darf nicht panicken.
fn main() { a::; }
```

```source
// Doppelte Namen muessen abgelehnt werden.
fn pick(x: int, x: int) int { return x; }
struct Bad { x: int, x: bool }
variant V { A, A }
```

```source
// Unicode-Spans duerfen nicht verrutschen.
fn main() {
    let c: char = 'ä';
}
```

## Bug-Hunting-Teststrategie

1. Lexer separat testen:
   - Alle Tokenklassen als Einzel- und Nachbarschaftstests: `= == ===`, `< <= << <<=`, `/ // /* */ /=`.
   - Zahlenmatrix: Dezimal, `0x`, `0b`, `0o`, `_`, Exponenten, ungueltige Digits, Grenzwerte `i64::MAX`, `i64::MIN`.
   - String/Char-Matrix: ASCII, Unicode direkt, Escape-Sequenzen, unbekannte Escapes, unterminierte Literale.
   - Span-Tests: Aus jedem Token-Span wieder den Quelltext rekonstruieren.

2. Parser separat testen:
   - Jede unterstuetzte Grammatikproduktion mit minimal gueltigem und minimal ungueltigem Code.
   - Operator-Prioritaet als AST-Snapshot.
   - Postfix-Ketten: `a.b(c)[d]++`, Struct-Literal plus Field-Access, malformed `::`.
   - Parser-Fuzzing mit kleinen Tokenfolgen aus unterstuetzten Tokens; Ziel: nie panic, immer `Ok` oder `ParseError`.

3. Semantik/Codegen separat testen:
   - Normale Calls, Method-Calls und Operator-Calls mit korrekten und falschen Argumenttypen.
   - Scopes: lokale Shadowing-Faelle, Parameter vs local/global/const, doppelte Namen.
   - Typkompatibilitaet: `null`, Struct-Pointer, Variants, primitive Operatoren, Return-Typen.
   - Kontrollfluss: `if` mit/ohne else, beide Arme returnen, loop mit break, loop ohne break, continue/break in nested if.

4. Pipeline-Tests:
   - Fuer jedes Minimalprogramm `Lexer -> Parser -> IR-Codegen` laufen lassen.
   - Erwartete Fehler als konkrete Error-Varianten testen, nicht nur `is_err()`.
   - Bei erfolgreichen Programmen IR-Invarianten pruefen: jeder erreichbare Block terminiert, Call-Argumenttypen passen zur Signatur, keine duplicate field/case/param Namen.

5. Random/Fuzz-aehnliche Inputs:
   - Kleine AST-Generatoren fuer bereits unterstuetzte Syntax bauen statt rein zufaelliger Bytes.
   - Gueltige Programme mutieren: Operator austauschen, Literalbasis wechseln, Namen duplizieren, Semikolons entfernen, `::`/`.` vertauschen.
   - Jeden Crash oder unerwarteten Erfolg automatisch reduzieren: erst Statements entfernen, dann Ausdruecke verkleinern, dann Literale/Namen minimieren.

