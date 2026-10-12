// name resolution and type checking by the rules of spec/core.md. Turns the parsed
// module into the checked core (typed.rs) and stops at the first error

use std::collections::HashMap;

use crate::ast::{self, Compare, Conversion, Overflow, Unary};
use crate::diag::{Diagnostic, Span, error};
use crate::ops::Arith;
use crate::typed::*;

pub fn check(module: &ast::Module) -> Result<Program, Diagnostic> {
	let mut checker = Checker { module, types: Types::default(), items: HashMap::new(), signatures: Vec::new(), exprs: Vec::new(), scopes: Vec::new(), frame: 0, loops: Vec::new(), next_loop: 0, ret: Types::UNIT };
	checker.declare()?;
	let mut functions = Vec::new();
	for item in &module.items {
		if let ast::Item::Fn(function) = item {
			functions.push(checker.function(function)?);
		}
	}
	let main = checker.main()?;
	Ok(Program { types: checker.types, functions, exprs: checker.exprs, main })
}

#[derive(Clone, Copy)]
enum ItemRef {
	Fn(FnId),
	Type(TypeId),
}

struct Signature {
	params: Vec<TypeId>,
	ret: TypeId,
}

struct LoopScope<'a> {
	label: Option<&'a str>,
	id: LoopId,
	broken: bool,
}

struct Checker<'m, 'a> {
	module: &'m ast::Module<'a>,
	types: Types,
	items: HashMap<&'a str, ItemRef>,
	signatures: Vec<Signature>,
	exprs: Vec<Expr>,
	// the function being checked
	scopes: Vec<Vec<(&'a str, Slot, TypeId)>>,
	frame: u32,
	loops: Vec<LoopScope<'a>>,
	next_loop: u32,
	ret: TypeId,
}

fn no_type(span: Span) -> Diagnostic {
	error(span, "this literal has no type here (let/a parameter/a typed operand gives it one)")
}

fn arith_symbol(op: Arith, overflow: Overflow) -> String {
	let prefix = match overflow {
		Overflow::Trap => "",
		Overflow::Wrap => "wrapping",
		Overflow::Saturate => "saturating",
		Overflow::Checked => "checked",
	};
	let symbol = match op {
		Arith::Add => "+",
		Arith::Sub => "-",
		Arith::Mul => "*",
		Arith::Div => "/",
		Arith::Rem => "%",
	};
	format!("{prefix}{symbol}")
}

fn conversion_name(conversion: Conversion) -> &'static str {
	match conversion {
		Conversion::Widen => "widen",
		Conversion::Narrow => "narrow",
		Conversion::Wrap => "wrap",
		Conversion::Saturate => "saturate",
		Conversion::Float => "float",
	}
}

impl<'m, 'a> Checker<'m, 'a> {
	fn push(&mut self, kind: ExprKind, ty: TypeId, span: Span) -> ExprId {
		self.exprs.push(Expr { kind, ty, span });
		ExprId(self.exprs.len() as u32 - 1)
	}

	fn ty(&self, id: ExprId) -> TypeId {
		self.exprs[id.0 as usize].ty
	}

	fn name(&self, ty: TypeId) -> String {
		self.types.name(ty)
	}

	// never fits wherever a value is expected (it never arrives)
	fn expect(&self, id: ExprId, expected: Option<TypeId>) -> Result<ExprId, Diagnostic> {
		let ty = self.ty(id);
		match expected {
			Some(want) if ty != want && ty != Types::NEVER => Err(error(self.exprs[id.0 as usize].span, format!("expected {} but this is {}", self.name(want), self.name(ty)))),
			_ => Ok(id),
		}
	}

	// ================================================================
	// DECLARATIONS
	// ================================================================

