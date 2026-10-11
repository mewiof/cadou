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
- [x] CI that builds every program and checks its output on Linux x86-64 and AArch64
- [x] first experiment on what a no-alias promise buys ([noalias.md](research/noalias.md))
- [ ] Linux reference run with perf counters (WSL2 on the same machine once it is back on)
- [ ] an AArch64 machine for timing
- [ ] k-nucleotide with a rule for which hash table each language may use
- [ ] systems workloads:
	- [ ] hash map insert/lookup
	- [ ] sorting
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

- [ ] written sketch of the core semantics with example programs (functions/integers/structs/tagged unions/slices/pointers/control flow/modules/C ABI calls)
- [ ] reference interpreter as the oracle for differential testing
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

1. Explain the baseline's largest gaps from the generated code (see the 2026-10-11 journal entry).
2. Linux run under WSL2 with perf counters as soon as Virtual Machine Platform is back on.
3. k-nucleotide and the first systems workloads (hash map + sorting).
4. EXPERT n-body and spectral-norm to see how far the idiomatic versions are from the machine.
5. Start phase 2 with the core semantics on paper and then the reference interpreter.
