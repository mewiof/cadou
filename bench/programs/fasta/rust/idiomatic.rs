// fasta: one repeated and two random DNA sequences in 60-character lines

use std::io::{self, BufWriter, Write};

const LINE_LENGTH: usize = 60;

const ALU: &[u8] = b"GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGG\
GAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGA\
CCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAAT\
ACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCA\
GCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGG\
AGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCC\
AGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAA";

struct Acid {
	symbol: u8,
	probability: f64,
}

struct Random {
	seed: u32,
}

impl Random {
	const IM: u32 = 139968;
	const IA: u32 = 3877;
	const IC: u32 = 29573;

	fn next(&mut self, max: f64) -> f64 {
		self.seed = (self.seed * Self::IA + Self::IC) % Self::IM;
		max * self.seed as f64 / Self::IM as f64
	}
}

fn repeat_fasta(out: &mut impl Write, header: &str, sequence: &[u8], mut n: usize) -> io::Result<()> {
	out.write_all(header.as_bytes())?;
	let mut line = [0u8; LINE_LENGTH + 1];
	let mut position = 0;
	while n > 0 {
		let line_length = n.min(LINE_LENGTH);
		for c in &mut line[..line_length] {
			*c = sequence[position];
			position += 1;
			if position == sequence.len() {
				position = 0;
			}
		}
		line[line_length] = b'\n';
		out.write_all(&line[..=line_length])?;
		n -= line_length;
	}
	Ok(())
}

fn random_fasta(out: &mut impl Write, header: &str, acids: &mut [Acid], mut n: usize, random: &mut Random) -> io::Result<()> {
	out.write_all(header.as_bytes())?;
	let mut cumulative = 0.0;
	for acid in acids.iter_mut() {
		cumulative += acid.probability;
		acid.probability = cumulative;
	}

	let mut line = [0u8; LINE_LENGTH + 1];
	while n > 0 {
		let line_length = n.min(LINE_LENGTH);
		for c in &mut line[..line_length] {
			let r = random.next(1.0);
			let mut k = 0;
			while k + 1 < acids.len() && r >= acids[k].probability {
				k += 1;
			}
			*c = acids[k].symbol;
		}
		line[line_length] = b'\n';
		out.write_all(&line[..=line_length])?;
		n -= line_length;
	}
	Ok(())
}

fn acid(symbol: u8, probability: f64) -> Acid {
	Acid { symbol, probability }
}

fn main() -> io::Result<()> {
	let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1000);
	let mut iub = [
		acid(b'a', 0.27), acid(b'c', 0.12), acid(b'g', 0.12), acid(b't', 0.27),
		acid(b'B', 0.02), acid(b'D', 0.02), acid(b'H', 0.02), acid(b'K', 0.02),
		acid(b'M', 0.02), acid(b'N', 0.02), acid(b'R', 0.02), acid(b'S', 0.02),
		acid(b'V', 0.02), acid(b'W', 0.02), acid(b'Y', 0.02),
	];
	let mut homo_sapiens = [
		acid(b'a', 0.3029549426680),
		acid(b'c', 0.1979883004921),
		acid(b'g', 0.1975473066391),
		acid(b't', 0.3015094502008),
	];

	// every fasta writes through a 64 KiB buffer
	let mut out = BufWriter::with_capacity(1 << 16, io::stdout().lock());
	let mut random = Random { seed: 42 };
	repeat_fasta(&mut out, ">ONE Homo sapiens alu\n", ALU, 2 * n)?;
	random_fasta(&mut out, ">TWO IUB ambiguity codes\n", &mut iub, 3 * n, &mut random)?;
	random_fasta(&mut out, ">THREE Homo sapiens frequency\n", &mut homo_sapiens, 5 * n, &mut random)?;
	out.flush()
}
