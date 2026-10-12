# Journal

Newest entry first.

## 2026-10-12

Overflow checks got measured and the reference interpreter runs its first two milestones.

### Done

- [overflow.md](research/overflow.md): the suite with overflow checks on and off (Rust) and with wrapping/assumed/trapping signed overflow (C by clang and gcc). [ADR 0005](decisions/0005-integer-overflow.md) settles spec section 11.4: checked builds trap and unchecked builds wrap.
- The reference interpreter in [compiler/](../compiler/) with Rust 1.99.0 pinned and no dependencies. It reads/checks/runs integers/floats/booleans/functions/control flow/structs/tagged unions/opt. 64 golden programs in [tests/golden/](../tests/golden/) + 10 unit tests (one compares every pair of 8-bit operands with Rust's own arithmetic). A new CI job runs them on Linux x86-64 and AArch64 (not run yet).
- [spec/notation.md](../spec/notation.md) lists the forms. [spec/core.md](../spec/core.md) got the rules the checker forced (sections 3.9/3.10/5/6/8.1/9.1/11.7). [ADR 0006](decisions/0006-literal-types-and-unused-values.md): literals take their type from context and every unused value needs `discard`.
- [COMPLEXITY.md](COMPLEXITY.md) counts something for the first time: 40 reserved words/2626 spec words/1948 compiler lines/16 features.

### Findings

- Overflow checks cost Rust 3.9%/clang 6.4%/gcc 11.4% over the suite (geometric means) and 0.7%/0.4%/1.5% without spectral-norm. There the checks take away the vector divisions in all three compilers (no `vdivpd` left in the binaries and 10 to 31 overflow branches instead).
- Assuming no overflow bought nothing over wrapping (clang -0.9%/gcc +0.2%). clang's wrapping spectral-norm is even 7% faster.
- gcc's checked fannkuch-redux is 14% slower (not analyzed).
- clang's vector code in spectral-norm comes from the SLP vectorizer during the LTO link ("Vectorized horizontal reduction" on the loop unrolled 4 times). The loop vectorizer refuses the loop because the sum has to stay in order. A plain `-O3` build without LTO keeps every division scalar.
- rustc 1.99.0 types an unconstrained literal as `i32` and accepts an ignored `i32` result without a warning. Zig 0.17.0 rejects both (`comptime_int` and "value of type 'i32' ignored").

### Failed or surprising

- The first golden run overflowed the interpreter's stack. An unoptimized evaluation level takes 14 KiB of stack against 0.8 KiB optimized and the depth test goes 100000 levels deep. The dev profile builds with opt-level 1 now.
- Rust's fasta ran 1.1% faster with overflow checks (significant and unexplained).

### Next

- The rest of [ROADMAP.md](ROADMAP.md#next-steps): interpreter milestone 3 (memory with provenance + unchecked code)/the Linux run/EXPERT versions/more workloads/a checked spectral-norm loop that keeps its vector code.

## 2026-10-11: baseline gaps and the core draft

The open baseline gaps got explained and four experiments ran. Three benchmarks joined the suite and phase 2 started on paper.

### Done

- CI passed on its first run after the push (checks + benchmark builds and validation on ubuntu-24.04 and ubuntu-24.04-arm). The worry about clang 18 and `<print>` in the first entry did not come true.
- [baseline-gaps.md](research/baseline-gaps.md) explains the n-body/fannkuch-redux/fasta gaps with 36 builds that change one thing each.
- fasta writes through a 64 KiB buffer in every language.
- Experiments: loops a run-time check cannot rescue ([noalias2.md](research/noalias2.md))/index types ([index-type.md](research/index-type.md))/clang's Windows defaults ([clang-msvc.md](research/clang-msvc.md)).
- k-nucleotide/hash-map/sort. The harness feeds generated input on stdin and pins khash for C. `bench/harness/experiment.py` holds what one-change experiments share.
- Second Windows baseline in the [report](../bench/results/2026-10-11-i7-14700kf-windows-2/report.md).
- [spec/core.md](../spec/core.md) is the first draft of the core semantics. [ADR 0004](decisions/0004-reference-interpreter.md) plans the reference interpreter.

### Baseline

Median time over the fastest variant from 30 runs each pinned to a P-core ([report](../bench/results/2026-10-11-i7-14700kf-windows-2/report.md)). fasta runs with the 64 KiB buffer everywhere. The geometric means cover nine benchmarks and do not compare with the first baseline's six.

| benchmark | c-clang | c-gcc | cpp-clang | cpp-gcc | rust | zig |
| --- | --- | --- | --- | --- | --- | --- |
| binary-trees | 1.69 | 1.67 | 1.88 | 1.70 | 1.72 | **1.00** |
| fannkuch-redux | 1.16 | **1.00** | 1.54 | 1.30 | 1.17 | 1.49 |
| fasta | 1.12 | 1.02 | 1.12 | **1.00** | 1.22 | 1.09 |
| hash-map | 1.07 | **1.00** | 4.25 | 4.08 | 1.45 | 1.03 |
| k-nucleotide | **1.00** | 1.06 | 1.46 | 1.04 | 1.41 | 1.07 |
| mandelbrot | **1.00** | 1.05 | 1.01 | 1.05 | 1.06 | 1.05 |
| n-body | 1.30 | 1.30 | 1.16 | 1.30 | 1.27 | **1.00** |
| sort | 7.01 | 7.43 | 4.54 | 3.66 | **1.00** | 2.57 |
| spectral-norm | 1.45 | 1.01 (tie) | 1.45 | **1.00** | 1.66 | 1.97 |
| geometric mean | 1.46 | 1.38 | 1.75 | 1.53 | 1.31 | 1.28 |

Fastest counts: gcc 4 (2 in C and 2 in C++)/clang 2/zig 2/rust 1. sort alone moves the geometric means a lot.

### Findings

- n-body: no language feature behind it. LLVM's vectorizer packs the unrolled step differently for each IR shape (the index type of one loop moves clang's C++ build by 12%). Zig's lead comes partly from its IR and partly from its own pass setup.
- fannkuch-redux: library reverse/rotate code carries setup for long ranges and pays for it on 2 to 11 elements.
- fasta: the UCRT flushed stdout after every `fwrite` because the harness's `NUL` is a character device.
- A no-alias promise is worth 16 to 18 times where a loop keeps its own bounds in memory it writes through. Around an unknown call LLVM keeps a sum in a register only with noalias + dereferenceable + align on the pointer (Rust's `&mut`). C's `restrict` gives one of the three. Zig reads `x` before it calls `f` in `x += f()` and needs no promise there.
- 64-bit index arithmetic loses AVX2's native multiply and conversion. The width of an index matters more than its sign.
- clang's MSVC-target defaults (`-fwrapv` and no type-based alias analysis) cost it nothing overall on this suite.
- Rust's `sort_unstable` sorts 10 million random integers in 0.14 s. Zig's `std.mem.sortUnstable` needs 0.37 s/`std::sort` 0.52 to 0.65 s/`qsort` 1.0 s. `std::unordered_map` takes 4 times as long as khash on hash-map. Rust's default SipHash costs 45% there (neither analyzed yet).

### Corrections

- The first entry said clang computes spectral-norm one element at a time. clang 23.1.3 vectorizes the loop 4 lanes wide like gcc (gcc's loop is wider). [ADR 0002](decisions/0002-initial-backend.md) carries the correction.
- The first entry put part of Rust's and Zig's spectral-norm gap on `usize` to `f64`. In vector code the 64-bit product costs more. Zig stays scalar because its LLVM 22.1.8 leaves this loop scalar (clang 22.1.8 through `zig cc` as well).

### Failed or surprising

- The first version of the aliasing experiment measured nothing for clang and rustc. They rewrote each kernel for its only caller (a struct argument arrived as two values). The kernels now get called through pointers.
- A chain of `add reg, 1` measured 28 GHz. Raptor Cove folds immediate adds at rename. `add reg, reg` gives 5.1 to 5.4 GHz.
- Zig 0.17.0's `--verbose-llvm-ir` prints `, , !prof` and debug metadata that LLVM 23 rejects. LLVM 23's loop idiom pass then turned a loop in Zig's start code into a `wcslen` call no library provides.
- Git Bash's `> NUL` creates a regular file. Only Python's `os.devnull` gave the character device the harness uses.
- The first "no output" fasta variants hashed every byte with FNV. That serial chain cost as much as the output it replaced.
- llvm-mca misses n-body's step by 12 to 37 cycles. Rust's n-body and clang's slower spectral-norm with standard semantics (a shorter loop that takes 8% longer) stay open until hardware counters exist.
- Identical Zig code ran 24% slower with `noalias`. Its loop spans two 64-byte lines (placement suspected and not tested).

### Next

- The rest of [ROADMAP.md](ROADMAP.md#next-steps): the reference interpreter (milestones 1 and 2)/the cost of overflow checks/the Linux run/EXPERT versions/more workloads.

## 2026-10-11: foundations and the first baseline

Phase 0 and phase 1 up to a first baseline.

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
