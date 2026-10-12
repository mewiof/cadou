// the reader of the core notation (spec/notation.md): lists in parentheses and atoms.
// `//` starts a comment that runs to the end of its line

use crate::diag::{Diagnostic, Span, error};

// NOTE: the passes after the reader recurse once per level. The limit keeps any input
// from overflowing their stack
pub const MAX_DEPTH: usize = 200;

#[derive(Debug)]
pub enum Sexp<'a> {
	Atom(&'a str, Span),
	List(Vec<Sexp<'a>>, Span),
}

impl Sexp<'_> {
	pub fn span(&self) -> Span {
		match self {
			Sexp::Atom(_, span) | Sexp::List(_, span) => *span,
		}
	}
}

pub fn read(source: &str) -> Result<Vec<Sexp<'_>>, Diagnostic> {
	let mut reader = Reader { source, bytes: source.as_bytes(), pos: 0 };
	let mut items = Vec::new();
	loop {
		reader.skip_blank();
		if reader.pos == reader.bytes.len() {
			return Ok(items);
		}
		items.push(reader.sexp(0)?);
	}
}

struct Reader<'a> {
	source: &'a str,
	bytes: &'a [u8],
	pos: usize,
}

impl<'a> Reader<'a> {
	fn comment_starts(&self) -> bool {
		self.bytes[self.pos] == b'/' && self.bytes.get(self.pos + 1) == Some(&b'/')
	}

	fn skip_blank(&mut self) {
		while self.pos < self.bytes.len() {
			if self.comment_starts() {
				while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
					self.pos += 1;
				}
			} else if matches!(self.bytes[self.pos], b' ' | b'\t' | b'\n' | b'\r') {
				self.pos += 1;
			} else {
				return;
			}
		}
	}

	fn sexp(&mut self, depth: usize) -> Result<Sexp<'a>, Diagnostic> {
		let start = self.pos;
		match self.bytes[start] {
			b'(' => {
				if depth == MAX_DEPTH {
					return Err(error(Span::new(start, start + 1), format!("lists nest deeper than {MAX_DEPTH} levels")));
				}
				self.pos += 1;
				let mut items = Vec::new();
				loop {
					self.skip_blank();
					match self.bytes.get(self.pos) {
						None => return Err(error(Span::new(start, start + 1), "this ( is never closed")),
						Some(b')') => {
							self.pos += 1;
							return Ok(Sexp::List(items, Span::new(start, self.pos)));
						}
						Some(_) => items.push(self.sexp(depth + 1)?),
					}
				}
			}
			b')' => Err(error(Span::new(start, start + 1), "this ) closes nothing")),
			_ => {
				while self.pos < self.bytes.len() && !matches!(self.bytes[self.pos], b' ' | b'\t' | b'\n' | b'\r' | b'(' | b')') && !self.comment_starts() {
					self.pos += 1;
				}
				Ok(Sexp::Atom(&self.source[start..self.pos], Span::new(start, self.pos)))
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn shape(sexp: &Sexp) -> String {
		match sexp {
			Sexp::Atom(text, _) => text.to_string(),
			Sexp::List(items, _) => format!("[{}]", items.iter().map(shape).collect::<Vec<_>>().join(" ")),
		}
	}

	#[test]
	fn atoms_lists_comments() {
		let items = read("(fn f () i32 // the answer\n\t42)//end\n(+ -1 x)").unwrap();
		assert_eq!(items.iter().map(shape).collect::<Vec<_>>(), ["[fn f [] i32 42]", "[+ -1 x]"]);
		assert_eq!(items[1].span(), Span::new(37, 45));
	}

	#[test]
	fn division_is_an_atom() {
		let items = read("(/ a b)").unwrap();
		assert_eq!(shape(&items[0]), "[/ a b]");
	}

	#[test]
	fn unbalanced() {
		assert_eq!(read("(a (b)").unwrap_err().message, "this ( is never closed");
		assert_eq!(read("(a))").unwrap_err().span, Span::new(3, 4));
	}

	#[test]
	fn depth_limit() {
		let deep = "(".repeat(MAX_DEPTH + 1) + &")".repeat(MAX_DEPTH + 1);
		assert!(read(&deep).unwrap_err().message.starts_with("lists nest deeper"));
		let fine = "(".repeat(MAX_DEPTH) + &")".repeat(MAX_DEPTH);
		assert!(read(&fine).is_ok());
	}
}
