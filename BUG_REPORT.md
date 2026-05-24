# Bug Report: Rython to IR, Manager und CLI

Stand: 2026-05-24

Dieser Report betrachtet den aktuellen Workspace-Zustand ohne alte geloeschte Bug-Reports,
Todo-Listen oder Tests. Der Bereich IR to ASM selbst ist nicht bewertet; bewertet wird nur,
ob Rython bis IR sowie Manager/CLI korrekt bis zum Backend-Uebergang arbeiten.

## Verwendete Zielregeln

- Boolesche Operatoren sind `&&`, `||` und `!`; Wortoperatoren wie `and`/`or` sind nicht Zielsyntax.
- `while` und `loop` duerfen als Statement und als Expression vorkommen.
- Es gibt keine Tail-Expressions. Normale Ausdruecke ohne Semikolon sind Fehler.
- `return` und `yield` muessen immer explizit mit Semikolon abgeschlossen werden.
- `return` verlaesst die ganze Funktion. `yield` liefert einen Wert an den naechsten Block in einem Wert-Kontext.
- `Type::case` ist Variantensyntax, `Type::method(args)` ist Syntax fuer assoziierte Funktionen,
  `obj.field`/`obj.method()` sind Instanzzugriff und Methodenaufruf.
- Generics werden monomorphisiert. Traits, `impl`, Bounds und `Self` werden statisch aufgeloest.
- Imports laden `.ry`-Dateien relativ zum Input-Root und mergen Top-Level-Items in den globalen Scope.
- String/List-Literale werden ueber eine Core-Stdlib/Prelude abgesenkt.
- `for` wird ueber statisch aufgeloeste Iterator-/Iterable-Traits abgesenkt.
- Inline-ASM ersetzt `%name` durch Wert-Temps und `%&name` durch Adress-Temps.
- Block-Shadowing lokaler Namen ist erlaubt; lokale Variablen/Parameter duerfen keine Globals/Consts verdecken.
- Die CLI soll alle Build-Optionen weiterfuehren und erst am echten IR-to-ASM-Uebergang in einen Backend-Stub laufen.

## Befunde

### B001 - `while` und `loop` sind als Statements nicht parsebar

Schweregrad: hoch

Beleg:
- `parse_primary` erkennt `while` und `loop` nur als Expressions (`crates/rython_to_ir/src/parser.rs:423`).
- `is_statement_start` kommentiert `While` und `Loop` aus (`crates/rython_to_ir/src/parser.rs:1369`).
- `examples/current_features.ry` nutzt `while` und `loop` als Statements (`examples/current_features.ry:48`, `examples/current_features.ry:96`).
- Reproduktion: `cargo run -q -p rython_cli -- --emit-ir --no-run examples/current_features.ry`
  endet mit `[parser] UnexpectedToken { expected: Semicolon, found: Return, token_idx: 211 }`.

Ist-Verhalten: Ein Loop-Block ohne Semikolon wird als Expression-Statement behandelt und danach wird ein Semikolon erwartet.

Erwartetes Verhalten: `while cond { ... }` und `loop { ... }` muessen als Statements ohne Semikolon gueltig sein.

### B002 - Loop-/While-Codegen panict mit `todo!()`

Schweregrad: hoch

Beleg:
- `gen_loop` ist `todo!()` (`crates/rython_to_ir/src/codegen/expr.rs:65`).
- `gen_while` ist `todo!()` (`crates/rython_to_ir/src/codegen/expr.rs:68`).

Ist-Verhalten: Wenn ein Loop/While als Expression bis Codegen gelangt, bricht der Compiler mit Panic ab.

Erwartetes Verhalten: Der Compiler muss IR erzeugen oder einen strukturierten `CodegenError` liefern.

### B003 - `yield`-Codegen panict und verwendet falschen Kontext

Schweregrad: hoch

