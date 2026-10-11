# CADOU

CADOU (a working name) is a research project for a systems programming language. The goal is to beat C/C++/Rust/Zig at run time by a measurable margin while staying memory-safe by default. It is an experiment rather than a product. There is no compiler yet.

Status on 2026-10-11: phase 1. The benchmark harness and six classic benchmarks exist in C/C++/Rust/Zig (measured on one Windows machine so far). In the [first baseline](bench/results/2026-10-11-i7-14700kf-windows/report.md) no competitor wins everywhere (fastest counts out of six: zig 3/gcc 2/clang 1). Phases and next steps live in [docs/ROADMAP.md](docs/ROADMAP.md).

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
| `compiler/` | bootstrap compiler (from phase 2) |
| `std/` | standard library |
| `tests/` | compiler tests |
| `fuzz/` | fuzzers and corpora |
| `examples/` | example programs and later a small kernel |
| `tools/` | repository scripts |

## Building and testing

Nothing to build yet besides the benchmarks. With Python 3.11+/git/rustup:

```sh
python bench/harness/bench.py fetch
python bench/harness/bench.py validate
python bench/harness/bench.py run --label my-machine
python -m unittest discover -s bench/harness/tests -t bench/harness
python -m unittest discover -s tools
python tools/check_text.py
```

## License

MIT (see [LICENSE](LICENSE)).
