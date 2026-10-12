// integer semantics of spec sections 6.1 to 6.4. A value of any integer type travels
// as an i128. That holds the exact sum/difference of two 64-bit values and every
// product except some of two large u64 values

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IntType {
	pub signed: bool,
	pub bits: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arith {
	Add,
	Sub,
	Mul,
	Div,
	Rem,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
	Overflow,
	DivideByZero,
	ShiftRange,
}

impl IntType {
	pub fn min(self) -> i128 {
		if self.signed { -(1 << (self.bits - 1)) } else { 0 }
	}

	pub fn max(self) -> i128 {
		if self.signed { (1 << (self.bits - 1)) - 1 } else { (1 << self.bits) - 1 }
	}

	pub fn contains(self, value: i128) -> bool {
		self.min() <= value && value <= self.max()
	}

	// does every value of `other` fit in this type
	pub fn holds(self, other: IntType) -> bool {
		self.min() <= other.min() && other.max() <= self.max()
	}

	// the value modulo 2^bits in this type's range
	pub fn wrap(self, value: i128) -> i128 {
		let low = value & ((1 << self.bits) - 1);
		if low > self.max() { low - (1 << self.bits) } else { low }
	}

	pub fn clamp(self, value: i128) -> i128 {
		value.clamp(self.min(), self.max())
	}
}

// the exact sum/difference/product of a and b. None only when it leaves i128 (a
// product of two large u64 values and thus positive)
fn exact(op: Arith, a: i128, b: i128) -> Option<i128> {
	match op {
		Arith::Add => a.checked_add(b),
		Arith::Sub => a.checked_sub(b),
		Arith::Mul => a.checked_mul(b),
		Arith::Div | Arith::Rem => unreachable!("division has no wrapping/saturating/checked form"),
	}
}

// + - * / % as written: overflow traps (6.1) and so does division by zero (6.2)
pub fn trapping(op: Arith, ty: IntType, a: i128, b: i128) -> Result<i128, Fault> {
	let result = match op {
		Arith::Div | Arith::Rem if b == 0 => return Err(Fault::DivideByZero),
		// NOTE: MIN % -1 is 0 but traps like MIN / -1 (one check covers both)
		Arith::Rem if a == ty.min() && b == -1 && ty.signed => return Err(Fault::Overflow),
		Arith::Div => a / b,
		Arith::Rem => a % b,
		_ => exact(op, a, b).ok_or(Fault::Overflow)?,
	};
	if ty.contains(result) { Ok(result) } else { Err(Fault::Overflow) }
}

pub fn wrapping(op: Arith, ty: IntType, a: i128, b: i128) -> i128 {
	// i128 arithmetic wraps modulo 2^128 and that keeps the low bits right
	let result = match op {
		Arith::Add => a.wrapping_add(b),
		Arith::Sub => a.wrapping_sub(b),
		Arith::Mul => a.wrapping_mul(b),
		Arith::Div | Arith::Rem => unreachable!("division has no wrapping form"),
	};
	ty.wrap(result)
}

pub fn saturating(op: Arith, ty: IntType, a: i128, b: i128) -> i128 {
	exact(op, a, b).map_or(ty.max(), |result| ty.clamp(result))
}

pub fn checked(op: Arith, ty: IntType, a: i128, b: i128) -> Option<i128> {
	exact(op, a, b).filter(|&result| ty.contains(result))
}

pub fn neg(ty: IntType, a: i128) -> Result<i128, Fault> {
	if ty.contains(-a) { Ok(-a) } else { Err(Fault::Overflow) }
}

// the shift amount has the type of the shifted value. Bits shifted out are gone (6.3)
pub fn shift_left(ty: IntType, a: i128, amount: i128) -> Result<i128, Fault> {
	if amount < 0 || amount >= ty.bits as i128 {
		return Err(Fault::ShiftRange);
	}
	Ok(ty.wrap(a << amount))
}

// arithmetic for signed types (values are sign-extended) and logical for unsigned ones
pub fn shift_right(ty: IntType, a: i128, amount: i128) -> Result<i128, Fault> {
	if amount < 0 || amount >= ty.bits as i128 {
		return Err(Fault::ShiftRange);
	}
	Ok(a >> amount)
}

// the integer part (toward zero) when it fits. None for NaN and out of range (6.4)
pub fn float_to_int(ty: IntType, x: f64) -> Option<i128> {
	let whole = x.trunc();
	// powers of two are exact in f64 (the bounds of every integer type are)
	let high = 2f64.powi(ty.bits as i32 - ty.signed as i32);
	let low = if ty.signed { -high } else { 0.0 };
	(whole >= low && whole < high).then_some(whole as i128)
}

pub fn float_to_int_saturating(ty: IntType, x: f64) -> i128 {
	match float_to_int(ty, x) {
		Some(value) => value,
		None if x.is_nan() => 0,
		None if x < 0.0 => ty.min(),
		None => ty.max(),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const I8: IntType = IntType { signed: true, bits: 8 };
	const U8: IntType = IntType { signed: false, bits: 8 };
	const I64: IntType = IntType { signed: true, bits: 64 };
	const U64: IntType = IntType { signed: false, bits: 64 };

	// every pair of 8-bit operands against Rust's own i8 and u8 methods
	#[test]
	fn eight_bit_exhaustive() {
		for a in i8::MIN..=i8::MAX {
			for b in i8::MIN..=i8::MAX {
				let (x, y) = (a as i128, b as i128);
				assert_eq!(trapping(Arith::Add, I8, x, y).ok(), a.checked_add(b).map(i128::from));
				assert_eq!(trapping(Arith::Sub, I8, x, y).ok(), a.checked_sub(b).map(i128::from));
				assert_eq!(trapping(Arith::Mul, I8, x, y).ok(), a.checked_mul(b).map(i128::from));
				assert_eq!(trapping(Arith::Div, I8, x, y).ok(), a.checked_div(b).map(i128::from));
				assert_eq!(trapping(Arith::Rem, I8, x, y).ok(), a.checked_rem(b).map(i128::from));
				assert_eq!(wrapping(Arith::Add, I8, x, y), a.wrapping_add(b) as i128);
				assert_eq!(wrapping(Arith::Sub, I8, x, y), a.wrapping_sub(b) as i128);
				assert_eq!(wrapping(Arith::Mul, I8, x, y), a.wrapping_mul(b) as i128);
				assert_eq!(saturating(Arith::Add, I8, x, y), a.saturating_add(b) as i128);
				assert_eq!(saturating(Arith::Sub, I8, x, y), a.saturating_sub(b) as i128);
				assert_eq!(saturating(Arith::Mul, I8, x, y), a.saturating_mul(b) as i128);
				assert_eq!(checked(Arith::Mul, I8, x, y), a.checked_mul(b).map(i128::from));
				assert_eq!(I8.wrap(x & y), (a & b) as i128);
				assert_eq!(I8.wrap(x | y), (a | b) as i128);
				assert_eq!(I8.wrap(x ^ y), (a ^ b) as i128);
				let amount = (0..8).contains(&b).then_some(b as u32);
				assert_eq!(shift_left(I8, x, y).ok(), amount.map(|n| (a << n) as i128));
				assert_eq!(shift_right(I8, x, y).ok(), amount.map(|n| (a >> n) as i128));
			}
			assert_eq!(neg(I8, a as i128).ok(), a.checked_neg().map(i128::from));
			assert_eq!(I8.wrap(!(a as i128)), !a as i128);
		}
		for a in u8::MIN..=u8::MAX {
			for b in u8::MIN..=u8::MAX {
				let (x, y) = (a as i128, b as i128);
				assert_eq!(trapping(Arith::Add, U8, x, y).ok(), a.checked_add(b).map(i128::from));
				assert_eq!(trapping(Arith::Sub, U8, x, y).ok(), a.checked_sub(b).map(i128::from));
				assert_eq!(trapping(Arith::Mul, U8, x, y).ok(), a.checked_mul(b).map(i128::from));
				assert_eq!(trapping(Arith::Div, U8, x, y).ok(), a.checked_div(b).map(i128::from));
				assert_eq!(trapping(Arith::Rem, U8, x, y).ok(), a.checked_rem(b).map(i128::from));
				assert_eq!(wrapping(Arith::Sub, U8, x, y), a.wrapping_sub(b) as i128);
				assert_eq!(wrapping(Arith::Mul, U8, x, y), a.wrapping_mul(b) as i128);
				assert_eq!(saturating(Arith::Sub, U8, x, y), a.saturating_sub(b) as i128);
				assert_eq!(saturating(Arith::Mul, U8, x, y), a.saturating_mul(b) as i128);
				assert_eq!(U8.wrap(!x), !a as i128);
				let amount = (b < 8).then_some(b as u32);
				assert_eq!(shift_left(U8, x, y).ok(), amount.map(|n| (a << n) as i128));
				assert_eq!(shift_right(U8, x, y).ok(), amount.map(|n| (a >> n) as i128));
			}
		}
	}

	#[test]
	fn sixty_four_bit_edges() {
		let edges = [i64::MIN, i64::MIN + 1, -3, -1, 0, 1, 2, 3, i64::MAX - 1, i64::MAX, 1 << 32, -(1 << 32), 3037000499, 3037000500];
		for &a in &edges {
			for &b in &edges {
				let (x, y) = (a as i128, b as i128);
				assert_eq!(trapping(Arith::Mul, I64, x, y).ok(), a.checked_mul(b).map(i128::from));
				assert_eq!(trapping(Arith::Div, I64, x, y).ok(), a.checked_div(b).map(i128::from));
				assert_eq!(trapping(Arith::Rem, I64, x, y).ok(), a.checked_rem(b).map(i128::from));
				assert_eq!(wrapping(Arith::Mul, I64, x, y), a.wrapping_mul(b) as i128);
				assert_eq!(saturating(Arith::Mul, I64, x, y), a.saturating_mul(b) as i128);
				let (p, q) = (a as u64, b as u64);
				let (x, y) = (p as i128, q as i128);
				assert_eq!(trapping(Arith::Add, U64, x, y).ok(), p.checked_add(q).map(i128::from));
				assert_eq!(trapping(Arith::Mul, U64, x, y).ok(), p.checked_mul(q).map(i128::from));
				assert_eq!(wrapping(Arith::Mul, U64, x, y), p.wrapping_mul(q) as i128);
				assert_eq!(saturating(Arith::Mul, U64, x, y), p.saturating_mul(q) as i128);
				assert_eq!(saturating(Arith::Sub, U64, x, y), p.saturating_sub(q) as i128);
			}
		}
	}

	#[test]
	fn holds() {
		assert!(IntType { signed: true, bits: 16 }.holds(U8));
		assert!(!U64.holds(I8));
		assert!(!I64.holds(U64));
		assert!(I64.holds(I64));
	}

	#[test]
	fn floats_to_integers() {
		assert_eq!(float_to_int(I8, 127.9), Some(127));
		assert_eq!(float_to_int(I8, -128.9), Some(-128));
		assert_eq!(float_to_int(I8, 128.0), None);
		assert_eq!(float_to_int(U8, -0.9), Some(0));
		assert_eq!(float_to_int(U8, -1.0), None);
		assert_eq!(float_to_int(I64, 9223372036854775807.0), None);
		assert_eq!(float_to_int(I64, -9223372036854775808.0), Some(i64::MIN as i128));
		assert_eq!(float_to_int(U64, 18446744073709549568.0), Some(18446744073709549568));
		assert_eq!(float_to_int(I8, f64::NAN), None);
		assert_eq!(float_to_int_saturating(I8, f64::NAN), 0);
		assert_eq!(float_to_int_saturating(I8, -1e9), -128);
		assert_eq!(float_to_int_saturating(U8, f64::INFINITY), 255);
		for x in [-1e30, -300.5, -1.5, -0.0, 0.7, 255.9, 1e30, f64::NEG_INFINITY, f64::NAN] {
			assert_eq!(float_to_int_saturating(U8, x), (x as u8) as i128);
			assert_eq!(float_to_int_saturating(I8, x), (x as i8) as i128);
		}
	}
}
