# Experiment: loops a run-time check cannot rescue

2026-10-11. Code in [bench/experiments/noalias2/](../../bench/experiments/noalias2/). Raw data in [bench/results/experiments/2026-10-11-noalias2.json](../../bench/results/experiments/2026-10-11-noalias2.json).

## Hypothesis

In the [first experiment](noalias.md) clang and rustc vectorized a loop without a no-alias promise. They compared the two pointers' extents at run time. Such a check needs every pointer and every extent before the loop starts. Three loops where that fails:

- fields: the bound and the data pointer of the loop live in a struct that the loop writes through. A byte store may change either one.
- arrays: three arrays written and three read in one loop. That makes twelve pairs to check.
- call: a sum behind a pointer around a call the compiler cannot see into.

A promise should pay in all three.

## Method

Same flags and timing protocol as [noalias.md](noalias.md). One binary per language takes the kernel and the mode on the command line. Each pair of functions differs only in the promise:

| language | without the promise | with it |
| --- | --- | --- |
| C | plain pointers | `restrict` on every pointer parameter |
| Rust | raw pointers | `&mut` and `&` |
| Zig | plain pointers and slices | `noalias` on every pointer and slice parameter |

All loops index on purpose. Idiomatic iteration over a slice or a range loads the bounds once and would dodge the fields case.

Sizes: fields adds to 16 KiB of bytes 100,000 times. arrays moves 1024 particles 1,000,000 times. call sums 4096 transformed values 25,000 times through a function pointer picked at run time.

## Results

Medians of 30 runs. The ratio is without the promise over with it (95% bootstrap interval).

| kernel | variant | without the promise | with it | ratio |
| --- | --- | --- | --- | --- |
| fields | c-clang | `plain` 311.2 ms | `restrict` 19.2 ms | 16.21 (16.04-16.28) |
| fields | c-gcc | `plain` 346.0 ms | `restrict` 19.3 ms | 17.95 (17.86-18.04) |
| fields | rust | `pointer` 311.6 ms | `reference` 19.7 ms | 15.80 (15.67-15.97) |
| fields | zig | `plain` 317.0 ms | `noalias` 176.1 ms | 1.80 (1.79-1.80) |
| arrays | c-clang | `plain` 141.6 ms | `restrict` 137.8 ms | 1.03 (1.03-1.03) |
| arrays | c-gcc | `plain` 135.8 ms | `restrict` 136.8 ms | 0.99 (0.99-1.00) |
| arrays | rust | `pointer` 141.9 ms | `reference` 139.9 ms | 1.01 (1.01-1.02) |
| arrays | zig | `plain` 592.3 ms | `noalias` 597.3 ms | 0.99 (0.99-1.00) |
| call | c-clang | `plain` 152.1 ms | `restrict` 152.3 ms | 1.00 (1.00-1.00) |
| call | c-gcc | `plain` 151.6 ms | `restrict` 151.8 ms | 1.00 (0.99-1.00) |
| call | rust | `pointer` 152.0 ms | `reference` 79.5 ms | 1.91 (1.90-1.92) |
| call | zig | `plain` 78.6 ms | `noalias` 97.3 ms | 0.81 (0.81-0.81) |

## Generated code

Listings from the same flags without LTO (the calls through pointers keep each kernel as compiled on its own).

fields:

- Without the promise clang/gcc/rustc reload the data pointer and the count around every byte (`mov r8, [rcx]`/`add byte ptr [r8 + rax], dl`/`cmp rax, [rcx + 8]`). No check can help. The trip count is unknown until the loop ends.
- With it they load both once and add 128 bytes per iteration with four 32-byte `vpaddb`.
- Zig's `noalias` hoists both loads as well. Zig 0.17.0 then unrolls the byte loop 8 times and never vectorizes it (as in the [first experiment](noalias.md)).

arrays:

- clang and gcc vectorize the plain loop behind run-time checks of all twelve pairs. rustc does the same for raw pointers. Twelve pairs are within what both compilers accept.
- Zig stays scalar both ways.

call:

- clang and gcc keep the sum in memory with or without `restrict`. Every iteration calls and then loads/adds/stores (`add qword ptr [rbx], rax`).
- rustc gives `&mut u64` three attributes: `noalias`/`dereferenceable(8)`/`align 8`. LLVM then loads the sum once before the loop and keeps it in a register. The store after each call stays (`mov qword ptr [r14], r15`) but the reload is gone.
- It takes all three. Adding them to clang's IR of `ApplyRestrict` one at a time and running `opt -O3` keeps the sum in a register only with all three ([attributes.py](../../bench/experiments/noalias2/attributes.py)). `restrict` supplies the first one. LLVM may move the load in front of the loop only where it cannot fault (a valid and aligned pointer).
- Zig keeps the sum in a register with or without `noalias`. The two functions compile to the same instructions. Zig evaluates `total.* += f(x)` by reading `total.*` before the call: a test program that changes the variable inside the called function prints 6 with Zig 0.17.0 and 101 with clang/gcc/rustc. Each iteration's load then directly follows the previous store and forwards without any alias information. C leaves that order open ("The evaluations of the operands are unsequenced" in C23 6.5.17.1 of N3220). The Rust Reference fixes the other order for primitive compound assignment ("the right hand side is evaluated first", read 2026-10-11). The Zig language reference of 0.17.0 does not specify one.
- The `noalias` version of Zig's loop starts 16 bytes before a 64-byte boundary and spans two cache lines. The plain version fits in one. That is the likely reason identical code runs 24% slower (not tested).

## Conclusion

The hypothesis holds where the loop keeps its own bounds or pointers in memory it writes through. There the promise is worth 16 to 18 times on clang/gcc/rustc and no check can stand in. With many arrays LLVM and gcc still check at run time and the promise buys nothing. Around calls `restrict` alone buys nothing in C. What helps is a reference that is non-aliasing/dereferenceable/aligned at once (Rust's `&mut`) or an evaluation order that reads the left operand first (Zig).

For CADOU:

- References that are valid/aligned/non-aliasing for the whole call give LLVM all three attributes it needs ([PRINCIPLES.md 3.2](../PRINCIPLES.md#32-give-the-optimizer-information)).
- Strict left-to-right evaluation that reads the old value before the right operand ([spec section 5](../../spec/core.md#5-evaluation)) removes the call case without any promise.
- A first version called the kernels directly. clang and rustc then rewrote each kernel for its only caller. The fields kernel received the pointer and the count as two values and the question disappeared (`Brighten(pixels, count, amount)` in clang's listing). Whole-program specialization is a third way out.
