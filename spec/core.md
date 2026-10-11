# Core semantics

Draft from 2026-10-11. Meaning comes before syntax ([PRINCIPLES.md](../docs/PRINCIPLES.md)). Examples use the parenthesized notation that the reference interpreter reads ([ADR 0004](../docs/decisions/0004-reference-interpreter.md)). It is a test format and not the surface syntax.

## 1. Scope

The phase 2 core: modules/functions/booleans/integers/floats/structs/tagged unions/arrays/slices/pointers/control flow/calls into C.

Outside the core for now: generics and compile-time execution/allocators other than one global heap/threads/strings beyond byte slices. Use after free has no static answer before phase 3 (section 11.1).

## 2. Modules

A program is a set of modules. A module holds items: functions/types/constants/C declarations. An item stays private to its module unless it is exported. Imports are explicit. An import cycle is an error.

The program starts at `main` in the root module. `main` takes nothing and returns an `i32` exit status or nothing (status 0).

## 3. Types

### 3.1 Scalars

| type | values |
| --- | --- |
| `bool` | `true`/`false` |
| `i8` `i16` `i32` `i64` | two's complement integers of that width |
| `u8` `u16` `u32` `u64` | unsigned integers of that width |
| `f32` `f64` | IEEE 754 binary32 and binary64 |
| `unit` | one value |

No integer type changes width with the target. Lengths and indices have one fixed type (section 11.3).

### 3.2 Pointers

`(ptr T)` points to one `T` and is never null. `(opt (ptr T))` may be null at no extra size. Safe code has no pointer arithmetic. Raw pointers with arithmetic belong to unchecked code (section 8.2).

### 3.3 Arrays and slices

`(array N T)` holds N values of `T` in place. N is known at compile time. `(slice T)` is a pointer plus a length. Both index from 0.

### 3.4 Structs

A struct has named fields. Fields lie in declared order at their natural alignment (the layout C gives the same declaration). Packed and over-aligned layouts have to be asked for.

### 3.5 Tagged unions

A tagged union is one of several named cases. Each case has a payload type. Only `match` reads the tag. The layout belongs to the compiler (a null pointer may encode the empty case of `(opt (ptr T))` e.g.).

### 3.6 Optionals

`(opt T)` is the tagged union of `none` and `(some T)`.

### 3.7 Functions

A function value is the address of code. There are no closures.

### 3.8 Conversions

No value changes type without a written operation (section 6.4).

## 4. Memory

### 4.1 Places

A place is memory that holds a value: a local/a global/a heap object or a field or element of one of them. Reading a place copies its value. Writing replaces it.

### 4.2 Allocations

Every place lies in an allocation. A pointer is the allocation it came from (its provenance) plus an offset. The interpreter keeps both. Compiled code keeps the address only.

### 4.3 Initialization

Safe code never reads a place before writing it (checked at compile time). The interpreter also tracks uninitialized bytes and stops when unchecked code reads one.

### 4.4 The heap

One global heap with allocate and free. Before phase 3 nothing stops a double free or a read after free at compile time. The interpreter detects both and traps.

## 5. Evaluation

Operands evaluate left to right before their operation. Arguments evaluate left to right before the call. An assignment evaluates the place first (index operands included) and the value second. The bounds check of the place comes after both and before the write.

The core has no compound assignment. One in the surface syntax lowers to `(set x (+ x e))` with the place of `x` evaluated once. The rules above then read the old value of `x` before `e` runs. C leaves that order open and Rust evaluates `e` first. Reading first lets a value kept in a register survive a call inside `e` without any aliasing promise ([noalias2.md](../docs/research/noalias2.md)).

`if` and `match` are expressions. A `match` covers every case or fails to compile. `loop` repeats until a `break` (labels pick the loop). `return` leaves the function.

## 6. Operations

### 6.1 Integer arithmetic

`+`/`-`/`*` take two operands of one type. A result that does not fit is an overflow. A checked build traps on it (section 11.4 has the unchecked side). Separate operations state another intent:

