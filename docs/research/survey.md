# Survey of C/C++/Rust/Zig and memory-safety work

First version from 2026-10-11. Versions covered: C23 (ISO/IEC 9899:2024 as working draft N3220)/C++26 in progress/Rust 1.99.0/Zig 0.17.0. "Read" dates are when a source was last checked. Full entries are in [REFERENCES.md](../REFERENCES.md).

## 1. C

### 1.1 Arrays forget their length

An expression of array type turns into a pointer to the first element (C23 6.3.2.1p3). The exceptions are `sizeof`/`typeof`/unary `&` and a string literal that initializes an array. From then on the length travels separately or not at all. No check is possible.

Ritchie explains the origin in The Development of the C Language (HOPL-II 1993). B and BCPL stored a real pointer for each array. Structures with array members (`struct { int inumber; char name[14]; }` for a directory entry) left no place for that pointer. C instead "caused the creation of the pointer when the array name is mentioned in an expression". Most B code kept working that way.

### 1.2 Strings end with a zero

"A string is a contiguous sequence of characters terminated by and including the first null character" (C23 7.1.1). Length is a scan. A missing terminator means a read past the end. B made this choice to escape the 8- or 9-bit length limit of BCPL's count byte and because "maintaining the count seemed, in our experience, less convenient than using a terminator" (Ritchie 1993). Poul-Henning Kamp called it "The Most Expensive One-byte Mistake" (ACM Queue 9(7) from 2011-07-25). ACM's pages refuse automated reads so this one was not re-read here.

### 1.3 Undefined behavior

C23's Annex J.2 (N3220) lists 221 numbered undefined behaviors. Item (1) also covers every violated "shall" outside a constraint (the real count is higher). Three of them carry optimizations:

- signed overflow (6.5.1p5) lets the compiler assume `i + 1 > i` and widen loop counters
- effective types or "strict aliasing" (6.5.1p7) mean a store through one type leaves objects of another alone (values stay in registers)
- reading an uninitialized automatic object that never had its address taken (6.3.2.1p2)

The optimizer assumes none of this happens. Nothing checks it. `restrict` (6.7.4.2) is the opt-in version for aliasing: a promise the compiler trusts and never verifies.

### 1.4 Text macros and headers

Preprocessing is a separate textual stage (translation phases 1-4 in 5.1.1.2). `#include` pastes one file into another (6.10.3). Clang's module documentation (read 2026-10-11) lists the costs:

- every header gets preprocessed and parsed again for every translation unit (M headers times N units)
- macros leak across headers and break them
- include guards and reserved-looking names are workarounds
- tools cannot tell where a library's interface ends

### 1.5 Silent integer conversions

Integer promotions and the usual arithmetic conversions (6.3.1.1 and 6.3.1.8) change types without a cast. `-1 < 1u` is false because the `int` becomes `unsigned`. Arithmetic on two `char` values happens in `int`.

### 1.6 Worth keeping

The C ABI is the one boundary every other language speaks (all four compared here included). `restrict` shows that "this pointer does not alias" can be said in source.

## 2. C++

### 2.1 Removal is rare

C++17 removed trigraphs (N4086)/`register` (P0001R1)/`++` on `bool` (P0002R1)/dynamic exception specifications (P0003R5)/`std::auto_ptr`/`std::random_shuffle` per cppreference's C++17 page (read 2026-10-11). C++23 removed garbage-collection support (P2186R2)/mixed wide string literal concatenation (P2201R1)/non-encodable wide character literals (P2362R3) per its C++23 page.

Stroustrup in "Remember the Vasa!" (P0977R0 from 2018-03-06): "Individually, many (most?) proposals make sense. Together they are insanity to the point of endangering the future of C++." He also says C++17 "added significant surface complexity and increased the number of features people need to learn".

### 2.2 Initialization

