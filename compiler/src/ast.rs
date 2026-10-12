// the parsed core notation: one node per form (spec/notation.md). Names stay
// unresolved and literals untyped until the checker

use crate::diag::{Diagnostic, Span, error};
use crate::ops::Arith;
use crate::read::Sexp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExprId(pub u32);

#[derive(Clone, Copy, Debug)]
pub struct Name<'a> {
	pub text: &'a str,
	pub span: Span,
}

pub struct Module<'a> {
	pub items: Vec<Item<'a>>,
	pub exprs: Vec<Expr<'a>>,
}

pub enum Item<'a> {
	Fn(Function<'a>),
	Struct(Name<'a>, Vec<(Name<'a>, TypeExpr<'a>)>),
	// a case without a type carries unit
	Union(Name<'a>, Vec<(Name<'a>, Option<TypeExpr<'a>>)>),
}

pub struct Function<'a> {
	pub name: Name<'a>,
	pub params: Vec<(Name<'a>, TypeExpr<'a>)>,
	pub ret: TypeExpr<'a>,
	pub body: Vec<ExprId>,
	pub span: Span,
}

pub enum TypeExpr<'a> {
	Name(Name<'a>),
	Opt(Box<TypeExpr<'a>>, Span),
}

impl TypeExpr<'_> {
	pub fn span(&self) -> Span {
		match self {
			TypeExpr::Name(name) => name.span,
			TypeExpr::Opt(_, span) => *span,
		}
	}
}

pub struct Expr<'a> {
	pub kind: ExprKind<'a>,
	pub span: Span,
}