Beleg:
- `gen_yield` sucht aktuell im `loop_stack` und panict danach mit `todo!()` (`crates/rython_to_ir/src/codegen/stmt.rs:61`).
- Die Zielregel ist: `yield` liefert an den naechsten Block in einem Wert-Kontext, nicht nur an eine Loop.

Ist-Verhalten: `yield` ausserhalb eines Loop-Kontexts wird als `YieldOutsideLoop` behandelt, innerhalb wuerde ein Panic folgen.

Erwartetes Verhalten: `yield` muss Block-Wert-Kontexte bedienen und darf nicht panicen.

### B004 - Block-Expressions sind im AST/Codegen unfertig

Schweregrad: hoch

Beleg:
- `Stmt::Block` existiert (`crates/rython_to_ir/src/ast.rs:141`), aber es gibt keinen `Expr::Block`.
- `gen_expr_block` ist `todo!()` und unbenutzt (`crates/rython_to_ir/src/codegen/stmt.rs:191`).
- Die Zielregel verlangt Wert-Kontexte fuer Bloecke via `yield`.

Ist-Verhalten: Bloecke koennen nur als Statement-Scope erzeugt werden; Werte aus Bloecken sind nicht ausdrueckbar.

Erwartetes Verhalten: Block-Wert-Kontexte muessen im AST, Parser, Typechecker und IR-Codegen abbildbar sein.

### B005 - Parser akzeptiert verbotene Tail-Expressions und wandelt sie in `return`

Schweregrad: hoch

Beleg:
- `parse_block` macht aus einer Expression vor `}` automatisch `Stmt::Return` (`crates/rython_to_ir/src/parser.rs:1109`).
- Die Zielregel sagt: Es gibt keine Tail-Expressions; Ausdruecke ohne Semikolon sind Fehler.

Ist-Verhalten: `fn f() int { 1 }` wird als implizites `return 1` geparst.

Erwartetes Verhalten: Der Parser muss an dieser Stelle einen Fehler melden.

### B006 - Parser akzeptiert `return`/`yield` ohne Semikolon vor `}`

Schweregrad: mittel

Beleg:
- `parse_return` erlaubt `RBrace` ohne Semikolon (`crates/rython_to_ir/src/parser.rs:1300`).
- `parse_yield` erlaubt `RBrace` ohne Semikolon (`crates/rython_to_ir/src/parser.rs:1283`).
- Die Zielregel verlangt immer Semikolon.

Ist-Verhalten: `return expr }` oder `yield expr }` koennen ohne Semikolon akzeptiert werden.

Erwartetes Verhalten: Fehlendes Semikolon muss Parserfehler sein.

### B007 - `for` wird geparst, aber im IR-Codegen immer abgelehnt

Schweregrad: hoch

Beleg:
- `Stmt::For` existiert (`crates/rython_to_ir/src/ast.rs:138`) und `parse_for` erzeugt es (`crates/rython_to_ir/src/parser.rs:1243`).
- `gen_stmt` hat keinen Arm fuer `Stmt::For`; es faellt in `InvalidStatement` (`crates/rython_to_ir/src/codegen/stmt.rs:170`).

Ist-Verhalten: Ein parsebarer `for`-Loop kann nicht zu IR werden.

Erwartetes Verhalten: `for item in expr { ... }` muss ueber statisch aufgeloeste Iterator-/Iterable-Traits IR erzeugen.

### B008 - Imports werden geparst, aber im Codegen als InvalidItem abgelehnt

Schweregrad: hoch

Beleg:
- `parse_import` erzeugt `Item::Import` (`crates/rython_to_ir/src/parser.rs:773`).
- `generate_code_inner` gibt fuer `Item::Import` direkt `CodegenError::InvalidItem` zurueck (`crates/rython_to_ir/src/codegen/generator.rs:597`).

Ist-Verhalten: Importierte Module werden nicht geladen, nicht aufgeloest und verhindern IR-Erzeugung.

Erwartetes Verhalten: Der Manager/Compiler muss `<input-root>/a/b.ry` fuer `import a.b;` laden, Abhaengigkeiten aufloesen und Items mergen.

