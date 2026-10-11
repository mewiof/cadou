// fannkuch-redux: count prefix reversals over all permutations of 0..n-1

const MAX_N: usize = 16;

fn main() {
	let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(7);
	if !(1..=MAX_N).contains(&n) {
		eprintln!("n must be in 1..{MAX_N}");
		std::process::exit(1);
	}

	let mut perm1 = [0usize; MAX_N];
	let mut count = [0usize; MAX_N];
	let (mut max_flips, mut checksum, mut perm_index) = (0, 0i32, 0u64);
	for (i, p) in perm1[..n].iter_mut().enumerate() {
		*p = i;
	}

	let mut r = n;
	loop {
		while r != 1 {
			count[r - 1] = r;
			r -= 1;
		}

		let mut perm = perm1;
		let mut flips = 0;
		let mut k = perm[0];
		while k != 0 {
			perm[..=k].reverse();
			flips += 1;
			k = perm[0];
		}
		max_flips = max_flips.max(flips);
		checksum += if perm_index % 2 == 0 { flips } else { -flips };

		// next permutation in counting order (rotates the first r + 1 elements)
		loop {
			if r == n {
				println!("{checksum}\nPfannkuchen({n}) = {max_flips}");
				return;
			}
			perm1[..=r].rotate_left(1);
			count[r] -= 1;
			if count[r] > 0 {
				break;
			}
			r += 1;
		}
		perm_index += 1;
	}
}
