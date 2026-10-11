# Principles

## 1. Priorities

In this order when they conflict:

1. **Speed.** Top run-time performance that is measured and never assumed. Compile speed is the second speed metric.
2. **Safety.** Memory-safe by default. Unsafe operations exist but they are explicit.
3. **Minimalism.** A small core/one obvious way to do each thing/an enforced complexity budget ([COMPLEXITY.md](COMPLEXITY.md)).
4. **Freedom.** Full low-level control through escape hatches that grep can find.

Developer experience (fast compiles/clear diagnostics/good tools) is a requirement rather than an extra.

When speed and safety pull apart, first look for a design that gives both (usually by telling the compiler more). Without one, safe is the default and the fast way is a visible opt-out. Both get measured.

## 2. How decisions get made

- Evidence over opinion. A performance claim needs reproducible data committed to this repository. "Should be faster" is not a result.
- Every significant change goes hypothesis -> experiment -> measurement -> written conclusion -> decision.
- Significant decisions become ADRs in [decisions/](decisions/) (context/options/evidence/decision/consequences/when to revisit).
- Every work session gets a dated entry in [JOURNAL.md](JOURNAL.md) with the failed experiments too.
- The repository always builds and passes its tests at the end of a session.
- Facts about other languages come from primary sources (documentation/RFCs/standards/papers) with a version or a date.
- Rewrites/deletions/breaking changes are fine whenever the evidence points there. Nothing is compatible with anything until a stable version gets declared.

## 3. Design hypotheses

Starting points. Each one gets challenged with data.

### 3.1 No hidden costs

- No garbage collector/mandatory runtime/hidden heap allocation/exceptions. Nothing implicit (constructors/copies/destructors) runs behind the reader's back. What you read is what executes.
- All allocation goes through explicit allocators. Arenas and regions are first-class and pleasant to use.

### 3.2 Give the optimizer information

- Mutable references do not alias by default (as in Fortran and Rust). The compiler can then reorder and vectorize. Aliasing has to be written down.
- Data layout is first-class: explicit alignment/packed structs/bitfields with a defined layout/an easy switch between array-of-structs and struct-of-arrays.
- Portable SIMD vector types and inline assembly are built in.
- Compile-time execution replaces templates and textual macros (one language instead of two).
- No undefined behavior in safe code. Where C leans on UB for speed (signed overflow e.g.) there are explicit operations instead: wrapping/saturating/checked arithmetic + an explicit "assume no overflow". Intent gets stated instead of inferred.

### 3.3 Safety

- Slices (pointer plus length) by default. Bounds checks are on in safe builds and removable per build mode or per marked block.
- Pointers are non-null by default. Absence is an optional type.
- Safe code never reads uninitialized memory.
- Errors are values the compiler does not let you ignore silently.
- Safe code has no data races (ownership transfer and marker traits are the candidates).
- Temporal safety (use after free/dangling references) is the central open problem. Phase 3 prototypes at least three models (e.g. simplified borrowing/regions and arenas/mutable value semantics/generational references with check elision/linear types). It compares them on soundness (argued formally and ideally mechanized)/run-time cost/annotation burden on the same real programs/compile-time cost/fitness for kernel code. The choice becomes an ADR backed by data.

### 3.4 Minimalism

- A small core: structs/tagged unions/functions/slices/pointers/compile-time execution/modules. No inheritance and no exceptions.
- One of each: string type/initialization syntax/generics mechanism/error mechanism.
- Every addition pays for itself in [COMPLEXITY.md](COMPLEXITY.md). Removing a feature beats adding one.
- An edition mechanism that can evolve and remove features exists from day one (layers must not pile up).

### 3.5 Freedom

- Raw pointers/unchecked blocks/inline assembly/custom allocators/a freestanding mode with no standard library and no runtime/interrupt handlers/precise control of calling conventions/full C ABI interop.
- Every escape hatch is explicit and greppable (a review can find all the dangerous code).

## 4. The long-term test

CADOU should later carry an experimental microkernel: x86-64 and AArch64/capability-based security/drivers in user space/compatibility layers for foreign binaries. Every language decision has to keep that possible.

## 5. Measuring against the others

- Competitors at their best release settings: C and C++ (clang and gcc)/Rust/Zig. Exact flags and protocol are in [ADR 0003](decisions/0003-benchmark-methodology.md). Compiler versions get recorded on every run.
- Same algorithm and data structures in every language in two categories. IDIOMATIC is what a competent programmer writes. EXPERT is hand-tuned with intrinsics allowed. CADOU competes in both.
- CADOU results come twice and labeled: with safety checks on and with explicit unchecked opt-outs.
- "Outperforms" means a statistically significant win. The suite-level goal is a geometric-mean win against the best competitor in each category.
- Losses get reported and explained from the generated assembly.
- At least one x86-64 and one AArch64 machine.
- Once dedicated hardware exists a performance regression beyond the noise threshold blocks a merge.
