#!/usr/bin/env python3
# integer overflow three ways: wrapping/assumed impossible/trapping. Rust with and
# without overflow checks (all integer types). C by clang and gcc in all three modes
# (signed types only since unsigned wrapping is defined in C)
#
#   python bench/experiments/overflow/run.py [--reps 30] [--cpu 4]
#
# write-up: docs/research/overflow.md

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2] / "harness"))

from experiment import Case, run

BENCHMARKS = ("binary-trees", "fannkuch-redux", "fasta", "hash-map", "k-nucleotide", "mandelbrot", "n-body", "sort", "spectral-norm")
TRAP = ("-fsanitize=signed-integer-overflow", "-fsanitize-trap=signed-integer-overflow")

# clang on the MSVC target wraps by default (see clang-msvc.md)
MODES = (
	("rust", "wrap", ()),
	("rust", "check", ("-C", "overflow-checks=on")),
	("c-clang", "wrap", ()),
	("c-clang", "assume", ("-fno-wrapv",)),
	("c-clang", "check", ("-fno-wrapv", *TRAP)),
	("c-gcc", "wrap", ("-fwrapv",)),
	("c-gcc", "assume", ()),
	("c-gcc", "check", TRAP),
)

CASES = tuple(Case(benchmark, f"{variant} {mode}", variant, flags=flags) for benchmark in BENCHMARKS for variant, mode, flags in MODES)

if __name__ == "__main__":
	run("overflow", CASES)
