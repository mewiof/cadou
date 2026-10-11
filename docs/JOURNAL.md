# Journal

Newest entry first.

## 2026-10-11

Session 1 covered phase 0 and phase 1 up to a first baseline.

### Done

- Repository layout/[STYLE.md](STYLE.md)/`tools/check_text.py` (used by CI).
- [PRINCIPLES.md](PRINCIPLES.md)/[COMPLEXITY.md](COMPLEXITY.md)/[REFERENCES.md](REFERENCES.md)/[ROADMAP.md](ROADMAP.md) and the ADR template.
- ADR [0001](decisions/0001-bootstrap-language.md) picks Rust for the bootstrap compiler. ADR [0002](decisions/0002-initial-backend.md) picks textual LLVM IR. ADR [0003](decisions/0003-benchmark-methodology.md) fixes the benchmark method.
- First [survey](research/survey.md). Every point from the starting list got checked against a primary source. Odin/Jai/Fortran/Vale/Hylo/Cyclone/SPARK/Austral/Verona/Fil-C and the Rust aliasing models came on top.
- Toolchains pinned to the newest stable releases (clang 23.1.3/GCC 16.2.0 from winlibs/Rust 1.99.0/Zig 0.17.0).
- Six benchmarks in C/C++/Rust/Zig with C and C++ built by both clang and gcc. That makes 36 programs plus empty `startup` programs. All 36 reproduce the published Benchmarks Game outputs and agree with each other at the timed sizes.
- Harness with 17 unit tests + the text checker with 8. CI builds and checks every program on Linux x86-64 and AArch64.
- Windows baseline in the [report](../bench/results/2026-10-11-i7-14700kf-windows/report.md). First experiment in [noalias.md](research/noalias.md).

### Baseline

Median time over the fastest variant from 30 runs each pinned to a P-core. Intervals/p-values/memory/sizes/compile times are in the report.

| benchmark | c-clang | c-gcc | cpp-clang | cpp-gcc | rust | zig |
| --- | --- | --- | --- | --- | --- | --- |
| binary-trees | 1.71 | 1.67 | 1.90 | 1.72 | 1.74 | **1.00** |
| fannkuch-redux | 1.16 | **1.00** | 1.52 | 1.30 | 1.15 | 1.48 |
| fasta | 1.36 | 1.27 | 1.35 | 1.26 | 1.11 | **1.00** |
| mandelbrot | **1.00** | 1.04 | 1.01 | 1.04 | 1.06 | 1.04 |
| n-body | 1.30 | 1.31 | 1.17 | 1.31 | 1.28 | **1.00** |
| spectral-norm | 1.46 | 1.01 (tie) | 1.45 | **1.00** | 1.66 | 1.97 |
| geometric mean | 1.31 | 1.20 | 1.37 | 1.25 | 1.31 | 1.20 |

No single competitor sets the bar (fastest counts: zig 3/gcc 2/clang 1). mandelbrot is the only near tie with everything within about 6%.

### Explained so far

- spectral-norm with gcc against clang: gcc vectorizes the index arithmetic/the `int` to `double` conversion/the division four lanes at a time (`vcvtdq2pd` + `vdivpd`). Only the additions of the sum stay in order (scalar `vaddsd`). clang does every element scalar (`vcvtsi2sd` + `vdivsd`) and needs 45% more time.
- spectral-norm with Rust (14% over clang) and Zig (36% over clang): on top of LLVM's scalar loop `usize` to `f64` takes five instructions (`vmovq`/`vpunpckldq`/`vsubpd`/`vshufpd`/`vaddsd`) instead of one `vcvtsi2sd`. This CPU has no AVX-512 and thus no unsigned conversion instruction. The idiomatic index type brings that cost along.
- binary-trees: Zig's SMP allocator against the Windows heap behind `malloc`/`std::make_unique`/`Box`. Zig also peaks at 35 MiB against 68 MiB for the others with 16-byte nodes (per-block heap headers are the likely difference but nobody checked yet).

### Not explained yet

- n-body: zig takes 1.08 s and everyone else 1.26 to 1.42 s. clang's C++ build also beats clang's C (1.26 s against 1.40 s) on the same loops.
- fannkuch-redux: cpp-clang (MSVC STL `std::reverse`/`std::rotate`) and zig (`std.mem.rotate`) need about 50% more time than gcc's C with plain loops.
- fasta: zig 1.81 s/Rust 2.01 s/C and C++ 2.29 to 2.46 s with the same generator. The output path is the suspect.
- startup: zig 4.1 ms against 4.9 to 5.7 ms for the rest. Significant in this run but the intervals are wide.

### Failed or surprising

- The noalias hypothesis failed for LLVM on the simplest loop. clang and rustc check for overlap at run time and vectorize anyway. The promise is worth 2.78x on gcc only and Zig wastes it.
- FP contraction was expected to change outputs. It did not at the timed sizes even though clang's mandelbrot binary gains FMA instructions with the default `-ffp-contract=on`.
- Unpacking the toolchains failed at first. Windows refused to rename a freshly unpacked directory (Access denied while Defender held it). The fetcher now unpacks in place.
- g++ on MinGW does not link `std::print` without `-lstdc++exp`.
- The assumption that hyperfine goes through a shell and reports no memory was out of date. The 2.0.0 README says otherwise.
- Graydon Hoare's post is gone from Dreamwidth (404) and the Internet Archive refused this machine (HTTP 429). The survey quotes it through Michael Tsai's blog.

### Not done

- No Linux numbers and no hardware counters while WSL2 stays off on this machine. No AArch64 timing.
- CI has not run yet. The likeliest failure is Ubuntu 24.04's clang 18 compiling libstdc++ 14's `<print>`.

### Next

- Explain the n-body/fannkuch-redux/fasta gaps from the generated code.
- A second aliasing experiment where a run-time check cannot rescue the loop (pointers loaded from memory or unknown extents).
- The rest of [ROADMAP.md](ROADMAP.md#next-steps).
