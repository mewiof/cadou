# Baseline gaps

2026-10-11. Code in [bench/experiments/gaps/](../../bench/experiments/gaps/). Raw data in [bench/results/experiments/2026-10-11-gaps.json](../../bench/results/experiments/2026-10-11-gaps.json). `python bench/experiments/gaps/run.py` rebuilds and reruns all 36 cases (about 40 minutes).

The [first baseline](../../bench/results/2026-10-11-i7-14700kf-windows/report.md) left three gaps unexplained. Each case changes one flag or one source line of an idiomatic program as of commit 92c5a20. Everything else comes from [ADR 0003](../decisions/0003-benchmark-methodology.md): benchmark flags/3 warmup and 30 timed rounds in shuffled order/logical CPU 4 at high priority. Every case prints the recorded full-size output. Times are medians. A dependent chain of register adds measured the core clock at 5.30 GHz before the rounds started ([clock.c](../../bench/experiments/gaps/clock.c)).

## n-body

Suspects were errno handling around `sqrt` in C and the way the five-body loops get unrolled.

| case | median |
| --- | --- |
| zig | 1.066 s |
| Zig's unoptimized IR through LLVM 23.1.3 `opt -O3` | 1.198 s |
| cpp-clang | 1.240 s |
| rust | 1.366 s |
| c-clang | 1.380 s |
| c-clang `-fno-math-errno` | 1.384 s |
| cpp-clang with `int` indices in the pair loop | 1.390 s |
| c-gcc | 1.400 s |
| c-gcc `-fno-math-errno` | 1.401 s |
| c-clang `-fno-slp-vectorize` | 1.406 s |
| C through `zig cc` (clang 22.1.8) | 1.410 s |
| cpp-clang `-fno-slp-vectorize` | 1.414 s |

- errno plays no part. clang turns the `sqrt` call into `llvm.sqrt` at `-O3` because a sum of squares is never negative and `sqrt` sets errno only for negative arguments. The call stays in `sqrt(x * x - y * y)` (checked 2026-10-11). gcc's step loop has a bare `vsqrtsd` too. `-fno-math-errno` changes neither compiler.
- Every build does 10 square roots and 10 divisions per step. gcc keeps the outer pair loop and computes in scalar registers. The LLVM builds unroll all 10 pairs into one straight block. Their SLP vectorizer then packs pairs of operations into vectors.
- That packing decides the time. Without it both clang builds take 1.41 s. With it C++ gains 12% and C gains 2%.
- clang's C and C++ builds differ in how the packed state passes through stack slots. Three loads in the C loop are 16 bytes wide where the value was stored as two pieces (8 + 16 bytes). Store-to-load forwarding cannot serve a load that spans two stores. Two of the three read the previous step's values. The C++ loop has no such load. Giving the C++ pair loop the `int` indices of the C version brings the split loads back and costs 12% ([split_loads.py](../../bench/experiments/gaps/split_loads.py)).
- C through clang 22.1.8 is as slow as through clang 23.1.3. Zig's own unoptimized IR through LLVM 23.1.3's standard pipeline runs in 1.198 s. Part of Zig's lead comes from the IR it emits and the rest from its own pass setup on LLVM 22.1.8.
- Zig's step loop is the shortest:

| case | instructions per step | llvm-mca cycles per step | measured cycles per step |
| --- | --- | --- | --- |
| zig | 287 | 76 | 113 |
| Zig IR through LLVM 23.1.3 | 382 | 115 | 127 |
| cpp-clang | 337 | 112 | 131 |
| rust | 361 | 110 | 145 |
| c-clang | 353 | 120 | 146 |
| cpp-clang with `int` indices | 341 | 111 | 147 |

llvm-mca 23.1.3 (`-mcpu=raptorlake`) gets the order mostly right and misses by 12 to 37 cycles. It ignores dependencies through memory. Rust's loop has no split load and still needs 145 cycles where llvm-mca predicts 110. That stays open until hardware counters are available.

No language feature explains the n-body spread. The vectorizer reacts to incidental IR (the index type of one loop moves a build by 12%).

## fannkuch-redux

At n = 11 the program makes 39,916,800 permutations with 277,931,375 flips and 68,588,311 rotations ([fannkuch_counts.c](../../bench/experiments/gaps/fannkuch_counts.c)). Flip lengths spread evenly over 2 to 11 elements. 58% of the rotations move 2 elements.

