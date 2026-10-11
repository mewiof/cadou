#!/usr/bin/env python3
# benchmark harness
#
#   python bench/harness/bench.py fetch [zig] [llvm] [gcc] [rust]
#   python bench/harness/bench.py validate [--bench NAME ...] [--variant NAME ...]
#   python bench/harness/bench.py run --label LABEL [--reps 30] [--warmup 3] [--cpu 4]
#   python bench/harness/bench.py report bench/results/DIR
#
# method and flags: docs/decisions/0003-benchmark-methodology.md

import argparse
import dataclasses
import datetime
import hashlib
import json
import os
import pathlib
import random
import shutil
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import inputs
import machine
import report
import runner
import suite
import toolchains as toolchains_module
from suite import BUILD, RESULTS, ROOT
from toolchains import EXE, display_path

FORMAT = 1


def log(message):
	print(message, file=sys.stderr, flush=True)


def file_sha256(path):
	return toolchains_module.sha256_of(path)


# ================================================================
# BUILD
# ================================================================

def build(tools, implementations, compile_reps):
	cache = BUILD / "zig-cache"
	builds = {}
	for implementation in implementations:
		directory = BUILD / implementation.benchmark.name
		directory.mkdir(parents=True, exist_ok=True)
		binary = directory / f"{implementation.category}-{implementation.variant.name}{EXE}"
		times = []
		# the first build warms file caches (and Zig's global cache) and is not timed
		for attempt in range(compile_reps + 1):
			shutil.rmtree(cache / "local", ignore_errors=True)
			command, env = toolchains_module.compile_command(tools, implementation, binary, cache)
			start = time.perf_counter_ns()
			result = subprocess.run(command, capture_output=True, text=True, cwd=directory, env=dict(os.environ, **env))
			elapsed = time.perf_counter_ns() - start
			if result.returncode != 0:
				raise SystemExit(f"build failed: {implementation.key}\n{' '.join(command)}\n{result.stdout}{result.stderr}")
			if attempt > 0:
				times.append(elapsed)

		stripped = None
		with tempfile.TemporaryDirectory() as scratch:
			strip = toolchains_module.strip_command(tools, binary, pathlib.Path(scratch) / binary.name)
			if strip and subprocess.run(strip, capture_output=True).returncode == 0:
				stripped = (pathlib.Path(scratch) / binary.name).stat().st_size

		builds[implementation.key] = {
			"binary": binary,
			"command": [display_path(part) for part in command],
			"compile_ns": times,
			"binary_bytes": binary.stat().st_size,
			"stripped_bytes": stripped,
			"compiler_output": (result.stdout + result.stderr).strip(),
		}
		log(f"built {implementation.key}" + (f" ({times[len(times) // 2] / 1e9:.2f} s)" if times else ""))
	return builds


# ================================================================
# VALIDATION
# ================================================================

def first_difference(expected, actual):
	for number, (a, b) in enumerate(zip(expected.split(b"\n"), actual.split(b"\n")), 1):
		if a != b:
			return f"line {number}: expected {a[:80]!r}, got {b[:80]!r}"
	return f"expected {len(expected)} bytes, got {len(actual)}"


def validate_stdin(benchmark):
	return inputs.materialize(benchmark.validate_stdin) if benchmark.validate_stdin else None


def time_stdin(benchmark):
	return inputs.materialize(benchmark.time_stdin, benchmark.time_stdin_sha256) if benchmark.time_stdin else None


def validate(implementations, builds):
	failed = []
	for implementation in implementations:
		benchmark = implementation.benchmark
		expected = benchmark.validate_output.read_bytes() if benchmark.validate_output else b""
		with tempfile.TemporaryFile() as out:
			runner.run([str(builds[implementation.key]["binary"]), *benchmark.validate_args], stdout=out, stdin=validate_stdin(benchmark))
			out.seek(0)
			actual = out.read()
		if actual == expected:
			log(f"valid {implementation.key}")
		else:
			failed.append(implementation.key)
			log(f"WRONG {implementation.key}: {first_difference(expected, actual)}")
	return failed


