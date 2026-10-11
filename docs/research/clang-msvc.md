# Experiment: clang's defaults on the MSVC target

2026-10-11. Code in [bench/experiments/clang-msvc/](../../bench/experiments/clang-msvc/). Raw data in [bench/results/experiments/2026-10-11-clang-msvc.json](../../bench/results/experiments/2026-10-11-clang-msvc.json).

## Question

clang 23.1.3 for `x86_64-pc-windows-msvc` hands `-fwrapv` and `-relaxed-aliasing` to its front end by default (seen with `-###` for C and C++). Signed overflow then wraps and type-based alias analysis is off. gcc 16.2.0 on MinGW keeps standard C semantics (`-fstrict-aliasing` on and `-fwrapv` off per `-Q --help=optimizers`). So does clang on Linux. The [first baseline](../../bench/results/2026-10-11-i7-14700kf-windows/report.md) measured clang with the Windows defaults. Do they cost clang time?

## Method

The six classic benchmarks in C and C++ built by clang twice: with the benchmark flags of [ADR 0003](../decisions/0003-benchmark-methodology.md) and with `-fno-wrapv -fstrict-aliasing` added. Both flags reach the front end (no `-fwrapv` and no `-relaxed-aliasing` in `-###`). None of the programs overflows a signed integer or reads an object through another type. Both kinds of build print the recorded outputs. The generated code differs for all six benchmarks. Benchmark protocol.

## Results

Medians of 30 runs. "standard" adds `-fno-wrapv -fstrict-aliasing`.

| benchmark | c-clang | c-clang standard | cpp-clang | cpp-clang standard |
| --- | --- | --- | --- | --- |
| binary-trees | 3.118 s | 3.130 s | 3.469 s | 3.463 s |
| fannkuch-redux | 1.532 s | 1.520 s | 2.037 s | 1.961 s |
| fasta | 1.799 s | 1.802 s | 1.821 s | 1.822 s |
| mandelbrot | 2.314 s | 2.315 s | 2.335 s | 2.318 s |
| n-body | 1.382 s | 1.388 s | 1.243 s | 1.212 s |
| spectral-norm | 0.660 s | 0.711 s | 0.660 s | 0.710 s |

Standard semantics changes the geometric mean by +1.3% for C (slower) and by 0.0% for C++. C++ gains 3.7% on fannkuch-redux and 2.5% on n-body. Both languages lose 8% on spectral-norm. There the main vector loop even gets shorter: without `-fwrapv` the product `(i + j) * (i + j + 1)` cannot be negative and the sign fix-up before the division by 2 goes away (23 instructions per 4 elements instead of 26). Why the program still takes longer stays open until hardware counters are available.

## Decision

The benchmark flags stay as they are. The Windows defaults do not hold clang back on this suite. [ADR 0003](../decisions/0003-benchmark-methodology.md) records the defaults. A Linux run will measure clang with standard semantics.
