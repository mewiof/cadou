#!/usr/bin/env python3
# which parameter attributes let LLVM keep the sum of the call kernel in a register.
# clang's unoptimized IR of kernels.c gets attributes added to the sum pointer and
# goes through the pinned LLVM's opt -O3
#
#   python bench/experiments/noalias2/attributes.py

import pathlib
import re
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent.parent / "harness"))

import toolchains

CASES = (
	("Apply", ""),
	("Apply", "dereferenceable(8) align 8"),
	("ApplyRestrict", ""),
	("ApplyRestrict", "align 8"),
	("ApplyRestrict", "dereferenceable(8)"),
	("ApplyRestrict", "dereferenceable(8) align 8"),
)


def main():
	llvm = toolchains.pinned_bin("llvm", toolchains.load_pins(), toolchains.host_key(), True)
	with tempfile.TemporaryDirectory() as scratch:
		scratch = pathlib.Path(scratch)
		raw = scratch / "kernels.ll"
		clang = [str(llvm / f"clang{toolchains.EXE}"), "-std=c17", "-O3", "-march=native", "-ffp-contract=off", "-S", "-emit-llvm", "-Xclang", "-disable-llvm-passes"]
		subprocess.run(clang + [str(HERE / "kernels.c"), "-o", str(raw)], check=True)
		text = raw.read_text()
		print("| function | added to the sum pointer | sum kept in a register |")
		print("| --- | --- | --- |")
		for function, attributes in CASES:
			pattern = re.compile(r"(define internal void @" + function + r"\(ptr )([^%]*)(%\w+)")
			patched, count = pattern.subn(lambda m: m.group(1) + m.group(2) + attributes + " " + m.group(3), text, count=1)
			if count != 1:
				raise SystemExit(f"no definition of {function}")
			source = scratch / "patched.ll"
			optimized = scratch / "optimized.ll"
			source.write_text(patched)
			subprocess.run([str(llvm / f"opt{toolchains.EXE}"), "-O3", str(source), "-S", "-o", str(optimized)], check=True)
			body = re.search(r"define internal void @" + function + r"\(.*?\n}\n", optimized.read_text(), re.S).group(0)
			# LLVM names the value it keeps for the whole loop *.promoted
			print(f"| {function} | {attributes or '(nothing)'} | {'yes' if 'promoted' in body else 'no'} |")


if __name__ == "__main__":
	main()
