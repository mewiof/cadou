// the reference interpreter: walks the checked core and stops at the first trap. A
// program prints the same and ends with the same status on every host (ADR 0004)

use std::cmp::Ordering;
use std::ops::{Add, Div, Mul, Sub};

use crate::ast::{Bits, Compare, Conversion, Overflow};
use crate::diag::Span;
use crate::ops::{self, Arith, Fault, IntType};
use crate::typed::*;

// NOTE: compiled code runs out of stack at a depth that depends on the host. The
// interpreter stops at a fixed nesting of calls and expressions instead
pub const MAX_DEPTH: usize = 100_000;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
	Unit,
	Bool(bool),
	Int(i128),
	F32(f32),
	F64(f64),
	Struct(Box<[Value]>),
	Case(u32, Box<Value>),
}

pub struct Trap {
	pub span: Span,
	pub message: String,
	// the section of spec/core.md that defines the check
	pub section: &'static str,
}

pub enum End {
	Exit(i32),
	Trap(Trap),
}

pub struct Outcome {
	pub output: String,
	pub end: End,
}

pub fn run(program: &Program) -> Outcome {
	let mut machine = Machine { program, locals: Vec::new(), base: 0, depth: 0, output: String::new() };
	let end = match machine.call(program.main, Vec::new()) {
		Ok(Value::Int(status)) => End::Exit(status as i32),
		Ok(_) => End::Exit(0),
		Err(Exit::Trap(trap)) => End::Trap(trap),
		Err(_) => unreachable!("break/continue/return stay inside their function"),
	};
	Outcome { output: machine.output, end }
}

pub fn int_name(ty: IntType) -> String {
	format!("{}{}", if ty.signed { 'i' } else { 'u' }, ty.bits)
}

// the shortest decimal that reads back as the same value
pub fn float_text<T: Into<f64> + Copy + std::fmt::Debug>(x: T) -> String {
	let wide: f64 = x.into();
	if wide.is_nan() {
		"nan".to_string()
	} else if wide.is_infinite() {
		if wide > 0.0 { "inf" } else { "-inf" }.to_string()
	} else {
		format!("{x:?}")
	}
}

fn arith_symbol(op: Arith) -> &'static str {
	match op {
		Arith::Add => "+",
		Arith::Sub => "-",
		Arith::Mul => "*",
		Arith::Div => "/",
		Arith::Rem => "%",
	}
}

fn float_arith<T: Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T>>(op: Arith, a: T, b: T) -> T {
	match op {
		Arith::Add => a + b,
		Arith::Sub => a - b,
		Arith::Mul => a * b,
		Arith::Div => a / b,
		Arith::Rem => unreachable!("the checker allows no float remainder"),
	}
}

// how evaluation leaves an expression early
enum Exit {
	Break(LoopId),
	Continue(LoopId),
	Return(Value),
	Trap(Trap),
}

struct Machine<'p> {
	program: &'p Program,
	// the frames of all active calls (one slot per local)
	locals: Vec<Value>,
	base: usize,
	depth: usize,
	output: String,
}

