# Roadmap

Phases without dates. Evidence can reopen any phase and rewrites are allowed at any time.

## Phase 0: foundations

Done on 2026-10-11 (the survey keeps growing).

- [x] repository layout/[STYLE.md](STYLE.md)/`tools/check_text.py`
- [x] [PRINCIPLES.md](PRINCIPLES.md)/the ADR template/[JOURNAL.md](JOURNAL.md)/[COMPLEXITY.md](COMPLEXITY.md)/[REFERENCES.md](REFERENCES.md)
- [x] first [survey](research/survey.md) of C/C++/Rust/Zig and memory-safety work
- [ ] survey second pass: Swift ownership and noncopyable types/Mojo/Carbon/Hare/CHERI/Arm MTE

## Phase 1: the bar

In progress.

- [x] benchmark harness with pinned toolchains ([ADR 0003](decisions/0003-benchmark-methodology.md))
- [x] idiomatic n-body/spectral-norm/fannkuch-redux/mandelbrot/binary-trees/fasta in C/C++/Rust/Zig + `startup`
- [x] first baseline on Windows 11 with an i7-14700KF ([report](../bench/results/2026-10-11-i7-14700kf-windows/report.md))
- [x] second baseline with ten benchmarks ([report](../bench/results/2026-10-11-i7-14700kf-windows-2/report.md))
- [x] CI that builds every program and checks its output on Linux x86-64 and AArch64
- [x] first experiment on what a no-alias promise buys ([noalias.md](research/noalias.md))
- [x] the baseline's open gaps explained from the generated code ([baseline-gaps.md](research/baseline-gaps.md))
- [x] loops a run-time check cannot rescue ([noalias2.md](research/noalias2.md))/index types ([index-type.md](research/index-type.md))/clang's Windows defaults ([clang-msvc.md](research/clang-msvc.md))
- [x] the cost of overflow checks ([overflow.md](research/overflow.md))
- [ ] Linux reference run with perf counters (WSL2 on the same machine once it is back on)
- [ ] an AArch64 machine for timing
- [x] k-nucleotide with a rule for which hash table each language may use
- [ ] systems workloads:
	- [x] hash map insert/lookup
	- [x] sorting
	- [ ] memcpy/memset-style kernels
	- [ ] JSON parsing and string processing
	- [ ] allocator-heavy work
	- [ ] matrix multiply
	- [ ] SIMD image/audio kernels
	- [ ] a lock-free queue
	- [ ] kernel-flavored tasks (page tables/ring buffers/a scheduler loop)
- [ ] EXPERT versions (SIMD/intrinsics/arenas)
- [ ] compile-speed benchmark (lines per second for clean and incremental builds)
- [ ] dedicated hardware so a regression beyond noise can block a merge

## Phase 2: core language

Semantics first and syntax last.

- [ ] written sketch of the core semantics with example programs (functions/integers/structs/tagged unions/slices/pointers/control flow/modules/C ABI calls). First draft in [spec/core.md](../spec/core.md)
- [ ] reference interpreter as the oracle for differential testing ([ADR 0004](decisions/0004-reference-interpreter.md))
	- [ ] integers/booleans/functions/control flow with every arithmetic check
	- [ ] structs/tagged unions/match
	- [ ] arrays/slices/pointers/the heap with bounds checks and provenance
	- [ ] modules
	- [ ] host functions in place of C calls
- [ ] bootstrap compiler in Rust ([ADR 0001](decisions/0001-bootstrap-language.md)) that emits LLVM IR ([ADR 0002](decisions/0002-initial-backend.md))
- [ ] first benchmarks compiled end to end as `cadou` variants with checks on and off

## Phase 3: safety research

- [ ] prototypes of at least three temporal-safety models ([PRINCIPLES.md 3.3](PRINCIPLES.md#33-safety) + candidates in [survey 7](research/survey.md#7-consequences-for-cadou))
- [ ] data-race freedom
- [ ] the choice as an ADR backed by measurements

## Phase 4: performance

- [ ] aliasing guarantees/layout control/SIMD/compile-time specialization/own optimization passes
- [ ] every benchmark gap closed or explained

## Phase 5: developer experience

- [ ] diagnostics/formatter/language server/build tool/DWARF debug info/documentation

## Phase 6: self-hosting

## Phase 7: freestanding and OS readiness

- [ ] a minimal CADOU kernel booting in QEMU on x86-64 and AArch64 (interrupts/paging/context switching) measured against equivalent C and Rust kernels

## Next steps

1. Reference interpreter milestones 1 and 2 with golden tests ([ADR 0004](decisions/0004-reference-interpreter.md)).
2. Linux run under WSL2 with perf counters as soon as Virtual Machine Platform is back on (Rust's n-body is waiting for counters).
3. EXPERT n-body and spectral-norm to see how far the idiomatic versions are from the machine.
4. More systems workloads (memcpy/memset kernels/JSON parsing/an allocator-heavy one/matrix multiply).
5. A checked spectral-norm loop that keeps its vector code (one check in front of the loop per [overflow.md](research/overflow.md)).