	fn declare(&mut self) -> Result<(), Diagnostic> {
		let module = self.module;
		// every name first (declarations may refer to each other in any order)
		let mut user_types = Vec::new();
		let mut functions = 0;
		for item in &module.items {
			let (name, item_ref) = match item {
				ast::Item::Fn(function) => {
					functions += 1;
					(function.name, ItemRef::Fn(FnId(functions - 1)))
				}
				ast::Item::Struct(name, _) => {
					self.types.structs.push(StructDef { name: name.text.to_string(), fields: Vec::new() });
					let id = self.types.intern(TypeKind::Struct(self.types.structs.len() as u32 - 1));
					user_types.push((id, name.span));
					(*name, ItemRef::Type(id))
				}
				ast::Item::Union(name, _) => {
					self.types.unions.push(UnionDef { name: name.text.to_string(), cases: Vec::new() });
					let id = self.types.intern(TypeKind::Union(self.types.unions.len() as u32 - 1));
					user_types.push((id, name.span));
					(*name, ItemRef::Type(id))
				}
			};
			if self.items.insert(name.text, item_ref).is_some() {
				return Err(error(name.span, format!("{} is declared twice", name.text)));
			}
		}
		let (mut struct_index, mut union_index) = (0, 0);
		for item in &module.items {
			match item {
				ast::Item::Fn(function) => {
					let params = function.params.iter().map(|(_, ty)| self.resolve(ty)).collect::<Result<_, _>>()?;
					let ret = self.resolve(&function.ret)?;
					self.signatures.push(Signature { params, ret });
				}
				ast::Item::Struct(_, fields) => {
					let mut resolved: Vec<(String, TypeId)> = Vec::new();
					for (field, ty) in fields {
						if resolved.iter().any(|(name, _)| name == field.text) {
							return Err(error(field.span, format!("field {} appears twice", field.text)));
						}
						resolved.push((field.text.to_string(), self.resolve(ty)?));
					}
					self.types.structs[struct_index].fields = resolved;
					struct_index += 1;
				}
				ast::Item::Union(name, cases) => {
					if cases.is_empty() {
						return Err(error(name.span, format!("union {} needs at least one case", name.text)));
					}
					let mut resolved: Vec<(String, TypeId)> = Vec::new();
					for (case, ty) in cases {
						if resolved.iter().any(|(name, _)| name == case.text) {
							return Err(error(case.span, format!("case {} appears twice", case.text)));
						}
						let ty = match ty {
							Some(ty) => self.resolve(ty)?,
							None => Types::UNIT,
						};
						resolved.push((case.text.to_string(), ty));
					}
					self.types.unions[union_index].cases = resolved;
					union_index += 1;
				}
			}
		}
		// a type that holds itself by value would be infinitely large
		for (id, span) in user_types {
			if self.holds(id, id, &mut Vec::new()) {
				return Err(error(span, format!("{} contains itself", self.name(id))));
			}
		}
		Ok(())
	}

	// does `ty` contain `target` by value (through fields/cases/opt payloads)
	fn holds(&self, ty: TypeId, target: TypeId, seen: &mut Vec<TypeId>) -> bool {
		let parts: Vec<TypeId> = match self.types.kind(ty) {
			TypeKind::Struct(index) => self.types.structs[*index as usize].fields.iter().map(|(_, ty)| *ty).collect(),
			TypeKind::Union(index) => self.types.unions[*index as usize].cases.iter().map(|(_, ty)| *ty).collect(),
			TypeKind::Opt(payload) => vec![*payload],
			_ => Vec::new(),
		};
		for part in parts {
			if part == target {
				return true;
			}
			if !seen.contains(&part) {
				seen.push(part);
				if self.holds(part, target, seen) {
					return true;
				}
			}
		}
		false
	}

	fn resolve(&mut self, ty: &ast::TypeExpr<'a>) -> Result<TypeId, Diagnostic> {
		match ty {
			ast::TypeExpr::Name(name) => {
				if let Some(id) = Types::builtin(name.text) {
					return Ok(id);
				}
				match self.items.get(name.text) {
					Some(ItemRef::Type(id)) => Ok(*id),
					Some(ItemRef::Fn(_)) => Err(error(name.span, format!("{} is a function and not a type", name.text))),
					None => Err(error(name.span, format!("unknown type {}", name.text))),
				}
			}
			ast::TypeExpr::Opt(inner, _) => {
				let inner = self.resolve(inner)?;
				Ok(self.types.intern(TypeKind::Opt(inner)))
			}
		}
	}

	fn main(&self) -> Result<FnId, Diagnostic> {
		let module = self.module;
		let Some(&ItemRef::Fn(id)) = self.items.get("main") else {
			return Err(error(Span::new(0, 0), "the program has no main function"));
		};
		let signature = &self.signatures[id.0 as usize];
		if !signature.params.is_empty() || (signature.ret != Types::UNIT && signature.ret != Types::I32) {
			let span = module.items.iter().find_map(|item| match item {
				ast::Item::Fn(function) if function.name.text == "main" => Some(function.name.span),
				_ => None,
			});
			return Err(error(span.unwrap(), "main takes nothing and returns i32 or unit"));
		}
		Ok(id)
	}

