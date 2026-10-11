# markdown report from a raw.json written by `bench.py run`

import stats

ALPHA = 0.05


def seconds(ns):
	if ns < 1e8:
		return f"{ns / 1e6:.2f} ms"
	return f"{ns / 1e9:.3f} s"


def span(low, high):
	if high < 1e8:
		return f"{low / 1e6:.2f}-{high / 1e6:.2f} ms"
	return f"{low / 1e9:.3f}-{high / 1e9:.3f} s"


def size(value, unit):
	if value is None:
		return "-"
	scale = {"KiB": 1024, "MiB": 1024 * 1024}[unit]
	return f"{value / scale:,.0f} {unit}" if value >= 10 * scale else f"{value / scale:,.1f} {unit}"


def pvalue(p):
	return "<0.001" if p < 0.001 else f"{p:.3f}"


def program_input(entry):
	parts = list(entry["args"])
	if entry.get("stdin"):
		parts.append(f"< {' '.join(entry['stdin'])}")
	return " ".join(parts)


def analyze(raw):
	# per benchmark: summaries + ratio to the fastest + Holm-adjusted p against it
	by_benchmark = {}
	for key, entry in raw["implementations"].items():
		walls = [s["wall_ns"] for s in entry["samples"]]
		row = {"key": key, "entry": entry, "walls": walls, "summary": stats.summarize(walls)}
		by_benchmark.setdefault((entry["benchmark"], entry["category"]), []).append(row)

	for rows in by_benchmark.values():
		rows.sort(key=lambda r: r["summary"]["median"])
		fastest = rows[0]
		others = rows[1:]
		raw_p = [stats.mann_whitney(r["walls"], fastest["walls"])[1] for r in others]
		for row, p in zip(others, stats.holm(raw_p)):
			row["p"] = p
			row["ratio"] = row["summary"]["median"] / fastest["summary"]["median"]
			row["ratio_ci"] = stats.bootstrap_ratio_ci(row["walls"], fastest["walls"], seed=raw["protocol"]["seed"])
		fastest["p"] = None
		fastest["ratio"] = 1.0
		fastest["ratio_ci"] = None
	return by_benchmark


def render(raw):
	machine = raw["machine"]
	protocol = raw["protocol"]
	tools = raw["toolchains"]
	analyzed = analyze(raw)
	variants = [v for v in raw["variants"] if any(r["entry"]["variant"] == v for rows in analyzed.values() for r in rows)]

	cpu = machine["cpu"]
	target = " and ".join(f"{k} {v}" for k, v in raw.get("target_cpu", {}).items()) or "unknown"
	os_info = machine["os"]
	os_text = f"{os_info['system']} {os_info['release']} build {os_info['build']}" if "build" in os_info else f"{os_info.get('distribution') or os_info['system']} (kernel {os_info.get('kernel', os_info['release'])})"
	commit = (machine["repository"]["commit"] or "unknown")[:10]
	modified = " plus uncommitted changes" if machine["repository"]["modified"] else ""
	kind = machine.get("pinned_cpu_kind") or {}
	pinned = f"pinned to logical CPU {protocol['cpu']} ({kind.get('kind', 'unknown')} core {kind.get('core', '?')})" if protocol["cpu"] is not None else "not pinned"
	counters = "Hardware counters: perf, see raw.json." if protocol.get("counters") else "No hardware counters on this host."

	lines = [
		f"# Benchmark run {raw['label']}",
		"",
		f"Recorded {machine['recorded_utc']} on {cpu.get('model') or cpu.get('model_name')} (`-march=native` means {target}) with {cpu['logical_cpus']} logical CPUs under {os_text}. Harness at `{commit}`{modified}.",
		"",
		f"Protocol: {pinned} at high priority. {protocol['warmup']} warmup rounds and {protocol['reps']} timed rounds. Each round runs every program once in a new random order (seed {protocol['seed']}). Times are whole-process wall clock with process start included (see `startup`). Medians come with 95% order-statistic intervals. \"vs fastest\" is the ratio of medians with a 95% bootstrap interval. p comes from a two-sided Mann-Whitney U test against the fastest variant (Holm-adjusted within each benchmark). Compile time is the median of {protocol['compile_reps']} clean builds (unpinned). {counters}",
		"",
		"| tool | version | pinned |",
		"| --- | --- | --- |",
	]
	for name, tool in tools.items():
		version = tool["version"].splitlines()[0]
		lines.append(f"| {name} | {version} | {'yes' if tool['pinned'] else 'no'} |")

	lines += [
		"",
		"## Summary",
		"",
		f"Median time over the fastest variant's median. **1.00** is the fastest. \"tie\" means no significant difference from it (Holm p >= {ALPHA}).",
		"",
		"| benchmark | input | " + " | ".join(variants) + " |",
		"| --- | --- | " + " | ".join("---" for _ in variants) + " |",
	]
	ratios = {v: [] for v in variants}
	for (benchmark, category), rows in sorted(analyzed.items()):
		if benchmark == "startup":
			continue
		cells = []
		by_variant = {r["entry"]["variant"]: r for r in rows}
		for v in variants:
			row = by_variant.get(v)
			if row is None:
				cells.append("-")
				continue
			ratios[v].append(row["ratio"])
			if row["p"] is None:
				cells.append("**1.00**")
			else:
				cells.append(f"{row['ratio']:.2f}" + (" tie" if row["p"] >= ALPHA else ""))
		name = benchmark if category == "idiomatic" else f"{benchmark} ({category})"
		lines.append(f"| {name} | {program_input(rows[0]['entry'])} | " + " | ".join(cells) + " |")
	lines.append("| geometric mean | | " + " | ".join(f"{stats.geomean(ratios[v]):.2f}" if ratios[v] else "-" for v in variants) + " |")

	for (benchmark, category), rows in sorted(analyzed.items(), key=lambda item: (item[0][0] == "startup", item[0])):
		first = rows[0]["entry"]
		title = benchmark if category == "idiomatic" else f"{benchmark} ({category})"
		args = program_input(first) or "(none)"
		lines += [
			"",
			f"## {title}",
			"",
			f"Input `{args}`. Output sha256 `{(first.get('output_sha256') or '-')[:16]}`.",
			"",
			"| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |",
			"| --- | --- | --- | --- | --- | --- | --- | --- | --- |",
		]
		for row in rows:
			s = row["summary"]
			entry = row["entry"]
			ci = span(s["ci_low"], s["ci_high"]) if s["ci_exact"] else "n too small"
			if row["p"] is None:
				versus, p = "fastest", ""
			else:
				low, high = row["ratio_ci"]
				versus = f"{(row['ratio'] - 1) * 100:+.1f}% ({(low - 1) * 100:+.1f}, {(high - 1) * 100:+.1f})"
				p = pvalue(row["p"])
			peaks = [x["peak_memory_bytes"] for x in entry["samples"] if x.get("peak_memory_bytes")]
			compile_ns = sorted(entry["compile_ns"])[len(entry["compile_ns"]) // 2] if entry["compile_ns"] else None
			lines.append(
				f"| {entry['variant']} | {seconds(s['median'])} | {ci} | {versus} | {p} | {size(max(peaks), 'MiB') if peaks else '-'} | "
				f"{size(entry['binary_bytes'], 'KiB')} | {size(entry.get('stripped_bytes'), 'KiB')} | {seconds(compile_ns) if compile_ns else '-'} |"
			)
	lines.append("")
	return "\n".join(lines)
