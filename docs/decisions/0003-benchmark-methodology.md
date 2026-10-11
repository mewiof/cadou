# 0003: Benchmark methodology

- Status: accepted
- Date: 2026-10-11

## Context

Rules set at the start of the project:

- the same algorithm and data structures in every language
- C and C++ with clang and gcc at `-O3 -march=native` plus LTO
- Rust with its release profile plus fat LTO/one codegen unit/`panic = "abort"`/`target-cpu=native`
- Zig in ReleaseFast with `-mcpu=native`
- warmups/many repetitions/medians with intervals/a statistical test/pinning/a random order/hardware counters where possible

The details are open. Two of them decide whether a comparison is fair. What exactly gets timed? Do the four languages execute the same floating-point operations?

The only machine today is a Windows 11 desktop with an Intel Core i7-14700KF (hybrid with 8 P-cores plus SMT and 12 E-cores for 28 logical CPUs). WSL2 stays off on it for a while. There is no Linux box and no AArch64 machine. The harness has to work on Linux from day one anyway.

## Options

- Time inside each program (one timer per language) or time the whole process from outside. Inside timing hides startup but needs four implementations of the measurement.
- An existing runner or our own. hyperfine 2.0.0 runs commands without a shell and reports peak memory ([README](https://github.com/sharkdp/hyperfine) read 2026-10-11). It runs commands in input order instead of interleaving them though. Output validation/compile timing/binary sizes/pinning on Windows/metadata would still be our code.
- Allow each compiler its default FP contraction or switch it off everywhere.

## Evidence

- Pinning works. The harness pins itself and every child to logical CPU 4. Windows' CPU set API puts that CPU in efficiency class 1 (logical CPUs 0-15 are the P-core threads) and not in class 0 (16-27 are the E-cores).
- Process start costs 4.1 to 5.7 ms on this machine (`startup` in the [first baseline](../../bench/results/2026-10-11-i7-14700kf-windows/report.md)). That is about 1% of the shortest timed run (spectral-norm at 0.46 s).
- FP contraction. clang defaults to `-ffp-contract=on` (fusion inside one statement) per the [Clang manual](https://clang.llvm.org/docs/UsersManual.html) (in-progress docs read 2026-10-11). GCC defaults to off for C in an ISO mode and to `fast` otherwise (so g++ fuses across statements) per the [GCC manual](https://gcc.gnu.org/onlinedocs/gcc/Optimize-Options.html) (read 2026-10-11). rustc and Zig never fuse unless the code calls `mul_add`/`@mulAdd`. Rebuilt with each compiler's defaults mandelbrot and n-body gave the same outputs at the timed sizes. The clang binary still went from 9 to 12 FMA instructions (checked 2026-10-11).
- g++ on MinGW does not link `std::print` without `-lstdc++exp` (GCC 16.2.0 reports `undefined reference to std::__open_terminal`).
- Since Rust 1.77 Cargo release builds strip debuginfo when `debug = 0` ([Rust 1.77.0 announcement](https://blog.rust-lang.org/2024/03/21/Rust-1.77.0/) from 2024-03-21).
- In Zig 0.17.0 `std.process.Init` hands `main` the SMP allocator as `gpa` in ReleaseFast when libc is not linked (`callMain` in `lib/std/start.zig`).
- clang 23.1.3 for `x86_64-pc-windows-msvc` passes `-fwrapv` and `-relaxed-aliasing` to its front end by default (MSVC compatibility). Adding `-fno-wrapv -fstrict-aliasing` changes clang's code for every benchmark and its time by -4% to +8% (geometric mean +1.3% for C and 0.0% for C++) per [clang-msvc.md](../research/clang-msvc.md) from 2026-10-11.

## Decision

Our own harness in `bench/harness/` (Python with the stdlib only).

What gets timed: the whole process in wall-clock time between two `perf_counter_ns` reads around spawn and wait. Output goes to the null device. The `startup` benchmark (empty programs) measures the fixed part. User/system CPU time and peak memory come from the OS (`GetProcessTimes` + the peak working set on Windows and `wait4` rusage on Linux). On Windows `QueryProcessCycleTime` lands in `reference_cycles` (TSC ticks rather than core clocks).

Correctness gates before any timing:

1. Every program runs the small validation input and must match `expected-*` byte for byte. Those files are the published Benchmarks Game outputs (all 36 implementations reproduce them).
2. Every program runs the timed input once with its output hashed. All variants must agree (and match the hash in `bench.toml` once one is recorded).

Protocol:

- 3 warmup rounds and 30 timed rounds. A round runs every implementation once in an order shuffled by a seeded RNG (drift spreads over all variants instead of hitting one).
- The harness pins itself to one logical CPU before spawning (children inherit the mask). Programs start at high priority. Default CPU 4. The run records which kind of core that was.
- Builds run unpinned. Compile time is the median of 3 clean builds after one untimed build. Zig gets a fresh local cache each time but keeps its global cache (compiler_rt gets built once per machine like for any user).
- Linux only: a separate pass of 5 `perf stat` runs per implementation for cycles/instructions/branches/branch misses/cache references/cache misses. Counters stay out of the timed runs.
- Not controlled on Windows without admin rights: turbo/frequency/background load. The power plan gets recorded.

Statistics:

- Median with a 95% interval from order statistics (ranks 10 and 21 of 30 with no distribution assumed).
- Ratio to the fastest variant's median with a 95% percentile bootstrap interval (10,000 resamples with a fixed seed).
- Two-sided Mann-Whitney U against the fastest variant (normal approximation with tie and continuity corrections) Holm-adjusted within a benchmark. A difference counts when the adjusted p is below 0.05. Otherwise the report says "tie".
- Suite score: the geometric mean of the per-benchmark ratios.

Flags on top of the required ones:

| variant | command |
| --- | --- |
| c-clang | `clang -std=c17 -O3 -march=native -flto -ffp-contract=off -fuse-ld=lld` (`-lm` on Linux) |
| c-gcc | `gcc -std=c17 -O3 -march=native -flto -flto-partition=one -ffp-contract=off -lm` (`-static` on Windows) |
| cpp-clang | as c-clang with `-std=c++23` |
| cpp-gcc | as c-gcc with `-std=c++23` (`-static -lstdc++exp` on Windows) |
| rust | `rustc --edition=2024 -C opt-level=3 -C codegen-units=1 -C lto=fat -C panic=abort -C target-cpu=native -C strip=debuginfo` |
| zig | `zig build-exe -O ReleaseFast -mcpu=native` |

- `-ffp-contract=off` everywhere. All four languages then execute the same IEEE operations (bit-identical results at any size). What contraction buys C is a separate experiment.
- `-flto-partition=one` gives gcc a single LTO unit as its counterpart of `codegen-units = 1` (it also silences the serial LTRANS warning).
- MinGW builds are static (no runtime DLL gets picked up from `PATH`).
- rustc runs directly with the release-profile values instead of through Cargo (compile time is then the compiler's alone). `-C strip=debuginfo` matches Cargo's release default.
- C and C++ programs switch stdout to binary mode under `_WIN32`. Text mode would turn every `\n` into `\r\n`.

Rules for IDIOMATIC programs:

- Single-threaded. Threads would measure schedulers and thread libraries. Parallel versions will be their own category.
- The language's default allocator: `malloc`/`std::make_unique`/`Box`/`init.gpa` (C/C++/Rust/Zig).
- Library calls where a competent programmer would make them. fannkuch-redux flips with `std::reverse`/`reverse`/`std.mem.reverse` (C++/Rust/Zig) while C swaps in a loop.
- The language's own index type (`int` in C/C++ and `usize` in Rust/Zig).
- The same allocation order everywhere (binary-trees goes left subtree -> right subtree -> node).
- Standard Benchmarks Game algorithms written from the published descriptions rather than copied.

Binary size: the file as built + a copy run through `llvm-strip --strip-all`. On Windows the debug info sits in separate PDB files that do not count.

## Consequences

- Windows numbers include Windows process creation. binary-trees also measures the Windows heap (UCRT `malloc` behind C/C++/Rust against Zig's own SMP allocator). These numbers carry the host label and do not compare with Linux numbers.
- clang on Windows uses the MSVC STL with UCRT. g++ uses libstdc++ with UCRT through MinGW. On Linux both would use libstdc++ and glibc.
- clang on Windows keeps its MSVC-compatible defaults (wrapping signed overflow and no type-based alias analysis). On Linux clang compiles with standard C and C++ semantics.
- The first baseline has no IPC or cache data (no hardware counters until a Linux host exists).
- Turning contraction off may cost C/C++ speed their default flags would get. Rust and Zig would need explicit fused calls for the same.

## Revisit when

- WSL2 or bare-metal Linux is available. Linux then becomes the reference host with perf counters/frequency control/an isolated core.
- An AArch64 machine is available.
- Run-to-run spread (IQR over median) exceeds 1% for a benchmark. Look at isolation before trusting small differences.
- CADOU programs join. Add `cadou` variants with checks on and with checks off (labeled).
