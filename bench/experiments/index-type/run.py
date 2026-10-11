#!/usr/bin/env python3
# index type experiment: spectral-norm with 32-bit/signed 64-bit/unsigned 64-bit
# indices. Benchmark flags and timing protocol
#
#   python bench/experiments/index-type/run.py [--reps 30] [--cpu 4]
#
# write-up: docs/research/index-type.md

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2] / "harness"))

from experiment import Case, run

C_INT64 = ((r"\bint (i|j|n)\b", r"long long \1", 13),)
C_SIZE = ((r"\bint (i|j|n)\b", r"size_t \1", 13),)
RUST_INT64 = (
	(r"fn a\(i: usize, j: usize\)", "fn a(i: i64, j: i64)", 1),
	(r"a\((i|j), (i|j)\) \* x", r"a(\1 as i64, \2 as i64) * x", 2),
)
# a signed division in Zig must spell out its rounding. @divTrunc is what C and Rust do
ZIG_INT64 = (
	(r"fn a\(i: usize, j: usize\)", "fn a(i: i64, j: i64)", 1),
	(r"\(i \+ j\) \* \(i \+ j \+ 1\) / 2", "@divTrunc((i + j) * (i + j + 1), 2)", 1),
	(r"a\((i|j), (i|j)\) \* x", r"a(@intCast(\1), @intCast(\2)) * x", 2),
)

CASES = (
	Case("spectral-norm", "c-clang int", "c-clang"),
	Case("spectral-norm", "c-clang long long", "c-clang", patches=C_INT64),
	Case("spectral-norm", "c-clang size_t", "c-clang", patches=C_SIZE),
	Case("spectral-norm", "c-gcc int", "c-gcc"),
	Case("spectral-norm", "c-gcc long long", "c-gcc", patches=C_INT64),
	Case("spectral-norm", "c-gcc size_t", "c-gcc", patches=C_SIZE),
	Case("spectral-norm", "rust usize", "rust"),
	Case("spectral-norm", "rust i64", "rust", patches=RUST_INT64),
	Case("spectral-norm", "zig usize", "zig"),
	Case("spectral-norm", "zig i64", "zig", patches=ZIG_INT64),
)

if __name__ == "__main__":
	run("index-type", CASES)
