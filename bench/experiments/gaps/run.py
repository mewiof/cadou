#!/usr/bin/env python3
# baseline gaps: n-body/fannkuch-redux/fasta rebuilt with one change at a time.
# Same flags and timing protocol as the benchmarks. The programs come from the
# commit of the first baseline's sources (later edits leave this experiment alone)
#
#   python bench/experiments/gaps/run.py [--reps 30] [--cpu 4] [--bench NAME]
#
# write-up: docs/research/baseline-gaps.md

import argparse
import dataclasses
import datetime
import hashlib
import json
import os
import pathlib
import random
import re
import statistics
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

BUILD = suite.BUILD / "experiments" / "gaps"
BASE = "92c5a20e18ac45b876eaa27d1dea4920a9fa4500"
VARIANTS = {v.name: v for v in suite.VARIANTS}
NBODY_STEPS = 50_000_000
CLOCK_ITERATIONS = 500_000_000


@dataclasses.dataclass(frozen=True)
class Case:
	benchmark: str
	name: str
	variant: str
	flags: tuple = ()
	# (old, new) replacements in the idiomatic source (each old text occurs once)
	patches: tuple = ()
	# a build the harness has no command for: "zig cc" or "zig ir"
	special: str = ""


# ================================================================
# PATCHES
# ================================================================

# n-body C++: the pair loop of Advance indexed like the C version
INT_PAIR_INDICES = (
	"for (std::size_t i = 0; i < bodies.size(); i++) {\n\t\t\tBody& a = bodies[i];\n\t\t\tfor (std::size_t j = i + 1; j < bodies.size(); j++) {",
	"for (int i = 0; i < 5; i++) {\n\t\t\tBody& a = bodies[i];\n\t\t\tfor (int j = i + 1; j < 5; j++) {",
)

# fannkuch-redux: library reverse/rotate replaced by the loops of the C version
CPP_FLIP_LOOP = (
	"\t\t\tstd::reverse(perm.begin(), perm.begin() + k + 1);\n",
	"\t\t\tfor (int i = 0, j = k; i < j; i++, j--)\n\t\t\t\tstd::swap(perm[i], perm[j]);\n",
)
CPP_ROTATE_LOOP = (
	"\t\t\tstd::rotate(perm1.begin(), perm1.begin() + 1, perm1.begin() + r + 1);\n",
	"\t\t\tint first = perm1[0];\n\t\t\tfor (int i = 0; i < r; i++)\n\t\t\t\tperm1[i] = perm1[i + 1];\n\t\t\tperm1[r] = first;\n",
)
ZIG_FLIP_LOOP = (
	"\t\t\tstd.mem.reverse(usize, perm[0 .. k + 1]);\n",
	"\t\t\tvar i: usize = 0;\n\t\t\tvar j = k;\n\t\t\twhile (i < j) : ({\n\t\t\t\ti += 1;\n\t\t\t\tj -= 1;\n\t\t\t}) std.mem.swap(usize, &perm[i], &perm[j]);\n",
)
ZIG_ROTATE_LOOP = (
	"\t\t\tstd.mem.rotate(usize, perm1[0 .. r + 1], 1);\n",
	"\t\t\tconst first = perm1[0];\n\t\t\tfor (0..r) |i| perm1[i] = perm1[i + 1];\n\t\t\tperm1[r] = first;\n",
)

# fasta: full buffering for stdout (the UCRT treats NUL as a terminal and leaves it unbuffered)
C_FULL_BUFFER = (
	"\tsize_t n = argc > 1 ? (size_t)atol(argv[1]) : 1000;\n",
	"\tsetvbuf(stdout, NULL, _IOFBF, 1 << 16);\n\tsize_t n = argc > 1 ? (size_t)atol(argv[1]) : 1000;\n",
)
CPP_FULL_BUFFER = (
	"\tstd::size_t n = argc > 1 ? std::strtoull(argv[1], nullptr, 10) : 1000;\n",
	"\tstd::setvbuf(stdout, nullptr, _IOFBF, 1 << 16);\n\tstd::size_t n = argc > 1 ? std::strtoull(argv[1], nullptr, 10) : 1000;\n",
)