	// ================================================================
	// FUNCTIONS AND BLOCKS
	// ================================================================

	fn function(&mut self, function: &ast::Function<'a>) -> Result<Function, Diagnostic> {
		let Some(&ItemRef::Fn(id)) = self.items.get(function.name.text) else {
			unreachable!("declare() registered every function");
		};
		let params = self.signatures[id.0 as usize].params.clone();
		let ret = self.signatures[id.0 as usize].ret;
		self.ret = ret;
		self.frame = 0;
		self.scopes = vec![Vec::new()];
		self.loops.clear();
		for ((name, _), &ty) in function.params.iter().zip(&params) {
			self.declare_local(*name, ty)?;
		}
		let body = self.block(&function.body, Some(ret), function.span)?;
		Ok(Function { name: function.name.text.to_string(), params, ret, frame: self.frame, body, span: function.span })
	}

	fn lookup(&self, name: &str) -> Option<(Slot, TypeId)> {
		self.scopes.iter().rev().flat_map(|scope| scope.iter().rev()).find(|(local, _, _)| *local == name).map(|&(_, slot, ty)| (slot, ty))
	}

	// a name stays unique while it is visible (no shadowing)
	fn declare_local(&mut self, name: ast::Name<'a>, ty: TypeId) -> Result<Slot, Diagnostic> {
		if self.lookup(name.text).is_some() {
			return Err(error(name.span, format!("{} is already a local here", name.text)));
		}
		if self.items.contains_key(name.text) {
			return Err(error(name.span, format!("{} already names a function or a type", name.text)));
		}
		let slot = self.frame;
		self.frame += 1;
		self.scopes.last_mut().unwrap().push((name.text, slot, ty));
		Ok(slot)
	}

	// a form whose value nobody reads: unit/never only (discard drops anything else)
	fn statement(&mut self, form: ast::ExprId) -> Result<ExprId, Diagnostic> {
		let module = self.module;
		let node = &module.exprs[form.0 as usize];
		let id = match &node.kind {
			ast::ExprKind::Let(name, ty, value) => {
				let ty = self.resolve(ty)?;
				let value = self.expr(*value, Some(ty))?;
				let slot = self.declare_local(*name, ty)?;
				return Ok(self.push(ExprKind::Let(slot, value), Types::UNIT, node.span));
			}
			_ => self.expr(form, None)?,
		};
		let ty = self.ty(id);
		if ty != Types::UNIT && ty != Types::NEVER {
			return Err(error(node.span, format!("this {} value goes unused (discard drops it)", self.name(ty))));
		}
		Ok(id)
	}

	// the forms of fn/do: the last one gives the value (unit when it is a let or missing)
	fn block(&mut self, forms: &[ast::ExprId], expected: Option<TypeId>, span: Span) -> Result<ExprId, Diagnostic> {
		let module = self.module;
		self.scopes.push(Vec::new());
		let mut typed = Vec::with_capacity(forms.len());
		let mut ty = Types::UNIT;
		for (index, &form) in forms.iter().enumerate() {
			let last = index + 1 == forms.len() && !matches!(module.exprs[form.0 as usize].kind, ast::ExprKind::Let(..));
			let id = if last { self.expr(form, expected)? } else { self.statement(form)? };
			if last {
				ty = self.ty(id);
			}
			typed.push(id);
		}
		self.scopes.pop();
		if let Some(want) = expected
			&& ty == Types::UNIT
			&& want != Types::UNIT
		{
			let end = forms.last().map_or(span, |&form| module.exprs[form.0 as usize].span);
			return Err(error(end, format!("this ends the block without a value of type {}", self.name(want))));
		}
		Ok(self.push(ExprKind::Block(typed), ty, span))
	}

	// ================================================================
	// EXPRESSIONS
	// ================================================================

