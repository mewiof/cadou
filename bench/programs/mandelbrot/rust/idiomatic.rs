// mandelbrot: up to 50 iterations per point for re in -1.5..0.5 and im in -1..1 (binary PBM out)

use std::io::Write;

const ITERATIONS: u32 = 50;
const LIMIT_SQUARED: f64 = 4.0;

fn main() -> std::io::Result<()> {
	let size: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(200);
	let row_bytes = size.div_ceil(8);
	let mut bitmap = vec![0u8; row_bytes * size];

	for (y, row) in bitmap.chunks_exact_mut(row_bytes).enumerate() {
		let ci = 2.0 * y as f64 / size as f64 - 1.0;
		for x in 0..size {
			let cr = 2.0 * x as f64 / size as f64 - 1.5;
			let (mut zr, mut zi, mut tr, mut ti) = (0.0, 0.0, 0.0, 0.0);
			let mut i = 0;
			while i < ITERATIONS && tr + ti <= LIMIT_SQUARED {
				zi = 2.0 * zr * zi + ci;
				zr = tr - ti + cr;
				tr = zr * zr;
				ti = zi * zi;
				i += 1;
			}
			if tr + ti <= LIMIT_SQUARED {
				row[x / 8] |= 0x80 >> (x % 8);
			}
		}
	}

	let mut out = std::io::stdout().lock();
	write!(out, "P4\n{size} {size}\n")?;
	out.write_all(&bitmap)
}
