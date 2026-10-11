// sort: n random 64-bit integers with the standard library sort. Prints the
// smallest/median/largest value and a checksum that depends on the order

// splitmix64 (the same stream in every language)
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
	let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1000);
	if n == 0 {
		std::process::exit(1);
	}
	let mut random = SplitMix64(0);
	let mut values: Vec<u64> = (0..n).map(|_| random.next()).collect();

	values.sort_unstable();

	let checksum = values.iter().enumerate().fold(0u64, |sum, (i, &value)| sum.wrapping_add(value.wrapping_mul(i as u64 + 1)));
	println!("{} {} {} {}", values[0], values[n / 2], values[n - 1], checksum);
}