def check_full_outputs(implementations, builds):
	# the timed input size runs once with its output hashed. Every variant must
	# agree (and match bench.toml once a hash is recorded there)
	hashes = {}
	for implementation in implementations:
		with tempfile.TemporaryDirectory() as scratch:
			path = pathlib.Path(scratch) / "out"
			with open(path, "wb") as out:
				runner.run([str(builds[implementation.key]["binary"]), *implementation.benchmark.time_args], stdout=out, high_priority=True, stdin=time_stdin(implementation.benchmark))
			hashes[implementation.key] = file_sha256(path)

	failed = []
	by_benchmark = {}
	for implementation in implementations:
		by_benchmark.setdefault(implementation.benchmark.name, []).append(implementation)
	for name, group in by_benchmark.items():
		recorded = group[0].benchmark.time_output_sha256
		reference = recorded or hashes[group[0].key]
		wrong = [i.key for i in group if hashes[i.key] != reference]
		for key in wrong:
			log(f"WRONG {key}: full-size output sha256 {hashes[key]} (expected {reference})")
		if not recorded and not wrong:
			log(f"{name}: all variants agree on {reference} (not yet in bench.toml)")
		failed += wrong
	return hashes, failed


# ================================================================
# MEASUREMENT
# ================================================================

def measure(implementations, builds, reps, warmup, seed):
	rng = random.Random(seed)
	samples = {implementation.key: [] for implementation in implementations}
	stdin = {implementation.key: time_stdin(implementation.benchmark) for implementation in implementations}
	rounds = warmup + reps
	start = time.monotonic()
	for index in range(rounds):
		order = list(implementations)
		rng.shuffle(order)
		for implementation in order:
			sample = runner.run([str(builds[implementation.key]["binary"]), *implementation.benchmark.time_args], high_priority=True, stdin=stdin[implementation.key])
			if index >= warmup:
				samples[implementation.key].append(dataclasses.asdict(sample))
		elapsed = time.monotonic() - start
		remaining = elapsed / (index + 1) * (rounds - index - 1)
		log(f"round {index + 1}/{rounds} ({'warmup' if index < warmup else 'timed'}): {elapsed / 60:.1f} min in and about {remaining / 60:.1f} min left")
	return samples


def collect_counters(implementations, builds, reps):
	counters = {}
	for implementation in implementations:
		command = [str(builds[implementation.key]["binary"]), *implementation.benchmark.time_args]
		stdin = time_stdin(implementation.benchmark)
		counters[implementation.key] = [runner.perf_counters(command, stdin=stdin) for _ in range(reps)]
		log(f"counted {implementation.key}")
	return counters


# ================================================================
# COMMANDS
# ================================================================

def select(args):
	benchmarks = suite.load_benchmarks(args.bench)
	implementations = suite.find_implementations(benchmarks, args.variant, args.category or suite.CATEGORIES)
	if not implementations:
		raise SystemExit("nothing to do")
	return implementations


def command_fetch(args):
	toolchains_module.fetch(args.names)


def command_validate(args):
	implementations = select(args)
	tools = toolchains_module.Toolchains(use_pins=not args.system_toolchains)
	builds = build(tools, implementations, compile_reps=0)
	failed = validate(implementations, builds)
	if args.full_size and not failed:
		_, failed = check_full_outputs(implementations, builds)
	if failed:
		raise SystemExit(f"{len(failed)} implementation(s) failed: {', '.join(failed)}")
	log(f"all {len(implementations)} implementations valid")