### B009 - Traits und Impl-Bloecke werden geparst, aber im Codegen abgelehnt

Schweregrad: hoch

Beleg:
- AST enthaelt `Trait`, `TraitImplementation`, Bounds und Function-Signatures (`crates/rython_to_ir/src/ast.rs:53`, `crates/rython_to_ir/src/ast.rs:93`).
- Parser erzeugt Traits/Impls (`crates/rython_to_ir/src/parser.rs:885`, `crates/rython_to_ir/src/parser.rs:1047`).
- Codegen lehnt `Item::Trait` und `Item::TraitImplementation` als `InvalidItem` ab (`crates/rython_to_ir/src/codegen/generator.rs:597`).

Ist-Verhalten: Trait-Syntax ist oberflaechlich vorhanden, aber nicht IR-fertig.

Erwartetes Verhalten: Traits, Impl-Bloecke, Bounds und `Self` muessen statisch geprueft und aufgeloest werden.

### B010 - `any Trait` panict in der Typkonvertierung

Schweregrad: hoch

Beleg:
- `Type::AnyTrait(_) => todo!()` in `convert_to_ir_type` (`crates/rython_to_ir/src/codegen/generator.rs:381`).

Ist-Verhalten: Ein parsebarer `any Trait`-Typ fuehrt im Codegen zu Panic.

Erwartetes Verhalten: Der Typ muss nach der statischen Trait-Strategie geprueft oder strukturiert abgelehnt werden; kein Panic.

### B011 - Generics werden geparst, aber nicht monomorphisiert oder semantisch aufgeloest

Schweregrad: hoch

Beleg:
- AST und Parser speichern `generic_params` (`crates/rython_to_ir/src/ast.rs:13`, `crates/rython_to_ir/src/parser.rs:601`).
- `parse_type_args` existiert (`crates/rython_to_ir/src/parser.rs:660`), aber `gen_call` ignoriert `type_args` (`crates/rython_to_ir/src/codegen/expr.rs:550`).
- `convert_to_ir_type` kennt nur primitive Namen, Struct-Namen und Variant-Namen (`crates/rython_to_ir/src/codegen/generator.rs:362`).

Ist-Verhalten: Typ-Parameter wie `T` werden als unbekannter Typ behandelt oder ignoriert.

Erwartetes Verhalten: Generische Funktionen/Structs muessen pro Typkombination monomorphisiert werden.

### B012 - Rust-aehnliche Pfade sind unvollstaendig

Schweregrad: hoch

Beleg:
- `Ident::Ident` wird nur als VariantLiteral geparst (`crates/rython_to_ir/src/parser.rs:466`).
- Der `ColonColon`-Arm in `parse_postfix` ist auskommentiert (`crates/rython_to_ir/src/parser.rs:350`).
- `gen_call` unterstuetzt normale Funktionsnamen und Instanzmethoden, aber keine `Type::method(args)`-Aufloesung (`crates/rython_to_ir/src/codegen/expr.rs:550`).
- Reproduktion: `examples/all_features.ry` enthaelt `Vec2::new(3, 4)` (`examples/all_features.ry:179`) und scheitert aktuell bereits im Parser mit `UnexpectedToken`.

Ist-Verhalten: `Type::case` funktioniert als spezieller Variantenknoten, aber assoziierte Funktionen und allgemeine Pfade fehlen.

Erwartetes Verhalten: `Type::case` und `Type::method(args)` muessen getrennt und korrekt aufgeloest werden.

### B013 - Top-Level-Operatorfunktionen werden parsebar akzeptiert

Schweregrad: mittel

Beleg:
- `parse_fn_def` erlaubt `operator` fuer jede Funktion (`crates/rython_to_ir/src/parser.rs:1014`).
- Die Zielregel erlaubt Operator-Overloads nur als Struct-Methoden.

Ist-Verhalten: `fn operator + name(...)` ist auch top-level syntaktisch gueltig.

