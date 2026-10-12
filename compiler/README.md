# Compiler

The bootstrap compiler in Rust ([ADR 0001](../docs/decisions/0001-bootstrap-language.md)) that will emit LLVM IR ([ADR 0002](../docs/decisions/0002-initial-backend.md)). So far it holds the reference interpreter of the core ([ADR 0004](../docs/decisions/0004-reference-interpreter.md)) in milestones 1 and 2.

| file | stage |
| --- | --- |
| `src/read.rs` | text to lists and atoms |
| `src/ast.rs` | lists to one node per form ([spec/notation.md](../spec/notation.md)) |
| `src/check.rs` | names/types/the rules of [spec/core.md](../spec/core.md) into the checked core of `src/typed.rs` |
| `src/eval.rs` + `src/ops.rs` | the interpreter and the arithmetic of spec section 6 |

```sh
cargo run -- run ../tests/golden/union/shape.core
cargo run -- check file.core
cargo test
```

`rust-toolchain.toml` pins the Rust release of the benchmarks. A trap exits with status 101 and a checker error with 1.
