# Tests

Unit and golden tests for each compiler stage/differential tests against the reference interpreter/a regression test for every fixed bug. The benchmark harness has its own tests in `bench/harness/tests/`.

`golden/` holds programs in the core notation ([spec/notation.md](../spec/notation.md)). Each `name.core` has a `name.out` next to it with what a run prints: the output followed by `exit N` or by the trap or the first error with its line and column. `cargo test` in `compiler/` runs them all. `CADOU_BLESS=1 cargo test` rewrites the `.out` files instead (read the diff before committing it). A program that traps or fails to check tests exactly one thing.
