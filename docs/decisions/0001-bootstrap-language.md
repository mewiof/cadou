# 0001: Bootstrap compiler language

- Status: accepted
- Date: 2026-10-11

## Context

The first CADOU compiler gets written in an existing language. It lives until self-hosting (phase 6) and that may take years. It has to:

- express a front end and several IRs comfortably (tagged unions with exhaustive matching)
- build and run fast (compile speed is a measured metric and part of developer experience)
- work for a long time on Windows and Linux (x86-64 and AArch64)
- reach LLVM now and maybe Cranelift or an own backend later ([ADR 0002](0002-initial-backend.md))
- port to CADOU itself at self-hosting time (CADOU has no garbage collector)

## Options

| language | for | against |
| --- | --- | --- |
| Rust 1.99 | tagged unions + exhaustive `match`. No GC (arena-and-index data structures carry over to CADOU almost line by line). Speed in C's class. `cargo test`/fuzzing/Miri. Cranelift is a Rust crate. Stable since 2015 with editions for breaking changes | slow builds of the compiler itself. The borrow checker fights graph-shaped data unless it lives in arenas |
| OCaml 5 | made for compilers (ADTs/matching/fast builds) | GC idioms (sharing/persistent structures/closures) do not port to a GC-free CADOU and self-hosting would become a rewrite. Native Windows support only since opam 2.2.0 (2024-07-01). No Cranelift |
| Zig 0.17 | closest in spirit (allocators + comptime) | breaks on every release ([survey 4.4](../research/survey.md#44-churn)) and had no async from 0.11 to 0.16 ([4.3](../research/survey.md#43-asyncawait-removed-then-redesigned)). A bootstrap compiler would chase the language instead of building ours |
| C++23 | fast with LLVM's own API | no exhaustive matching (`std::variant` plus `visit`)/undefined behavior throughout/slow builds |
| C | fast and tiny | no tagged unions or generics. Rich IRs turn into casts and conventions |
| Lean 4 | one language for the compiler and the mechanized semantics | GC with a small ecosystem for native code generation |

## Evidence

- Rust 1.99.0 builds and runs all six Rust benchmarks on this machine. Its LLVM (23.1.1) has the major version of the pinned clang (23.1.3). A compiler emitting LLVM IR shares an optimizer with two competitors.
- Rust's compile-time problem is real. Average build satisfaction is 6 out of 10 and 55% of respondents wait more than ten seconds for an incremental rebuild (Rust compiler performance survey from 2025-09-10). A bootstrap compiler stays small for a long time and that bounds the damage. The revisit conditions put numbers on it.
- Phase 3 compares temporal-safety models against Rust's borrow checker ([survey 3](../research/survey.md#3-rust) + [6](../research/survey.md#6-ownership-research)). Writing the compiler in Rust gives daily first-hand data on that annotation burden.
- opam 2.2.0's release notes (2024-07-01) announce native Windows support "after 8 years' effort".

## Decision

Rust on the stable channel with edition 2024. The version gets pinned in `compiler/rust-toolchain.toml` when the crate exists.

Ground rules for the compiler code:

- AST and IR nodes live in arenas and point at each other through typed indices (never `Rc<RefCell<_>>` graphs). A CADOU port needs exactly that shape.
- No dependency beyond `std` without a written reason. Each one costs build time and supply-chain trust.
- No `unsafe` unless a measurement asks for it (and then in one module).
- The repository's text and comment rules apply ([STYLE.md](../STYLE.md)). Naming follows rustc's lints.

## Consequences

- Self-hosting translates arenas and indices instead of redesigning around a missing GC.
- Rust's build times become the bootstrap compiler's build times. They get measured like everything else.
- Cranelift stays one crate away for a fast debug backend.
- Contributors need rustup (the benchmarks already do).

## Revisit when

- An incremental rebuild after a one-line change takes more than 10 s on the reference machine after a sensible split into crates. Or a clean build takes more than 2 minutes.
- Self-hosting starts. The bootstrap compiler freezes then instead of growing.
- A Rust release or edition breaks the compiler badly enough to cost more than a day.
