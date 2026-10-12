# Core notation

The test format of the core ([ADR 0004](../docs/decisions/0004-reference-interpreter.md)) and not the surface syntax. Each form maps to one node of the checked core. [core.md](core.md) defines what the forms mean. `compiler/` reads/checks/runs it: `cargo run -- run file.core`.

## Reading

- A program is a sequence of declarations. A form is a list in parentheses that starts with a word.
- `//` starts a comment that runs to the end of the line.
- Names are letters/digits/`_` with no digit first. The words of this page are reserved.
- Integer literals are decimal or hex (`0xff`) with an optional `-` and `_` between digits. A float literal has a fraction or an exponent or both: `1.5`/`-0.0`/`2.5e-3`.
- Labels are names with a colon in front: `:outer`.
- Lists nest at most 200 deep.

## Declarations

```
(fn name ((param type) ...) result body ...)
(struct Name (field type) ...)
(union Name (case type) ... (case) ...)
```

A union case written `(case)` carries `unit`.

## Types

`bool` `i8` `i16` `i32` `i64` `u8` `u16` `u32` `u64` `f32` `f64` `unit` + the names of structs and unions + `(opt type)`.

## Expressions

| form | meaning | core.md |
| --- | --- | --- |
| `name` | reads a local | 4.1 |
| `(f arg ...)` | calls a function | 5 |
| `(let name type value)` | a new local for the rest of the block (only directly in a block) | 5 |
| `(set place value)` | writes a local or a field of one | 5 |
| `(do form ...)` | a block: the value of its last form | 5 |
| `(if condition then)` `(if condition then else)` | | 5 |
| `(loop :label form ...)` | repeats its forms (the label is optional) | 5 |
| `(break :label)` `(continue :label)` | leave the innermost or the labeled loop | 5 |
| `(return value)` | leaves the function (no value for unit) | 5 |
| `(discard value)` | drops a value | 5 |
| `(out value)` | writes a value and a newline | 9.1 |
| `(+ a b)` `(- a b)` `(* a b)` `(neg a)` | trap on overflow | 6.1 |
| `(wrapping+ a b)` `(saturating+ a b)` `(checked+ a b)` | the same for `-` and `*` | 6.1 |
| `(/ a b)` `(% a b)` | | 6.2 |
| `(& a b)` `(\| a b)` `(^ a b)` `(~ a)` `(<< a n)` `(>> a n)` | | 6.3 |
| `(widen type v)` `(narrow type v)` `(wrap type v)` `(saturate type v)` `(float type v)` | | 6.4 |
| `(fma a b c)` | a * b + c rounded once | 6.5 |
| `(== a b)` `(!= a b)` `(< a b)` `(<= a b)` `(> a b)` `(>= a b)` | | 6.6 |
| `(and a b)` `(or a b)` `(not a)` | | 6.6 |
| `(make type (field value) ...)` | a struct with every field once | 3.4 |
| `(make type (case value))` `(make type (case))` | a union or opt value | 3.5 |
| `(. value field)` | a field of a struct | 3.4 |
| `(match value ((case name) body) ((case) body) ...)` | one arm per case | 3.5 |
