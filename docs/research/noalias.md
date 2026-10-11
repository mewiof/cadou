# Experiment: what a no-alias promise buys

2026-10-11. Code in [bench/experiments/noalias/](../../bench/experiments/noalias/). Raw data in [bench/results/experiments/2026-10-11-noalias.json](../../bench/results/experiments/2026-10-11-noalias.json).

## Hypothesis

[PRINCIPLES.md 3.2](../PRINCIPLES.md#32-give-the-optimizer-information) assumes that mutable references which cannot alias let the compiler keep values in registers and vectorize. It also assumes the gain is big enough to make non-aliasing the default. The smallest loop where this should show adds an array into `*total` through a pointer. If `total` might point into the array then every iteration has to store the running sum and load it back.

## Method

One loop in three languages with and without the promise. Each language gets one binary where a command-line argument picks the version. The functions are `noinline` so the caller cannot prove anything for them.

| language | without the promise | with it |
| --- | --- | --- |
| C | `uint32_t* total` + `const uint32_t* values` | both `restrict` |
| Rust | `*mut u32` + `*const u32` | `&mut u32` + `&[u32]` |
| Zig | `*u32` + `[]const u32` | both `noalias` |

2^20 elements of `i & 7` get summed 1000 times with wrapping `uint32_t` arithmetic. The build uses the benchmark flags of [ADR 0003](../decisions/0003-benchmark-methodology.md) with C built by both clang and gcc. Timing follows the same protocol (3 warmup and 30 timed rounds/shuffled/pinned to logical CPU 4 at high priority). `python bench/experiments/noalias/run.py` reproduces it.

## Results

All eight runs print 3670016000. Medians of 30 runs with a 95% bootstrap interval for the ratio:

| variant | without the promise | with it | ratio |
| --- | --- | --- | --- |
| c-clang | `plain` 71.6 ms | `restrict` 71.6 ms | 1.00 (0.97-1.04) |
| c-gcc | `plain` 205.7 ms | `restrict` 74.1 ms | 2.78 (2.71-2.86) |
| rust | `pointer` 71.6 ms | `reference` 70.5 ms | 1.02 (0.97-1.05) |
| zig | `plain` 205.9 ms | `noalias` 206.3 ms | 1.00 (0.98-1.01) |

The listings below come from the same flags without LTO. clang and gcc got `-S`/`-emit-llvm`. rustc got `--emit=llvm-ir,asm` and Zig got `-femit-asm`/`-femit-llvm-ir`.

- clang 23.1.3 vectorizes both versions with 256-bit adds. Before the plain loop it compares the address of `total` with the extent of `values` (two `icmp ult ptr` in the IR). It takes the vector loop when they cannot overlap. `restrict` only saves that check.
- rustc 1.99.0 puts `noalias` on both pointer parameters of `accumulate(total: &mut u32, values: &[u32])`. `accumulate_raw` gets no `noalias` but the same run-time check as clang's plain loop. It runs just as fast.
- gcc 16.2.0 keeps the plain sum in a register but stores it to `*total` after every element (`addl (%rdx), %eax` then `movl %eax, (%rcx)`). There is no run-time check and no vector code. With `restrict` it vectorizes with 256-bit registers.
- Zig 0.17.0 turns the plain loop into gcc's scalar shape unrolled 16 times. `noalias` reaches LLVM (both parameters carry it in the IR) and the stores go away. The loop still vectorizes badly with 128-bit adds and a horizontal reduction inside every step of 8 elements. It is no faster. Zig's own unoptimized IR for both functions comes out of LLVM 23.1.3's `opt -O3 -mcpu=raptorlake` vectorized like clang's (the plain one behind the run-time check). The gap sits in Zig 0.17.0's pipeline or in its LLVM 22.1.8 rather than in the source.

## Conclusion

For this loop the hypothesis fails on LLVM. A run-time overlap check recovers the whole speed and the promise only removes the check. It holds on gcc (the promise is worth 2.78x there). Zig shows that a backend can waste the promise.

A no-alias guarantee pays where a compiler cannot build such a check cheaply: many pointers/extents it does not know/pointers loaded from memory/calls inside the loop. The next aliasing experiment needs one of those. A CADOU backend on LLVM gets a loop like this right either way. A backend without alias analysis (Cranelift or an early one of our own) would need the guarantee.

[PRINCIPLES.md 3.2](../PRINCIPLES.md#32-give-the-optimizer-information) stays a hypothesis. [ADR 0002](../decisions/0002-initial-backend.md) uses these numbers.