CASES = (
	Case("n-body", "c-clang", "c-clang"),
	Case("n-body", "c-clang -fno-math-errno", "c-clang", flags=("-fno-math-errno",)),
	Case("n-body", "c-clang -fno-slp-vectorize", "c-clang", flags=("-fno-slp-vectorize",)),
	Case("n-body", "c through zig cc (clang 22.1.8)", "c-clang", special="zig cc"),
	Case("n-body", "c-gcc", "c-gcc"),
	Case("n-body", "c-gcc -fno-math-errno", "c-gcc", flags=("-fno-math-errno",)),
	Case("n-body", "cpp-clang", "cpp-clang"),
	Case("n-body", "cpp-clang -fno-slp-vectorize", "cpp-clang", flags=("-fno-slp-vectorize",)),
	Case("n-body", "cpp-clang with int pair indices", "cpp-clang", patches=(INT_PAIR_INDICES,)),
	Case("n-body", "rust", "rust"),
	Case("n-body", "zig", "zig"),
	Case("n-body", "zig ir through llvm 23.1.3 -O3", "zig", special="zig ir"),
	Case("fannkuch-redux", "c-gcc", "c-gcc"),
	Case("fannkuch-redux", "c-clang", "c-clang"),
	Case("fannkuch-redux", "cpp-clang", "cpp-clang"),
	Case("fannkuch-redux", "cpp-clang -D_USE_STD_VECTOR_ALGORITHMS=0", "cpp-clang", flags=("-D_USE_STD_VECTOR_ALGORITHMS=0",)),
	Case("fannkuch-redux", "cpp-clang with a flip loop", "cpp-clang", patches=(CPP_FLIP_LOOP,)),
	Case("fannkuch-redux", "cpp-clang with a rotate loop", "cpp-clang", patches=(CPP_ROTATE_LOOP,)),
	Case("fannkuch-redux", "cpp-clang with both loops", "cpp-clang", patches=(CPP_FLIP_LOOP, CPP_ROTATE_LOOP)),
	Case("fannkuch-redux", "cpp-gcc", "cpp-gcc"),
	Case("fannkuch-redux", "cpp-gcc with a flip loop", "cpp-gcc", patches=(CPP_FLIP_LOOP,)),
	Case("fannkuch-redux", "cpp-gcc with both loops", "cpp-gcc", patches=(CPP_FLIP_LOOP, CPP_ROTATE_LOOP)),
	Case("fannkuch-redux", "rust", "rust"),
	Case("fannkuch-redux", "zig", "zig"),
	Case("fannkuch-redux", "zig with a rotate loop", "zig", patches=(ZIG_ROTATE_LOOP,)),
	Case("fannkuch-redux", "zig with both loops", "zig", patches=(ZIG_FLIP_LOOP, ZIG_ROTATE_LOOP)),
	Case("fasta", "c-clang", "c-clang"),
	Case("fasta", "c-clang with setvbuf", "c-clang", patches=(C_FULL_BUFFER,)),
	Case("fasta", "c-gcc", "c-gcc"),
	Case("fasta", "c-gcc with setvbuf", "c-gcc", patches=(C_FULL_BUFFER,)),
	Case("fasta", "cpp-clang", "cpp-clang"),
	Case("fasta", "cpp-clang with setvbuf", "cpp-clang", patches=(CPP_FULL_BUFFER,)),
	Case("fasta", "cpp-gcc", "cpp-gcc"),
	Case("fasta", "cpp-gcc with setvbuf", "cpp-gcc", patches=(CPP_FULL_BUFFER,)),
	Case("fasta", "rust", "rust"),
	Case("fasta", "zig", "zig"),
)


# ================================================================
# BUILD
# ================================================================

def slug(name):
	return re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-")