	// literals and operators over nothing but literals have no type of their own. They
	// take it from their surroundings
	fn untyped(&self, id: ast::ExprId) -> bool {
		match &self.module.exprs[id.0 as usize].kind {
			ast::ExprKind::Int(_) | ast::ExprKind::Float(_) => true,
			ast::ExprKind::Unary(Unary::Neg | Unary::BitNot, operand) => self.untyped(*operand),
			ast::ExprKind::Arith(_, overflow, left, right) => *overflow != Overflow::Checked && self.untyped(*left) && self.untyped(*right),
			ast::ExprKind::Bits(_, left, right) => self.untyped(*left) && self.untyped(*right),
			ast::ExprKind::Fma(a, b, c) => self.untyped(*a) && self.untyped(*b) && self.untyped(*c),
			ast::ExprKind::If(_, then, Some(other)) => self.untyped(*then) && self.untyped(*other),
			ast::ExprKind::Do(forms) => forms.last().is_some_and(|&last| self.untyped(last)),
			_ => false,
		}
	}

	// operands that share one type. Without a hint the first operand with a type of
	// its own sets it. The order of evaluation stays as written
	fn operands(&mut self, ids: &[ast::ExprId], hint: Option<TypeId>) -> Result<Vec<ExprId>, Diagnostic> {
		let mut typed = vec![None; ids.len()];
		let mut ty = hint;
		if ty.is_none()
			&& let Some(first) = ids.iter().position(|&id| !self.untyped(id))
		{
			let operand = self.expr(ids[first], None)?;
			typed[first] = Some(operand);
			ty = Some(self.ty(operand)).filter(|&t| t != Types::NEVER);
		}
		for (index, &id) in ids.iter().enumerate() {
			if typed[index].is_none() {
				let operand = self.expr(id, ty)?;
				ty = ty.or(Some(self.ty(operand)).filter(|&t| t != Types::NEVER));
				typed[index] = Some(operand);
			}
		}
		Ok(typed.into_iter().map(Option::unwrap).collect())
	}

	// the first type that is not never
	fn common(&self, ids: &[ExprId]) -> TypeId {
		ids.iter().map(|&id| self.ty(id)).find(|&ty| ty != Types::NEVER).unwrap_or(Types::NEVER)
	}