| case | median |
| --- | --- |
| cpp-gcc with both loops of the C version | 1.171 s |
| cpp-gcc with the flip loop of the C version | 1.174 s |
| c-gcc | 1.322 s |
| zig with both loops | 1.438 s |
| cpp-clang with both loops | 1.502 s |
| rust | 1.521 s |
| c-clang | 1.523 s |
| cpp-clang `-D_USE_STD_VECTOR_ALGORITHMS=0` | 1.631 s |
| cpp-clang with the rotate loop | 1.738 s |
| cpp-gcc | 1.739 s |
| cpp-clang with the flip loop | 1.791 s |
| zig with the rotate loop | 1.957 s |
| zig | 1.976 s |
| cpp-clang | 2.021 s |

- gcc's C build unrolls the flip loop completely (the `int[16]` array bounds it at 15 swaps) with an exit test after each swap. Its rotation is two overlapping 32-byte moves.
- clang turns both copy loops of the C version into library calls (`memcpy` per permutation and `memmove` per rotation). Each call brings a `vzeroupper` and four reloaded vector constants. The Windows x64 ABI keeps xmm6-xmm15 across calls but not the upper halves of the ymm registers ([x64 ABI conventions](https://learn.microsoft.com/en-us/cpp/build/x64-software-conventions) updated 2026-06-02). That costs 15% against gcc.
- The MSVC STL of Visual Studio 18.7 (MSVC 14.51.36231) calls out of line for both operations. `std::reverse` goes to `__std_reverse_trivially_swappable_4` with vector paths from 32 bytes up. `std::rotate` goes to `__std_rotate` with a stack cookie and three `memmove` calls. Plain loops save 0.52 s (each operation about half of it).
- libstdc++ 16 reverses with the same swap loop as C over pointers. gcc vectorizes it with a 32-byte permute path/a 16-byte path/a scalar tail and picks one at run time. The flip loop of the C version saves 0.57 s. With both loops C++ beats C because `perm = perm1` copies a fixed `std::array<int, 16>` in two 32-byte moves where C copies n elements.
- Zig's `std.mem.reverse` gets inlined with a 32-byte path for 8 elements or more plus an unrolled tail. The flip costs the time. `std.mem.rotate` (three reversals with two of them out of line) costs nothing measurable.
- Rust's `reverse` inlines as a short swap loop.

Every library version pays setup meant for long ranges on 2 to 11 elements. The fastest flip came from a bound known at compile time.

## fasta

The harness sends stdout to the null device. Windows reports `NUL` as a character device ([stdout_probe.c](../../bench/experiments/gaps/stdout_probe.c) prints `_isatty` = 64 and `GetFileType` = `FILE_TYPE_CHAR`). Microsoft documents what follows: `stdout` and `stderr` "are flushed whenever they're full or, if you're writing to a character device, after each library call" ([Stream I/O](https://learn.microsoft.com/en-us/cpp/c-runtime-library/stream-i-o) updated 2025-05-30). fasta writes 4.2 million lines with one `fwrite` each. Rust's `BufWriter` and Zig's writer buffer 8 and 4 KiB in the program.

| case | median |
| --- | --- |
| cpp-gcc with `setvbuf` | 1.612 s |
| c-gcc with `setvbuf` | 1.642 s |
| zig | 1.762 s |
| c-clang with `setvbuf` | 1.792 s |
| cpp-clang with `setvbuf` | 1.819 s |
| rust | 1.980 s |
| c-gcc | 2.253 s |
| cpp-gcc | 2.261 s |
| cpp-clang | 2.393 s |
| c-clang | 2.402 s |

`setvbuf(stdout, NULL, _IOFBF, 1 << 16)` saves 0.57 to 0.65 s. gcc's C/C++ builds then lead. Rust stays 10% behind clang's buffered C. Its loop is the same except for one index check per character after the acid search and no unrolling (not measured further).

fasta now writes through a 64 KiB buffer in every language ([bench/README.md](../../bench/README.md)).

## Consequences

- None of the three gaps comes from language semantics. n-body is vectorizer luck/fannkuch-redux is library code built for long inputs/fasta was the C runtime's buffering on Windows.
- A CADOU on LLVM inherits the n-body lottery. Explicit vector types or a packing pass in our own MIR are the ways to make such loops predictable. Both are hypotheses for phase 4.
- Standard library routines need fast paths for short inputs and should use lengths known at compile time.
- Output-heavy programs set their buffering explicitly.