// what + - * do when the result does not fit (spec 6.1)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overflow {
	Trap,
	Wrap,
	Saturate,
	Checked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bits {
	And,
	Or,
	Xor,
	Shl,
	Shr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Compare {
	Eq,
	Ne,
	Lt,
	Le,
	Gt,
	Ge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unary {
	Neg,
	Not,
	BitNot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Conversion {
	Widen,
	Narrow,
	Wrap,
	Saturate,
	Float,
}

pub enum ExprKind<'a> {
	Int(i128),
	// parsed by the checker once it knows f32 or f64 (one rounding either way)
	Float(&'a str),
	Bool(bool),
	Unit,
	Name(&'a str),
	Call(Name<'a>, Vec<ExprId>),
	Let(Name<'a>, TypeExpr<'a>, ExprId),
	Set(ExprId, ExprId),
	Do(Vec<ExprId>),
	If(ExprId, ExprId, Option<ExprId>),
	Loop(Option<Name<'a>>, Vec<ExprId>),
	Break(Option<Name<'a>>),
	Continue(Option<Name<'a>>),
	Return(Option<ExprId>),
	Discard(ExprId),
	Out(ExprId),
	Unary(Unary, ExprId),
	Arith(Arith, Overflow, ExprId, ExprId),
	Bits(Bits, ExprId, ExprId),
	Compare(Compare, ExprId, ExprId),
	And(ExprId, ExprId),
	Or(ExprId, ExprId),
	Fma(ExprId, ExprId, ExprId),
	Convert(Conversion, TypeExpr<'a>, ExprId),
	Make(TypeExpr<'a>, Vec<(Name<'a>, Option<ExprId>)>),
	Field(ExprId, Name<'a>),
	Match(ExprId, Vec<Arm<'a>>),
}

pub struct Arm<'a> {
	pub case: Name<'a>,
	pub binding: Option<Name<'a>>,
	pub body: ExprId,
}

// words with a meaning of their own. None of them can name a function/type/local
const RESERVED: &[&str] = &[
	"fn", "struct", "union", "let", "set", "do", "if", "loop", "break", "continue", "return", "discard", "out", "make", "match", "fma", "neg", "not", "and", "or", "widen", "narrow", "wrap", "saturate", "float", "true", "false", "unit", "bool", "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64", "opt",
];

enum Binary {
	Arith(Arith, Overflow),
	Bits(Bits),
	Compare(Compare),
}

fn binary(head: &str) -> Option<Binary> {
	use Overflow::*;
	Some(match head {
		"+" => Binary::Arith(Arith::Add, Trap),
		"-" => Binary::Arith(Arith::Sub, Trap),
		"*" => Binary::Arith(Arith::Mul, Trap),
		"/" => Binary::Arith(Arith::Div, Trap),
		"%" => Binary::Arith(Arith::Rem, Trap),
		"wrapping+" => Binary::Arith(Arith::Add, Wrap),
		"wrapping-" => Binary::Arith(Arith::Sub, Wrap),
		"wrapping*" => Binary::Arith(Arith::Mul, Wrap),
		"saturating+" => Binary::Arith(Arith::Add, Saturate),
		"saturating-" => Binary::Arith(Arith::Sub, Saturate),
		"saturating*" => Binary::Arith(Arith::Mul, Saturate),
		"checked+" => Binary::Arith(Arith::Add, Checked),
		"checked-" => Binary::Arith(Arith::Sub, Checked),
		"checked*" => Binary::Arith(Arith::Mul, Checked),
		"&" => Binary::Bits(Bits::And),
		"|" => Binary::Bits(Bits::Or),
		"^" => Binary::Bits(Bits::Xor),
		"<<" => Binary::Bits(Bits::Shl),
		">>" => Binary::Bits(Bits::Shr),
		"==" => Binary::Compare(Compare::Eq),
		"!=" => Binary::Compare(Compare::Ne),
		"<" => Binary::Compare(Compare::Lt),
		"<=" => Binary::Compare(Compare::Le),
		">" => Binary::Compare(Compare::Gt),
		">=" => Binary::Compare(Compare::Ge),
		_ => return None,
	})
}

pub fn parse<'a>(items: &[Sexp<'a>]) -> Result<Module<'a>, Diagnostic> {
	let mut parser = Parser { exprs: Vec::new() };
	let items = items.iter().map(|item| parser.item(item)).collect::<Result<_, _>>()?;
	Ok(Module { items, exprs: parser.exprs })
}

fn list<'s, 'a>(sexp: &'s Sexp<'a>, what: &str) -> Result<&'s [Sexp<'a>], Diagnostic> {
	match sexp {
		Sexp::List(items, _) => Ok(items),
		Sexp::Atom(..) => Err(error(sexp.span(), format!("expected {what} in parentheses"))),
	}
}

fn word(text: &str) -> bool {
	text.bytes().next().is_some_and(|c| c.is_ascii_alphabetic() || c == b'_') && text.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
}

fn name<'a>(sexp: &Sexp<'a>) -> Result<Name<'a>, Diagnostic> {
	match *sexp {
		Sexp::Atom(text, span) if RESERVED.contains(&text) => Err(error(span, format!("{text} is reserved and cannot name anything"))),
		Sexp::Atom(text, span) if word(text) => Ok(Name { text, span }),
		_ => Err(error(sexp.span(), "expected a name (letters/digits/_ with no digit first)")),
	}
}

// :name picks a loop
fn label<'a>(sexp: &Sexp<'a>) -> Option<Name<'a>> {
	match *sexp {
		Sexp::Atom(text, span) => text.strip_prefix(':').filter(|rest| word(rest)).map(|rest| Name { text: rest, span }),
		Sexp::List(..) => None,
	}
}

fn arity(head: &str, args: &[Sexp], count: usize, span: Span, what: &str) -> Result<(), Diagnostic> {
	if args.len() == count { Ok(()) } else { Err(error(span, format!("{head} takes {what}"))) }
}

fn digits_ok(text: &str, digit: fn(u8) -> bool) -> bool {
	text.bytes().next().is_some_and(digit) && text.bytes().all(|c| digit(c) || c == b'_')
}

// digits(.digits)(e(+|-)digits) with at least the fraction or the exponent
fn float_text(text: &str) -> bool {
	let (mantissa, exponent) = match text.find(['e', 'E']) {
		Some(at) => (&text[..at], Some(&text[at + 1..])),
		None => (text, None),
	};
	let (whole, fraction) = match mantissa.split_once('.') {
		Some((whole, fraction)) => (whole, Some(fraction)),
		None => (mantissa, None),
	};
	let exponent_ok = exponent.is_none_or(|e| digits_ok(e.strip_prefix(['+', '-']).unwrap_or(e), |c| c.is_ascii_digit()));
	(fraction.is_some() || exponent.is_some()) && digits_ok(whole, |c| c.is_ascii_digit()) && fraction.is_none_or(|f| digits_ok(f, |c| c.is_ascii_digit())) && exponent_ok
}

fn number<'a>(text: &'a str, span: Span) -> Result<ExprKind<'a>, Diagnostic> {
	let unsigned = text.strip_prefix('-').unwrap_or(text);
	let negative = unsigned.len() != text.len();
	let (digits, radix) = match unsigned.strip_prefix("0x") {
		Some(hex) => (hex, 16),
		None => (unsigned, 10),
	};
	if radix == 10 && !digits_ok(digits, |c| c.is_ascii_digit()) {
		return if float_text(unsigned) { Ok(ExprKind::Float(text)) } else { Err(error(span, format!("{text} is not a number"))) };
	}
	if radix == 16 && !digits_ok(digits, |c| c.is_ascii_hexdigit()) {
		return Err(error(span, format!("{text} is not a number")));
	}
	let mut value: i128 = 0;
	for c in digits.bytes().filter(|&c| c != b'_') {
		let digit = (c as char).to_digit(radix).unwrap() as i128;
		value = value.checked_mul(radix as i128).and_then(|v| v.checked_add(digit)).ok_or_else(|| error(span, format!("{text} is too large for any integer type")))?;
	}
	Ok(ExprKind::Int(if negative { -value } else { value }))
}

