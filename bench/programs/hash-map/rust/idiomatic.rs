// hash-map: insert n random 64-bit keys. Look up each of them plus n absent keys
// from a second stream

use std::collections::HashMap;

// splitmix64 (the same streams in every language)
struct SplitMix64(u64);

impl SplitMix64 {
	fn next(&mut self) -> u64 {
		self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
		let mut z = self.0;
		z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
		z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
		z ^ (z >> 31)
	}
}

fn main() {
	let n: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1000);
	let mut map = HashMap::new();
	let mut keys = SplitMix64(0);
	for i in 0..n {
		map.insert(keys.next(), i);
	}

	let (mut again, mut others) = (SplitMix64(0), SplitMix64(1));
	let (mut hits, mut sum) = (0u64, 0u64);
	for _ in 0..n {
		if let Some(&value) = map.get(&again.next()) {
			hits += 1;
			sum += value;
		}
		if let Some(&value) = map.get(&others.next()) {
			hits += 1;
			sum += value;
		}
	}
	println!("{hits} {sum}");
}
