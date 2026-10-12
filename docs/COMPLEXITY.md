# Complexity budget

Five numbers that every change touching the language updates. An addition states what it costs here and why it is worth it. A removal needs no justification.

| metric | counted as | now |
| --- | --- | --- |
| keywords | reserved words of the core notation ([spec/notation.md](../spec/notation.md)) until a surface syntax exists | 40 |
| grammar productions | rules in the grammar in `spec/` | 0 |
| spec size | words in `spec/` without code blocks | 2626 (draft of the core + the notation) |
| core compiler size | non-blank non-comment lines in `compiler/src/` without test modules | 1948 (reader/checker/reference interpreter) |
| language features | rows in the table below | 16 |

There are no limits yet. The phase 2 core sets them from its own numbers. Raising one later takes an ADR.

## Features

Forms are the heads a feature adds to the core notation (reserved words and operator symbols). The spec column names the sections of [core.md](../spec/core.md) that define it.

| feature | added | forms | spec | why it pays |
| --- | --- | --- | --- | --- |
| `bool`/fixed-width integers/`f32`/`f64`/`unit` | 2026-10-12 | 14 | 3.1 | one width per type on every target |
| literals typed by their place | 2026-10-12 | 0 | 3.10 | no hidden default type ([ADR 0006](decisions/0006-literal-types-and-unused-values.md)) |
| never | 2026-10-12 | 0 | 3.9 | `break`/`return` fit anywhere |
| functions and calls | 2026-10-12 | 2 | 3.7/5 | the unit of code |
| blocks/locals/`discard` | 2026-10-12 | 4 | 5 | no value gets dropped unseen (ADR 0006) |
| `if` | 2026-10-12 | 1 | 5 | choice |
| `loop`/`break`/`continue` with labels | 2026-10-12 | 3 | 5 | the one loop (while and for lower to it) |
| integer arithmetic with wrapping/saturating/checked forms | 2026-10-12 | 15 | 6.1/6.2 | intent stated instead of inferred ([ADR 0005](decisions/0005-integer-overflow.md)) |
| bit operations and shifts | 2026-10-12 | 6 | 6.3 | systems code |
| five conversions | 2026-10-12 | 5 | 6.4 | no implicit conversion (3.8) |
| `fma` | 2026-10-12 | 1 | 6.5 | fusion only where written (C compilers fuse on their own per [ADR 0003](decisions/0003-benchmark-methodology.md)) |
| comparisons and `and`/`or`/`not` | 2026-10-12 | 9 | 6.6 | conditions |
| structs with `make` and `.` | 2026-10-12 | 3 | 3.4 | aggregates in C layout with one way to build them |
| tagged unions with `match` | 2026-10-12 | 2 | 3.5/5 | the only way to read a tag and an exhaustive one |
| `opt` | 2026-10-12 | 1 | 3.6 | absence without null |
| `out` | 2026-10-12 | 1 | 9.1 | output for golden tests until calls into C exist |
