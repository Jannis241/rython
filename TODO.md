# Todo: Rython to IR, Manager und CLI fertigstellen

Stand: 2026-05-24

Diese Liste beschreibt die Arbeit, die noetig ist, damit Rython bis IR inklusive Manager und CLI
als fertig gelten kann. IR to ASM selbst ist nicht Teil dieser Liste, ausser dem expliziten
Backend-Uebergang/Stub.

## 1. Sprache und Spezifikation festziehen

- Aktuelle Zielsyntax im Repo dokumentieren:
  `&&`, `||`, `!`; keine Tail-Expressions; Semikolonpflicht fuer `return`/`yield`;
  `while`/`loop` als Statement und Expression; Rust-aehnliche Pfade; statische Traits;
  Monomorphisierung; Core-Stdlib/Prelude; Iterator-basierte `for`-Loops.
- Beispiele bereinigen und trennen:
  funktionierende Beispiele, negative Compile-Fail-Beispiele und Feature-Showcases mit klarer Erwartung.
- Festlegen und dokumentieren, welche Fehler Phase-spezifisch sind:
  Lexerfehler, Parserfehler, Name-Resolution-Fehler, Typechecking-Fehler, IR-Codegen-Fehler.
- Alle alten/irrefuehrenden Kommentare im Code entfernen oder aktualisieren, besonders Kommentare, die nicht mehr zur Zielsemantik passen.

## 2. Lexer fertigstellen

- Token-Spans auf stabile Source-Spans umstellen:
  Datei, Byte-/Char-Range, Zeile und Spalte oder eine eindeutig daraus ableitbare Struktur.
- Negative Token-Laengen und `start_char_idx + 1` durch konsistente 0- oder 1-basierte Span-Konvention ersetzen.
- `--emit-tokens` so korrigieren, dass Token-Slices und Start-/Endpositionen stimmen.
- Unbekannte Escape-Sequenzen als `InvalidEscape` melden.
- Zahlenliterale streng validieren:
  ungueltige Suffixe, fehlerhafte `_`-Positionen, Radix-Digits und Float-Exponenten.
- Lexer-Fehler mit `Display` und Span versehen.
- Unit-Tests fuer Keywords, Operatoren, Kommentare, ASM-Blocks, Strings/Chars, Escapes, Zahlen und Spans schreiben.

## 3. Parser fertigstellen

- `while` und `loop` in `is_statement_start` aufnehmen und als Statements ohne Semikolon parsebar machen.
- Loop-/While-Expressions weiterhin in Expression-Kontexten erlauben.
- Tail-Expression-Umwandlung zu implizitem `return` aus `parse_block` entfernen.
- Fehlendes Semikolon bei `return` und `yield` immer als Parserfehler melden.
- Block-Expressions in AST und Parser einfuehren, damit Wert-Kontexte per `yield` moeglich sind.
- `yield` syntaktisch als explizites Statement mit optionalem Wert und Pflicht-Semikolon definieren.
- Rust-aehnliche Pfade modellieren:
  Variant-Literal `Type::Case`, assoziierter Call `Type::method(args)`, optional spaeter qualifizierte Modulpfade.
- Top-Level-Operatorfunktionen ablehnen; `operator` nur in Struct-Methoden erlauben.
- Generic-Type-Args und Pfade im Parser ohne Spezialfaelle fuer nur Varianten abbilden.
- Parser-Fehler mit Source-Spans und menschenlesbarem `Display` ausstatten.
- Parser-Tests fuer positive und negative Syntaxfaelle schreiben.

## 4. Name Resolution und Module

- Eine eigene Name-Resolution-Phase vor IR-Codegen einfuehren.
- Imports im Manager/Compiler laden:
  `import a.b;` => `<input-root>/a/b.ry`, inklusive Zyklus- und Duplikatpruefung.
- Importierte Top-Level-Items in einen globalen Programmscope mergen.
- Eindeutige Top-Level-Funktionsnamen erzwingen; normales Overloading nicht erlauben.
- Struct-, Variant-, Trait-, Function-, Const- und Global-Namen getrennt und eindeutig validieren.
- Lokale Scope-Regeln umsetzen:
  Block-Shadowing erlauben, Duplikate im selben Scope ablehnen, lokale/Parameter-Namen gegen Globals/Consts verbieten.
- `Self` im passenden Struct-/Impl-/Trait-Kontext aufloesen.
- Qualifizierte Pfade fuer Varianten und assoziierte Funktionen aufloesen.
- Name-Resolution-Tests fuer Imports, Duplikate, Shadowing, `Self`, qualifizierte Namen und Fehlerfaelle schreiben.

## 5. Typchecking und Semantik

