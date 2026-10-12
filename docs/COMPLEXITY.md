# Complexity budget

Five numbers that every change touching the language updates. An addition states what it costs here and why it is worth it. A removal needs no justification.

| metric | counted as | now |
| --- | --- | --- |
| keywords | reserved words in the lexer | 0 |
| grammar productions | rules in the grammar in `spec/` | 0 |
| spec size | words in `spec/` without code blocks | 1405 (draft of the core) |
| core compiler size | non-blank non-comment lines in `compiler/` without tests | 0 |
| language features | rows in the table below | 0 |

There are no limits yet. The phase 2 core sets them from its own numbers. Raising one later takes an ADR.

## Features

| feature | added | cost (keywords/productions/spec words/compiler lines) | why it pays |
| --- | --- | --- | --- |
