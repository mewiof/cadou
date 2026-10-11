#!/usr/bin/env python3
# reference for the hash-map benchmark (wrote expected-1000.txt)
#
#   python bench/programs/hash-map/reference.py 1000

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
	keys = splitmix64(0)
	table = {}
	for i in range(n):
		table[next(keys)] = i
	again, others = splitmix64(0), splitmix64(1)
	hits = total = 0
	for _ in range(n):
		for key in (next(again), next(others)):
			if key in table:
				hits += 1
				total += table[key]
	print(f"{hits} {total & MASK}")


if __name__ == "__main__":
	main(int(sys.argv[1]) if len(sys.argv) > 1 else 1000)
