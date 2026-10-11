#!/usr/bin/env python3
# reference for the sort benchmark (wrote expected-1000.txt)
#
#   python bench/programs/sort/reference.py 1000

import sys

MASK = (1 << 64) - 1


def splitmix64(state):
	while True:
		state = (state + 0x9E3779B97F4A7C15) & MASK
		z = state
		z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK
		z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK
		yield z ^ (z >> 31)


def main(n):
	random = splitmix64(0)
	values = sorted(next(random) for _ in range(n))
	checksum = sum(value * (i + 1) for i, value in enumerate(values)) & MASK
	print(f"{values[0]} {values[n // 2]} {values[-1]} {checksum}")


if __name__ == "__main__":
	main(int(sys.argv[1]) if len(sys.argv) > 1 else 1000)