struct Parser<'a> {
	exprs: Vec<Expr<'a>>,
}

impl<'a> Parser<'a> {
	fn push(&mut self, kind: ExprKind<'a>, span: Span) -> ExprId {
		self.exprs.push(Expr { kind, span });
		ExprId(self.exprs.len() as u32 - 1)
	}

	fn item(&mut self, sexp: &Sexp<'a>) -> Result<Item<'a>, Diagnostic> {
		let items = list(sexp, "a declaration")?;
		let span = sexp.span();
		match items.first() {
			Some(Sexp::Atom("fn", _)) => {
				let [_, fn_name, params, ret, body @ ..] = items else {
					return Err(error(span, "fn takes a name/a parameter list/a return type/a body"));
				};
				let params = list(params, "a parameter list")?.iter().map(|param| self.typed_name(param, "a parameter (name type)")).collect::<Result<_, _>>()?;
				let ret = self.type_expr(ret)?;
				let body = self.block(body)?;
				Ok(Item::Fn(Function { name: name(fn_name)?, params, ret, body, span }))
			}
			Some(Sexp::Atom("struct", _)) if items.len() >= 2 => {
				let fields = items[2..].iter().map(|field| self.typed_name(field, "a field (name type)")).collect::<Result<_, _>>()?;
				Ok(Item::Struct(name(&items[1])?, fields))
			}
			Some(Sexp::Atom("union", _)) if items.len() >= 2 => {
				let mut cases = Vec::new();
				for case in &items[2..] {
					match list(case, "a case (name type)")? {
						[case_name] => cases.push((name(case_name)?, None)),
						[case_name, ty] => cases.push((name(case_name)?, Some(self.type_expr(ty)?))),
						_ => return Err(error(case.span(), "a case is (name) or (name type)")),
					}
				}
				Ok(Item::Union(name(&items[1])?, cases))
			}
			_ => Err(error(span, "expected a declaration: (fn ...)/(struct ...)/(union ...)")),
		}
	}

	fn typed_name(&mut self, sexp: &Sexp<'a>, what: &str) -> Result<(Name<'a>, TypeExpr<'a>), Diagnostic> {
		match list(sexp, what)? {
			[item_name, ty] => Ok((name(item_name)?, self.type_expr(ty)?)),
			_ => Err(error(sexp.span(), format!("expected {what}"))),
		}
	}

	fn type_expr(&mut self, sexp: &Sexp<'a>) -> Result<TypeExpr<'a>, Diagnostic> {
		match sexp {
			&Sexp::Atom(text, span) if word(text) => Ok(TypeExpr::Name(Name { text, span })),
			Sexp::List(items, span) => match items.as_slice() {
				[Sexp::Atom("opt", _), inner] => Ok(TypeExpr::Opt(Box::new(self.type_expr(inner)?), *span)),
				_ => Err(error(*span, "a type is a name or (opt type)")),
			},
			Sexp::Atom(_, span) => Err(error(*span, "expected a type")),
		}
	}

	// forms in a fn/do/loop body (the only places where let may stand)
	fn block(&mut self, sexps: &[Sexp<'a>]) -> Result<Vec<ExprId>, Diagnostic> {
		let mut forms = Vec::with_capacity(sexps.len());
		for sexp in sexps {
			let form = match sexp {
				Sexp::List(items, span) if matches!(items.first(), Some(Sexp::Atom("let", _))) => {
					let [_, local, ty, value] = items.as_slice() else {
						return Err(error(*span, "let takes a name/a type/a value"));
					};
					let kind = ExprKind::Let(name(local)?, self.type_expr(ty)?, self.expr(value)?);
					self.push(kind, *span)
				}
				_ => self.expr(sexp)?,
			};
			forms.push(form);
		}
		Ok(forms)
	}

	fn expr(&mut self, sexp: &Sexp<'a>) -> Result<ExprId, Diagnostic> {
		let (items, span) = match sexp {
			&Sexp::Atom(text, span) => {
				let kind = match text {
					"true" => ExprKind::Bool(true),
					"false" => ExprKind::Bool(false),
					"unit" => ExprKind::Unit,
					_ if text.trim_start_matches('-').starts_with(|c: char| c.is_ascii_digit()) => number(text, span)?,
					_ => ExprKind::Name(name(sexp)?.text),
				};
				return Ok(self.push(kind, span));
			}
			Sexp::List(items, span) => (items, *span),
		};
		let Some(Sexp::Atom(head, _)) = items.first() else {
			return Err(error(span, "a form starts with a word: (if ...)/(+ ...)/(f ...)"));
		};
		let args = &items[1..];
		let kind = match *head {
			"let" => return Err(error(span, "let stands only directly in a block (the body of fn/do/loop)")),
			"set" => {
				arity(head, args, 2, span, "a place and a value")?;
				ExprKind::Set(self.expr(&args[0])?, self.expr(&args[1])?)
			}
			"do" => ExprKind::Do(self.block(args)?),
			"if" => match args {
				[condition, then] => ExprKind::If(self.expr(condition)?, self.expr(then)?, None),
				[condition, then, other] => ExprKind::If(self.expr(condition)?, self.expr(then)?, Some(self.expr(other)?)),
				_ => return Err(error(span, "if takes a condition/a then branch/an optional else branch")),
			},
			"loop" => match args.first().and_then(label) {
				Some(name) => ExprKind::Loop(Some(name), self.block(&args[1..])?),
				None => ExprKind::Loop(None, self.block(args)?),
			},
			"break" | "continue" => {
				let target = match args {
					[] => None,
					[target] => Some(label(target).ok_or_else(|| error(target.span(), "expected a label like :outer"))?),
					_ => return Err(error(span, format!("{head} takes an optional label"))),
				};
				if *head == "break" { ExprKind::Break(target) } else { ExprKind::Continue(target) }
			}
			"return" => match args {
				[] => ExprKind::Return(None),
				[value] => ExprKind::Return(Some(self.expr(value)?)),
				_ => return Err(error(span, "return takes an optional value")),
			},
			"discard" | "out" | "neg" | "not" | "~" => {
				arity(head, args, 1, span, "one operand")?;
				let operand = self.expr(&args[0])?;
				match *head {
					"discard" => ExprKind::Discard(operand),
					"out" => ExprKind::Out(operand),
					"neg" => ExprKind::Unary(Unary::Neg, operand),
					"not" => ExprKind::Unary(Unary::Not, operand),
					_ => ExprKind::Unary(Unary::BitNot, operand),
				}
			}
			"and" | "or" => {
				arity(head, args, 2, span, "two operands")?;
				let (left, right) = (self.expr(&args[0])?, self.expr(&args[1])?);
				if *head == "and" { ExprKind::And(left, right) } else { ExprKind::Or(left, right) }
			}
			"fma" => {
				arity(head, args, 3, span, "three operands (a * b + c)")?;
				ExprKind::Fma(self.expr(&args[0])?, self.expr(&args[1])?, self.expr(&args[2])?)
			}
			"widen" | "narrow" | "wrap" | "saturate" | "float" => {
				arity(head, args, 2, span, "a type and a value")?;
				let conversion = match *head {
					"widen" => Conversion::Widen,
					"narrow" => Conversion::Narrow,
					"wrap" => Conversion::Wrap,
					"saturate" => Conversion::Saturate,
					_ => Conversion::Float,
				};
				ExprKind::Convert(conversion, self.type_expr(&args[0])?, self.expr(&args[1])?)
			}
			"make" => {
				let Some((ty, parts)) = args.split_first() else {
					return Err(error(span, "make takes a type and its parts"));
				};
				let ty = self.type_expr(ty)?;
				let mut values = Vec::with_capacity(parts.len());
				for part in parts {
					match list(part, "a part (name value)")? {
						[part_name] => values.push((name(part_name)?, None)),
						[part_name, value] => values.push((name(part_name)?, Some(self.expr(value)?))),
						_ => return Err(error(part.span(), "a part is (name value) or (name)")),
					}
				}
				ExprKind::Make(ty, values)
			}
			"." => {
				arity(head, args, 2, span, "a value and a field name")?;
				ExprKind::Field(self.expr(&args[0])?, name(&args[1])?)
			}
			"match" => {
				let Some((value, arms)) = args.split_first() else {
					return Err(error(span, "match takes a value and its arms"));
				};
				let value = self.expr(value)?;
				let mut parsed = Vec::with_capacity(arms.len());
				for arm in arms {
					let [pattern, body] = list(arm, "an arm ((case binding) body)")? else {
						return Err(error(arm.span(), "an arm is ((case binding) body)"));
					};
					let (case, binding) = match list(pattern, "a pattern (case binding)")? {
						[case] => (name(case)?, None),
						[case, binding] => (name(case)?, Some(name(binding)?)),
						_ => return Err(error(pattern.span(), "a pattern is (case) or (case binding)")),
					};
					parsed.push(Arm { case, binding, body: self.expr(body)? });
				}
				ExprKind::Match(value, parsed)
			}
			_ => match binary(head) {
				Some(op) => {
					arity(head, args, 2, span, "two operands")?;
					let (left, right) = (self.expr(&args[0])?, self.expr(&args[1])?);
					match op {
						Binary::Arith(arith, overflow) => ExprKind::Arith(arith, overflow, left, right),
						Binary::Bits(bits) => ExprKind::Bits(bits, left, right),
						Binary::Compare(compare) => ExprKind::Compare(compare, left, right),
					}
				}
				None => {
					let callee = name(&items[0])?;
					let args = args.iter().map(|arg| self.expr(arg)).collect::<Result<_, _>>()?;
					ExprKind::Call(callee, args)
				}
			},
		};
		Ok(self.push(kind, span))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::read::read;

	fn expr_kind(text: &str) -> Result<String, String> {
		let items = read(text).map_err(|d| d.message)?;
		let mut parser = Parser { exprs: Vec::new() };
		let id = parser.expr(&items[0]).map_err(|d| d.message)?;
		Ok(match &parser.exprs[id.0 as usize].kind {
			ExprKind::Int(value) => format!("int {value}"),
			ExprKind::Float(text) => format!("float {text}"),
			ExprKind::Name(text) => format!("name {text}"),
			_ => "other".to_string(),
		})
	}

	#[test]
	fn numbers() {
		assert_eq!(expr_kind("-128"), Ok("int -128".into()));
		assert_eq!(expr_kind("0xff_ff"), Ok("int 65535".into()));
		assert_eq!(expr_kind("1_000"), Ok("int 1000".into()));
		assert_eq!(expr_kind("2.5e-3"), Ok("float 2.5e-3".into()));
		assert_eq!(expr_kind("-0.0"), Ok("float -0.0".into()));
		assert_eq!(expr_kind("1e9"), Ok("float 1e9".into()));
		assert!(expr_kind("1.").is_err());
		assert!(expr_kind("1e").is_err());
		assert!(expr_kind("0x").is_err());
		assert!(expr_kind("12ab").is_err());
		assert!(expr_kind("99999999999999999999999999999999999999999").unwrap_err().contains("too large"));
	}

	#[test]
	fn names() {
		assert_eq!(expr_kind("total_2"), Ok("name total_2".into()));
		assert!(expr_kind("loop").unwrap_err().contains("reserved"));
		assert!(expr_kind("a-b").is_err());
	}
}