def command_run(args):
	implementations = select(args)
	tools = toolchains_module.Toolchains(use_pins=not args.system_toolchains)
	runner.unpin()
	builds = build(tools, implementations, args.compile_reps)
	failed = validate(implementations, builds)
	if failed:
		raise SystemExit(f"validation failed: {', '.join(failed)}")

	runner.pin(args.cpu)
	# taken before measuring (time and commit are those of the start)
	machine_info = machine.describe(args.cpu)
	hashes, failed = check_full_outputs(implementations, builds)
	if failed:
		raise SystemExit(f"full-size outputs differ: {', '.join(failed)}")
	samples = measure(implementations, builds, args.reps, args.warmup, args.seed)
	counters = collect_counters(implementations, builds, args.counter_reps) if args.counter_reps and runner.perf_available() else {}
	runner.unpin()

	raw = {
		"format": FORMAT,
		"label": args.label,
		"machine": machine_info,
		"target_cpu": toolchains_module.target_cpu(tools),
		"toolchains": tools.describe(),
		"protocol": {
			"reps": args.reps,
			"warmup": args.warmup,
			"cpu": args.cpu,
			"priority": "high",
			"seed": args.seed,
			"order": "every round runs each implementation once, shuffled",
			"compile_reps": args.compile_reps,
			"counters": bool(counters),
		},
		"variants": [v.name for v in suite.VARIANTS],
		"implementations": {},
	}
	for implementation in implementations:
		entry = builds[implementation.key]
		raw["implementations"][implementation.key] = {
			"benchmark": implementation.benchmark.name,
			"category": implementation.category,
			"variant": implementation.variant.name,
			"source": implementation.source.relative_to(ROOT).as_posix(),
			"source_sha256": file_sha256(implementation.source),
			"args": list(implementation.benchmark.time_args),
			"stdin": list(implementation.benchmark.time_stdin) if implementation.benchmark.time_stdin else None,
			"stdin_sha256": inputs.sha256_of(time_stdin(implementation.benchmark)) if implementation.benchmark.time_stdin else None,
			"command": entry["command"],
			"compile_ns": entry["compile_ns"],
			"binary_bytes": entry["binary_bytes"],
			"stripped_bytes": entry["stripped_bytes"],
			"output_sha256": hashes[implementation.key],
			"samples": samples[implementation.key],
			"counters": counters.get(implementation.key),
		}

	date = raw["machine"]["recorded_utc"][:10]
	directory = RESULTS / f"{date}-{args.label}"
	directory.mkdir(parents=True, exist_ok=True)
	with open(directory / "raw.json", "w", newline="\n") as f:
		json.dump(raw, f, indent="\t")
		f.write("\n")
	write_report(directory)
	log(f"wrote {directory.relative_to(ROOT).as_posix()}")


def write_report(directory):
	with open(directory / "raw.json") as f:
		raw = json.load(f)
	with open(directory / "report.md", "w", newline="\n") as f:
		f.write(report.render(raw))


def command_report(args):
	write_report(pathlib.Path(args.directory))


def main(argv=None):
	parser = argparse.ArgumentParser(prog="bench.py", description="CADOU benchmark harness")
	commands = parser.add_subparsers(dest="command", required=True)

	fetch = commands.add_parser("fetch", help="download the pinned toolchains")
	fetch.add_argument("names", nargs="*")
	fetch.set_defaults(handler=command_fetch)

	def add_selection(sub):
		sub.add_argument("--bench", action="append", help="benchmark name (repeatable)")
		sub.add_argument("--variant", action="append", help="variant name (repeatable)")
		sub.add_argument("--category", action="append", choices=suite.CATEGORIES, help="default: all")
		sub.add_argument("--system-toolchains", action="store_true", help="ignore the pins and use PATH")

	validate_parser = commands.add_parser("validate", help="build everything and check outputs")
	add_selection(validate_parser)
	validate_parser.add_argument("--full-size", action="store_true", help="also compare the outputs of the timed input sizes")
	validate_parser.set_defaults(handler=command_validate)

	run = commands.add_parser("run", help="build + validate + measure, then write bench/results/DATE-LABEL")
	add_selection(run)
	run.add_argument("--label", required=True, help="e.g. i7-14700kf-windows")
	run.add_argument("--reps", type=int, default=30)
	run.add_argument("--warmup", type=int, default=3)
	run.add_argument("--cpu", type=int, default=4, help="logical CPU to pin to (-1 disables pinning)")
	run.add_argument("--seed", type=int, default=1)
	run.add_argument("--compile-reps", type=int, default=3)
	run.add_argument("--counter-reps", type=int, default=5, help="perf stat runs per implementation (Linux)")
	run.set_defaults(handler=command_run)

	report_parser = commands.add_parser("report", help="re-render report.md from raw.json")
	report_parser.add_argument("directory")
	report_parser.set_defaults(handler=command_report)

	args = parser.parse_args(argv)
	if getattr(args, "cpu", None) == -1:
		args.cpu = None
	args.handler(args)


if __name__ == "__main__":
	main()