	fn expr(&mut self, id: ast::ExprId, expected: Option<TypeId>) -> Result<ExprId, Diagnostic> {
		let module = self.module;
		let node = &module.exprs[id.0 as usize];
		let span = node.span;
		let typed = match &node.kind {
			ast::ExprKind::Int(value) => return self.int_literal(*value, expected, span),
			ast::ExprKind::Float(text) => return self.float_literal(text, expected, span),
			ast::ExprKind::Bool(value) => self.push(ExprKind::Bool(*value), Types::BOOL, span),
			ast::ExprKind::Unit => self.push(ExprKind::Unit, Types::UNIT, span),
			ast::ExprKind::Name(name) => match self.lookup(name) {
				Some((slot, ty)) => self.push(ExprKind::Local(slot), ty, span),
				None if self.items.contains_key(name) => return Err(error(span, format!("{name} is not a local"))),
				None => return Err(error(span, format!("unknown name {name}"))),
			},
			ast::ExprKind::Call(callee, args) => self.call(*callee, args, span)?,
			ast::ExprKind::Let(..) => unreachable!("the parser keeps let in blocks"),
			ast::ExprKind::Set(place, value) => {
				let place = self.place(*place)?;
				let value = self.expr(*value, Some(self.ty(place)))?;
				self.push(ExprKind::Set(place, value), Types::UNIT, span)
			}
			ast::ExprKind::Do(forms) => return self.block(forms, expected, span),
			ast::ExprKind::If(condition, then, other) => {
				let condition = self.expr(*condition, Some(Types::BOOL))?;
				match other {
					None => {
						let then = self.expr(*then, Some(Types::UNIT))?;
						self.push(ExprKind::If(condition, then, None), Types::UNIT, span)
					}
					Some(other) => {
						let branches = self.operands(&[*then, *other], expected)?;
						let ty = self.common(&branches);
						return Ok(self.push(ExprKind::If(condition, branches[0], Some(branches[1])), ty, span));
					}
				}
			}
			ast::ExprKind::Loop(label, body) => self.loop_(*label, body, span)?,
			ast::ExprKind::Break(label) => {
				let index = self.target(*label, span, "break")?;
				self.loops[index].broken = true;
				self.push(ExprKind::Break(self.loops[index].id), Types::NEVER, span)
			}
			ast::ExprKind::Continue(label) => {
				let index = self.target(*label, span, "continue")?;
				self.push(ExprKind::Continue(self.loops[index].id), Types::NEVER, span)
			}
			ast::ExprKind::Return(value) => {
				let value = match value {
					Some(value) => Some(self.expr(*value, Some(self.ret))?),
					None if self.ret == Types::UNIT => None,
					None => return Err(error(span, format!("return needs a value of type {}", self.name(self.ret)))),
				};
				self.push(ExprKind::Return(value), Types::NEVER, span)
			}
			ast::ExprKind::Discard(value) => {
				let value = self.expr(*value, None)?;
				self.push(ExprKind::Discard(value), Types::UNIT, span)
			}
			ast::ExprKind::Out(value) => {
				let value = self.expr(*value, None)?;
				self.push(ExprKind::Out(value), Types::UNIT, span)
			}
			ast::ExprKind::Unary(op, operand) => self.unary(*op, *operand, expected, span)?,
			ast::ExprKind::Arith(op, overflow, left, right) => self.arith(*op, *overflow, *left, *right, expected, span)?,
			ast::ExprKind::Bits(op, left, right) => {
				let hint = expected.filter(|&t| self.types.int(t).is_some());
				let operands = self.operands(&[*left, *right], hint)?;
				let ty = self.common(&operands);
				if self.types.int(ty).is_none() && ty != Types::NEVER {
					return Err(error(span, format!("bit operations take integers and not {}", self.name(ty))));
				}
				self.push(ExprKind::Bits(*op, operands[0], operands[1]), ty, span)
			}
			ast::ExprKind::Compare(op, left, right) => {
				let operands = self.operands(&[*left, *right], None)?;
				let ty = self.common(&operands);
				let comparable = match self.types.kind(ty) {
					TypeKind::Int(_) | TypeKind::Float(_) | TypeKind::Never => true,
					TypeKind::Bool => matches!(op, Compare::Eq | Compare::Ne),
					_ => false,
				};
				if !comparable {
					return Err(error(span, format!("{} values do not compare this way", self.name(ty))));
				}
				self.push(ExprKind::Compare(*op, operands[0], operands[1]), Types::BOOL, span)
			}
			ast::ExprKind::And(left, right) | ast::ExprKind::Or(left, right) => {
				let left = self.expr(*left, Some(Types::BOOL))?;
				let right = self.expr(*right, Some(Types::BOOL))?;
				let kind = if matches!(node.kind, ast::ExprKind::And(..)) { ExprKind::And(left, right) } else { ExprKind::Or(left, right) };
				self.push(kind, Types::BOOL, span)
			}
			ast::ExprKind::Fma(a, b, c) => {
				let hint = expected.filter(|&t| self.types.float(t).is_some());
				let operands = self.operands(&[*a, *b, *c], hint)?;
				let ty = self.common(&operands);
				if self.types.float(ty).is_none() && ty != Types::NEVER {
					return Err(error(span, format!("fma takes floats and not {}", self.name(ty))));
				}
				self.push(ExprKind::Fma(operands[0], operands[1], operands[2]), ty, span)
			}
			ast::ExprKind::Convert(conversion, target, value) => self.convert(*conversion, target, *value, span)?,
			ast::ExprKind::Make(ty, parts) => self.make(ty, parts, span)?,
			ast::ExprKind::Field(base, field) => {
				let base = self.expr(*base, None)?;
				self.field(base, *field, span)?
			}
			ast::ExprKind::Match(value, arms) => self.match_(*value, arms, expected, span)?,
		};
		self.expect(typed, expected)
	}

	fn int_literal(&mut self, value: i128, expected: Option<TypeId>, span: Span) -> Result<ExprId, Diagnostic> {
		let ty = expected.ok_or_else(|| no_type(span))?;
		match self.types.kind(ty) {
			TypeKind::Int(int) if int.contains(value) => Ok(self.push(ExprKind::Int(value), ty, span)),
			TypeKind::Int(_) => Err(error(span, format!("{value} does not fit in {}", self.name(ty)))),
			TypeKind::Float(_) => Err(error(span, format!("{} takes float literals ({value}.0)", self.name(ty)))),
			_ => Err(error(span, format!("expected {} but this is an integer literal", self.name(ty)))),
		}
	}