- Eine Typchecking-Phase vor oder eng getrennt von IR-Codegen aufbauen.
- Primitive Typen, Struct-Pointer-Konvention, Varianten, Funktionen, Methoden und Void sauber typisieren.
- Nicht-void Funktionen auf vollstaendige Return-Pfade pruefen.
- Alle erreichbaren Kontrollflussbloecke terminatorisch validieren.
- `yield`-Typisierung implementieren:
  alle erreichbaren `yield`-Werte eines Wert-Kontexts muessen denselben Typ haben.
- `return` und `yield` klar trennen:
  `return` Funktionsterminator, `yield` Blockwert/Kontrollfluss innerhalb des naechsten Wert-Kontexts.
- Generics monomorphisieren:
  generische Funktionen/Structs pro verwendeter Typkombination spezialisieren.
- Traits statisch aufloesen:
  Bounds pruefen, Impl-Auswahl treffen, Trait-Methoden auf konkrete Funktionen abbilden.
- `any Trait` gemaess statischer Trait-Strategie behandeln oder klar als nicht gueltiger Zieltyp ablehnen, ohne Panic.
- Operator-Overloads nur als Struct-Methoden typpruefen und registrieren.
- Methoden-/Operator-Dispatch ueber konkrete, monomorphisierte Signaturen statt reiner String-Namen aufloesen.
- Tests fuer Typfehler, Return-Pfade, Yield-Werte, Generics, Traits, Operatoren und Method Calls schreiben.

## 6. IR-Modell bereinigen

- `IrInstruction::Call` fuer Void-Rueckgaben eindeutig modellieren:
  entweder optionales Temp oder separate Void-Call-Form.
- Blocklabels konsistent ohne eingebautes `:` oder mit eindeutig dokumentierter Konvention speichern.
- IR fuer Blockwerte/Yield einfuehren:
  Speicherplatz/Temp fuer yielded Value, Branches zum Zielblock, Typvalidierung.
- IR fuer `loop` und `while` implementieren:
  Header/Body/Continue/Break/Yield/Merge-Bloecke.
- IR fuer `for` ueber Iterator-/Iterable-Traits erzeugen.
- IR fuer assoziierte Funktionen und qualifizierte Namen erzeugen.
- Variant-IR erweitern, falls Cases spaeter Daten tragen sollen; sonst Case-only-Semantik dokumentieren und testen.
- Struct-Pointer-Konvention pruefen und durchgehend dokumentieren:
  wann `Named(T)` Wert ist und wann `Pointer(Named(T))`.
- IR-Pretty-Printer und Debug-Ausgabe an das bereinigte IR-Modell anpassen.

## 7. Codegen fertigstellen

- Alle `todo!()` im Rython->IR-Pfad entfernen:
  `gen_loop`, `gen_while`, `gen_yield`, `gen_expr_block`, `Type::AnyTrait`.
- `Stmt::For` in `gen_stmt` implementieren.
- `Item::Import`, `Item::Trait` und `Item::TraitImplementation` nicht mehr im Codegen als rohe AST-Items sehen:
  sie muessen vorher aufgeloest oder in Codegen-relevante Datenstrukturen transformiert werden.
- RHS-Operator-Dispatch korrigieren:
  Typpruefung und Call-Argumentreihenfolge muessen uebereinstimmen.
- Fehlende Terminatoren/Returns richtig pruefen:
  Entry und alle erreichbaren Bloecke in Kontrollflussanalyse aufnehmen.
- Variable Lookup innersten lokalen Scope zuerst ausfuehren und globale Konfliktregeln separat validieren.
- Parameterduplikate und lokale Duplikate vor der IR-Erzeugung melden.
- String-/List-Literale ueber Core-Stdlib/Prelude absenken.
- Inline-ASM-Ersetzung vervollstaendigen:
  `%name` Wert-Temp, `%&name` Adress-Temp, lokale und globale Symbole, klare Fehler bei unbekannten Namen.
- Codegen-Fehler mit Spans und Display-Meldungen ausstatten.
- IR-Codegen-Tests fuer alle aktuellen AST-Knoten schreiben.

## 8. Core-Stdlib/Prelude

- Einen klaren Ort fuer Core-Stdlib-Dateien anlegen.
- Prelude-Laden im Manager/Compiler definieren:
  immer automatisch, vor User-Imports, mit Duplikatregeln.
- Minimaltypen und Methoden bereitstellen:
  `string`, `list`, Speicher-/Initialisierungsroutinen, `push_char`, `push_element`, Iterator-/Iterable-Traits.
- Abhaengigkeit zwischen Core-Stdlib und Inline-ASM sauber dokumentieren.
- Tests fuer String-Literale, Char-Push, List-Literale und `for` ueber Listen schreiben.

## 9. Manager fertigstellen

- Input-Root bestimmen und an Import-Aufloesung weitergeben.
- Pipeline explizit strukturieren:
  read -> lex -> parse -> resolve imports -> name resolution -> typecheck/monomorphize -> IR -> backend stub.
