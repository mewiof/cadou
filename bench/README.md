# Benchmarks

The bar CADOU has to clear. The same programs in C/C++/Rust/Zig get built with each language's best release settings and timed by one harness. Method/flags/statistics: [ADR 0003](../docs/decisions/0003-benchmark-methodology.md).

## Running

Needs Python 3.11+ and git. The harness fetches the Zig/LLVM/GCC builds pinned in [toolchains.toml](toolchains.toml) into `.toolchains/`. Rust comes from rustup.

```sh
python bench/harness/bench.py fetch
python bench/harness/bench.py validate
python bench/harness/bench.py run --label i7-14700kf-windows
```

`run` builds everything and checks every output before it measures anything. Results land in `results/DATE-LABEL/` as `raw.json` + `report.md`. `--bench`/`--variant`/`--reps` narrow a run down and `--system-toolchains` ignores the pins. Harness tests: `python -m unittest discover -s bench/harness/tests -t bench/harness`.

## Programs

One folder per benchmark under `programs/` with one subfolder per language. `idiomatic.*` is what a competent programmer writes. `expert.*` (none yet) will be the best hand-tuned version. `bench.toml` holds the inputs and the expected output.

| benchmark | what it stresses | validation | timed |
| --- | --- | --- | --- |
| n-body | scalar doubles/`sqrt`/a 5-element array of structs | 1000 | 50,000,000 steps |
| spectral-norm | int to double conversion/division/long dependent sums | 100 | 5500 |
| fannkuch-redux | small int arrays/short reversals and rotations/branches | 7 | 11 |
| mandelbrot | a tight double loop with an early exit/bit packing | 200 | 8000 |
| binary-trees | one heap allocation per node/recursion | 10 | 19 |
| fasta | an LCG/table lookups/buffered output (254 MB at the timed size) | 1000 | 25,000,000 |
| startup | nothing (process creation and runtime init) | none | none |

The algorithms are the [Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/) ones written from its descriptions. Its published outputs are the validation files. mandelbrot and binary-trees run below the Benchmarks Game sizes (16000 and 21) to keep a full run under an hour.

Rules for every language:

- Single-threaded. The fastest Benchmarks Game entries are mostly parallel but ours measure code generation.
- binary-trees takes every node from the language's default heap in one order (left subtree -> right subtree -> node).
- fasta keeps the reference generator exactly (doubles included). Its output depends on every rounding.
- fasta writes through a 64 KiB output buffer in every language (`setvbuf` in C and C++). Without it the UCRT flushes stdout after every call when stdout is a character device like the harness's `NUL` ([baseline-gaps.md](../docs/research/baseline-gaps.md#fasta)).

## Results

`results/` has one folder per run. `raw.json` keeps every sample with the build commands/toolchain versions/machine data. `report.md` gets rendered from it and never edited by hand (`bench.py report DIR` re-renders it).