	fn float_literal(&mut self, text: &str, expected: Option<TypeId>, span: Span) -> Result<ExprId, Diagnostic> {
		let ty = expected.ok_or_else(|| no_type(span))?;
		let digits = text.replace('_', "");
		// NOTE: the parse rounds once to the nearest value of the target type
		let kind = match self.types.float(ty) {
			Some(FloatType::F32) => digits.parse::<f32>().ok().filter(|x| x.is_finite()).map(ExprKind::F32),
			Some(FloatType::F64) => digits.parse::<f64>().ok().filter(|x| x.is_finite()).map(ExprKind::F64),
			None => return Err(error(span, format!("expected {} but this is a float literal", self.name(ty)))),
		};
		let kind = kind.ok_or_else(|| error(span, format!("{text} is too large for {}", self.name(ty))))?;
		Ok(self.push(kind, ty, span))
	}

	fn call(&mut self, callee: ast::Name<'a>, args: &[ast::ExprId], span: Span) -> Result<ExprId, Diagnostic> {
		let id = match self.items.get(callee.text) {
			Some(&ItemRef::Fn(id)) => id,
			Some(ItemRef::Type(_)) => return Err(error(callee.span, format!("{} is a type (make builds its values)", callee.text))),
			None if self.lookup(callee.text).is_some() => return Err(error(callee.span, format!("{} is a local and not a function", callee.text))),
			None => return Err(error(callee.span, format!("unknown function {}", callee.text))),
		};
		let params = self.signatures[id.0 as usize].params.clone();
		let ret = self.signatures[id.0 as usize].ret;
		if args.len() != params.len() {
			return Err(error(span, format!("{} takes {} arguments and gets {}", callee.text, params.len(), args.len())));
		}
		let args = args.iter().zip(params).map(|(&arg, param)| self.expr(arg, Some(param))).collect::<Result<_, _>>()?;
		Ok(self.push(ExprKind::Call(id, args), ret, span))
	}

	// what set writes to: a local or a field of a place
	fn place(&mut self, id: ast::ExprId) -> Result<ExprId, Diagnostic> {
		let module = self.module;
		let node = &module.exprs[id.0 as usize];
		match &node.kind {
			ast::ExprKind::Name(_) => self.expr(id, None),
			ast::ExprKind::Field(base, field) => {
				let base = self.place(*base)?;
				self.field(base, *field, node.span)
			}
			_ => Err(error(node.span, "set writes to a local or a field of one")),
		}
	}

	fn field(&mut self, base: ExprId, field: ast::Name<'a>, span: Span) -> Result<ExprId, Diagnostic> {
		let ty = self.ty(base);
		let TypeKind::Struct(index) = *self.types.kind(ty) else {
			return Err(error(span, format!("{} has no fields", self.name(ty))));
		};
		let def = &self.types.structs[index as usize];
		let Some(position) = def.fields.iter().position(|(name, _)| name == field.text) else {
			return Err(error(field.span, format!("{} has no field {}", def.name, field.text)));
		};
		let field_ty = def.fields[position].1;
		Ok(self.push(ExprKind::Field(base, position as u32), field_ty, span))
	}

	fn loop_(&mut self, label: Option<ast::Name<'a>>, body: &[ast::ExprId], span: Span) -> Result<ExprId, Diagnostic> {
		if let Some(label) = label
			&& self.loops.iter().any(|scope| scope.label == Some(label.text))
		{
			return Err(error(label.span, format!("an enclosing loop already has the label :{}", label.text)));
		}
		let id = LoopId(self.next_loop);
		self.next_loop += 1;
		self.loops.push(LoopScope { label: label.map(|label| label.text), id, broken: false });
		self.scopes.push(Vec::new());
		let forms = body.iter().map(|&form| self.statement(form)).collect::<Result<_, _>>()?;
		self.scopes.pop();
		// a loop nothing breaks out of never ends
		let ty = if self.loops.pop().unwrap().broken { Types::UNIT } else { Types::NEVER };
		Ok(self.push(ExprKind::Loop(id, forms), ty, span))
	}

	// the loop a break/continue leaves: the innermost one or the one with the label
	fn target(&self, label: Option<ast::Name<'a>>, span: Span, what: &str) -> Result<usize, Diagnostic> {
		match label {
			None => self.loops.len().checked_sub(1).ok_or_else(|| error(span, format!("{what} outside a loop"))),
			Some(label) => self.loops.iter().rposition(|scope| scope.label == Some(label.text)).ok_or_else(|| error(label.span, format!("no enclosing loop has the label :{}", label.text))),
		}
	}