def check_run(command, directory, env=None):
	result = subprocess.run([str(part) for part in command], capture_output=True, text=True, cwd=directory, env=dict(os.environ, **(env or {})))
	if result.returncode != 0:
		raise SystemExit(f"failed: {' '.join(map(str, command))}\n{result.stdout}{result.stderr}")


def build_zig_ir(tools, llvm, source, binary, directory):
	# the unoptimized IR of Zig through the plain -O3 pipeline of the pinned LLVM. Zig 0.17.0
	# prints a stray ", ," before !prof (fixed in the text). LLVM 23 would turn a loop in
	# Zig's start code into a wcslen call no library provides (hence -disable-builtin)
	zig = tools.require("zig").path
	cache = ["--cache-dir", directory / "zig-cache", "--global-cache-dir", suite.BUILD / "zig-cache" / "global"]
	ir = directory / "zig-ir.ll"
	check_run([zig, "build-exe", source, "-O", "ReleaseFast", "-mcpu=native", "-fstrip", f"--verbose-llvm-ir={ir}", f"-femit-bin={directory / 'zig-ir-reference.exe'}", *cache], directory)
	ir.write_text(ir.read_text().replace(", , !prof", ", !prof"), newline="\n")
	optimized = directory / "zig-ir.bc"
	obj = directory / "zig-ir.obj"
	check_run([llvm / f"opt{toolchains.EXE}", "-O3", "-disable-builtin=wcslen", ir, "-o", optimized], directory)
	check_run([llvm / f"llc{toolchains.EXE}", "-O3", "-filetype=obj", optimized, "-o", obj], directory)
	check_run([zig, "build-exe", obj, "-O", "ReleaseFast", "-mcpu=native", "-fstrip", "-lntdll", "-lkernel32", f"-femit-bin={binary}", *cache], directory)
	return ["zig build-exe --verbose-llvm-ir", "opt -O3 -disable-builtin=wcslen", "llc -O3 -filetype=obj", "zig build-exe -lntdll -lkernel32"]


def build(tools, llvm, case):
	directory = BUILD / case.benchmark
	directory.mkdir(parents=True, exist_ok=True)
	variant = VARIANTS[case.variant]
	program = f"bench/programs/{case.benchmark}/{variant.language}/idiomatic{variant.extension}"
	text = subprocess.run(["git", "show", f"{BASE}:{program}"], cwd=suite.ROOT, capture_output=True, check=True).stdout.decode()
	for old, new in case.patches:
		if text.count(old) != 1:
			raise SystemExit(f"{case.name}: a patch does not apply to {program}")
		text = text.replace(old, new)
	source = directory / f"{slug(case.name)}{variant.extension}"
	source.write_text(text, newline="\n")
	binary = directory / f"{slug(case.name)}{toolchains.EXE}"
	if case.special == "zig cc":
		# no -flto: zig cc fails to link LTO objects against its own libc on Windows
		command = [tools.require("zig").path, "cc", "-std=c17", "-O3", "-march=native", "-ffp-contract=off", str(source), "-o", str(binary)]
		check_run(command, directory)
	elif case.special == "zig ir":
		command = build_zig_ir(tools, llvm, source, binary, directory)
	else:
		implementation = suite.Implementation(None, "experiment", variant, source)
		command, env = toolchains.compile_command(tools, implementation, binary, suite.BUILD / "zig-cache")
		command = command + list(case.flags)
		check_run(command, directory, env)
	return {"binary": binary, "source": source, "command": [toolchains.display_path(part) for part in command]}


# ================================================================
# GENERATED CODE
# ================================================================

INSTRUCTION = re.compile(r"^\s*([0-9a-f]+):\s+(\S+)\s*(.*)$")


def lanes(mnemonic, operands):
	if mnemonic.endswith("sd"):
		return 1
	return 4 if "%ymm" in operands else 2