cppreference's initialization page (read 2026-10-11) names default/value/zero/copy/direct/list/aggregate/reference initialization + constant/static/dynamic initialization for non-local variables. Which one runs depends on the spelling (`T x;` vs `T x{};` vs `T x = y;` vs `T x(y);`) and on the type (aggregate or not/user-provided constructor or not/`std::initializer_list` constructor or not).

### 2.3 Value categories

"Every expression belongs to exactly one of the fundamental categories in this taxonomy: lvalue, xvalue, or prvalue" with glvalue and rvalue as groupings ([basic.lval] in the current draft on eel.is, read 2026-10-11). The split decides when a move is allowed and when a temporary gets materialized. Programmers meet it through `&&` overloads and the deduction rules (`T&&` is a forwarding reference only when `T` gets deduced).

### 2.4 Overload resolution meets templates

Overload resolution ranks candidates by the conversions each argument needs ([over.match] and [over.ics.rank]). Function templates join the set after deduction ([temp.deduct]). A constructor template taking `T&&` beats the copy constructor when the source is a non-const lvalue:

```cpp
struct Wrapper {
	Wrapper(const Wrapper&);           // copy
	template <class T> Wrapper(T&&);   // forwarding
};

Wrapper a{1};
Wrapper b{a};        // the template: T = Wrapper& needs no added const
const Wrapper c{2};
Wrapper d{c};        // the copy constructor: equal match and non-templates win ties
```

clang 23.1.3 and g++ 16.2.0 both pick the template for `b` and the copy constructor for `d` (checked 2026-10-11).

### 2.5 The safety debate

Safe C++ (P3390R0 by Baxter and Mazakas from 2024-09-11) proposed a `safe` function context/borrow checking/lifetime parameters/choice types/relocation. The safety study group voted to put profiles first. Baxter then declared "Safe C++ is not being continued" (as reported by The Register on 2025-09-16). Profiles did not make C++26 either. EWG "encouraged pursuing a white paper to systematically address undefined behavior that could be concurrent with C++26" according to Herb Sutter's trip report (2025-02-17). C++26 does get a hardened standard library (P3471) and no more undefined behavior for reads of uninitialized variables (same report).

## 3. Rust

### 3.1 Leaking became safe

RFC 1066 (opened 2015-04-15 a month before Rust 1.0) made `std::mem::forget` safe. It also stated that not running a destructor is not unsafe. The trigger was `thread::scoped` whose guard joined the child thread in `Drop`. A cycle of `Rc` pointers could leak that guard in safe code and leave the child on a dead stack frame. Since then no safe API may depend on a destructor running (no must-run destructors and no linear types). withoutboats on adding a `Leak` trait today ("Changing the rules of Rust" from 2023-09-17): "adding `?Leak` would create a permanent scar across the ecosystem, as the vast majority of generics would gain a `T: ?Leak` bound."

### 3.2 Pin instead of a Move trait

Immovable values arrived in 2018 as the library type `Pin` instead of a `?Move` trait. withoutboats in "Pin" (2024-07-19): "It is simply not backward compatible to add another new ?Trait, and an edition cannot be easily used to solve the problem, because it is necessary that the interfaces of traits remain the same so that two crates in different editions can be composed together." `Pin` restricts pinned references only and so broke nothing. Its cost shows in every `Future::poll(self: Pin<&mut Self>, ...)` and in the later "pinned places" and `UnpinCell` designs on the same blog (2024).

### 3.3 Range types

`a..b` builds `core::ops::Range`. It implements `Iterator` itself and therefore cannot be `Copy`. RFC 3550 (2023-12-18) proposed `Copy` + `IntoIterator` range types for edition 2024 because "implementing `Iterator` directly [...] is now widely considered to be a mistake". Edition 2024 shipped in Rust 1.85.0 (2025-02-20) without the switch. The new types are stable in `std::range` since 1.96.0. The 1.99.0 docs say they will replace the legacy ones "in a future edition".

### 3.4 Mutex poisoning