impl Machine<'_> {
	fn trap<T>(&self, span: Span, section: &'static str, message: String) -> Result<T, Exit> {
		Err(Exit::Trap(Trap { span, message, section }))
	}

	fn int_type(&self, id: ExprId) -> IntType {
		self.program.types.int(self.program.expr(id).ty).expect("an integer operand")
	}

	fn call(&mut self, function: FnId, args: Vec<Value>) -> Result<Value, Exit> {
		let program = self.program;
		let function = &program.functions[function.0 as usize];
		let base = self.locals.len();
		self.locals.extend(args);
		self.locals.resize(base + function.frame as usize, Value::Unit);
		let caller = std::mem::replace(&mut self.base, base);
		let result = self.eval(function.body);
		self.base = caller;
		self.locals.truncate(base);
		match result {
			Err(Exit::Return(value)) => Ok(value),
			other => other,
		}
	}

	fn eval(&mut self, id: ExprId) -> Result<Value, Exit> {
		if self.depth == MAX_DEPTH {
			return self.trap(self.program.expr(id).span, "8.1", format!("calls and expressions nest deeper than {MAX_DEPTH} levels"));
		}
		self.depth += 1;
		let result = self.eval_inner(id);
		self.depth -= 1;
		result
	}

	fn eval_inner(&mut self, id: ExprId) -> Result<Value, Exit> {
		let program = self.program;
		let expr = program.expr(id);
		let span = expr.span;
		match &expr.kind {
			ExprKind::Int(value) => Ok(Value::Int(*value)),
			ExprKind::F32(x) => Ok(Value::F32(*x)),
			ExprKind::F64(x) => Ok(Value::F64(*x)),
			ExprKind::Bool(value) => Ok(Value::Bool(*value)),
			ExprKind::Unit => Ok(Value::Unit),
			ExprKind::Local(slot) => Ok(self.locals[self.base + *slot as usize].clone()),
			ExprKind::Call(function, args) => {
				let mut values = Vec::with_capacity(args.len());
				for &arg in args {
					values.push(self.eval(arg)?);
				}
				self.call(*function, values)
			}
			ExprKind::Let(slot, value) => {
				let value = self.eval(*value)?;
				self.locals[self.base + *slot as usize] = value;
				Ok(Value::Unit)
			}
			ExprKind::Set(place, value) => {
				// the place first and the value second (spec 5)
				let (slot, path) = self.place(*place);
				let value = self.eval(*value)?;
				let mut target = &mut self.locals[self.base + slot];
				for index in path {
					target = match target {
						Value::Struct(fields) => &mut fields[index as usize],
						_ => unreachable!("a field path leads through structs"),
					};
				}
				*target = value;
				Ok(Value::Unit)
			}
			ExprKind::Block(forms) => {
				let mut value = Value::Unit;
				for &form in forms {
					value = self.eval(form)?;
				}
				Ok(value)
			}
			ExprKind::If(condition, then, other) => {
				if self.eval(*condition)? == Value::Bool(true) {
					self.eval(*then)
				} else if let Some(other) = other {
					self.eval(*other)
				} else {
					Ok(Value::Unit)
				}
			}
			ExprKind::Loop(this, forms) => loop {
				for &form in forms {
					match self.eval(form) {
						Ok(_) => {}
						Err(Exit::Break(target)) if target == *this => return Ok(Value::Unit),
						Err(Exit::Continue(target)) if target == *this => break,
						Err(exit) => return Err(exit),
					}
				}
			},
			ExprKind::Break(target) => Err(Exit::Break(*target)),
			ExprKind::Continue(target) => Err(Exit::Continue(*target)),
			ExprKind::Return(value) => {
				let value = match value {
					Some(value) => self.eval(*value)?,
					None => Value::Unit,
				};
				Err(Exit::Return(value))
			}
			ExprKind::Discard(value) => {
				self.eval(*value)?;
				Ok(Value::Unit)
			}
			ExprKind::Out(value) => {
				let printed = self.eval(*value)?;
				let text = self.format(program.expr(*value).ty, &printed);
				self.output.push_str(&text);
				self.output.push('\n');
				Ok(Value::Unit)
			}
			ExprKind::Neg(operand) => match self.eval(*operand)? {
				Value::Int(a) => {
					let ty = self.int_type(*operand);
					match ops::neg(ty, a) {
						Ok(result) => Ok(Value::Int(result)),
						Err(_) => self.trap(span, "6.1", format!("neg {a} overflows {}", int_name(ty))),
					}
				}
				Value::F32(x) => Ok(Value::F32(-x)),
				Value::F64(x) => Ok(Value::F64(-x)),
				_ => unreachable!("neg takes numbers"),
			},
			ExprKind::Not(operand) => Ok(Value::Bool(self.eval(*operand)? == Value::Bool(false))),
			ExprKind::BitNot(operand) => match self.eval(*operand)? {
				Value::Int(a) => Ok(Value::Int(self.int_type(*operand).wrap(!a))),
				_ => unreachable!("~ takes integers"),
			},
			ExprKind::Arith(op, overflow, left, right) => {
				let a = self.eval(*left)?;
				let b = self.eval(*right)?;
				match (a, b) {
					(Value::Int(a), Value::Int(b)) => self.int_arith(*op, *overflow, self.int_type(*left), a, b, span),
					(Value::F32(a), Value::F32(b)) => Ok(Value::F32(float_arith(*op, a, b))),
					(Value::F64(a), Value::F64(b)) => Ok(Value::F64(float_arith(*op, a, b))),
					_ => unreachable!("arithmetic on one type of numbers"),
				}
			}
			ExprKind::Bits(op, left, right) => {
				let (Value::Int(a), Value::Int(b)) = (self.eval(*left)?, self.eval(*right)?) else {
					unreachable!("bit operations take integers");
				};
				let ty = self.int_type(*left);
				let (result, symbol) = match op {
					Bits::And => (Ok(ty.wrap(a & b)), "&"),
					Bits::Or => (Ok(ty.wrap(a | b)), "|"),
					Bits::Xor => (Ok(ty.wrap(a ^ b)), "^"),
					Bits::Shl => (ops::shift_left(ty, a, b), "<<"),
					Bits::Shr => (ops::shift_right(ty, a, b), ">>"),
				};
				match result {
					Ok(result) => Ok(Value::Int(result)),
					Err(_) => self.trap(span, "6.3", format!("{a} {symbol} {b} shifts {} by {b} bits (0 to {} allowed)", int_name(ty), ty.bits - 1)),
				}
			}
			ExprKind::Compare(op, left, right) => {
				let (a, b) = (self.eval(*left)?, self.eval(*right)?);
				// partial_cmp gives None with a NaN (IEEE 754: every comparison but != is false)
				let ordering = match (&a, &b) {
					(Value::Int(a), Value::Int(b)) => Some(a.cmp(b)),
					(Value::Bool(a), Value::Bool(b)) => Some(a.cmp(b)),
					(Value::F32(a), Value::F32(b)) => a.partial_cmp(b),
					(Value::F64(a), Value::F64(b)) => a.partial_cmp(b),
					_ => unreachable!("comparison of one type"),
				};
				Ok(Value::Bool(match op {
					Compare::Eq => ordering == Some(Ordering::Equal),
					Compare::Ne => ordering != Some(Ordering::Equal),
					Compare::Lt => ordering == Some(Ordering::Less),
					Compare::Le => matches!(ordering, Some(Ordering::Less | Ordering::Equal)),
					Compare::Gt => ordering == Some(Ordering::Greater),
					Compare::Ge => matches!(ordering, Some(Ordering::Greater | Ordering::Equal)),
				}))
			}
			ExprKind::And(left, right) => {
				if self.eval(*left)? == Value::Bool(false) {
					return Ok(Value::Bool(false));
				}
				self.eval(*right)
			}
			ExprKind::Or(left, right) => {
				if self.eval(*left)? == Value::Bool(true) {
					return Ok(Value::Bool(true));
				}
				self.eval(*right)
			}
			ExprKind::Fma(a, b, c) => match (self.eval(*a)?, self.eval(*b)?, self.eval(*c)?) {
				// mul_add rounds once
				(Value::F32(a), Value::F32(b), Value::F32(c)) => Ok(Value::F32(a.mul_add(b, c))),
				(Value::F64(a), Value::F64(b), Value::F64(c)) => Ok(Value::F64(a.mul_add(b, c))),
				_ => unreachable!("fma takes floats of one type"),
			},
			ExprKind::Convert(conversion, operand) => {
				let value = self.eval(*operand)?;
				self.convert(*conversion, value, expr.ty, span)
			}
			ExprKind::Struct(fields) => {
				let mut values = vec![Value::Unit; fields.len()];
				for &(index, value) in fields {
					values[index as usize] = self.eval(value)?;
				}
				Ok(Value::Struct(values.into_boxed_slice()))
			}
			ExprKind::Case(case, payload) => Ok(Value::Case(*case, Box::new(self.eval(*payload)?))),
			ExprKind::Field(base, index) => match self.eval(*base)? {
				Value::Struct(fields) => Ok(fields[*index as usize].clone()),
				_ => unreachable!("fields belong to structs"),
			},
			ExprKind::Match(value, arms) => {
				let Value::Case(case, payload) = self.eval(*value)? else {
					unreachable!("match reads cases");
				};
				let arm = &arms[case as usize];
				if let Some(slot) = arm.binding {
					self.locals[self.base + slot as usize] = *payload;
				}
				self.eval(arm.body)
			}
		}
	}

	// the local and the field path that set writes to
	fn place(&self, id: ExprId) -> (usize, Vec<u32>) {
		match &self.program.expr(id).kind {
			ExprKind::Local(slot) => (*slot as usize, Vec::new()),
			ExprKind::Field(base, index) => {
				let (slot, mut path) = self.place(*base);
				path.push(*index);
				(slot, path)
			}
			_ => unreachable!("the checker allows locals and fields"),
		}
	}

	fn int_arith(&self, op: Arith, overflow: Overflow, ty: IntType, a: i128, b: i128, span: Span) -> Result<Value, Exit> {
		let symbol = arith_symbol(op);
		match overflow {
			Overflow::Trap => match ops::trapping(op, ty, a, b) {
				Ok(result) => Ok(Value::Int(result)),
				Err(Fault::DivideByZero) => self.trap(span, "6.2", format!("{a} {symbol} 0 divides by zero")),
				Err(_) => {
					let section = if matches!(op, Arith::Div | Arith::Rem) { "6.2" } else { "6.1" };
					self.trap(span, section, format!("{a} {symbol} {b} overflows {}", int_name(ty)))
				}
			},
			Overflow::Wrap => Ok(Value::Int(ops::wrapping(op, ty, a, b))),
			Overflow::Saturate => Ok(Value::Int(ops::saturating(op, ty, a, b))),
			// the cases of opt: none first and some second
			Overflow::Checked => Ok(match ops::checked(op, ty, a, b) {
				Some(result) => Value::Case(1, Box::new(Value::Int(result))),
				None => Value::Case(0, Box::new(Value::Unit)),
			}),
		}
	}

	fn convert(&self, conversion: Conversion, value: Value, to: TypeId, span: Span) -> Result<Value, Exit> {
		let target = self.program.types.kind(to);
		match (value, target) {
			(Value::Int(v), TypeKind::Int(int)) => match conversion {
				Conversion::Narrow if !int.contains(v) => self.trap(span, "6.4", format!("{v} does not fit in {}", int_name(*int))),
				Conversion::Wrap => Ok(Value::Int(int.wrap(v))),
				Conversion::Saturate => Ok(Value::Int(int.clamp(v))),
				_ => Ok(Value::Int(v)),
			},
			// both casts round to nearest with ties to even
			(Value::Int(v), TypeKind::Float(FloatType::F32)) => Ok(Value::F32(v as f32)),
			(Value::Int(v), TypeKind::Float(FloatType::F64)) => Ok(Value::F64(v as f64)),
			(Value::F32(x), TypeKind::Float(FloatType::F32)) => Ok(Value::F32(x)),
			(Value::F32(x), TypeKind::Float(FloatType::F64)) => Ok(Value::F64(x as f64)),
			(Value::F64(x), TypeKind::Float(FloatType::F32)) => Ok(Value::F32(x as f32)),
			(Value::F64(x), TypeKind::Float(FloatType::F64)) => Ok(Value::F64(x)),
			(value @ (Value::F32(_) | Value::F64(_)), TypeKind::Int(int)) => {
				let (x, text) = match value {
					Value::F32(x) => (x as f64, float_text(x)),
					Value::F64(x) => (x, float_text(x)),
					_ => unreachable!(),
				};
				match conversion {
					Conversion::Saturate => Ok(Value::Int(ops::float_to_int_saturating(*int, x))),
					_ => match ops::float_to_int(*int, x) {
						Some(result) => Ok(Value::Int(result)),
						None => self.trap(span, "6.4", format!("{text} does not fit in {}", int_name(*int))),
					},
				}
			}
			_ => unreachable!("the checker allows numeric conversions only"),
		}
	}

	fn format(&self, ty: TypeId, value: &Value) -> String {
		let types = &self.program.types;
		match value {
			Value::Unit => "unit".to_string(),
			Value::Bool(value) => value.to_string(),
			Value::Int(value) => value.to_string(),
			Value::F32(x) => float_text(*x),
			Value::F64(x) => float_text(*x),
			Value::Struct(values) => {
				let TypeKind::Struct(index) = types.kind(ty) else {
					unreachable!("a struct value has a struct type");
				};
				let def = &types.structs[*index as usize];
				let fields: Vec<String> = def.fields.iter().zip(values.iter()).map(|((name, ty), value)| format!(" ({name} {})", self.format(*ty, value))).collect();
				format!("({}{})", def.name, fields.concat())
			}
			Value::Case(case, payload) => {
				let cases = types.cases(ty).expect("a case value has a union or opt type");
				let (name, payload_ty) = &cases[*case as usize];
				if *payload_ty == Types::UNIT { format!("({name})") } else { format!("({name} {})", self.format(*payload_ty, payload)) }
			}
		}
	}
}
