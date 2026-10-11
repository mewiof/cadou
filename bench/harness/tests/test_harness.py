import unittest

import report
import runner
import suite
import toolchains


class SuiteTest(unittest.TestCase):
	def test_every_benchmark_has_all_idiomatic_variants(self):
		benchmarks = suite.load_benchmarks()
		self.assertGreaterEqual(len(benchmarks), 6)
		for benchmark in benchmarks:
			found = {i.variant.name for i in suite.find_implementations([benchmark], categories=("idiomatic",))}
			self.assertEqual(found, {v.name for v in suite.VARIANTS}, benchmark.name)

	def test_expected_outputs_exist(self):
		for benchmark in suite.load_benchmarks():
			if benchmark.validate_output is not None:
				self.assertTrue(benchmark.validate_output.exists(), benchmark.name)
			self.assertTrue(benchmark.time_args or benchmark.name == "startup", benchmark.name)


class PerfParseTest(unittest.TestCase):
	def test_hybrid_events_are_summed_and_unsupported_skipped(self):
		text = "\n".join([
			"# started on Sat Oct 11 10:00:00 2026",
			"",
			"1000,,cpu_core/cycles/,100.00,,",
			"20,,cpu_atom/cycles/,0.50,,",
			"3000,,cpu_core/instructions/,100.00,3.00,insn per cycle",
			"<not counted>,,cpu_atom/instructions/,0.00,,",
			"<not supported>,,cache-misses,0.00,,",
			"7,,branch-misses:u,100.00,,",
		])
		self.assertEqual(runner.parse_perf_csv(text), {"cycles": 1020, "instructions": 3000, "branch-misses": 7})


class DisplayPathTest(unittest.TestCase):
	def test_home_and_root_are_hidden(self):
		inside = str(suite.ROOT / "bench" / "build" / "x.exe")
		self.assertNotIn(str(suite.ROOT), toolchains.display_path(f"-femit-bin={inside}"))


def fake_raw():
	def samples(base):
		return [{"wall_ns": base + 1000 * (i % 5), "user_ns": None, "system_ns": None, "peak_memory_bytes": 4 << 20, "reference_cycles": None, "exit_code": 0} for i in range(30)]

	implementations = {}
	for benchmark, base in (("n-body", 1_200_000_000), ("startup", 3_000_000)):
		for offset, variant in enumerate(("c-clang", "rust")):
			implementations[f"{benchmark}/idiomatic/{variant}"] = {
				"benchmark": benchmark, "category": "idiomatic", "variant": variant,
				"args": ["50000000"] if benchmark == "n-body" else [],
				"compile_ns": [500_000_000, 510_000_000, 520_000_000],
				"binary_bytes": 150_000, "stripped_bytes": 140_000, "output_sha256": "ab" * 32,
				"samples": samples(base + offset * base // 10),
			}
	return {
		"label": "test",
		"machine": {
			"recorded_utc": "2026-10-11T00:00:00+00:00",
			"cpu": {"model": "Test CPU", "logical_cpus": 8},
			"os": {"system": "Windows", "release": "11", "build": 26300},
			"repository": {"commit": "0123456789abcdef", "modified": False},
			"pinned_cpu_kind": {"core": 2, "kind": "performance"},
		},
		"target_cpu": {"clang": "raptorlake"},
		"toolchains": {"clang": {"version": "clang version 23.1.3", "pinned": True}},
		"protocol": {"reps": 30, "warmup": 3, "cpu": 4, "seed": 1, "compile_reps": 3, "counters": False},
		"variants": ["c-clang", "c-gcc", "rust"],
		"implementations": implementations,
	}


class ReportTest(unittest.TestCase):
	def test_renders_ascii_with_fastest_first(self):
		text = report.render(fake_raw())
		self.assertTrue(text.isascii())
		# c-gcc has no samples (so no column)
		self.assertIn("| benchmark | input | c-clang | rust |", text)
		self.assertIn("| n-body | 50000000 | **1.00** | 1.10 |", text)
		nbody = text.split("## n-body", 1)[1]
		self.assertLess(nbody.index("| c-clang |"), nbody.index("| rust |"))


if __name__ == "__main__":
	unittest.main()