def step_loop(llvm, binary):
	# the loop with the most sqrt lanes. A fully unrolled n-body step has 10 of them
	text = subprocess.run([str(llvm / f"llvm-objdump{toolchains.EXE}"), "-d", "--no-show-raw-insn", str(binary)], capture_output=True, text=True).stdout
	code = []
	for line in text.splitlines():
		match = INSTRUCTION.match(line)
		if match:
			code.append((int(match.group(1), 16), match.group(2), match.group(3).split("#")[0].strip()))
	index = {address: i for i, (address, _, _) in enumerate(code)}
	best = None
	for i, (address, mnemonic, operands) in enumerate(code):
		target = re.match(r"0x([0-9a-f]+)", operands) if mnemonic.startswith("j") else None
		if not target or int(target.group(1), 16) not in index or int(target.group(1), 16) >= address:
			continue
		body = code[index[int(target.group(1), 16)]: i]
		sqrt = sum(lanes(m, o) for _, m, o in body if m.startswith("vsqrt"))
		if sqrt and (best is None or sqrt > best[0] or (sqrt == best[0] and len(body) < len(best[1]))):
			best = (sqrt, body)
	if best is None or best[0] != 10 or any(m.startswith("j") for _, m, _ in best[1]):
		return None
	body = best[1]
	listing = "\n".join(f"{m} {re.sub(r'<[^>]*>', '', o).strip()}" for _, m, o in body if not m.startswith("nop")) + "\n"
	mca = subprocess.run([str(llvm / f"llvm-mca{toolchains.EXE}"), "-mcpu=raptorlake", "-iterations=1000"], input=listing, capture_output=True, text=True).stdout
	cycles = re.search(r"Total Cycles:\s+(\d+)", mca)
	return {
		"instructions": len(body) + 1,
		"div_lanes": sum(lanes(m, o) for _, m, o in body if m.startswith("vdiv")),
		"mca_cycles": int(cycles.group(1)) / 1000 if cycles else None,
	}


# ================================================================
# MEASUREMENT
# ================================================================

def output_sha256(binary, args):
	with tempfile.TemporaryDirectory() as scratch:
		path = pathlib.Path(scratch) / "out"
		with open(path, "wb") as out:
			runner.run([str(binary), *args], stdout=out)
		return hashlib.sha256(path.read_bytes()).hexdigest()


def clock_hz(tools, cpu):
	directory = BUILD / "clock"
	directory.mkdir(parents=True, exist_ok=True)
	binary = directory / f"clock{toolchains.EXE}"
	check_run([tools.require("clang").path, "-O2", HERE / "clock.c", "-o", binary], directory)
	runner.pin(cpu)
	walls = [runner.run([str(binary), str(CLOCK_ITERATIONS)], high_priority=True).wall_ns for _ in range(7)]
	runner.unpin()
	return 20 * CLOCK_ITERATIONS / (statistics.median(walls) / 1e9)


