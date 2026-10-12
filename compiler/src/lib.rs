// the bootstrap compiler (ADR 0001). So far the reader/checker/reference interpreter
// of the core (ADR 0004)

pub mod ast;
pub mod check;
pub mod diag;
pub mod eval;
pub mod ops;
pub mod read;
pub mod typed;

use diag::{Diagnostic, line_col};
use eval::End;

// eval::MAX_DEPTH levels take about 85 MB (reserved and touched as needed)
const STACK: usize = 256 << 20;

pub fn compile(source: &str) -> Result<typed::Program, Diagnostic> {
	let items = read::read(source)?;
	let module = ast::parse(&items)?;
	check::check(&module)
}

// a run as text: the output then `exit N` or the trap. A program that does not check
// gives its first error instead (golden tests compare this)
pub fn transcript(source: &str) -> String {
	let program = match compile(source) {
		Ok(program) => program,
		Err(diagnostic) => {
			let (line, column) = line_col(source, diagnostic.span.start);
			return format!("{line}:{column}: error: {}\n", diagnostic.message);
		}
	};
	let outcome = eval::run(&program);
	let end = match outcome.end {
		End::Exit(status) => format!("exit {status}\n"),
		End::Trap(trap) => {
			let (line, column) = line_col(source, trap.span.start);
			format!("{line}:{column}: trap: {} (spec {})\n", trap.message, trap.section)
		}
	};
	outcome.output + &end
}

// the passes recurse and a deep program needs more stack than a main thread has
pub fn with_stack<T: Send>(work: impl FnOnce() -> T + Send) -> T {
	std::thread::scope(|scope| {
		let thread = std::thread::Builder::new().stack_size(STACK).spawn_scoped(scope, work).expect("a thread for the interpreter");
		thread.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic))
	})
}
