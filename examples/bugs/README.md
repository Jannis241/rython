# Bug demonstrations

Each `.ry` file in this directory demonstrates one currently known source-level bug or unfinished
Rython-to-IR behavior. Most files are expected to fail today; files whose names contain `accepted`
compile today but should become errors.

Not every bug in `BUG_REPORT.md` can be demonstrated by a standalone Rython program. CLI stream
routing, Rust warnings, missing Rust tests, and internal naming/cleanup issues need command-level
or Rust-level tests instead of `.ry` fixtures.

