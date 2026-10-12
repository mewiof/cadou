// the checked core: names resolved/every expression typed/every form legal. The
// interpreter runs it and the compiler will lower it (ADR 0004)

use std::collections::HashMap;

use crate::ast::{Bits, Compare, Conversion, Overflow};
use crate::diag::Span;
use crate::ops::{Arith, IntType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FloatType {
	F32,
	F64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeKind {
	Unit,
	Bool,
	// the type of break/continue/return and of loops nothing breaks out of (no value has it)
	Never,
	Int(IntType),
	Float(FloatType),
	Struct(u32),
	Union(u32),
	Opt(TypeId),
}

pub struct StructDef {
	pub name: String,
	pub fields: Vec<(String, TypeId)>,
}

pub struct UnionDef {
	pub name: String,
	pub cases: Vec<(String, TypeId)>,
}

// every type once (equal types get equal ids)
pub struct Types {
	kinds: Vec<TypeKind>,
	ids: HashMap<TypeKind, TypeId>,
	pub structs: Vec<StructDef>,
	pub unions: Vec<UnionDef>,
}

const fn int(signed: bool, bits: u32) -> TypeKind {
	TypeKind::Int(IntType { signed, bits })
}

const BUILTIN: [(&str, TypeKind); 13] = [
	("unit", TypeKind::Unit),
	("bool", TypeKind::Bool),
	("never", TypeKind::Never),
	("i8", int(true, 8)),
	("i16", int(true, 16)),
	("i32", int(true, 32)),
	("i64", int(true, 64)),
	("u8", int(false, 8)),
	("u16", int(false, 16)),
	("u32", int(false, 32)),
	("u64", int(false, 64)),
	("f32", TypeKind::Float(FloatType::F32)),
	("f64", TypeKind::Float(FloatType::F64)),
];

impl Default for Types {
	fn default() -> Types {
		let mut types = Types { kinds: Vec::new(), ids: HashMap::new(), structs: Vec::new(), unions: Vec::new() };
		for (_, kind) in BUILTIN {
			types.intern(kind);
		}
		types
	}
}

impl Types {
	pub const UNIT: TypeId = TypeId(0);
	pub const BOOL: TypeId = TypeId(1);
	pub const NEVER: TypeId = TypeId(2);
	pub const I32: TypeId = TypeId(5);

	// the type a built-in name stands for (never cannot be written)
	pub fn builtin(name: &str) -> Option<TypeId> {
		BUILTIN.iter().position(|(builtin, _)| *builtin == name && name != "never").map(|index| TypeId(index as u32))
	}

	pub fn intern(&mut self, kind: TypeKind) -> TypeId {
		if let Some(&id) = self.ids.get(&kind) {
			return id;
		}
		let id = TypeId(self.kinds.len() as u32);
		self.kinds.push(kind.clone());
		self.ids.insert(kind, id);
		id
	}

	pub fn kind(&self, id: TypeId) -> &TypeKind {
		&self.kinds[id.0 as usize]
	}

	pub fn int(&self, id: TypeId) -> Option<IntType> {
		match self.kind(id) {
			TypeKind::Int(int) => Some(*int),
			_ => None,
		}
	}

	pub fn float(&self, id: TypeId) -> Option<FloatType> {
		match self.kind(id) {
			TypeKind::Float(float) => Some(*float),
			_ => None,
		}
	}

	pub fn numeric(&self, id: TypeId) -> bool {
		matches!(self.kind(id), TypeKind::Int(_) | TypeKind::Float(_))
	}

	// the cases of a union or an opt (none first)
	pub fn cases(&self, id: TypeId) -> Option<Vec<(String, TypeId)>> {
		match self.kind(id) {
			TypeKind::Union(index) => Some(self.unions[*index as usize].cases.clone()),
			TypeKind::Opt(payload) => Some(vec![("none".to_string(), Types::UNIT), ("some".to_string(), *payload)]),
			_ => None,
		}
	}

	pub fn name(&self, id: TypeId) -> String {
		match self.kind(id) {
			TypeKind::Struct(index) => self.structs[*index as usize].name.clone(),
			TypeKind::Union(index) => self.unions[*index as usize].name.clone(),
			TypeKind::Opt(payload) => format!("(opt {})", self.name(*payload)),
			kind => BUILTIN.iter().find(|(_, builtin)| builtin == kind).unwrap().0.to_string(),
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExprId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FnId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopId(pub u32);

// a local's index in its function's frame (parameters come first)
pub type Slot = u32;

pub struct Expr {
	pub kind: ExprKind,
	pub ty: TypeId,
	pub span: Span,
}

pub enum ExprKind {
	Int(i128),
	F32(f32),
	F64(f64),
	Bool(bool),
	Unit,
	Local(Slot),
	Call(FnId, Vec<ExprId>),
	Let(Slot, ExprId),
	// a place (a local or a field of one) and the value
	Set(ExprId, ExprId),
	Block(Vec<ExprId>),
	If(ExprId, ExprId, Option<ExprId>),
	Loop(LoopId, Vec<ExprId>),
	Break(LoopId),
	Continue(LoopId),
	Return(Option<ExprId>),
	Discard(ExprId),
	Out(ExprId),
	Neg(ExprId),
	Not(ExprId),
	BitNot(ExprId),
	Arith(Arith, Overflow, ExprId, ExprId),
	Bits(Bits, ExprId, ExprId),
	Compare(Compare, ExprId, ExprId),
	And(ExprId, ExprId),
	Or(ExprId, ExprId),
	Fma(ExprId, ExprId, ExprId),
	// the target is the expression's type
	Convert(Conversion, ExprId),
	// (field index/value) in written order (the order of evaluation)
	Struct(Vec<(u32, ExprId)>),
	// a union or opt case with its payload
	Case(u32, ExprId),
	Field(ExprId, u32),
	// one arm per case in case order
	Match(ExprId, Vec<Arm>),
}

pub struct Arm {
	pub binding: Option<Slot>,
	pub body: ExprId,
}

pub struct Function {
	pub name: String,
	pub params: Vec<TypeId>,
	pub ret: TypeId,
	pub frame: u32,
	pub body: ExprId,
	pub span: Span,
}

pub struct Program {
	pub types: Types,
	pub functions: Vec<Function>,
	pub exprs: Vec<Expr>,
	pub main: FnId,
}

impl Program {
	pub fn expr(&self, id: ExprId) -> &Expr {
		&self.exprs[id.0 as usize]
	}
}
