# 0002: Initial backend

- Status: accepted
- Date: 2026-10-11

## Context

CADOU needs machine code good enough to compete with clang and rustc at `-O3` soon after the front end exists. The compiler lowers to its own SSA mid-level IR (MIR) first. The backend then sits behind one interface and can be swapped. x86-64 and AArch64 matter from day one (Linux userspace first). The phase 4 hypotheses (non-aliasing references/layout control/specialization) need a backend that can be told about aliasing and about well-defined arithmetic.

## Options

- **LLVM through textual IR** (`.ll` files compiled by the pinned LLVM). It is the optimizer of clang 23.1.3 and rustc 1.99.0 (LLVM 23.1.1). Differences against those two then come from the language rather than the optimizer. On both targets the IR can state `noalias` parameters/scoped alias metadata/`nsw` and `nuw`/`poison` and `freeze`/`llvm.assume`/vector types/calling conventions/inline assembly/DWARF. Nothing to link. IR text is easy to golden-test and `opt -passes=verify` is a ready IR validator. Against it: "The textual format is not backwards compatible. We don't change it too often, but there are no specific promises" (LLVM Developer Policy, read 2026-10-11). LLVM is also slow for debug builds.
- **LLVM through the C API or inkwell.** No text round trip. The compiler then links LLVM and follows its API version by version.
- **C as the target.** Any C compiler would do. Aliasing can only be stated with `restrict` on pointers (nothing per field or per scope). Well-defined overflow means unsigned arithmetic and losing `nsw`. The calling conventions are C's. The numbers would mostly measure the C compiler.
- **Cranelift.** Its code generation is "about an order of magnitude faster than an equivalent LLVM-based system" with output "~14% slower than an LLVM-based system". It has "explicitly avoided" advanced alias analysis and any use of undefined behavior (cranelift.dev, read 2026-10-11). Good for debug builds but blind to what aliasing information buys.
- **QBE.** Aims at "70% of the performance of industrial optimizing compilers in 10% of the code" (c9x.me/compile, read 2026-10-11). Too far from the bar.
- **Our own backend now.** Years of work before it beats LLVM at `-O3`. It belongs in phases 4 and 5 as a measured experiment per build mode.

## Evidence

- The [noalias experiment](../research/noalias.md) adds an array into `*total` with and without a no-alias promise. clang 23.1.3 and rustc 1.99.0 vectorize both versions behind a run-time overlap check. gcc 16.2.0 needs `restrict` and is 2.78 times slower without it. Zig 0.17.0 (LLVM 22.1.8) stays slow with and without `noalias` although LLVM 23.1.3's `opt -O3` vectorizes the very same Zig IR. rustc already emits `noalias` for `&mut` and `&` parameters (the attribute CADOU's references would map to).
- LLVM is not best everywhere. On identical C clang needs 45% more time than gcc for spectral-norm. gcc vectorizes the independent part of the inner loop (index arithmetic/int-to-double conversion/division on 4 lanes) and keeps only the additions in order. LLVM leaves the whole loop scalar ([baseline report](../../bench/results/2026-10-11-i7-14700kf-windows/report.md) + assembly read on 2026-10-11).
- The pinned clang (23.1.3) and rustc 1.99.0 (LLVM 23.1.1) share a major LLVM version. Against them a CADOU on the same LLVM measures the language rather than the optimizer.

## Decision

The MIR lowers to textual LLVM IR that the pinned LLVM (`bench/toolchains.toml` with 23.1.3 today) optimizes and links. Tests run every emitted module through `opt -passes=verify`. The MIR does not depend on LLVM types. A second backend means a new lowering rather than a rewrite.

## Consequences

- One LLVM version at a time. Upgrades happen on purpose with the benchmark suite run before and after.
- Debug builds compile at LLVM's speed until a faster backend (Cranelift or our own) is measured and chosen.
- Golden tests can check the IR text directly.

## Revisit when

- Writing and parsing IR text takes more than 10% of compile time in a profile.
- A debug-build backend gets prototyped (phase 5).
- An LLVM upgrade breaks the emitted IR. Or an own backend beats LLVM in some build mode on the suite.
- A transformation LLVM lacks (gcc's partial vectorization of in-order sums e.g.) costs CADOU a benchmark. The first answer is a MIR pass of our own before any backend change.
