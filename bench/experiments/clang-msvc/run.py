#!/usr/bin/env python3
# clang on the MSVC target defaults to -fwrapv and to no type-based alias analysis.
# Each C and C++ benchmark built by clang with those defaults and with the standard
# semantics that clang uses on Linux
#
#   python bench/experiments/clang-msvc/run.py [--reps 30] [--cpu 4]
#
# write-up: docs/research/clang-msvc.md

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2] / "harness"))

from experiment import Case, run

STANDARD = ("-fno-wrapv", "-fstrict-aliasing")
BENCHMARKS = ("binary-trees", "fannkuch-redux", "fasta", "mandelbrot", "n-body", "spectral-norm")

CASES = tuple(
	Case(benchmark, f"{variant}{' ' + ' '.join(flags) if flags else ''}", variant, flags=flags)
	for benchmark in BENCHMARKS
	for variant in ("c-clang", "cpp-clang")
	for flags in ((), STANDARD)
)

if __name__ == "__main__":
	run("clang-msvc", CASES)
