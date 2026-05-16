Du arbeitest als unabhängiger Test-Reviewer für mein Rust-Compiler-Projekt.

Die Programmiersprache heißt Rython. Der relevante Compiler-Pfad ist aktuell:

Rython → IR

Der Teil IR → Assembly ist noch nicht relevant und soll nicht bewertet werden.

Deine Aufgabe ist es, die vorhandenen Tests auf Vollständigkeit, fachliche Korrektheit und Qualität zu prüfen. Du sollst nicht primär neue Tests schreiben, sondern zuerst bewerten, ob die vorhandenen Tests wirklich ausreichend sind.

Die relevanten Directories sind:

- `rython_to_ir/`
- `manger/`
- `rython_cli/`

Im Directory `rython_to_ir/` gibt es bereits ein Testdirectory. Prüfe dort die vorhandenen Tests besonders gründlich.

## Wichtige Grundregel

Bewerte die Tests nicht danach, ob sie aktuell grün laufen.

Bewerte sie danach, ob sie das fachlich korrekte Verhalten der Sprache Rython und des Compiler-Pfads Rython → IR absichern.

Die Tests dürfen nicht einfach das aktuelle Verhalten der Implementierung bestätigen. Sie sollen echte Fehler finden können.

Wenn Tests nur das aktuelle Verhalten des Codes nachbilden, ohne zu prüfen, ob dieses Verhalten semantisch korrekt ist, markiere sie als schwach oder problematisch.

## Was du prüfen sollst

Prüfe, ob die Tests ausreichend abdecken:


1. Lexer
   - Keywords
   - Identifier
   - Literale
   - Operatoren
   - Trennzeichen
   - Kommentare
   - Whitespace
   - ungültige Zeichen
   - fehlerhafte Eingaben
   - Edge Cases

2. Parser
   - gültige Syntax
   - ungültige Syntax
   - Operatorpräzedenz
   - Assoziativität
   - Klammerung
   - Deklarationen
   - Zuweisungen
   - Funktionen
   - Funktionsaufrufe
   - Kontrollfluss
   - Blöcke
   - verschachtelte Konstrukte
   - Fehlerfälle

3. AST
   - korrekter Aufbau der Nodes
   - korrekte Verschachtelung
   - korrekte Namen, Typen, Literale und Operatoren
   - Spans / Positionsdaten, falls vorhanden
   - Unterscheidung ähnlicher Konstrukte
   - Edge Cases

4. IR
   - Konstanten
   - Variablen
   - temporäre Werte
   - Operationen
   - Kontrollfluss
   - Labels / Basic Blocks, falls vorhanden
   - Funktionssignaturen
   - Rückgabewerte
   - Typinformationen
   - Validität und Konsistenz der IR

5. Codegeneration Rython → IR
   - einfache Programme
   - arithmetische Ausdrücke
   - boolesche Ausdrücke
   - Variablen
   - Zuweisungen
   - Funktionen
   - Funktionsaufrufe
   - Rückgabewerte
   - if / else
   - Schleifen, falls vorhanden
   - Scoping
   - Typen
   - mehrere Funktionen
   - komplexe Ausdrücke
   - Fehlerfälle

6. `manger/`
   - Prüfe, welche Funktionalität dieses Modul hat.
   - Bewerte, ob es sinnvolle Unit- und Integrationstests gibt.
   - Prüfe Fehlerfälle, Edge Cases und typische Nutzungsfälle.
   - Prüfe, ob Tests nur oberflächlich sind oder echte Funktionalität absichern.

7. `rython_cli/`
   - Prüfe, ob CLI-Verhalten ausreichend getestet ist.
   - Argumente und Flags
   - gültige und ungültige Eingaben
   - Fehlermeldungen
   - Exit Codes
   - Dateipfade
   - Integration mit `rython_to_ir`
   - Verhalten bei fehlenden Dateien
   - Verhalten bei fehlerhaftem Rython-Code

