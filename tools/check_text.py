#!/usr/bin/env python3
# check tracked text files against docs/STYLE.md
#
#   python tools/check_text.py                  every tracked file
#   python tools/check_text.py --staged         the index (pre-commit)
#   python tools/check_text.py --message FILE   a commit message (commit-msg)
#   python tools/check_text.py PATH...          given files
#
# exit status 1 lists every violation as path:line: message

import pathlib
import re
import subprocess
import sys
import unicodedata

ROOT = pathlib.Path(__file__).resolve().parent.parent

LINE_COMMENT = {".c": "//", ".h": "//", ".cpp": "//", ".hpp": "//", ".rs": "//", ".zig": "//", ".core": "//", ".py": "#", ".toml": "#", ".yml": "#", ".yaml": "#", ".sh": "#"}
HASH_COMMENT_NAMES = {".gitignore", ".gitattributes", ".editorconfig", ".clang-format"}

# YAML forbids tabs (.clang-format is YAML too)
SPACE_INDENTED = {".yml", ".yaml"}
SPACE_INDENTED_NAMES = {".clang-format"}

# data the harness compares byte for byte
DATA = re.compile(r"^bench/programs/[^/]+/expected-")

TAGS = ("NOTE:", "TODO:", "BUG:", "FD")

# a dash between words (arithmetic like 1 - 2 passes)
DASH_ASIDE = re.compile(r"[A-Za-z,;:)] -{1,2} [A-Za-z(]")

# a clause hung on a comma ("X, and Y")
CONNECTOR = re.compile(r",\s+(and|but|so|which|while|plus|since|though|yet|or|nor|whereas)\b", re.IGNORECASE)


def git(*args):
	return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=True).stdout


def tracked():
	return [p for p in git("ls-files", "-z").decode().split("\0") if p]


def staged():
	names = git("diff", "--cached", "--name-only", "--diff-filter=ACMR", "-z").decode().split("\0")
	return [(p, git("show", f":{p}")) for p in names if p]


# ================================================================
# PROSE
# ================================================================

def mask(text):
	# code/quotes/links keep their own punctuation
	text = re.sub(r"`[^`]*`", "``", text)
	text = re.sub(r"\]\([^)]*\)", "]()", text)
	text = re.sub(r"https?://\S+", "URL", text)
	return re.sub(r'"[^"]*"', '""', text)


def without_parentheses(text):
	depth = 0
	kept = []
	for character in text:
		if character == "(":
			depth += 1
		elif character == ")":
			depth = max(0, depth - 1)
		elif depth == 0:
			kept.append(character)
	return "".join(kept)


def prose_problems(text):
	text = mask(text)
	problems = []
	if ";" in text:
		problems.append("semicolon (use parentheses or a new sentence)")
	connector = CONNECTOR.search(text)
	if connector:
		problems.append(f"', {connector.group(1)}' joins a clause (split the sentence or use parentheses)")
	if DASH_ASIDE.search(text):
		problems.append("dash used as an aside (use parentheses)")
	for sentence in re.split(r"(?<=[.!?:])\s+", without_parentheses(text)):
		commas = len(re.findall(r",", sentence)) - len(re.findall(r"(?<=\d),(?=\d)", sentence))
		if commas > 1:
			problems.append(f"{commas} commas in one sentence (restructure it): {sentence.strip()[:60]!r}")
	return problems


# ================================================================
# FILES
# ================================================================

def punctuation_problem(line):
	for character in line:
		if ord(character) < 128:
			continue
		if unicodedata.category(character)[0] not in "LM":
			return f"non-ASCII {unicodedata.name(character, hex(ord(character)))} (U+{ord(character):04X})"
	return None


