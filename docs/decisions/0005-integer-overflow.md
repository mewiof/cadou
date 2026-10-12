# 0005: Integer overflow

- Status: accepted
- Date: 2026-10-12

## Context

[spec/core.md](../../spec/core.md) section 6.1 makes an overflow of `+`/`-`/`*` trap in a checked build. Section 11.4 left open what an unchecked build does with it. It can wrap or it can assume that no overflow happens (C's undefined behavior for signed types). The choice waited for the cost of the checks.

## Options

Checked builds:

- trap on every overflow (the draft)
- wrap and keep the checks for debug builds (Rust's release builds as measured)

Unchecked builds and unchecked blocks:

- wrap: every program keeps a defined meaning
- assume no overflow: the optimizer may build on it and an overflow is undefined

## Evidence

From [overflow.md](../research/overflow.md) (nine benchmarks on the reference machine):

- Checks cost 3.9% for Rust/6.4% for clang/11.4% for gcc (geometric means). Without spectral-norm they cost 0.7%/0.4%/1.5%. In spectral-norm the checks take away the vector divisions in all three compilers.
- Assuming no overflow buys nothing over wrapping. Wrapping moves the geometric mean by -0.9% for clang (faster) and +0.2% for gcc. Eight benchmarks tie for each compiler. On spectral-norm clang's wrapping build is 7% faster and gcc's 2% slower.
- The usual case for the undefined behavior did not show up. Every C program indexes with 32-bit `int`. Wrapping slowed one of them down by more than noise (gcc's spectral-norm).
- [clang-msvc.md](../research/clang-msvc.md) found the same for clang's whole Windows default (`-fwrapv` plus no type-based alias analysis).

## Decision

- Checked builds trap on overflow as section 6 says. The safe default costs about 4% on the suite and most of it in one loop.
- Unchecked builds and unchecked blocks wrap. No build mode makes plain arithmetic undefined.
- `unchecked+`/`unchecked-`/`unchecked*` stay the explicit way to promise that no overflow happens. A measurement has to show that the promise pays before code uses it.

## Consequences

- Without unchecked operations and removed bounds checks a program means the same in every build mode. An unchecked build agrees with the interpreter on every run that does not overflow.
- The LLVM lowering ([ADR 0002](0002-initial-backend.md)) emits checked arithmetic through the overflow intrinsics plus a trap. Unchecked builds emit plain `add`/`sub`/`mul` without `nsw`. Only the explicit unchecked operations carry `nsw`/`nuw`.
- A checked loop that loses its vector code is a gap phase 4 has to close. The first candidate is one check in front of the loop that covers every round.

## Revisit when

- A benchmark shows a significant win for assuming no overflow over wrapping on any host.
- Linux (where clang uses standard semantics by default) or AArch64 numbers disagree.
- Phase 4 cannot get checked loops back to vector code.
