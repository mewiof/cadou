// add every element into *total. rustc marks `&mut` and `&` parameters noalias
// (raw pointers get no such promise)

const COUNT: usize = 1 << 20;
const ROUNDS: usize = 1000;

#[inline(never)]
fn accumulate(total: &mut u32, values: &[u32]) {
	for value in values {
		*total = total.wrapping_add(*value);
	}
}

#[inline(never)]
unsafe fn accumulate_raw(total: *mut u32, values: *const u32, count: usize) {
	for i in 0..count {
		unsafe { *total = (*total).wrapping_add(*values.add(i)) };
	}
}

fn main() {
	let mode = std::env::args().nth(1).unwrap_or_default();
	if mode != "reference" && mode != "pointer" {
		eprintln!("usage: accumulate reference|pointer");
		std::process::exit(1);
	}
	let values: Vec<u32> = (0..COUNT).map(|i| (i & 7) as u32).collect();
	let mut total = 0u32;
	for _ in 0..ROUNDS {
		if mode == "reference" {
			accumulate(&mut total, &values);
		} else {
			unsafe { accumulate_raw(&mut total, values.as_ptr(), values.len()) };
		}
	}
	println!("{total}");
}