Erwartetes Verhalten: Top-Level-Operatorfunktionen muessen Parser- oder Semantikfehler sein.

### B014 - RHS-Operator-Dispatch prueft Argumenttypen in falscher Reihenfolge

Schweregrad: hoch

Beleg:
- Wenn der rechte Operand den Operator bereitstellt, wird der Call mit `args: vec![temp_id_2, temp_id_1]` erzeugt (`crates/rython_to_ir/src/codegen/expr.rs:965`).
- Die Typpruefung davor verwendet aber `got_arg_types = vec![ir_type_1, ir_type_2]` (`crates/rython_to_ir/src/codegen/expr.rs:951`).

Ist-Verhalten: Der Generator vergleicht Typen in anderer Reihenfolge als er die Argumente uebergibt.

Erwartetes Verhalten: Typpruefung und Call-Argumentliste muessen dieselbe Reihenfolge verwenden.

### B015 - Methoden-/Operator-Dispatch ist nur namensbasiert und nicht generisch/traitbasiert

Schweregrad: hoch

Beleg:
- Methoden werden direkt als `"{struct_name}_{method_name}"` gesucht (`crates/rython_to_ir/src/codegen/expr.rs:619`, `crates/rython_to_ir/src/codegen/expr.rs:679`).
- Operatoren werden ueber `(struct_name, operator_string)` registriert (`crates/rython_to_ir/src/codegen/generator.rs:636`).

Ist-Verhalten: Trait-Impls, Monomorphisierung und qualifizierte Namen sind nicht Teil der Aufloesung.

Erwartetes Verhalten: Dispatch muss nach der Zielregel statisch ueber konkrete Typen, Traits/Impls und monomorphisierte Signaturen laufen.

### B016 - Void-Calls haben im IR ein Pflicht-Temp

Schweregrad: mittel

Beleg:
- `IrInstruction::Call` hat `temp_id: TempId`, obwohl der Kommentar `None` fuer void erwaehnt (`crates/rython_to_ir/src/ir.rs:111`).
- `ExprValue::from` verwirft Void-Call-Werte spaeter nur anhand des Return-Typs (`crates/rython_to_ir/src/codegen/generator.rs:62`).

Ist-Verhalten: Auch Void-Calls erzeugen eine Temp-ID, obwohl diese semantisch keinen Wert hat.

Erwartetes Verhalten: IR muss Void-Calls eindeutig ohne Wert-Temp darstellen oder konsistent definieren, warum ein Temp existiert.

### B017 - Fehlende Terminator-/Return-Pruefung ist falsch

Schweregrad: hoch

Beleg:
- `finish_blocks` prueft fehlende Terminatoren nur, wenn der Blockname schon in `reachable_labels` steht (`crates/rython_to_ir/src/codegen/generator.rs:140`).
- Der Entry-Block ist zu Beginn nicht in `reachable_labels`.

Ist-Verhalten: Eine nicht-void Funktion ohne Return kann in manchen Faellen implizit `Ret(None)` erhalten.

Erwartetes Verhalten: Jede erreichbare nicht-void Kontrollflussbahn muss einen Wert terminieren; fehlende Returns muessen Fehler sein.

### B018 - Lokale Duplikate und Parameter-Duplikate werden zu spaet oder gar nicht korrekt erkannt

Schweregrad: mittel

Beleg:
- `insert_variable` ueberschreibt vorhandene Namen im aktuellen Scope ohne Fehler (`crates/rython_to_ir/src/codegen/scope.rs:31`).
- Parameterduplikate werden erst nach `gen_func_struct` geprueft (`crates/rython_to_ir/src/codegen/generator.rs:560`), aber `handle_parameters` hat sie vorher schon in die Scope-Map eingefuegt (`crates/rython_to_ir/src/codegen/generator.rs:206`).

Ist-Verhalten: Die Codegen-Phase kann mit ueberschriebenen Symbolen arbeiten, bevor sie den Fehler meldet; lokale Duplikate im selben Scope werden nicht explizit verhindert.