| form | on overflow |
| --- | --- |
| `(wrapping+ a b)` | keeps the low bits |
| `(saturating+ a b)` | clamps to the type's range |
| `(checked+ a b)` | gives `none` |
| `(unchecked+ a b)` | undefined (section 8.2) |

### 6.2 Division

Division truncates toward zero. Dividing by zero traps. The minimum value divided by -1 overflows (section 6.1).

### 6.3 Shifts and bits

A shift by the bit width or more traps. Bitwise operations are defined for every input.

### 6.4 Conversions

- widening to a type that holds every source value: exact
- narrowing: checked (traps when the value does not fit) or truncating (keeps the low bits)
- integer to float: rounds to nearest with ties to even
- float to integer: traps on NaN or out of range. A saturating form clamps instead (NaN gives 0)

### 6.5 Floats

IEEE 754 arithmetic rounded to nearest with ties to even. Every operation rounds once. `(+ (* a b) c)` rounds twice. Fused multiply-add is an operation of its own. Nothing reorders float arithmetic. NaN payloads are unspecified.

### 6.6 Comparisons

Both operands have one type. `and`/`or` evaluate their right operand only when the left one does not decide.

### 6.7 Indexing and slicing

`(index a i)` traps unless `0 <= i < length`. `(slice a i j)` traps unless `0 <= i <= j <= length`.

## 7. Errors

An error is a value. A function that can fail returns a tagged union of its result and an error. A value of a must-use type cannot be dropped without a written discard.

## 8. Traps and undefined behavior

### 8.1 Traps

A trap ends the program with a nonzero status and a message that names the check. Nothing unwinds.

### 8.2 Unchecked code

Undefined behavior exists only in unchecked code: a marked block or a build mode that removes checks. Both are greppable. In unchecked code these are undefined:

- an index or a slice bound outside section 6.7 once its check is gone
- dereferencing a raw pointer that does not point into a live allocation
- an overflow of an unchecked operation (section 6.1)
- a call into C that breaks the C function's contract

### 8.3 What the interpreter checks

The interpreter stops with a report on everything in section 8.2 except C contracts. It never continues past undefined behavior.

## 9. Calls into C

A C function gets declared with C types: fixed-width integers/`f32`/`f64`/pointers/structs with C layout. Calling it is unchecked code. The call follows the platform's C calling convention.

## 10. Examples

The sum of a slice with `i64` indices (section 11.3 keeps that open). Overflow traps in a checked build:

```
(fn sum ((values (slice i64))) i64
	(let total i64 0)
	(let i i64 0)
	(loop
		(if (== i (len values)) (break))
		(set total (+ total (index values i)))
		(set i (+ i 1)))
	total)
```

A tagged union and a match that covers both cases:

```
(union Shape
	(circle f64)
	(square f64))

(fn area ((shape Shape)) f64
	(match shape
		((circle r) (* 3.141592653589793 (* r r)))
		((square side) (* side side))))
```

## 11. Open questions

### 11.1 Temporal safety

Phase 3 compares at least three models ([PRINCIPLES.md 3.3](../docs/PRINCIPLES.md#33-safety)). Until then section 4.4 holds.

### 11.2 Aliasing of references

Whether mutable references promise no aliasing by default. Evidence so far: [noalias.md](../docs/research/noalias.md) and [noalias2.md](../docs/research/noalias2.md).

### 11.3 Index type

The sign and the width of lengths and indices. Rust and Zig index with 64-bit `usize`. On the reference machine 64-bit index arithmetic loses AVX2's native multiply and conversion. spectral-norm with 64-bit signed indices then runs 1.8 times as long as with C's 32-bit `int` under clang ([index-type.md](../docs/research/index-type.md)). Lengths need 64 bits. Index arithmetic might not.

### 11.4 Overflow in unchecked builds

An unchecked build either wraps or assumes no overflow. The cost of overflow checks in checked builds gets measured first.

### 11.5 Struct layout

Declared order (section 3.4) keeps layout predictable. Reordering by size would save padding in some structs.

### 11.6 Strings

One string type. The candidate is UTF-8 bytes in a slice.
