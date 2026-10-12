// cadou run <file>: checks a program in the core notation and runs it in the
// reference interpreter. cadou check <file> only checks it

use std::io::Write;
use std::process::exit;

use cadou::diag::line_col;
use cadou::eval::{End, run};

// a trap ends the program with this status (spec 8.1)
const TRAP_STATUS: i32 = 101;

fn main() {
	let args: Vec<String> = std::env::args().collect();
	let (command, path) = match args.as_slice() {
		[_, command, path] if command == "run" || command == "check" => (command.as_str(), path.as_str()),
		_ => {
			eprintln!("usage: cadou run|check <file>");
			exit(2);
		}
	};
	let source = std::fs::read_to_string(path).unwrap_or_else(|error| {
		eprintln!("{path}: {error}");
		exit(2);
	});
	let status = cadou::with_stack(|| {
		let program = match cadou::compile(&source) {
			Ok(program) => program,
			Err(diagnostic) => {
				let (line, column) = line_col(&source, diagnostic.span.start);
				eprintln!("{path}:{line}:{column}: error: {}", diagnostic.message);
				return 1;
			}
		};
		if command == "check" {
			return 0;
		}
		let outcome = run(&program);
		print!("{}", outcome.output);
		match outcome.end {
			End::Exit(status) => status,
			End::Trap(trap) => {
				let (line, column) = line_col(&source, trap.span.start);
				eprintln!("{path}:{line}:{column}: trap: {} (spec {})", trap.message, trap.section);
				TRAP_STATUS
			}
		}
	});
	std::io::stdout().flush().ok();
	exit(status);
}