Erwartetes Verhalten: Namen muessen vor Codegen-Effekten validiert werden. Block-Shadowing ist nur zwischen innerem und aeusserem Scope erlaubt.

### B019 - Lookup-Prioritaet widerspricht der Shadowing-Regel

Schweregrad: mittel

Beleg:
- `gen_let` verbietet lokale Namen, die Globals/Consts gleichen (`crates/rython_to_ir/src/codegen/stmt.rs:10`).
- `gen_variable` sucht aber zuerst Consts, dann Globals und erst danach lokale Variablen (`crates/rython_to_ir/src/codegen/expr.rs:1043`).

Ist-Verhalten: Die Lookup-Reihenfolge ist nicht auf innersten Scope zuerst ausgelegt.

Erwartetes Verhalten: Lokale Block-Scopes muessen konsistent innerste zuerst aufgeloest werden; globale Konfliktregeln muessen separat validiert werden.

### B020 - String- und List-Literale setzen nicht existierende Builtins voraus

Schweregrad: hoch

Beleg:
- String-Literale erzeugen ein Struct `string` und rufen `init_start`/`push_char` auf (`crates/rython_to_ir/src/codegen/expr.rs:122`).
- List-Literale erzeugen ein Struct `list` und rufen `init_start`/`push_element` auf (`crates/rython_to_ir/src/codegen/expr.rs:156`).
- Es gibt keine Core-Stdlib/Prelude im Workspace (`rg --files` zeigt nur `crates/*`, `examples/*`, `Cargo.toml`, `README.md`).

Ist-Verhalten: Literale funktionieren nur, wenn der User zufaellig passende Structs und Methoden selbst definiert.

Erwartetes Verhalten: Manager/Compiler muss eine Core-Stdlib/Prelude laden oder Literale strukturiert als fehlende Builtins melden.

### B021 - Inline-ASM kann `%&name` nicht ersetzen

Schweregrad: hoch

Beleg:
- `substitute_asm_variables` behandelt nach `%` nur Ident-Startzeichen (`crates/rython_to_ir/src/codegen/stmt.rs:130`).
- `&` ist kein Ident-Startzeichen (`crates/rython_to_ir/src/codegen/stmt.rs:288`).
- `examples/main.ry` nutzt `%&to_print_start` und `%&to_print_length` (`examples/main.ry:36`, `examples/main.ry:37`).

Ist-Verhalten: `%&name` bleibt als roher ASM-Text erhalten und erhaelt kein Adress-Temp.

Erwartetes Verhalten: `%&name` muss durch die Adresse der lokalen/globalen Variable ersetzt werden.

### B022 - Inline-ASM-Ersetzung findet nur lokale Variablen

Schweregrad: mittel

Beleg:
- `substitute_asm_variables` nutzt ausschliesslich `lookup_variable` (`crates/rython_to_ir/src/codegen/stmt.rs:147`).
- `gen_left_value_addr` kann dagegen auch Globals adressieren (`crates/rython_to_ir/src/codegen/expr.rs:299`).

Ist-Verhalten: `%global_name` in ASM kann nicht ersetzt werden, obwohl normale Ausdruecke Globals lesen/schreiben koennen.

Erwartetes Verhalten: ASM-Ersetzung muss dieselbe Namensauflösung wie Expressions verwenden und Wert-/Adressmodus trennen.

### B023 - Lexer akzeptiert unbekannte Escape-Sequenzen als Backslash

Schweregrad: mittel

Beleg:
- `handle_escaped_char` gibt fuer unbekannte Escape-Sequenzen `Ok('\\')` zurueck (`crates/rython_to_ir/src/lexer.rs:663`).
- `LexingError::InvalidEscape` existiert (`crates/rython_to_ir/src/lexer.rs:128`).

Ist-Verhalten: Ein Escape wie `\q` wird nicht als Fehler gemeldet und verliert das Folgezeichen erst in der weiteren Verarbeitung.

