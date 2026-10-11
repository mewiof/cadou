# Style

Applies to every text file in the repository. `python tools/check_text.py` checks the mechanical parts (used by CI).

## Formatting

- Tabs for indentation shown 4 wide. YAML is the exception (the format forbids tabs).
- LF line endings/a final newline/no trailing whitespace.
- ASCII punctuation and symbols only. Letters with diacritics are fine in names.
- No column limit. C and C++ follow [.clang-format](../.clang-format). Rust follows [rustfmt.toml](../rustfmt.toml).
- Never run a formatter over lines you did not change. Formatter versions disagree and a format pass buries the real change in rewritten lines. Indent by hand.

## Comments

- `//` in C/C++/Rust/Zig and `#` in Python/TOML/YAML. No `/* */` blocks and no `///` doc comments.
- Start lowercase. No period at the end of a comment. A multi-line comment uses one `//` per line (periods and capitals only between its sentences).
- Short. Say why rather than what. A mental model where the code is not obvious. "e.g." lists for examples.
- Tags:
	- `NOTE:` a requirement/side effect/trap
	- `TODO:` work not done yet
	- `BUG:` a known issue being worked around
	- `FD` next to a forward declaration
- Major blocks get a 64-character divider with a short uppercase title:

```c
// ================================================================
// HARDWARE COUNTERS
// ================================================================
```

## Naming

C/C++/Python follow the rules below. Rust and Zig keep their own conventions (rustc's lints and the Zig style guide). Naming in CADOU itself is not decided yet.

- C/C++: types/enum members/template parameters/functions are `PascalCase`. Locals/parameters/members/statics are `camelCase`. Constants and macros are `UPPER_SNAKE` unless the constant names a value instead of tuning a number (then `PascalCase`).
- A leading `_` marks non-public fields and file-scope statics (no `m_`/`s_`/`g_`). C code drops it because file-scope names starting with `_` are reserved there (C17 7.1.3).
- Casing goes word by word. An initialism is all caps only when every letter starts a different word (`CPU`/`UI`/`LOD`). A shortened single word is a word (`Id`/`Idx`). Plural `s` stays lowercase (`IDs`). Prefer full words.
- `#pragma once` instead of include guards.
- Python: types in `PascalCase`. Functions/variables/modules/files in `snake_case`. Constants in `UPPER_SNAKE`.

## Prose

- American English. Commas and periods still go outside quotation marks: "Texture Loaded", "Server Connected".
- Short sentences with few commas. Restructure a sentence rather than piling up commas (one per sentence at most, outside parentheses and quotes).
- No semicolons. No clause hung on a comma (", and"/", so"/", which"). Write a second sentence or use short parentheses like "(used by CI)".
- A list inside a sentence uses slashes or `+`: C/C++/Rust/Zig.
- Asides go in parentheses and never between dashes.
- Headings are labels. They never end with a period.
- One fact lives in one place. Elsewhere it gets a link.
- A numbered section that something cites keeps its number. Rewrite inside it but never renumber.
- A claim about another language or tool names its source with a version or a date.

## Commits

- Plain imperative subject ("Add the n-body benchmark"). A body only when it carries something (reasons/numbers/deferred work).
- One change per commit. Every commit builds and passes its tests on its own.
