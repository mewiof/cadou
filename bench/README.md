# Benchmarks

The bar CADOU has to clear. The same programs in C/C++/Rust/Zig get built with each language's best release settings and timed by one harness. Method/flags/statistics: [ADR 0003](../docs/decisions/0003-benchmark-methodology.md).

## Running

Needs Python 3.11+ and git. The harness fetches the Zig/LLVM/GCC builds and khash pinned in [toolchains.toml](toolchains.toml) into `.toolchains/`. Rust comes from rustup.

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
| k-nucleotide | reading stdin/hash map updates on 2-bit packed keys | fasta 1000 on stdin | fasta 2,500,000 on stdin |
| hash-map | inserts and lookups of random 64-bit keys | 1000 | 4,000,000 |
| sort | the standard library sort on random 64-bit integers | 1000 | 10,000,000 |
| startup | nothing (process creation and runtime init) | none | none |

The first seven are [Benchmarks Game](https://benchmarksgame-team.pages.debian.net/benchmarksgame/) algorithms written from its descriptions. Its published outputs are the validation files. mandelbrot/binary-trees/k-nucleotide run below the Benchmarks Game sizes (16000/21/25,000,000) to keep a full run near an hour. hash-map and sort have no Benchmarks Game counterpart. A `reference.py` next to each wrote its expected output and reproduces the full-size output too.

Rules for every language:

- Single-threaded. The fastest Benchmarks Game entries are mostly parallel but ours measure code generation.
- binary-trees takes every node from the language's default heap in one order (left subtree -> right subtree -> node).
- fasta keeps the reference generator exactly (doubles included). Its output depends on every rounding.
- fasta writes through a 64 KiB output buffer in every language (`setvbuf` in C and C++). Without it the UCRT flushes stdout after every call when stdout is a character device like the harness's `NUL` ([baseline-gaps.md](../docs/research/baseline-gaps.md#fasta)).
- Hash tables are the standard library's hash map with its default hash function: `std::unordered_map`/`std::collections::HashMap`/`std.AutoHashMap` (C++/Rust/Zig). C has none and includes khash from klib (pinned in [toolchains.toml](toolchains.toml)). Every table starts at its default size. Keys are 64-bit integers (k-nucleotide packs two bits per nucleotide).
- sort uses the standard library's unstable sort: `qsort`/`std::sort`/`sort_unstable`/`std.mem.sortUnstable` (C/C++/Rust/Zig).
- k-nucleotide reads fasta's output from stdin. The harness generates that input ([inputs.py](harness/inputs.py)) and checks its sha256. The published k-nucleotide input is the fasta output for 1000.

## Results

`results/` has one folder per run. `raw.json` keeps every sample with the build commands/toolchain versions/machine data. `report.md` gets rendered from it and never edited by hand (`bench.py report DIR` re-renders it).
