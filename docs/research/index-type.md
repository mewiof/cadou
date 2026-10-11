# Experiment: index type

2026-10-11. Code in [bench/experiments/index-type/](../../bench/experiments/index-type/). Raw data in [bench/results/experiments/2026-10-11-index-type.json](../../bench/results/experiments/2026-10-11-index-type.json).

## Hypothesis

Rust and Zig index with `usize`. The first journal entry put part of their spectral-norm gap on `usize` to `f64`: five instructions on this machine without AVX-512 against one `vcvtsi2sd` for a signed integer. A signed 64-bit index would then be the cheaper choice for CADOU ([spec section 11.3](../../spec/core.md#113-index-type)).

## Method

spectral-norm at 5500 with the index arithmetic of `A(i, j)` and its loops in three types:

| language | variants |
| --- | --- |
| C (clang and gcc) | `int`/`long long`/`size_t` |
| Rust | `usize`/`i64` |
| Zig | `usize`/`i64` (with `@divTrunc` for the signed division like C and Rust) |

The patches are regular expressions in [run.py](../../bench/experiments/index-type/run.py). Benchmark flags and protocol of [ADR 0003](../decisions/0003-benchmark-methodology.md). Every variant prints the recorded output.

## Results

Medians of 30 runs:

| index type | c-clang | c-gcc | rust | zig |
| --- | --- | --- | --- | --- |
| 32-bit signed (`int`) | 0.658 s | **0.453 s** | - | - |
| 64-bit unsigned (`size_t`/`usize`) | 0.753 s | 0.891 s | 0.756 s | 0.890 s |
| 64-bit signed (`long long`/`i64`) | 1.210 s | 0.891 s | 1.213 s | 0.891 s |

## Generated code

The loop body divides once per element. [uops.info](https://uops.info/) measured one `vdivsd` per 4 cycles and one 32-byte `vdivpd` (4 lanes) per 8 cycles on Alder Lake-P (Golden Cove, the core Raptor Cove builds on, read 2026-10-11). 1.21 billion divisions at the measured 5.30 GHz ([baseline-gaps.md](baseline-gaps.md)) then take at least 0.91 s scalar and 0.46 s with 4 lanes.

- `int`: clang and gcc vectorize with native AVX2 instructions (`vpmulld` for the index product/`vcvtdq2pd` for the conversion). gcc runs 8 elements per iteration at about 2 cycles per element (the `vdivpd` bound). clang runs 4 and rebuilds its index vector from a 64-bit counter in every iteration.
- `long long` and `i64`: clang and rustc still vectorize 4 lanes. AVX2 has no 64-bit multiply and no 64-bit integer to double conversion (`vpmullq` and `vcvtqq2pd` need AVX512DQ per the [x86 instruction reference](https://www.felixcloutier.com/x86/) read 2026-10-11). LLVM builds the product from three `vpmuludq` plus shifts and converts each lane alone (`vpextrq` + scalar `vcvtsi2sd`). That vector loop is slower than a scalar one would be.
- `size_t` and `usize`: the same emulated product with a cheaper unsigned conversion.
- gcc with either 64-bit type and Zig with both types stay at the scalar `vdivsd` bound. Zig's LLVM 22.1.8 leaves even the `int` loop scalar when it compiles the C version through `zig cc`. Zig's scalar loop comes from that LLVM version rather than from the index type.

## Conclusion

The hypothesis fails. On this machine the width of the index matters more than its sign. 64-bit index arithmetic loses AVX2's native multiply and conversion. LLVM vectorizes the signed 64-bit loop anyway and ends 33% above the scalar bound.

The first journal entry was half right. `usize` to `f64` does cost extra instructions in scalar code. In the vector loop the cost sits in the 64-bit product and in the lane-by-lane conversion of signed values.

Correction: the first journal entry and [ADR 0002](../decisions/0002-initial-backend.md) said clang computes spectral-norm one element at a time. clang 23.1.3 vectorizes it 4 lanes wide like gcc (see the `int` row). gcc's lead comes from its wider loop with the index vector kept in registers. ADR 0002 carries the correction.

For CADOU the index type stays open ([spec section 11.3](../../spec/core.md#113-index-type)) with width as a question of its own. A 64-bit length does not have to force 64-bit arithmetic on every index expression. A program that states a smaller range (a 32-bit index type or a bounded integer) would give the compiler the native AVX2 forms.
