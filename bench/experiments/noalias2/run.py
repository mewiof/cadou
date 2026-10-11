#!/usr/bin/env python3
# second aliasing experiment: three loops where a run-time overlap check cannot stand
# in for a no-alias promise. Benchmark flags and timing protocol
#
#   python bench/experiments/noalias2/run.py [--reps 30] [--cpu 4]
#
# write-up: docs/research/noalias2.md

import argparse
import dataclasses
import datetime
import json
import os
import pathlib
import random
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent.parent / "harness"))

import machine
import runner
import stats
import suite
import toolchains

# (variant, source, mode without the promise, mode with it)
PROGRAMS = (
	("c-clang", "kernels.c", "plain", "restrict"),
	("c-gcc", "kernels.c", "plain", "restrict"),
	("rust", "kernels.rs", "pointer", "reference"),
	("zig", "kernels.zig", "plain", "noalias"),
)
KERNELS = ("fields", "arrays", "call")


def build(tools):
	directory = suite.BUILD / "experiments" / "noalias2"
	directory.mkdir(parents=True, exist_ok=True)
	binaries = {}
	variants = {v.name: v for v in suite.VARIANTS}
	for name, source, _, _ in PROGRAMS:
		implementation = suite.Implementation(None, "experiment", variants[name], HERE / source)
		binary = directory / f"{name}{toolchains.EXE}"
		command, env = toolchains.compile_command(tools, implementation, binary, suite.BUILD / "zig-cache")
		subprocess.run(command, check=True, cwd=directory, env={**os.environ, **env})
		binaries[name] = binary
	return binaries


def main(argv=None):
	parser = argparse.ArgumentParser()
	parser.add_argument("--reps", type=int, default=30)
	parser.add_argument("--warmup", type=int, default=3)
	parser.add_argument("--cpu", type=int, default=4)
	parser.add_argument("--seed", type=int, default=1)
	args = parser.parse_args(argv)

	tools = toolchains.Toolchains()
	binaries = build(tools)
	runs = [(name, kernel, mode) for kernel in KERNELS for name, _, plain, promised in PROGRAMS for mode in (plain, promised)]

	# every run of a kernel must print the same result
	outputs = {}
	for name, kernel, mode in runs:
		with tempfile.TemporaryFile() as out:
			runner.run([str(binaries[name]), kernel, mode], stdout=out)
			out.seek(0)
			outputs.setdefault(kernel, set()).add(out.read())
	for kernel, values in outputs.items():
		if len(values) != 1:
			raise SystemExit(f"{kernel}: outputs differ: {values}")

	runner.pin(args.cpu)
	info = machine.describe(args.cpu)
	rng = random.Random(args.seed)
	samples = {run: [] for run in runs}
	for index in range(args.warmup + args.reps):
		order = list(runs)
		rng.shuffle(order)
		for name, kernel, mode in order:
			sample = runner.run([str(binaries[name]), kernel, mode], high_priority=True)
			if index >= args.warmup:
				samples[(name, kernel, mode)].append(dataclasses.asdict(sample))
		print(f"round {index + 1}/{args.warmup + args.reps}", file=sys.stderr, flush=True)
	runner.unpin()

	for kernel in KERNELS:
		print(f"\n{kernel}\n")
		print("| variant | without the promise | with it | ratio |")
		print("| --- | --- | --- | --- |")
		for name, _, plain, promised in PROGRAMS:
			a = [s["wall_ns"] for s in samples[(name, kernel, plain)]]
			b = [s["wall_ns"] for s in samples[(name, kernel, promised)]]
			sa, sb = stats.summarize(a), stats.summarize(b)
			low, high = stats.bootstrap_ratio_ci(a, b, seed=args.seed)
			print(f"| {name} | `{plain}` {sa['median'] / 1e6:.1f} ms | `{promised}` {sb['median'] / 1e6:.1f} ms | {sa['median'] / sb['median']:.2f} ({low:.2f}-{high:.2f}) |")

	raw = {
		"experiment": "noalias2",
		"outputs": {kernel: values.pop().decode().strip() for kernel, values in outputs.items()},
		"machine": info,
		"toolchains": tools.describe(),
		"protocol": {"reps": args.reps, "warmup": args.warmup, "cpu": args.cpu, "seed": args.seed},
		"samples": {f"{name}/{kernel}/{mode}": values for (name, kernel, mode), values in samples.items()},
	}
	date = datetime.date.today().isoformat()
	path = suite.RESULTS / "experiments" / f"{date}-noalias2.json"
	path.parent.mkdir(parents=True, exist_ok=True)
	with open(path, "w", newline="\n") as f:
		json.dump(raw, f, indent="\t")
		f.write("\n")
	print(f"\nwrote {path.relative_to(suite.ROOT).as_posix()}")


if __name__ == "__main__":
	main()
