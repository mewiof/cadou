# shared parts of the experiments in bench/experiments/: a case changes flags or
# source lines of one idiomatic program. Builds use the benchmark commands. Timing
# follows the benchmark protocol (ADR 0003)

import argparse
import dataclasses
import datetime
import hashlib
import json
import os
import random
import re
import statistics
import subprocess
import sys
import tempfile

import inputs
import machine
import runner
import stats
import suite
import toolchains

VARIANTS = {v.name: v for v in suite.VARIANTS}


@dataclasses.dataclass(frozen=True)
class Case:
	benchmark: str
	name: str
	variant: str
	flags: tuple = ()
	# (pattern, replacement, expected count) regular expression substitutions
	patches: tuple = ()


def log(message):
	print(message, file=sys.stderr, flush=True)


def arguments(argv=None):
	parser = argparse.ArgumentParser()
	parser.add_argument("--reps", type=int, default=30)
	parser.add_argument("--warmup", type=int, default=3)
	parser.add_argument("--cpu", type=int, default=4)
	parser.add_argument("--seed", type=int, default=1)
	return parser.parse_args(argv)


def build(tools, case, directory):
	directory.mkdir(parents=True, exist_ok=True)
	variant = VARIANTS[case.variant]
	program = suite.PROGRAMS / case.benchmark / variant.language / f"idiomatic{variant.extension}"
	text = program.read_text(encoding="utf-8")
	for pattern, replacement, expected in case.patches:
		text, count = re.subn(pattern, replacement, text)
		if count != expected:
			raise SystemExit(f"{case.benchmark}/{case.name}: {pattern!r} matched {count} times instead of {expected}")
	name = re.sub(r"[^a-z0-9]+", "-", f"{case.benchmark} {case.name}".lower()).strip("-")
	source = directory / f"{name}{variant.extension}"
	source.write_text(text, encoding="utf-8", newline="\n")
	binary = directory / f"{name}{toolchains.EXE}"
	implementation = suite.Implementation(None, "experiment", variant, source)
	command, env = toolchains.compile_command(tools, implementation, binary, suite.BUILD / "zig-cache")
	command = command + list(case.flags)
	result = subprocess.run(command, capture_output=True, text=True, cwd=directory, env=dict(os.environ, **env))
	if result.returncode != 0:
		raise SystemExit(f"build failed: {case.benchmark}/{case.name}\n{result.stdout}{result.stderr}")
	log(f"built {case.benchmark}: {case.name}")
	return {
		"binary": binary,
		"source_sha256": hashlib.sha256(text.encode("utf-8")).hexdigest(),
		"command": [toolchains.display_path(part) for part in command],
	}


def timed_input(benchmark):
	return inputs.materialize(benchmark.time_stdin, benchmark.time_stdin_sha256) if benchmark.time_stdin else None


def check_outputs(cases, builds, benchmarks):
	# every case must print its benchmark's recorded full-size output
	for case in cases:
		benchmark = benchmarks[case.benchmark]
		with tempfile.TemporaryFile() as out:
			runner.run([str(builds[case].get("binary")), *benchmark.time_args], stdout=out, stdin=timed_input(benchmark))
			out.seek(0)
			digest = hashlib.sha256(out.read()).hexdigest()
		builds[case]["output_sha256"] = digest
		if digest != benchmark.time_output_sha256:
			raise SystemExit(f"{case.benchmark}/{case.name} prints a different output ({digest})")


def measure(cases, builds, benchmarks, args):
	runner.pin(args.cpu)
	info = machine.describe(args.cpu)
	stdin = {case: timed_input(benchmarks[case.benchmark]) for case in cases}
	rng = random.Random(args.seed)
	samples = {case: [] for case in cases}
	for index in range(args.warmup + args.reps):
		order = list(cases)
		rng.shuffle(order)
		for case in order:
			sample = runner.run([str(builds[case]["binary"]), *benchmarks[case.benchmark].time_args], high_priority=True, stdin=stdin[case])
			if index >= args.warmup:
				samples[case].append(dataclasses.asdict(sample))
		log(f"round {index + 1}/{args.warmup + args.reps}")
	runner.unpin()
	return samples, info


def print_tables(cases, samples, benchmarks, seed):
	for name in dict.fromkeys(case.benchmark for case in cases):
		group = [case for case in cases if case.benchmark == name]
		walls = {case: [s["wall_ns"] for s in samples[case]] for case in group}
		fastest = min(group, key=lambda case: statistics.median(walls[case]))
		benchmark = benchmarks[name]
		shown = " ".join([*benchmark.time_args, *(["<", *benchmark.time_stdin] if benchmark.time_stdin else [])])
		print(f"\n{name} {shown}\n")
		print("| case | median | 95% CI | vs fastest |")
		print("| --- | --- | --- | --- |")
		for case in group:
			summary = stats.summarize(walls[case])
			low, high = stats.bootstrap_ratio_ci(walls[case], walls[fastest], seed=seed)
			ratio = summary["median"] / statistics.median(walls[fastest])
			print(f"| {case.name} | {summary['median'] / 1e9:.3f} s | {summary['ci_low'] / 1e9:.3f}-{summary['ci_high'] / 1e9:.3f} s | {ratio:.2f} ({low:.2f}-{high:.2f}) |")


def save(experiment, cases, builds, samples, info, tools, args, extra=None):
	raw = {
		"experiment": experiment,
		"machine": info,
		"toolchains": tools.describe(),
		"protocol": {"reps": args.reps, "warmup": args.warmup, "cpu": args.cpu, "seed": args.seed, "priority": "high"},
		**(extra or {}),
		"cases": [
			{
				"benchmark": case.benchmark,
				"name": case.name,
				"variant": case.variant,
				"flags": list(case.flags),
				"patches": [list(p) for p in case.patches],
				"source_sha256": builds[case]["source_sha256"],
				"command": builds[case]["command"],
				"output_sha256": builds[case]["output_sha256"],
				"samples": samples[case],
			}
			for case in cases
		],
	}
	path = suite.RESULTS / "experiments" / f"{datetime.date.today().isoformat()}-{experiment}.json"
	path.parent.mkdir(parents=True, exist_ok=True)
	with open(path, "w", newline="\n") as f:
		json.dump(raw, f, indent="\t")
		f.write("\n")
	print(f"\nwrote {path.relative_to(suite.ROOT).as_posix()}")


def run(experiment, cases, argv=None, extra=None):
	# build/check/time/report/save in that order
	args = arguments(argv)
	tools = toolchains.Toolchains()
	benchmarks = {b.name: b for b in suite.load_benchmarks(sorted({case.benchmark for case in cases}))}
	directory = suite.BUILD / "experiments" / experiment
	builds = {case: build(tools, case, directory) for case in cases}
	check_outputs(cases, builds, benchmarks)
	samples, info = measure(cases, builds, benchmarks, args)
	print_tables(cases, samples, benchmarks, args.seed)
	save(experiment, cases, builds, samples, info, tools, args, extra)
