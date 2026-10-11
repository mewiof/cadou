# 0004: Reference interpreter

- Status: accepted
- Date: 2026-10-11

## Context

Phase 2 needs an interpreter for the core ([spec/core.md](../../spec/core.md)) before the compiler. It is the oracle that compiled programs get compared against (differential testing). It also makes the draft semantics executable while they still change.

The surface syntax does not exist yet and comes last. Unchecked code can do things no compiler checks. The oracle should catch those. Speed matters only as far as fuzzing needs throughput.

## Options

Input:

- the future surface syntax: not designed yet (every syntax change would break the tests)
- a parenthesized form of the core AST with one form per construct: a reader of about a hundred lines/tests that survive syntax changes/no sugar to maintain
- Rust code that builds ASTs: no reader at all (tests become Rust code)

Evaluation:

- walking the checked AST: closest to the spec text and the easiest to audit
- a bytecode machine: faster and a second translation that can hide bugs of its own
- an executable semantics in a proof assistant (Lean 4 e.g.): the long-term goal for the semantics but years away from running real programs

Memory:

- values only (no addresses): cannot run unchecked code or C layouts
- bytes with provenance as in Miri (each pointer carries its allocation and an offset, each byte knows whether it was written): catches out-of-bounds/use after free/reads of unwritten memory in unchecked code

## Evidence

- Miri interprets Rust with this kind of memory model. Its README (read 2026-10-11) lists what that buys: "Out-of-bounds memory accesses and use-after-free"/"Invalid use of uninitialized data"/misaligned accesses/broken type invariants/data races. Stacked Borrows and Tree Borrows ([REFERENCES.md](../REFERENCES.md)) were both built and checked in Miri.
- The bootstrap compiler is Rust ([ADR 0001](0001-bootstrap-language.md)). An interpreter in Rust can reuse the compiler's reader and checker while its evaluator stays independent of the compiler's IRs and backend.

## Decision

- A tree-walking interpreter written in Rust evaluates the checked core AST. It shares the reader and the type checker with the compiler and nothing after them.
- Input is the parenthesized core notation of the spec examples. Each form maps to one AST node. Sugar is not allowed (it would grow into a second language).
- Memory is bytes with provenance plus per-byte initialization. Every check of [spec section 8](../../spec/core.md#8-traps-and-undefined-behavior) runs. A trap or a detected undefined behavior stops the run with a report that names the spec section.
- Results are deterministic: the same program and input give the same output and the same exit status on every host.
- Tests are golden files: a program plus its expected output and exit status.

Milestones (also in [ROADMAP.md](../ROADMAP.md)):

1. integers/booleans/functions/control flow with every arithmetic check
2. structs/tagged unions/match
3. arrays/slices/pointers/the heap with bounds checks and provenance
4. modules
5. calls into a fixed set of host functions (output first) as the stand-in for C calls

## Consequences

- Tests written now outlive the syntax. The surface syntax will lower to the same AST.
- The checker is shared. A front-end bug can hide from differential testing. Golden tests of the checker have to cover that.
- The interpreter is slow by design. It is no benchmark contestant.

## Revisit when

- Interpretation limits fuzzing throughput (programs per second below what the fuzzer can generate).
- A mechanized semantics can run the golden tests (then it becomes the oracle).
- The surface syntax lands. The core notation either stays as a test format or goes.