Erwartetes Verhalten: Unbekannte Escapes muessen `InvalidEscape` sein.

### B024 - Token-Spans sind ungenau und fuer Diagnostics ungeeignet

Schweregrad: mittel

Beleg:
- `Token::new` verschiebt jeden Start um `+1` (`crates/rython_to_ir/src/lexer.rs:102`).
- Viele Tokens speichern negative Laengen.
- `--emit-tokens examples/test.ry` zeigt z.B. fuer `global`: `start=6`, `end=0`, obwohl das Token am Dateianfang steht.

Ist-Verhalten: Token-Positionen sind nicht stabile Source-Spans.

Erwartetes Verhalten: Lexer, Parser und Codegen-Fehler brauchen konsistente Datei-/Zeilen-/Spalten-Spans.

### B025 - Zahlen-Lexing validiert Suffixe und Separatoren nicht streng

Schweregrad: mittel

Beleg:
- Dezimalzahlen brechen bei ungueltigen Zeichen einfach ab (`crates/rython_to_ir/src/lexer.rs:715`).
- Radix-Zahlen erlauben beliebige `_` im Radix-Teil ohne Positionsvalidierung (`crates/rython_to_ir/src/lexer.rs:683`).

Ist-Verhalten: Inputs wie `123abc` werden als `Int(123)` plus `Ident(abc)` tokenisiert statt als ungueltige Zahl erkannt zu werden.

Erwartetes Verhalten: Zahlenliterale muessen klar validiert und bei ungueltigen Suffixen/Separatoren abgelehnt werden.

### B026 - Variant-Duplikate melden den falschen Fehler-Typ

Schweregrad: niedrig

Beleg:
- `CodegenError` enthaelt `DuplicateVariant` (`crates/rython_to_ir/src/codegen/error.rs:53`).
- Doppelte Variant-Cases melden aber `DuplicateField` (`crates/rython_to_ir/src/codegen/generator.rs:529`).

Ist-Verhalten: Die Fehlerkategorie ist ungenau.

Erwartetes Verhalten: Variant-Cases muessen mit einem passenden Variant-/Case-Fehler gemeldet werden.

### B027 - Beispiele widersprechen dem aktuellen Compilerzustand

Schweregrad: mittel

Beleg:
- `examples/current_features.ry` sagt, es zeige Features, die durch Parser und IR-Codegen funktionieren (`examples/current_features.ry:1`).
- Die Datei enthaelt `while`, `loop` und `and` (`examples/current_features.ry:48`, `examples/current_features.ry:68`, `examples/current_features.ry:96`) und scheitert aktuell im Parser.
- `examples/all_features.ry` enthaelt bewusst nicht fertig unterstuetzte Features, scheitert aber bereits an Syntaxkonflikten wie Wort-Bool-Operatoren und `Vec2::new`.

Ist-Verhalten: Beispielnamen und Kommentare fuehren zu falschen Erwartungen.

Erwartetes Verhalten: Beispiele muessen in aktuelle Pass-/Fail-Fixtures getrennt und an die Zielsyntax angepasst werden.

### B028 - Manager beendet die Pipeline vor dem Backend-Uebergang mit Erfolg

Schweregrad: hoch

Beleg:
- `manager::run` gibt direkt nach optionalem IR-Print `Ok(0)` zurueck (`crates/manager/src/run.rs:146`).
- Der ASM-/Assemble-/Link-/Run-Block ist komplett auskommentiert (`crates/manager/src/run.rs:153`).
- User-Ziel: CLI soll alle Optionen weiterfuehren und am echten IR-to-ASM-Aufruf in einen Backend-Stub laufen.

Ist-Verhalten: `rython_cli examples/test.ry` meldet Erfolg, obwohl weder Backend-Stub noch Build/Run-Pfad erreicht werden.

Erwartetes Verhalten: Nach IR-Erzeugung muss der Backend-Uebergang explizit erreicht werden; bis IR-to-ASM fertig ist, darf dort ein `todo!()`/Stub liegen.