- Backend-Uebergang herstellen:
  nach erfolgreicher IR-Erzeugung den IR-to-ASM-Aufrufpunkt erreichen und dort bis zur Backend-Implementierung bewusst stubben.
- BuildOptions voll verdrahten:
  `emit_tokens`, `emit_ast`, `emit_ir`, `emit_asm`, `keep_intermediates`, `release`, `run_after_build`, `output_path`.
- Emit-Ziele konsistent machen:
  entweder Usage auf stdout anpassen oder IR/ASM wirklich nach stderr schreiben.
- Fehlerdiagnostik mit Datei/Span durchreichen.
- Manager-Integrationstests mit Temp-Projekten und Imports schreiben.

## 10. CLI fertigstellen

- Unbenutzten Import `rython_to_ir::ast::Let` entfernen.
- Exit-Code-Policy definieren und testen:
  Usage/Argumentfehler, Compilefehler, Backend-Stub, erfolgreicher Build.
- Bei fehlendem Input entscheiden, ob Erfolg mit Usage oder Fehlercode gewollt ist, und dokumentieren.
- `--emit-asm` bis zum Backend-Stub korrekt behandeln.
- `--no-run`, `-o`, `--keep` und `--release` nicht still wirkungslos lassen:
  sie muessen den Buildpfad beeinflussen oder klar als aktuell no-op dokumentiert sein.
- CLI-Integrationstests fuer Hilfe, unbekannte Optionen, mehrere Inputs, falsche Extension, erfolgreiche IR-Erzeugung und Fehlerausgabe schreiben.

## 11. Diagnose und Fehlerqualitaet

- `Display` fuer `LexingError`, `ParseError`, `CodegenError` implementieren.
- Fehler mit Source-Spans versehen:
  Parserfehler duerfen nicht nur `token_idx` enthalten; Codegen-Fehler brauchen Ursprungs-AST-Spans.
- Fehlerkategorien schaerfen:
  `DuplicateVariant`/Case-Duplikate, `DuplicateField`, `DuplicateFunction`, `DuplicateType`, `Ambiguous*`.
- Panics/`expect` im normalen Compilerpfad durch strukturierte Fehler ersetzen.
- CLI-Ausgabe im Format `path:line:column: error: ...` oder aequivalent stabilisieren.
- Negative Tests fuer alle Fehlerklassen schreiben.

## 12. Tests und Qualitaet

- Teststruktur anlegen:
  Unit-Tests in `rython_to_ir`, Integrationstests fuer `manager`/`rython_cli`, Fixture-Verzeichnis fuer `.ry`-Programme.
- Positive Compile-to-IR-Tests:
  Globals/Consts, Funktionen, Structs, Methoden, Field Access, Varianten, Operatoren, Kontrollfluss, Literale, Imports.
- Negative Tests:
  falsche Typen, unbekannte Namen, doppelte Namen, fehlende Returns, ungueltige Syntax, ungueltige Escapes, kaputte Imports.
- Snapshot- oder strukturierte IR-Tests fuer stabile IR-Ausgabe einrichten.
- `cargo test` muss echte Tests ausfuehren und darf nicht nur `0 tests` melden.
- `cargo check --all-targets` und `cargo test` warnungsfrei machen.
- Optional Clippy in CI/Checkliste aufnehmen, sobald Warnungen bereinigt sind.

## 13. Aufraeumen

- Auskommentierten Backend-Code im Manager entweder in einen klaren Stub umbauen oder entfernen, bis IR-to-ASM implementiert wird.
- Tote Imports, Variablen, Felder und Funktionen entfernen oder bewusst markieren.
- `claude_print_ir` in neutralen Namen umbenennen, falls es als offizieller IR-Printer bleibt.
- Schreibweisen wie `mangel`/`preprocces`/`Signatur` vereinheitlichen, wenn die APIs oeffentlich bleiben.
- `Stmt::Yield` im AST-Printer ausgeben.
- Beispiele aktualisieren:
  `current_features.ry` muss wirklich durch Parser und IR-Codegen laufen;
  `all_features.ry` muss entweder Zielsyntax zeigen oder klar als Nicht-Compile-Fixture markiert sein.

## 14. Akzeptanzkriterien fuer "Rython -> IR fertig"

- Jedes gueltige Feature aus der dokumentierten Zielsyntax erzeugt deterministische, typgepruefte IR.
- Ungueltige Programme liefern strukturierte Fehler ohne Panic.
- Imports, Core-Stdlib, Generics, Traits, Impl-Bloecke, `Self`, Loops, `yield`, `for`, String/List-Literale und Inline-ASM-Ersetzung sind abgedeckt.
- Manager und CLI fuehren die Pipeline bis zum IR-to-ASM-Stub aus und behandeln alle Optionen konsistent.
- `cargo check --all-targets` und `cargo test` laufen warnungsfrei.
- Die Test-Suite enthaelt positive und negative Faelle fuer Lexer, Parser, Semantik, IR-Codegen, Manager und CLI.
