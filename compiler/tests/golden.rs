// every tests/golden/**/*.core goes through the reader/checker/interpreter. Its
// transcript must match the .out file next to it. CADOU_BLESS=1 rewrites the .out files
// (review the diff before committing it)

use std::fs;
use std::path::{Path, PathBuf};

fn programs(directory: &Path, found: &mut Vec<PathBuf>) {
	let mut entries: Vec<PathBuf> = fs::read_dir(directory).unwrap().map(|entry| entry.unwrap().path()).collect();
	entries.sort();
	for path in entries {
		if path.is_dir() {
			programs(&path, found);
		} else if path.extension().is_some_and(|extension| extension == "core") {
			found.push(path);
		}
	}
}

#[test]
fn golden() {
	let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/golden");
	let mut paths = Vec::new();
	programs(&root, &mut paths);
	assert!(!paths.is_empty(), "no programs under {}", root.display());
	let bless = std::env::var_os("CADOU_BLESS").is_some();
	let mut failures = Vec::new();
	for path in &paths {
		let source = fs::read_to_string(path).unwrap();
		let actual = cadou::with_stack(|| cadou::transcript(&source));
		let expected_path = path.with_extension("out");
		if bless {
			fs::write(&expected_path, &actual).unwrap();
			continue;
		}
		let expected = fs::read_to_string(&expected_path).unwrap_or_default();
		if actual != expected {
			let name = path.strip_prefix(&root).unwrap().display();
			failures.push(format!("{name}\n--- expected\n{expected}--- actual\n{actual}"));
		}
	}
	assert!(failures.is_empty(), "{} of {} programs differ\n\n{}", failures.len(), paths.len(), failures.join("\n"));
}
