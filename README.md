# CADOU

CADOU (a working name) is a research project for a systems programming language. The goal is to beat C/C++/Rust/Zig at run time by a measurable margin while staying memory-safe by default. It is an experiment rather than a product. There is no compiler yet but a reference interpreter runs the first part of the core.

Status on 2026-10-12: phase 1 is in progress and phase 2 has started. The benchmark harness and nine benchmarks exist in C/C++/Rust/Zig (measured on one Windows machine so far). In the [latest baseline](bench/results/2026-10-11-i7-14700kf-windows-2/report.md) no competitor wins everywhere (fastest counts out of nine: gcc 4/clang 2/zig 2/rust 1). A first draft of the core semantics is in [spec/core.md](spec/core.md). Its integers/floats/functions/control flow/structs/tagged unions run in the reference interpreter in [compiler/](compiler/). Phases and next steps live in [docs/ROADMAP.md](docs/ROADMAP.md).

## Priorities

In this order when they conflict:

1. Speed that is measured and never assumed. Compile speed counts too as the second speed metric.
2. Memory safety by default with explicit unsafe operations.
3. A small language with one obvious way to do each thing.
4. Full low-level control through escape hatches you can grep for.

The design hypotheses behind them are in [docs/PRINCIPLES.md](docs/PRINCIPLES.md).

## Non-goals

- Shipping/selling/supporting CADOU as a product.
- Backward compatibility before a stable version gets declared.
- Deadlines.

## Repository

| path | contents |
| --- | --- |
| `bench/` | benchmark harness/programs/results ([bench/README.md](bench/README.md)) |
| `docs/` | principles/roadmap/journal/decisions/research/style |
| `spec/` | language specification (from phase 2) |
| `compiler/` | bootstrap compiler (so far the reference interpreter) |
| `std/` | standard library |
| `tests/` | golden tests of the core and later compiler tests |
| `fuzz/` | fuzzers and corpora |
| `examples/` | example programs and later a small kernel |
| `tools/` | repository scripts |

## Building and testing

With Python 3.11+/git/rustup:

```sh
python bench/harness/bench.py fetch
python bench/harness/bench.py validate
python bench/harness/bench.py run --label my-machine
python -m unittest discover -s bench/harness/tests -t bench/harness
python -m unittest discover -s tools
python tools/check_text.py
cd compiler && cargo test
```

## License

MIT (see [LICENSE](LICENSE)).