def comment_problems(lines, marker):
	# full-line comments only (a block is a run of consecutive comment lines)
	problems = []
	block = []

	def finish():
		if not block:
			return
		first_number, first = block[0]
		last_number, last = block[-1]
		word = first.split()[0] if first.split() else ""
		if re.fullmatch(r"[A-Z][a-z']+[,:]?", word) and not first.startswith(TAGS):
			problems.append((first_number, f"comment starts with a capital: {first[:40]!r}"))
		if last.endswith(".") and not last.endswith("..."):
			problems.append((last_number, "comment ends with a period"))
		for problem in prose_problems(" ".join(text for _, text in block)):
			problems.append((first_number, problem))
		block.clear()

	for number, line in enumerate(lines, 1):
		stripped = line.strip()
		if stripped.startswith(marker) and not stripped.startswith("#!"):
			text = stripped[len(marker):].strip()
			# dividers, titles and blank comment lines end a block
			if not text or set(text) <= {"="} or text.isupper():
				finish()
				continue
			block.append((number, text))
		else:
			finish()
	finish()
	return problems


def markdown_problems(lines):
	problems = []
	fenced = False
	for number, line in enumerate(lines, 1):
		if line.startswith("```"):
			fenced = not fenced
			continue
		if fenced:
			continue
		if line.startswith("#") and line.rstrip().endswith("."):
			problems.append((number, "heading ends with a period"))
		cells = line.strip().strip("|").split("|") if line.lstrip().startswith("|") else [line]
		for cell in cells:
			for problem in prose_problems(cell):
				problems.append((number, problem))
	return problems


def check(path, data):
	if b"\0" in data or DATA.match(path):
		return []
	try:
		text = data.decode("utf-8")
	except UnicodeDecodeError as error:
		return [(1, f"not UTF-8 ({error.reason})")]
	problems = []
	if "\r" in text:
		problems.append((1, "CR line endings"))
	if text and not text.endswith("\n"):
		problems.append((text.count("\n") + 1, "no final newline"))

	name = pathlib.PurePosixPath(path)
	suffix = name.suffix
	spaces_allowed = suffix in SPACE_INDENTED or name.name in SPACE_INDENTED_NAMES
	lines = text.split("\n")
	fenced = False
	for number, line in enumerate(lines, 1):
		if line.startswith("```"):
			fenced = not fenced
		problem = punctuation_problem(line)
		if problem:
			problems.append((number, problem))
		if line != line.rstrip():
			problems.append((number, "trailing whitespace"))
		indent = line[: len(line) - len(line.lstrip())]
		if " " in indent and not spaces_allowed and not fenced and line.strip():
			problems.append((number, "indented with spaces"))

	marker = LINE_COMMENT.get(suffix) or ("#" if name.name in HASH_COMMENT_NAMES else None)
	if marker:
		problems += comment_problems(lines, marker)
	if suffix == ".md":
		problems += markdown_problems(lines)
	return problems


def check_message(text):
	# git's own comment lines are not part of the message
	lines = [line for line in text.split("\n") if not line.startswith("#")]
	problems = [(number, problem) for number, line in enumerate(lines, 1) if (problem := punctuation_problem(line))]
	paragraphs = re.split(r"\n\s*\n", "\n".join(lines).strip())
	for paragraph in paragraphs:
		for problem in prose_problems(" ".join(paragraph.split())):
			problems.append((1, problem))
	return problems


def main(argv):
	if argv[:1] == ["--message"]:
		results = [(argv[1], check_message(pathlib.Path(argv[1]).read_text(encoding="utf-8")))]
	else:
		if "--staged" in argv:
			files = staged()
		else:
			paths = [a for a in argv if not a.startswith("-")] or tracked()
			files = [(p, (ROOT / p).read_bytes()) for p in paths]
		results = [(path, check(path.replace("\\", "/"), data)) for path, data in files]
	count = 0
	for path, problems in results:
		for number, message in problems:
			print(f"{path}:{number}: {message}")
			count += 1
	if count:
		print(f"{count} problem(s)")
		return 1
	return 0


if __name__ == "__main__":
	sys.exit(main(sys.argv[1:]))