	fn unary(&mut self, op: Unary, operand: ast::ExprId, expected: Option<TypeId>, span: Span) -> Result<ExprId, Diagnostic> {
		if op == Unary::Not {
			let operand = self.expr(operand, Some(Types::BOOL))?;
			return Ok(self.push(ExprKind::Not(operand), Types::BOOL, span));
		}
		let hint = expected.filter(|&t| self.types.numeric(t));
		let operand = self.expr(operand, hint)?;
		let ty = self.ty(operand);
		let fits = match (op, self.types.kind(ty)) {
			(_, TypeKind::Never) => true,
			(Unary::Neg, TypeKind::Int(int)) => int.signed,
			(Unary::Neg, TypeKind::Float(_)) => true,
			(Unary::BitNot, TypeKind::Int(_)) => true,
			_ => false,
		};
		let (symbol, kind) = if op == Unary::Neg { ("neg", ExprKind::Neg(operand)) } else { ("~", ExprKind::BitNot(operand)) };
		if !fits {
			return Err(error(span, format!("{symbol} does not apply to {}", self.name(ty))));
		}
		Ok(self.push(kind, ty, span))
	}

	fn arith(&mut self, op: Arith, overflow: Overflow, left: ast::ExprId, right: ast::ExprId, expected: Option<TypeId>, span: Span) -> Result<ExprId, Diagnostic> {
		let hint = match overflow {
			Overflow::Checked => expected.and_then(|t| match *self.types.kind(t) {
				TypeKind::Opt(payload) if self.types.int(payload).is_some() => Some(payload),
				_ => None,
			}),
			_ => expected.filter(|&t| self.types.numeric(t)),
		};
		let operands = self.operands(&[left, right], hint)?;
		let ty = self.common(&operands);
		let fits = match self.types.kind(ty) {
			TypeKind::Never | TypeKind::Int(_) => true,
			TypeKind::Float(_) => overflow == Overflow::Trap && op != Arith::Rem,
			_ => false,
		};
		if !fits {
			return Err(error(span, format!("{} does not apply to {}", arith_symbol(op, overflow), self.name(ty))));
		}
		let result = if overflow == Overflow::Checked && ty != Types::NEVER { self.types.intern(TypeKind::Opt(ty)) } else { ty };
		Ok(self.push(ExprKind::Arith(op, overflow, operands[0], operands[1]), result, span))
	}

	fn convert(&mut self, conversion: Conversion, target: &ast::TypeExpr<'a>, value: ast::ExprId, span: Span) -> Result<ExprId, Diagnostic> {
		let target = self.resolve(target)?;
		let value = self.expr(value, None)?;
		let source = self.ty(value);
		let (from, to) = (self.types.kind(source), self.types.kind(target));
		let fits = match (conversion, from, to) {
			(_, TypeKind::Never, _) => true,
			(Conversion::Widen, TypeKind::Int(from), TypeKind::Int(to)) => to.holds(*from),
			(Conversion::Widen, TypeKind::Float(from), TypeKind::Float(to)) => from == to || *to == FloatType::F64,
			(Conversion::Narrow | Conversion::Saturate, TypeKind::Int(from), TypeKind::Int(to)) => !to.holds(*from),
			(Conversion::Narrow | Conversion::Saturate, TypeKind::Float(_), TypeKind::Int(_)) => true,
			(Conversion::Wrap, TypeKind::Int(_), TypeKind::Int(_)) => true,
			(Conversion::Float, TypeKind::Int(_), TypeKind::Float(_)) => true,
			(Conversion::Float, TypeKind::Float(FloatType::F64), TypeKind::Float(FloatType::F32)) => true,
			_ => false,
		};
		if !fits {
			let name = conversion_name(conversion);
			let hint = match (conversion, from, to) {
				(Conversion::Widen, TypeKind::Int(_), TypeKind::Int(_)) => " (some values do not fit: narrow/wrap/saturate)",
				(Conversion::Narrow | Conversion::Saturate, TypeKind::Int(_), TypeKind::Int(_)) => " (every value fits: widen)",
				_ => "",
			};
			return Err(error(span, format!("{name} does not convert {} to {}{hint}", self.name(source), self.name(target))));
		}
		Ok(self.push(ExprKind::Convert(conversion, value), target, span))
	}

