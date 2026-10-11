# inputs that programs read from stdin. Generated once into bench/build/inputs/
#
#   ("fasta", "N")   the output of the fasta benchmark for N (k-nucleotide reads it)

import hashlib
import sys

from suite import BUILD

LINE_LENGTH = 60
IM = 139968
IA = 3877
IC = 29573

ALU = (
	"GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGG"
	"GAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGA"
	"CCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAAT"
	"ACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCA"
	"GCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGG"
	"AGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCC"
	"AGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAA"
)

IUB = (
	("a", 0.27), ("c", 0.12), ("g", 0.12), ("t", 0.27),
	("B", 0.02), ("D", 0.02), ("H", 0.02), ("K", 0.02),
	("M", 0.02), ("N", 0.02), ("R", 0.02), ("S", 0.02),
	("V", 0.02), ("W", 0.02), ("Y", 0.02),
)

HOMO_SAPIENS = (
	("a", 0.3029549426680),
	("c", 0.1979883004921),
	("g", 0.1975473066391),
	("t", 0.3015094502008),
)


def repeat(text, count):
	whole, rest = divmod(count, len(text))
	return text * whole + text[:rest]


def lines(sequence):
	return "".join(sequence[i:i + LINE_LENGTH] + "\n" for i in range(0, len(sequence), LINE_LENGTH))


def random_sequence(acids, count, seed):
	# the generator runs through all IM states before it repeats (full period). One
	# period of symbols covers any length
	cumulative = []
	total = 0.0
	for _, probability in acids:
		total += probability
		cumulative.append(total)
	symbols = []
	states = []
	state = seed
	for _ in range(min(count, IM)):
		state = (state * IA + IC) % IM
		r = state / IM
		k = 0
		while k + 1 < len(acids) and r >= cumulative[k]:
			k += 1
		symbols.append(acids[k][0])
		states.append(state)
	final = states[(count - 1) % IM] if count else seed
	return repeat("".join(symbols), count), final


def fasta(n):
	n = int(n)
	iub, seed = random_sequence(IUB, 3 * n, 42)
	homo_sapiens, _ = random_sequence(HOMO_SAPIENS, 5 * n, seed)
	parts = [
		">ONE Homo sapiens alu\n", lines(repeat(ALU, 2 * n)),
		">TWO IUB ambiguity codes\n", lines(iub),
		">THREE Homo sapiens frequency\n", lines(homo_sapiens),
	]
	return "".join(parts).encode("ascii")


GENERATORS = {"fasta": fasta}


def sha256_of(path):
	digest = hashlib.sha256()
	with open(path, "rb") as f:
		for chunk in iter(lambda: f.read(1 << 20), b""):
			digest.update(chunk)
	return digest.hexdigest()


def materialize(spec, sha256=None):
	# the file for spec (written on first use). A recorded sha256 must match
	name, *args = spec
	if name not in GENERATORS:
		raise SystemExit(f"unknown input generator {name!r}")
	path = BUILD / "inputs" / f"{'-'.join(spec)}.txt"
	if not path.exists():
		path.parent.mkdir(parents=True, exist_ok=True)
		partial = path.with_name(path.name + ".part")
		partial.write_bytes(GENERATORS[name](*args))
		partial.replace(path)
		print(f"generated input {' '.join(spec)} with sha256 {sha256_of(path)}", file=sys.stderr, flush=True)
	if sha256 and sha256_of(path) != sha256:
		raise SystemExit(f"input {' '.join(spec)} has sha256 {sha256_of(path)} instead of {sha256}")
	return path
