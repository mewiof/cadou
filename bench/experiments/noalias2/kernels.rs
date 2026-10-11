// three loops where a run-time overlap check cannot stand in for a no-alias promise.
// rustc marks `&mut` and `&` parameters noalias (raw pointers get no such promise).
// black_box hides which function gets called (direct calls let LLVM specialize a
// kernel to its only caller)
//
//   kernels fields|arrays|call pointer|reference

use std::hint::black_box;

// ================================================================
// FIELDS
// ================================================================

const PIXEL_COUNT: usize = 16384;
const FIELD_ROUNDS: usize = 100000;

struct Image {
	pixels: *mut u8,
	count: usize,
}

// a byte store through image.pixels may land in *image unless image is a reference
#[inline(never)]
unsafe fn brighten_raw(image: *mut Image, amount: u8) {
	let mut i = 0;
	while i < unsafe { (*image).count } {
		unsafe {
			let pixel = (*image).pixels.add(i);
			*pixel = (*pixel).wrapping_add(amount);
		}
		i += 1;
	}
}

#[inline(never)]
fn brighten(image: &mut Image, amount: u8) {
	let mut i = 0;
	while i < image.count {
		unsafe {
			let pixel = image.pixels.add(i);
			*pixel = (*pixel).wrapping_add(amount);
		}
		i += 1;
	}
}

fn run_fields(reference: bool) -> u64 {
	let mut buffer: Vec<u8> = (0..PIXEL_COUNT).map(|i| (i * 7) as u8).collect();
	let mut image = Image { pixels: buffer.as_mut_ptr(), count: buffer.len() };
	let kernel: fn(&mut Image, u8) = black_box(brighten);
	let kernel_raw: unsafe fn(*mut Image, u8) = black_box(brighten_raw);
	for round in 0..FIELD_ROUNDS {
		let amount = (round & 15) as u8;
		if reference {
			kernel(&mut image, amount);
		} else {
			unsafe { kernel_raw(&mut image, amount) };
		}
	}
	buffer.iter().enumerate().map(|(i, &p)| p as u64 * (i as u64 + 1)).sum()
}

// ================================================================
// ARRAYS
// ================================================================

const PARTICLE_COUNT: usize = 1024;
const ARRAY_ROUNDS: usize = 1000000;

#[inline(never)]
unsafe fn advance_raw(x: *mut f32, y: *mut f32, z: *mut f32, vx: *const f32, vy: *const f32, vz: *const f32, dt: f32, count: usize) {
	for i in 0..count {
		unsafe {
			*x.add(i) += *vx.add(i) * dt;
			*y.add(i) += *vy.add(i) * dt;
			*z.add(i) += *vz.add(i) * dt;
		}
	}
}

#[inline(never)]
fn advance(x: &mut [f32], y: &mut [f32], z: &mut [f32], vx: &[f32], vy: &[f32], vz: &[f32], dt: f32) {
	// equal lengths up front let the bounds checks go
	let count = x.len();
	let (y, z, vx, vy, vz) = (&mut y[..count], &mut z[..count], &vx[..count], &vy[..count], &vz[..count]);
	for i in 0..count {
		x[i] += vx[i] * dt;
		y[i] += vy[i] * dt;
		z[i] += vz[i] * dt;
	}
}

fn run_arrays(reference: bool) -> f64 {
	let mut data = vec![0.0f32; 6 * PARTICLE_COUNT];
	for i in 0..PARTICLE_COUNT {
		data[i] = i as f32;
		data[PARTICLE_COUNT + i] = (2 * i) as f32;
		data[2 * PARTICLE_COUNT + i] = (3 * i) as f32;
		data[3 * PARTICLE_COUNT + i] = (i % 7) as f32 - 3.0;
		data[4 * PARTICLE_COUNT + i] = (i % 5) as f32 - 2.0;
		data[5 * PARTICLE_COUNT + i] = (i % 3) as f32 - 1.0;
	}
	let kernel: fn(&mut [f32], &mut [f32], &mut [f32], &[f32], &[f32], &[f32], f32) = black_box(advance);
	let kernel_raw: unsafe fn(*mut f32, *mut f32, *mut f32, *const f32, *const f32, *const f32, f32, usize) = black_box(advance_raw);
	for _ in 0..ARRAY_ROUNDS {
		if reference {
			let (x, rest) = data.split_at_mut(PARTICLE_COUNT);
			let (y, rest) = rest.split_at_mut(PARTICLE_COUNT);
			let (z, rest) = rest.split_at_mut(PARTICLE_COUNT);
			let (vx, rest) = rest.split_at(PARTICLE_COUNT);
			let (vy, vz) = rest.split_at(PARTICLE_COUNT);
			kernel(x, y, z, vx, vy, vz, 0.001);
		} else {
			let p = data.as_mut_ptr();
			unsafe {
				kernel_raw(p, p.add(PARTICLE_COUNT), p.add(2 * PARTICLE_COUNT), p.add(3 * PARTICLE_COUNT), p.add(4 * PARTICLE_COUNT), p.add(5 * PARTICLE_COUNT), 0.001, PARTICLE_COUNT)
			};
		}
	}
	let mut sum = 0.0f64;
	for i in 0..PARTICLE_COUNT {
		sum += data[i] as f64 + data[PARTICLE_COUNT + i] as f64 + data[2 * PARTICLE_COUNT + i] as f64;
	}
	sum
}

// ================================================================
// CALL
// ================================================================

const VALUE_COUNT: usize = 4096;
const CALL_ROUNDS: usize = 25000;

fn square(value: u32) -> u32 {
	value.wrapping_mul(value)
}

// the callee is unknown. Through a raw pointer it may read or write *total
#[inline(never)]
unsafe fn apply_raw(total: *mut u64, values: *const u32, count: usize, transform: fn(u32) -> u32) {
	for i in 0..count {
		unsafe { *total += transform(*values.add(i)) as u64 };
	}
}

#[inline(never)]
fn apply(total: &mut u64, values: &[u32], transform: fn(u32) -> u32) {
	for i in 0..values.len() {
		*total += transform(values[i]) as u64;
	}
}

fn run_call(reference: bool) -> u64 {
	let values: Vec<u32> = (0..VALUE_COUNT).map(|i| (i as u32).wrapping_mul(2654435761)).collect();
	let transform: fn(u32) -> u32 = black_box(square);
	let kernel: fn(&mut u64, &[u32], fn(u32) -> u32) = black_box(apply);
	let kernel_raw: unsafe fn(*mut u64, *const u32, usize, fn(u32) -> u32) = black_box(apply_raw);
	let mut total = 0u64;
	for _ in 0..CALL_ROUNDS {
		if reference {
			kernel(&mut total, &values, transform);
		} else {
			unsafe { kernel_raw(&mut total, values.as_ptr(), values.len(), transform) };
		}
	}
	total
}

fn main() {
	let args: Vec<String> = std::env::args().collect();
	let kernel = args.get(1).map(String::as_str).unwrap_or("");
	let mode = args.get(2).map(String::as_str).unwrap_or("");
	if mode != "pointer" && mode != "reference" {
		eprintln!("usage: kernels fields|arrays|call pointer|reference");
		std::process::exit(1);
	}
	let reference = mode == "reference";
	match kernel {
		"fields" => println!("{}", run_fields(reference)),
		"arrays" => println!("{:.3}", run_arrays(reference)),
		"call" => println!("{}", run_call(reference)),
		_ => {
			eprintln!("usage: kernels fields|arrays|call pointer|reference");
			std::process::exit(1);
		}
	}
}