### B029 - CLI-Usage verspricht stderr fuer `--emit-ir`, Manager schreibt aber stdout

Schweregrad: niedrig

Beleg:
- Usage sagt `--emit-ir      Print IR module to stderr` (`crates/rython_cli/src/main.rs:14`).
- `claude_print_ir::print_ir` nutzt `print!` (`crates/manager/src/claude_print_ir.rs:6`).
- `manager::run` ruft diese Funktion direkt auf (`crates/manager/src/run.rs:148`).

Ist-Verhalten: IR wird nach stdout geschrieben.

Erwartetes Verhalten: Entweder stderr verwenden oder Usage korrigieren.

### B030 - CLI/Manager-Optionen sind teilweise tote Optionen

Schweregrad: mittel

Beleg:
- `--emit-asm`, `--keep`, `-o`, `--no-run` und `--release` werden geparst (`crates/rython_cli/src/main.rs:40`), aber wegen `Ok(0)` vor dem Backend nicht wirksam (`crates/manager/src/run.rs:151`).
- `release` wird explizit nur weggebunden (`crates/manager/src/run.rs:97`).

Ist-Verhalten: Optionen werden akzeptiert, ohne den dokumentierten Build-Ablauf zu beeinflussen.

Erwartetes Verhalten: Optionen muessen bis zum Backend-Stub verdrahtet, dokumentiert und testbar sein.

### B031 - Fehlerausgabe ist Debug-basiert und ohne Source-Position

Schweregrad: mittel

Beleg:
- `BuildError::Lex`, `BuildError::Parse` und `BuildError::IrCodegen` formatieren mit `{e:?}` (`crates/manager/src/run.rs:76`).
- Parser- und Codegen-Fehler tragen meist nur `token_idx` oder gar keine Source-Spans (`crates/rython_to_ir/src/parser.rs:5`, `crates/rython_to_ir/src/codegen/error.rs:5`).

Ist-Verhalten: CLI-Fehler enthalten keine Datei-/Zeilen-/Spaltenposition und keine stabile menschenlesbare Diagnose.

Erwartetes Verhalten: Fehler muessen `Display`/Spans haben und in der CLI mit Datei/Position ausgegeben werden.

### B032 - Keine Tests im Workspace

Schweregrad: hoch

Beleg:
- `cargo test` meldet fuer alle Crates `running 0 tests`.

Ist-Verhalten: Es gibt keine automatischen Regressions-, Parser-, Codegen-, Manager- oder CLI-Tests.

Erwartetes Verhalten: Rython->IR-Fertigstellung braucht gezielte Unit- und Integrationstests.

### B033 - Workspace baut mit Warnungen

Schweregrad: niedrig

Beleg:
- `cargo test` erzeugt Warnungen zu unbenutzten Imports/Variablen, `dead_code` und `non_snake_case`.
- Beispiele: unbenutzter Import `rython_to_ir::ast::Let` (`crates/rython_cli/src/main.rs:5`),
  unbenutzter Import `crate::codegen::generator` (`crates/rython_to_ir/src/codegen/generator.rs:4`),
  unbenutzte Backend-Helfer nach auskommentierter Pipeline (`crates/manager/src/run.rs:191`).

Ist-Verhalten: Warnungen verdecken echte Regressionssignale.

Erwartetes Verhalten: `cargo check --all-targets` und `cargo test` sollen warnungsfrei laufen, ausser bewusst dokumentierte Backend-Stubs.

### B034 - AST-Debug-Printer verschluckt `yield`

Schweregrad: niedrig

Beleg:
- `print_stmt` fuer `Stmt::Yield` hat einen leeren Arm (`crates/rython_to_ir/src/ast.rs:431`).

Ist-Verhalten: Ein AST-Printer wuerde `yield`-Statements nicht anzeigen.

Erwartetes Verhalten: Debug-/AST-Ausgabe muss alle AST-Knoten sichtbar machen.