def main(argv=None):
	parser = argparse.ArgumentParser()
	parser.add_argument("--reps", type=int, default=30)
	parser.add_argument("--warmup", type=int, default=3)
	parser.add_argument("--cpu", type=int, default=4)
	parser.add_argument("--seed", type=int, default=1)
	parser.add_argument("--bench", action="append", help="limit to these benchmarks")
	args = parser.parse_args(argv)

	tools = toolchains.Toolchains()
	llvm = toolchains.pinned_bin("llvm", tools.pins, toolchains.host_key(), True)
	benchmarks = {b.name: b for b in suite.load_benchmarks()}
	cases = [c for c in CASES if not args.bench or c.benchmark in args.bench]

	builds = {}
	for case in cases:
		builds[case.name, case.benchmark] = build(tools, llvm, case)
		print(f"built {case.benchmark}: {case.name}", file=sys.stderr, flush=True)

	# every case must print the benchmark's recorded full-size output
	for case in cases:
		benchmark = benchmarks[case.benchmark]
		entry = builds[case.name, case.benchmark]
		entry["output_sha256"] = output_sha256(entry["binary"], benchmark.time_args)
		if entry["output_sha256"] != benchmark.time_output_sha256:
			raise SystemExit(f"{case.benchmark}: {case.name} prints a different output")

	hz = clock_hz(tools, args.cpu)
	runner.pin(args.cpu)
	info = machine.describe(args.cpu)
	rng = random.Random(args.seed)
	samples = {(c.name, c.benchmark): [] for c in cases}
	for index in range(args.warmup + args.reps):
		order = list(cases)
		rng.shuffle(order)
		for case in order:
			sample = runner.run([str(builds[case.name, case.benchmark]["binary"]), *benchmarks[case.benchmark].time_args], high_priority=True)
			if index >= args.warmup:
				samples[case.name, case.benchmark].append(dataclasses.asdict(sample))
		print(f"round {index + 1}/{args.warmup + args.reps}", file=sys.stderr, flush=True)
	runner.unpin()

	print(f"core clock {hz / 1e9:.2f} GHz (dependent add chain on CPU {args.cpu})")
	raw_cases = {}
	for name in dict.fromkeys(c.benchmark for c in cases):
		group = [c for c in cases if c.benchmark == name]
		walls = {c.name: [s["wall_ns"] for s in samples[c.name, name]] for c in group}
		fastest = min(group, key=lambda c: statistics.median(walls[c.name]))
		print(f"\n{name} {' '.join(benchmarks[name].time_args)}\n")
		print("| case | median | 95% CI | vs fastest |")
		print("| --- | --- | --- | --- |")
		for case in sorted(group, key=lambda c: statistics.median(walls[c.name])):
			summary = stats.summarize(walls[case.name])
			low, high = stats.bootstrap_ratio_ci(walls[case.name], walls[fastest.name], seed=args.seed)
			ratio = summary["median"] / statistics.median(walls[fastest.name])
			print(f"| {case.name} | {summary['median'] / 1e9:.3f} s | {summary['ci_low'] / 1e9:.3f}-{summary['ci_high'] / 1e9:.3f} s | {ratio:.2f} ({low:.2f}-{high:.2f}) |")
		entries = {}
		for case in group:
			entry = builds[case.name, name]
			loop = step_loop(llvm, entry["binary"]) if name == "n-body" else None
			entries[case.name] = {
				"variant": case.variant,
				"flags": list(case.flags),
				"patches": [list(p) for p in case.patches],
				"special": case.special,
				"source_sha256": hashlib.sha256(pathlib.Path(entry["source"]).read_bytes()).hexdigest(),
				"command": entry["command"],
				"output_sha256": entry["output_sha256"],
				"loop": loop,
				"samples": samples[case.name, name],
			}
		raw_cases[name] = entries
		if name == "n-body":
			print("\n| case | step loop instructions | div lanes | llvm-mca cycles per step | measured cycles per step |")
			print("| --- | --- | --- | --- | --- |")
			for case in group:
				loop = entries[case.name]["loop"]
				measured = statistics.median(walls[case.name]) / 1e9 * hz / NBODY_STEPS
				if loop:
					print(f"| {case.name} | {loop['instructions']} | {loop['div_lanes']} | {loop['mca_cycles']:.1f} | {measured:.0f} |")
				else:
					print(f"| {case.name} | not one straight-line step | | | {measured:.0f} |")

	raw = {
		"experiment": "gaps",
		"machine": info,
		"toolchains": tools.describe(),
		"protocol": {"reps": args.reps, "warmup": args.warmup, "cpu": args.cpu, "seed": args.seed, "priority": "high"},
		"clock_hz": hz,
		"cases": raw_cases,
	}
	date = datetime.date.today().isoformat()
	path = suite.RESULTS / "experiments" / f"{date}-gaps.json"
	path.parent.mkdir(parents=True, exist_ok=True)
	with open(path, "w", newline="\n") as f:
		json.dump(raw, f, indent="\t")
		f.write("\n")
	print(f"\nwrote {path.relative_to(suite.ROOT).as_posix()}")


if __name__ == "__main__":
	main()
