# benchmarks/competitor variants/the implementations found on disk

import dataclasses
import pathlib
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[2]
PROGRAMS = ROOT / "bench" / "programs"
BUILD = ROOT / "bench" / "build"
RESULTS = ROOT / "bench" / "results"
CATEGORIES = ("idiomatic", "expert")


@dataclasses.dataclass(frozen=True)
class Variant:
	name: str
	language: str
	toolchain: str
	extension: str


VARIANTS = (
	Variant("c-clang", "c", "llvm", ".c"),
	Variant("c-gcc", "c", "gcc", ".c"),
	Variant("cpp-clang", "cpp", "llvm", ".cpp"),
	Variant("cpp-gcc", "cpp", "gcc", ".cpp"),
	Variant("rust", "rust", "rust", ".rs"),
	Variant("zig", "zig", "zig", ".zig"),
)


@dataclasses.dataclass(frozen=True)
class Benchmark:
	name: str
	directory: pathlib.Path
	validate_args: tuple
	validate_output: pathlib.Path | None
	time_args: tuple
	time_output_sha256: str | None
	# stdin comes from a generated input such as ("fasta", "1000") (see inputs.py)
	validate_stdin: tuple | None = None
	time_stdin: tuple | None = None
	time_stdin_sha256: str | None = None


@dataclasses.dataclass(frozen=True)
class Implementation:
	benchmark: Benchmark
	category: str
	variant: Variant
	source: pathlib.Path

	@property
	def key(self):
		return f"{self.benchmark.name}/{self.category}/{self.variant.name}"


def load_benchmark(directory):
	with open(directory / "bench.toml", "rb") as f:
		spec = tomllib.load(f)
	expected = spec.get("validate_output")
	return Benchmark(
		name=directory.name,
		directory=directory,
		validate_args=tuple(spec.get("validate_args", ())),
		validate_output=directory / expected if expected else None,
		time_args=tuple(spec.get("time_args", ())),
		time_output_sha256=spec.get("time_output_sha256") or None,
		validate_stdin=tuple(spec["validate_stdin"]) if "validate_stdin" in spec else None,
		time_stdin=tuple(spec["time_stdin"]) if "time_stdin" in spec else None,
		time_stdin_sha256=spec.get("time_stdin_sha256") or None,
	)


def load_benchmarks(names=None):
	benchmarks = [load_benchmark(p.parent) for p in sorted(PROGRAMS.glob("*/bench.toml"))]
	if names:
		unknown = set(names) - {b.name for b in benchmarks}
		if unknown:
			raise SystemExit(f"unknown benchmark(s): {', '.join(sorted(unknown))}")
		benchmarks = [b for b in benchmarks if b.name in names]
	return benchmarks


def find_implementations(benchmarks, variants=None, categories=CATEGORIES):
	if variants:
		unknown = set(variants) - {v.name for v in VARIANTS}
		if unknown:
			raise SystemExit(f"unknown variant(s): {', '.join(sorted(unknown))}")
	found = []
	for benchmark in benchmarks:
		for category in categories:
			for variant in VARIANTS:
				if variants and variant.name not in variants:
					continue
				source = benchmark.directory / variant.language / f"{category}{variant.extension}"
				if source.exists():
					found.append(Implementation(benchmark, category, variant, source))
	return found