`std::sync::Mutex` gets poisoned when a thread panics while holding it. The docs for 1.99.0 admit that "most usage of a mutex will simply `unwrap()` these results". A non-poisoning `Mutex`/`RwLock`/`Condvar` set exists only on nightly (`std::sync::nonpoison` behind feature `sync_nonpoison` with issue #134645, checked on 1.101.0-nightly of 2026-10-10).

### 3.5 Iterator cannot lend

`fn next(&mut self) -> Option<Self::Item>` gives `Item` no lifetime tied to `self`. An item cannot borrow from its iterator. Generic associated types (stable since Rust 1.65) make a lending iterator writable as a separate trait (`type Item<'a> where Self: 'a;` in the stabilization post of 2022-10-28). `std`'s `Iterator` keeps the old shape and so does every `for` loop built on it.

### 3.6 Compilation model and ABI

The crate is the unit of compilation. Generic code gets instantiated again in every crate that uses it. There is no stable ABI. The default representation guarantees alignment and non-overlap and then "There are no other guarantees of data layout" (Rust Reference on type layout, read 2026-10-11). Field order is free. `extern "C"` is the only stable boundary. `extern "crabi"` (a proposed stable higher-level ABI) exists only behind an experimental feature gate.

Graydon Hoare: "I wanted crates to allow inlining inside but present stable entrypoints to the outside". He counts the lack of it toward "Rust's terrible compile times" and the missing stable ABI. His post "The Rust I Wanted Had No Future" (June 2023) returned 404 on 2026-10-11 and is quoted here from Michael Tsai's blog (2023-06-08). The Rust project's own compiler performance survey (2025-09-10 with over 3,700 responses) puts build satisfaction at 6 out of 10. Also "55% of respondents have to wait more than ten seconds for a rebuild".

### 3.7 What it gets right

Shared XOR mutable references let rustc mark `&mut` and `&` parameters `noalias` (visible in rustc 1.99.0's IR in the [noalias experiment](noalias.md)). LLVM still gets the simplest loops fast without it by checking for overlap at run time. `Send`/`Sync` make data races a type error in safe code. The core type system and several unsafe library types have a machine-checked soundness proof (RustBelt at POPL 2018). Tree Borrows (PLDI 2025) is the current proposal for what unsafe code may do with aliases.

Hoare would have "traded performance and expressivity away for simplicity". He wanted tail calls ("I actually wanted them!") and an integer type that overflows into a bignum (same post). CADOU puts speed first (the opposite trade).

## 4. Zig

### 4.1 What it gets right

"If Zig code doesn't look like it's jumping away to call a function, then it isn't" (ziglang.org overview, read 2026-10-11). "Any functions that need to allocate memory accept an allocator parameter". That is also what lets the standard library run freestanding. `comptime` runs ordinary Zig at compile time in place of macros and templates.

Safety belongs to the build mode and to the scope. Debug and ReleaseSafe check illegal behavior such as overflow at run time. ReleaseFast and ReleaseSmall do not. `@setRuntimeSafety` switches the checks for one scope (0.17.0 language reference).

### 4.2 Duck-typed generics

"Compile-time parameters is how Zig implements generics. It is compile-time duck typing." An `anytype` parameter's type "will be inferred when the function is called" (0.17.0 language reference). A generic body gets checked only when instantiated. A `largest(items: anytype)` that compares elements with `>` fails inside `largest` when called on an array of structs. The call site shows up only in the trace (Zig 0.17.0, checked 2026-10-11):

```text
largest.zig:6:12: error: operator > not allowed for type 'largest.Point'
  if (item > best) best = item;
      ~~~~~^~~~~~
referenced by:
    main: largest.zig:15:13
```

### 4.3 async/await removed then redesigned

Zig 0.10.0 (2022-10-31) made the self-hosted compiler the default and noted "async/await is not done yet (#6025)". Zig 0.11.0 (2023-08-04) deleted the old compiler and async went with it. Issue #6025 (bring the old async back) was closed as not planned on 2025-07-15. Zig 0.16.0 (2026-04-13) brought it back as a library design. `io.async` creates a `Future(T)` with `await` and `cancel` on top of an `Io` implementation passed around like an allocator. `Io.Threaded` is "feature-complete and well-tested" while `Io.Evented` is "work-in-progress, experimental". Every file system and network API now takes an `Io`. From 0.11 to 0.16 there was no async at all.

### 4.4 Churn

0.16.0 changed `switch`/`@Type`/pointer coercion rules and moved the file system/networking/process APIs onto `Io`. 0.17.0 (2026-10-01) removed `@cImport` (C translation moves to a package)/array repetition with `**`/`void{}`/`errdefer |err|` captures/the `i0` type. It also changed `@bitCast` and `@hasDecl`. The benchmarks here had to be written against 0.17's `std.process.Init` and `std.Io.File` writers by reading `lib/std/start.zig` and `lib/std/Io/File.zig` because most published examples target 0.15 or older.

`std.process.Init` has a cost of its own. Before `main` runs it sets up an `Io` implementation/an arena/an environment map/the SMP allocator. In the first baseline every Zig benchmark binary is 857 to 874 KiB against 4.5 KiB for the empty `startup` program (whose `main` takes no `Init`). A ReleaseFast build takes 5.5 to 8.2 s against 0.1 to 0.3 s for the C programs ([report](../../bench/results/2026-10-11-i7-14700kf-windows/report.md)).

## 5. Other languages

### 5.1 Odin

An implicit `context` carries the current allocator. `#soa[N]T` turns an array of structs into a struct of arrays with the same indexing syntax. Bounds checks are on by default and `#no_bounds_check`/`#bounds_check` toggle them per block. `distinct` makes a new type with the same representation (odin-lang.org overview, read 2026-10-11).

### 5.2 Jai

Jonathan Blow's language with compile-time execution of arbitrary code and data-layout control. Still an invite-only beta as of early 2026 (Wikipedia, read 2026-10-11). The primary material is talks and streams. Nothing here can be checked hands-on.

### 5.3 Fortran

A procedure may assume its dummy arguments do not alias when either one is written. Callers must not pass overlapping storage in that case (Fortran 2018 15.5.2.13 as summarized in Flang's aliasing document, read 2026-10-11). Pointers can only point at objects declared `TARGET` (aliasing through pointers is opt-in and visible). This is the model behind "non-aliasing by default" in [PRINCIPLES.md](../PRINCIPLES.md).

### 5.4 Vale

Generational references: every allocation carries a generation number. A non-owning reference remembers the generation it saw and every dereference compares the two. Evan Ovadia measured 10.84% over unsafe code on the BenchmarkRL terrain generator against 25.29% for reference counting (post updated 2023-07-09). Owning references/linear-style moves/borrows of immutable regions skip the check.

### 5.5 Hylo and mutable value semantics

Racordon/Shabalin/Zheng/Abrahams/Saeta in "Implementation Strategies for Mutable Value Semantics" (JOT 21(2) from 2022 with DOI 10.5381/jot.2022.21.2.a2): "references become second-class citizens: they are only created implicitly, at function boundaries, and cannot be stored in variables or object fields. Hence, variables can never share mutable state." Part-wise in-place mutation avoids the copies of pure functional updates. Fixed-size values live on the stack and dynamic containers use copy-on-write.

### 5.6 Cyclone

Grossman/Morrisett/Jim/Hicks/Wang/Cheney in "Region-Based Memory Management in Cyclone" (PLDI 2002 with DOI 10.1145/512529.512563). It is a type-safe C dialect with region subtyping/stack allocation/an optional GC under one scheme. Separate compilation forces some explicit region annotations. Defaults/local inference/region effects cut them down.

### 5.7 Ada and SPARK

SPARK assumes "there can be no aliasing of mutable data". Access types follow an ownership policy where assignment moves the value. Temporary access goes through anonymous access types (observing for access-to-constant and borrowing for access-to-variable). GNATprove also proves that no memory leaks (SPARK user's guide, read 2026-10-11).

### 5.8 Austral

Types live in a free universe or in a linear one. "A value of a linear type must be used once and only once. Not _can_: _must_." A linear variable from outside a loop cannot appear in the loop body. A branch must consume it in every arm or in none. Borrowing adds read-only and mutable references that do not consume the value (Austral specification, read 2026-10-11).

### 5.9 Verona

Every object belongs to one region in a forest of isolated regions. Each region has its own memory management strategy. A thread mutates one region at a time and reference capabilities enforce the isolation (Arvidsson et al. in "Reference Capabilities for Flexible Memory Management" at OOPSLA 2023 with DOI 10.1145/3622846).

### 5.10 Fil-C

Checked C and C++ with a concurrent GC and "invisible capabilities" that keep pointers 64 bits wide. "Every possibly-unsafe C and C++ operation is checked" and existing code "most likely compiles and runs in Fil-C with zero changes" (fil-c.org at version 0.686 from 2026-10-04).

## 6. Ownership research

- RustBelt (Jung/Jourdan/Krebbers/Dreyer at POPL 2018): a semantic soundness proof of Rust's type system machine-checked in Coq with Iris. It covers unsafe library types too.
- Stacked Borrows (Jung/Dang/Kang/Dreyer at POPL 2020): an aliasing model for unsafe Rust that Miri checks dynamically.
- Tree Borrows (Villani/Hostert/Dreyer/Jung at PLDI 2025 with DOI 10.1145/3735592): a tree in place of the stack. On the 30,000 most used crates it rejects 54% fewer test cases than Stacked Borrows. It keeps most of the old optimizations and adds read-read reordering (proofs in Rocq).

## 7. Consequences for CADOU

Each line is a hypothesis to test rather than a decision. Decisions go to ADRs with data.

| observed | what to try |
| --- | --- |
| arrays and strings lose their length (C) | slices everywhere and strings as byte slices. Measure the bounds-check cost per benchmark with checks on and off |
| UB bought speed (C) | name each assumption as an operation (wrapping/checked/saturating/"assume no overflow"). Measure what the assumption buys on the suite |
| `restrict` is opt-in and unchecked (C) | non-aliasing references by default with aliasing spelled out. Measure vectorization with and without ([first try](noalias.md)) |
| textual headers (C and C++) | modules from day one. Clean and incremental compile speed in lines per second |
| removal is rare (C++) | an edition mechanism that can remove features before the first release |
| many initialization forms (C++) | one initialization syntax |
| leak safety fixed before 1.0 (Rust) | settle must-run destructors and linear types in phase 3 before anything stable |
| `Pin` bolted on (Rust) | movability in the type system from the start |
| `Iterator` cannot lend (Rust) | iteration that allows lending before the standard library exists |
| poisoning by default (Rust) | no poisoning in the base primitives |
| crate-wide compilation and no stable ABI (Rust) | compile time per module measured from the first compiler on + a C ABI boundary always |
| duck-typed generics (Zig) | constraints checked at the definition with one generics mechanism |
| no async from 0.11 to 0.16 (Zig) | concurrency stays out of the core until a design survives the kernel use case |
| safety per build mode and per scope (Zig and Odin) | checks on by default with `unchecked` blocks that grep finds |

Temporal safety candidates for phase 3 with what is known before measuring anything:

| model | run-time cost | annotation cost | soundness work | notes |
| --- | --- | --- | --- | --- |
| simplified ownership and borrowing | none on access | lifetimes in signatures | RustBelt + Tree Borrows | the known quantity |
| regions/arenas | bulk free with none per access | region parameters | Cyclone + Verona | kernels already use arenas |
| mutable value semantics | copies or copy-on-write where values escape | low | Hylo | no stored references at all |
| generational references | 10.84% before elision (one benchmark by Vale) | none | little published | needs a generation word per allocation |
| linear types | none | every resource handle | Austral | fits capabilities and handles in a kernel |
