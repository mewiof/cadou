#!/usr/bin/env python3
# stack traffic of the n-body step loops that run.py built: loads that read a value
# from the previous step and loads wider than the store they read (store-to-load
# forwarding cannot serve those)
#
#   python bench/experiments/gaps/split_loads.py

import pathlib
import re
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2] / "harness"))

import suite
import toolchains

BUILD = suite.BUILD / "experiments" / "gaps" / "n-body"
LINE = re.compile(r"^\s*([0-9a-f]+):\s+(\S+)\s*(.*)$")
STACK = re.compile(r"(-?0x[0-9a-f]+|-?\d+)?\((%r[sb]p)\)")
EIGHT_BYTES = {"vmovddup", "vbroadcastsd", "vmovq", "vmovlpd", "vmovhpd"}
SIXTEEN_BYTES = {"vinsertf128", "vpunpcklqdq", "vpunpckhqdq", "vunpcklpd", "vunpckhpd"}


def lanes(mnemonic, operands):
	if mnemonic.endswith("sd"):
		return 1
	return 4 if "%ymm" in operands else 2


def width(mnemonic, operands):
	if mnemonic in EIGHT_BYTES or mnemonic.endswith("sd"):
		return 8
	if mnemonic in SIXTEEN_BYTES:
		return 16
	return 32 if "%ymm" in operands else 16


def step_loop(objdump, binary):
	# the loop with all 10 sqrt lanes of a step
	text = subprocess.run([str(objdump), "-d", "--no-show-raw-insn", str(binary)], capture_output=True, text=True).stdout
	code = [(int(m.group(1), 16), m.group(2), m.group(3).split("#")[0].strip()) for m in map(LINE.match, text.splitlines()) if m]
	index = {address: i for i, (address, _, _) in enumerate(code)}
	for i, (address, mnemonic, operands) in enumerate(code):
		target = re.match(r"0x([0-9a-f]+)", operands) if mnemonic.startswith("j") else None
		if target and int(target.group(1), 16) in index and int(target.group(1), 16) < address:
			body = code[index[int(target.group(1), 16)]: i]
			if sum(lanes(m, o) for _, m, o in body if m.startswith("vsqrt")) == 10 and not any(m.startswith("j") for _, m, _ in body):
				return body
	return None


def accesses(body):
	found = []
	for position, (_, mnemonic, operands) in enumerate(body):
		match = STACK.search(operands)
		if not match:
			continue
		# AT&T order: a store names the memory last
		last = re.split(r",(?![^(]*\))", operands)[-1]
		store = STACK.search(last) is not None and mnemonic.startswith(("vmov", "mov"))
		found.append((position, store, int(match.group(1) or "0", 0), width(mnemonic, operands)))
	return found


def main():
	objdump = toolchains.pinned_bin("llvm", toolchains.load_pins(), toolchains.host_key(), True) / f"llvm-objdump{toolchains.EXE}"
	print("| case | stack loads | loads of the previous step | loads wider than their store | of those from the previous step |")
	print("| --- | --- | --- | --- | --- |")
	for binary in sorted(BUILD.glob(f"*{toolchains.EXE}")):
		body = step_loop(objdump, binary)
		if body is None:
			continue
		memory = accesses(body)
		loads = [a for a in memory if not a[1]]
		carried = split = split_carried = 0
		for position, _, offset, size in loads:
			overlapping = lambda a: a[1] and a[2] < offset + size and offset < a[2] + a[3]
			before = [a for a in memory if a[0] < position and overlapping(a)]
			# the last store before the load or else the last one of the previous step
			source = before[-1:] or [a for a in memory if a[0] > position and overlapping(a)][-1:]
			if not before and source:
				carried += 1
			if source and not (source[0][2] <= offset and offset + size <= source[0][2] + source[0][3]):
				split += 1
				split_carried += not before
		print(f"| {binary.stem} | {len(loads)} | {carried} | {split} | {split_carried} |")


if __name__ == "__main__":
	main()