	fn make(&mut self, ty: &ast::TypeExpr<'a>, parts: &[(ast::Name<'a>, Option<ast::ExprId>)], span: Span) -> Result<ExprId, Diagnostic> {
		let ty = self.resolve(ty)?;
		if let TypeKind::Struct(index) = *self.types.kind(ty) {
			let fields = self.types.structs[index as usize].fields.clone();
			let mut values: Vec<(u32, ExprId)> = Vec::with_capacity(parts.len());
			for (name, value) in parts {
				let Some(position) = fields.iter().position(|(field, _)| field == name.text) else {
					return Err(error(name.span, format!("{} has no field {}", self.name(ty), name.text)));
				};
				if values.iter().any(|&(given, _)| given == position as u32) {
					return Err(error(name.span, format!("field {} is given twice", name.text)));
				}
				let Some(value) = value else {
					return Err(error(name.span, format!("field {} needs a value", name.text)));
				};
				values.push((position as u32, self.expr(*value, Some(fields[position].1))?));
			}
			if let Some((missing, _)) = fields.iter().enumerate().find(|(position, _)| !values.iter().any(|&(given, _)| given == *position as u32)) {
				return Err(error(span, format!("field {} is missing", fields[missing].0)));
			}
			return Ok(self.push(ExprKind::Struct(values), ty, span));
		}
		let Some(cases) = self.types.cases(ty) else {
			return Err(error(span, format!("make builds structs/unions/opt values and not {}", self.name(ty))));
		};
		let [(name, value)] = parts else {
			return Err(error(span, format!("a {} value is exactly one case", self.name(ty))));
		};
		let Some(case) = cases.iter().position(|(case, _)| case == name.text) else {
			return Err(error(name.span, format!("{} has no case {}", self.name(ty), name.text)));
		};
		let payload_ty = cases[case].1;
		let payload = match value {
			Some(value) => self.expr(*value, Some(payload_ty))?,
			None if payload_ty == Types::UNIT => self.push(ExprKind::Unit, Types::UNIT, name.span),
			None => return Err(error(name.span, format!("case {} carries a value of type {}", name.text, self.name(payload_ty)))),
		};
		Ok(self.push(ExprKind::Case(case as u32, payload), ty, span))
	}

	fn match_(&mut self, value: ast::ExprId, arms: &[ast::Arm<'a>], expected: Option<TypeId>, span: Span) -> Result<ExprId, Diagnostic> {
		let value = self.expr(value, None)?;
		let ty = self.ty(value);
		let Some(cases) = self.types.cases(ty) else {
			return Err(error(self.exprs[value.0 as usize].span, format!("match reads unions and opt values and not {}", self.name(ty))));
		};
		let mut case_of_arm = Vec::with_capacity(arms.len());
		let mut arm_of_case: Vec<Option<usize>> = vec![None; cases.len()];
		for (index, arm) in arms.iter().enumerate() {
			let Some(case) = cases.iter().position(|(case, _)| case == arm.case.text) else {
				return Err(error(arm.case.span, format!("{} has no case {}", self.name(ty), arm.case.text)));
			};
			if arm_of_case[case].replace(index).is_some() {
				return Err(error(arm.case.span, format!("case {} has two arms", arm.case.text)));
			}
			case_of_arm.push(case);
		}
		if let Some(missing) = arm_of_case.iter().position(Option::is_none) {
			return Err(error(span, format!("no arm for case {}", cases[missing].0)));
		}
		// like operands(): an arm with a type of its own goes first
		let mut order: Vec<usize> = (0..arms.len()).collect();
		if expected.is_none() {
			order.sort_by_key(|&index| self.untyped(arms[index].body));
		}
		let mut result = expected;
		let mut checked: Vec<Option<Arm>> = (0..arms.len()).map(|_| None).collect();
		for index in order {
			self.scopes.push(Vec::new());
			let binding = match arms[index].binding {
				Some(name) => Some(self.declare_local(name, cases[case_of_arm[index]].1)?),
				None => None,
			};
			let body = self.expr(arms[index].body, result)?;
			self.scopes.pop();
			if result.is_none() && self.ty(body) != Types::NEVER {
				result = Some(self.ty(body));
			}
			checked[index] = Some(Arm { binding, body });
		}
		let in_case_order = arm_of_case.iter().map(|arm| checked[arm.unwrap()].take().unwrap()).collect();
		Ok(self.push(ExprKind::Match(value, in_case_order), result.unwrap_or(Types::NEVER), span))
	}
}
