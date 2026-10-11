// spectral-norm: power method on A(i, j) = 1 / ((i + j)(i + j + 1) / 2 + i + 1)

fn a(i: usize, j: usize) -> f64 {
	1.0 / ((i + j) * (i + j + 1) / 2 + i + 1) as f64
}

fn multiply_av(v: &[f64], out: &mut [f64]) {
	for (i, o) in out.iter_mut().enumerate() {
		*o = v.iter().enumerate().map(|(j, x)| a(i, j) * x).sum();
	}
}

fn multiply_atv(v: &[f64], out: &mut [f64]) {
	for (i, o) in out.iter_mut().enumerate() {
		*o = v.iter().enumerate().map(|(j, x)| a(j, i) * x).sum();
	}
}

fn multiply_atav(v: &[f64], out: &mut [f64], scratch: &mut [f64]) {
	multiply_av(v, scratch);
	multiply_atv(scratch, out);
}

fn main() {
	let n: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(100);
	let mut u = vec![1.0; n];
	let mut v = vec![0.0; n];
	let mut scratch = vec![0.0; n];
	for _ in 0..10 {
		multiply_atav(&u, &mut v, &mut scratch);
		multiply_atav(&v, &mut u, &mut scratch);
	}

	let vbv: f64 = u.iter().zip(&v).map(|(a, b)| a * b).sum();
	let vv: f64 = v.iter().map(|x| x * x).sum();
	println!("{:.9}", (vbv / vv).sqrt());
}
