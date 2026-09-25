# rython

My first compiler, for my own statically typed language, written in Rust together with my brother [@jesko7](https://github.com/jesko7).

rython reads the source code, turns it into tokens (lexer), builds an AST (parser) and then translates it into our own intermediate representation (IR). While generating the IR, it also checks the program for errors, like wrong types or unknown variables. The last step, turning the IR into assembly is not done yet.

> **Status:** paused. At the moment I'm working on [jcc](https://github.com/Jannis241/jcc), a smaller compiler written in C, to learn C.

## Status

| Stage | Status |
|---|---|
| Lexer | ✅ done |
| Parser (AST) | ✅ done |
| Checks + translation to IR | ✅ works, but has some known bugs (see [`examples/bugs`](examples/bugs)) |
| IR to assembly | ⏳ not started |

## The language

```
struct Point {
    x: int,
    y: int,

    fn add(this, other: Point) Point {
        return Point {
            x: this.x + other.x,
            y: this.y + other.y,
        };
    }

    fn operator + add_op(this, rhs: Point) Point {
        return this.add(rhs);
    }
}

variant Status {
    Ready,
    Done,
}

const max_value: int = 100;

fn main() int {
    let a: Point = Point { x: 3, y: 4 };
    let b: Point = Point { x: 5, y: 6 };
    let c: Point = a + b;

    if c.x > max_value {
        return 0;
    }

    return c.x;
}
```

What already works (parsed, checked and translated to IR):

- constants, globals and local variables with types (`int`, `float`, `bool`, `char`)
- structs with methods and operator overloading (`+`, `-`, `[]`)
- variants (like enums)
- `if` / `else`, blocks and scopes
- arithmetic, comparison, logical and bitwise operators, `+=`, `++` and similar
- inline `asm` blocks

All of this is in [`examples/implemented_features.ry`](examples/implemented_features.ry).

We also planned bigger features like traits, generics, imports and iterators. The parser can already read some of this syntax, but it is not translated to IR yet. Our plans are in [`examples/target_features.ry`](examples/target_features.ry).

## How it works

```
source code -> lexer -> tokens -> parser -> AST -> checks + IR generation -> IR -> (assembly)
```

**Lexer** (`crates/rython_to_ir/src/lexer.rs`): turns the source code into tokens.

**Parser** (`crates/rython_to_ir/src/parser.rs`): a handwritten recursive descent parser. For expressions there is one function per precedence level, from `or` down to `primary`, so operator precedence works automatically.

**Checks + IR generation** (`crates/rython_to_ir/src/codegen/`): goes through the AST, checks the program and generates the IR at the same time. It keeps track of scopes, so it knows which variables are visible where. Some of the errors it finds:

- unknown variables, functions, types or fields
- mismatched types, for example `1 + true` or returning the wrong type
- calling a function with the wrong number of arguments
- assigning to a constant
- `break` or `continue` outside of a loop
- duplicate names (functions, types, fields, parameters)

**The IR** (`crates/rython_to_ir/src/ir.rs`): our IR is a bit similar to LLVM IR. Every function is made of basic blocks, and every block ends with a terminator (`ret`, `jump` or `branch`). Local variables live in memory (`alloca`, `load`, `store`), and the results of calculations are saved in numbered temporaries.

## Build and run

```sh
cargo build
cargo run -p rython_cli -- --emit-ir --no-run examples/implemented_features.ry
```

Options:

```
-o <path>        set the output path
--emit-tokens    print the tokens
--emit-ast       print the AST
--emit-ir        print the IR
--keep           keep intermediate files
--no-run         only build, don't run the program
```

Because the backend is missing, you need `--no-run` at the moment.

## Project structure

```
crates/rython_to_ir    lexer, parser, AST, checks and IR generation
crates/manager         runs all compiler steps one after another
crates/rython_cli      command line interface
crates/ir_to_assembly  backend (not started)
examples/              example programs and bug examples
```

## Known bugs

Every file in [`examples/bugs`](examples/bugs) shows one known bug. More details are in [`BUG_REPORT.md`](BUG_REPORT.md).