Dies warne beispiele für Bereich, falls es manche Bereiche nicht gibt müssen sie natürlich auch nicht getestet werden oder falls noch irgendwas vergessen wurde musst du diese natürlich noch hinzufügen, diese beispiels liste gilt nur als beispiel und nicht als klare vorlage.

## Vorgehensweise

1. Verschaffe dir einen Überblick über die Projektstruktur.
2. Lies vorhandene Dokumentation, README-Dateien, Beispiele und vorhandene Tests.
3. Analysiere die erwartete Semantik von Rython.
4. Prüfe die Tests in `rython_to_ir/`, `manger/` und `rython_cli/`.
5. Unterscheide klar zwischen:
   - gut abgedeckten Bereichen
   - teilweise abgedeckten Bereichen
   - gar nicht abgedeckten Bereichen
   - schwachen Tests
   - veralteten Tests
   - Tests, die nur aktuelles Verhalten bestätigen
6. Prüfe, ob negative Tests vorhanden sind.
7. Prüfe, ob Edge Cases vorhanden sind.
8. Prüfe, ob Integrationstests vorhanden sind.
9. Prüfe, ob Tests konkrete Erwartungen prüfen oder nur oberflächlich sind.
10. Prüfe, ob Snapshot-Tests oder Output-Vergleiche sinnvoll sind oder nur bestehendes Verhalten zementieren.

## Besonders wichtig

Passe keine Tests automatisch an, nur damit sie grün laufen.

Wenn ein Test fehlschlägt, bewerte zuerst, ob der Test fachlich korrekt ist.

Wenn ein Test fachlich korrekt ist und fehlschlägt, ist das wahrscheinlich ein Compiler-Bug oder eine fehlende Implementierung.

Wenn Verhalten unklar ist, stelle mir Rückfragen.

Du sollst mir ausdrücklich Rückfragen stellen, falls:
- die Rython-Semantik unklar ist
- mehrere Interpretationen möglich sind
- vorhandene Tests und Dokumentation widersprüchlich sind
- du nicht sicher bist, ob ein Verhalten gewollt oder zufällig ist
- eine wichtige Designentscheidung fehlt

## Ergebnis

Erstelle am Ende einen ausführlichen Review-Bericht mit folgenden Abschnitten:

1. Gesamtbewertung
   - Wie vollständig ist das Testset insgesamt?
   - Welche Bereiche sind stark?
   - Welche Bereiche sind schwach?

2. Abdeckung nach Bereich
   - Lexer
   - Parser
   - AST
   - IR
   - Codegeneration Rython → IR
   - `manger/`
   - `rython_cli/`

3. Fehlende Testfälle
   - Liste konkrete fehlende Testbereiche auf.
   - Gruppiere sie nach Priorität: hoch, mittel, niedrig.

4. Problematische Tests
   - Tests, die zu oberflächlich sind
   - Tests, die nur aktuelles Verhalten bestätigen
   - Tests, die veraltet wirken
   - Tests mit unklaren Erwartungen
   - Tests, die zu spröde sind

5. Qualität der Teststruktur
   - Dateistruktur
   - Benennung
   - Hilfsfunktionen
   - Lesbarkeit
   - Wartbarkeit
   - Redundanz

6. Einschätzung der funktionalen Korrektheit
   - Wo sichern die Tests wirklich korrekte Semantik ab?
   - Wo besteht die Gefahr, dass Bugs unentdeckt bleiben?

7. Konkrete Empfehlungen
   - Welche Tests sollten ergänzt werden?
   - Welche Tests sollten überarbeitet werden?
   - Welche Tests können entfernt werden?
   - Welche Test-Hilfsfunktionen wären sinnvoll?

8. Offene Fragen an mich
   - Stelle alle Rückfragen, die nötig sind, um die Testabdeckung wirklich beurteilen zu können.

## Wichtiges Ziel

Das Ziel ist nicht, möglichst viele grüne Tests zu haben.

Das Ziel ist ein Testset, das zuverlässig erkennt, ob der Compiler-Pfad Rython → IR funktional korrekt ist.

Sei kritisch, gründlich und unabhängig.
