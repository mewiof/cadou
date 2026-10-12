// source spans and the errors the reader/parser/checker report

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
	pub start: usize,
	pub end: usize,
}

impl Span {
	pub fn new(start: usize, end: usize) -> Span {
		Span { start, end }
	}
}

#[derive(Debug)]
pub struct Diagnostic {
	pub span: Span,
	pub message: String,
}

pub fn error(span: Span, message: impl Into<String>) -> Diagnostic {
	Diagnostic { span, message: message.into() }
}

// line and column of a byte offset (both from 1, a tab is one column)
pub fn line_col(source: &str, offset: usize) -> (usize, usize) {
	let before = &source[..offset.min(source.len())];
	let line = before.matches('\n').count() + 1;
	let column = before.rsplit('\n').next().map_or(0, |text| text.chars().count()) + 1;
	(line, column)
}
