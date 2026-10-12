# 0006: Literal types and unused values

- Status: accepted
- Date: 2026-10-12

## Context

The checker of the reference interpreter ([ADR 0004](0004-reference-interpreter.md)) needs two rules that the draft of [spec/core.md](../../spec/core.md) left open. What type does a literal like `1` have? What happens to a value nobody reads (a call made for its effect)? Section 7 of the draft planned must-use types for errors and said nothing about other values.

## Options

Literals:

- a default type where nothing else decides (Rust takes `i32` and `f64`)
- an integer of unlimited size at compile time that converts where it lands (Zig's `comptime_int`)
- the type the surroundings expect and an error where nothing decides

Unused values:

- a warning for marked types only (Rust's `#[must_use]`)
- an error for every unused value that is not unit (Zig)
- nothing

## Evidence

Checked on 2026-10-12 with the pinned compilers:

- rustc 1.99.0 rejects `let x = 3_000_000_000;` with "literal out of range for `i32`". The literal got the default type without anyone writing it.
- rustc 1.99.0 compiles `f();` for an `f` that returns `i32` without a warning.
- Zig 0.17.0 rejects the same call: "value of type 'i32' ignored" + "all non-void values must be used" + "to discard the value, assign it to '_'".
- Zig 0.17.0 rejects `var x = 3;` because the literal is a `comptime_int` ("must be const or comptime"). A runtime variable needs a written type there as well.
- The checker needs no inference for context typing. An expected type flows down into operands and the first operand with a type of its own sets it for the rest (`operands` in `compiler/src/check.rs`). Golden tests: `tests/golden/check/literal-no-type.core`/`literal-range.core`/`unused.core`.

## Decision

- A literal takes the type its place expects: the type of a `let`/a parameter/the other operand/the function's result. A literal whose type nothing decides is an error. There is no default type and no unlimited integer.
- A form whose value nobody reads has type unit or never. `(discard v)` drops any other value. Section 7's must-use types are gone because every value is must-use.

## Consequences

- No literal silently becomes `i32`. `(out 5)` is an error and a typed local fixes it.
- Neither the interpreter nor the compiler needs arithmetic on unlimited integers for the core.
- A call made only for its effect needs `discard` when it returns a value.
- Errors as values (section 7) need nothing beyond this rule to be impossible to ignore.

## Revisit when

- Ported benchmarks show `discard` on more than a few percent of their lines.
- Compile-time execution arrives (an unlimited integer type could serve literals and compile-time code alike).
