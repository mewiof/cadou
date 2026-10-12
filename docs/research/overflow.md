# Experiment: what overflow checks cost

2026-10-12. Code in [bench/experiments/overflow/](../../bench/experiments/overflow/). Raw data in [bench/results/experiments/2026-10-12-overflow.json](../../bench/results/experiments/2026-10-12-overflow.json).

## Question

[Spec section 11.4](../../spec/core.md#114-overflow-in-unchecked-builds) left two things open. What do overflow checks cost a checked build? Should an unchecked build wrap or assume that nothing overflows?

## Method

The nine timed benchmarks with the flags of [ADR 0003](../decisions/0003-benchmark-methodology.md) and one change per case:

| case | overflow | flags added |
| --- | --- | --- |
| rust wrap | wraps (the release default) | |
| rust check | traps for every integer type | `-C overflow-checks=on` |
| c-clang wrap | wraps (clang's default on the MSVC target) | |
| c-clang assume | undefined (standard C) | `-fno-wrapv` |
| c-clang check | traps for signed types | `-fno-wrapv -fsanitize=signed-integer-overflow -fsanitize-trap=signed-integer-overflow` |
| c-gcc wrap | wraps | `-fwrapv` |
| c-gcc assume | undefined (standard C) | |
| c-gcc check | traps for signed types | `-fsanitize=signed-integer-overflow -fsanitize-trap=signed-integer-overflow` |

C checks signed arithmetic only. Unsigned arithmetic wraps by definition there and the C programs use it on purpose (hashing). Rust checks every integer type in the program (its standard library stays as shipped). All 72 builds print the recorded outputs. Benchmark protocol (30 rounds on CPU 4) recorded at `f464007` without local changes.

## Results

Median of the first case over the median of the second with a 95% bootstrap interval. "tie" marks a Holm-adjusted Mann-Whitney p of 0.05 or more (adjusted over the nine benchmarks of a column).

| benchmark | rust check / wrap | clang check / assume | gcc check / assume | clang wrap / assume | gcc wrap / assume |
| --- | --- | --- | --- | --- | --- |
| binary-trees | 1.016 (1.002-1.032) | 1.019 (1.003-1.026) | 1.008 (1.002-1.018) | 0.998 tie | 1.002 tie |
| fannkuch-redux | 1.035 (1.020-1.042) | 0.992 tie | 1.142 (1.137-1.148) | 1.011 tie | 1.003 tie |
| fasta | 0.989 (0.980-0.997) | 1.002 tie | 0.999 tie | 0.998 tie | 0.999 tie |
| hash-map | 0.993 tie | 0.991 tie | 0.986 tie | 0.980 tie | 0.988 tie |
| k-nucleotide | 1.021 (1.013-1.027) | 1.014 tie | 0.998 tie | 1.003 tie | 1.000 tie |
| mandelbrot | 1.006 tie | 1.013 (1.008-1.016) | 0.998 tie | 1.001 tie | 0.999 tie |
| n-body | 1.002 tie | 1.001 tie | 0.999 tie | 1.008 tie | 1.001 tie |
| sort | 0.998 tie | 1.002 tie | 1.000 tie | 0.999 tie | 1.001 tie |
| spectral-norm | 1.331 (1.322-1.341) | 1.684 (1.674-1.693) | 2.340 (2.322-2.368) | 0.928 (0.922-0.939) | 1.021 (1.016-1.033) |
| geometric mean | 1.039 | 1.064 | 1.114 | 0.991 | 1.002 |
| without spectral-norm | 1.007 | 1.004 | 1.015 | 0.999 | 0.999 |

- Rust's and clang's checks cost at most 3.5% outside spectral-norm. gcc's cost 14% on fannkuch-redux (not analyzed) and at most 1% on the other seven. On spectral-norm checks cost 33% to 134%.
- spectral-norm's inner loop loses its vector code. Without checks the measured binaries contain vector divisions (`vdivpd`: 4 in clang's and Rust's/16 in gcc's). With checks they contain none and 10 to 31 overflow branches (`jo`). Every element then gets a division of its own (`vdivsd`).
- Assuming that nothing overflows gains nothing. Eight benchmarks tie for each compiler. On spectral-norm clang is 7% faster when it wraps (as in [clang-msvc.md](clang-msvc.md)) and gcc 2% slower.
- Rust's fasta runs 1.1% faster with checks. Unexplained (code placement is a guess).

## Decision

[ADR 0005](../decisions/0005-integer-overflow.md): checked builds trap on overflow and unchecked builds wrap. The open work is a checked loop that keeps its vector code. A candidate is one check in front of a loop that covers every round (spectral-norm's indices stay below `n` and `n` is known before the loop). Phase 4 tests it.
