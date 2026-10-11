// k-nucleotide: frequencies of 1- and 2-nucleotide sequences and counts of five
// longer ones in fasta's third sequence (read from stdin)

use std::collections::HashMap;
use std::io::{self, BufRead};

fn code(c: u8) -> u8 {
	match c {
		b'A' | b'a' => 0,
		b'C' | b'c' => 1,
		b'G' | b'g' => 2,
		_ => 3,
	}
}

fn read_three() -> io::Result<Vec<u8>> {
	let mut input = io::stdin().lock();
	let mut line = Vec::new();
	loop {
		line.clear();
		if input.read_until(b'\n', &mut line)? == 0 {
			return Ok(Vec::new());
		}
		if line.starts_with(b">THREE") {
			break;
		}
	}
	let mut sequence = Vec::new();
	loop {
		line.clear();
		if input.read_until(b'\n', &mut line)? == 0 || line[0] == b'>' {
			break;
		}
		let bases = line.strip_suffix(b"\n").unwrap_or(&line);
		sequence.extend(bases.iter().map(|&c| code(c)));
	}
	Ok(sequence)
}

// keys pack two bits per nucleotide (A/C/G/T = 0/1/2/3) with the first one highest
fn count(sequence: &[u8], k: usize) -> HashMap<u64, u32> {
	let mut counts = HashMap::new();
	let mask = (1u64 << (2 * k)) - 1;
	let mut key = 0u64;
	for (i, &c) in sequence.iter().enumerate() {
		key = ((key << 2) | c as u64) & mask;
		if i + 1 >= k {
			*counts.entry(key).or_insert(0) += 1;
		}
	}
	counts
}

fn decode(key: u64, k: usize) -> String {
	(0..k).map(|i| b"ACGT"[((key >> (2 * (k - 1 - i))) & 3) as usize] as char).collect()
}

fn print_frequencies(sequence: &[u8], k: usize) {
	let mut entries: Vec<(u64, u32)> = count(sequence, k).into_iter().collect();
	entries.sort_unstable_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
	let total = (sequence.len() - k + 1) as f64;
	for (key, count) in entries {
		println!("{} {:.3}", decode(key, k), 100.0 * count as f64 / total);
	}
	println!();
}

fn print_count(sequence: &[u8], pattern: &str) {
	let counts = count(sequence, pattern.len());
	let key = pattern.bytes().fold(0u64, |key, c| (key << 2) | code(c) as u64);
	println!("{}\t{}", counts.get(&key).copied().unwrap_or(0), pattern);
}

fn main() -> io::Result<()> {
	let sequence = read_three()?;
	print_frequencies(&sequence, 1);
	print_frequencies(&sequence, 2);
	for pattern in ["GGT", "GGTA", "GGTATT", "GGTATTTTAATT", "GGTATTTTAATTTATAGT"] {
		print_count(&sequence, pattern);
	}
	Ok(())
}
