# References

Sources behind the survey/ADRs/benchmarks. "read" is the date a source was last checked.

## Standards and language documentation

| source | version or date | link |
| --- | --- | --- |
| C standard ISO/IEC 9899:2024 as working draft N3220 by J. Meneide and F. Wiedijk | 2024-02-22 (read 2026-10-11) | https://www.open-std.org/jtc1/sc22/wg14/www/docs/n3220.pdf |
| C++ working draft | read 2026-10-11 | https://eel.is/c++draft/ |
| cppreference pages for C++17/C++23/initialization | read 2026-10-11 | https://en.cppreference.com/w/cpp |
| The Rust Reference on type layout | read 2026-10-11 | https://doc.rust-lang.org/reference/type-layout.html |
| Rust std docs for `std::sync::Mutex` + `std::range` | 1.99.0 | https://doc.rust-lang.org/stable/std/ |
| Rust std docs for `std::sync::nonpoison` | 1.101.0-nightly of 2026-10-10 | https://doc.rust-lang.org/nightly/std/sync/nonpoison/ |
| Rust Edition Guide on Rust 2024 | stable in 1.85.0 (2025-02-20) | https://doc.rust-lang.org/edition-guide/rust-2024/ |
| Zig language reference (`doc/langref.html` in the release archive) | 0.17.0 (2026-10-01) | https://ziglang.org/documentation/0.17.0/ |
| Zig overview | read 2026-10-11 | https://ziglang.org/learn/overview/ |
| Zig release notes for 0.10.0/0.11.0/0.16.0/0.17.0 | 2022-10-31/2023-08-04/2026-04-13/2026-10-01 | https://ziglang.org/download/ |
| Zig issue #6025 on async/await (closed as not planned) | 2025-07-15 | https://github.com/ziglang/zig/issues/6025 |
| Zig issue #23446 on stackless coroutines | read 2026-10-11 | https://github.com/ziglang/zig/issues/23446 |
| Clang Users Manual (`-ffp-contract`) + Clang Modules | in-progress docs read 2026-10-11 | https://clang.llvm.org/docs/ |
| GCC manual on optimize options (`-ffp-contract`) | read 2026-10-11 | https://gcc.gnu.org/onlinedocs/gcc/Optimize-Options.html |
| LLVM Developer Policy on IR backwards compatibility | read 2026-10-11 | https://llvm.org/docs/DeveloperPolicy.html |
| Microsoft C runtime docs on stream I/O (stdout flushing on character devices) | updated 2025-05-30 (read 2026-10-11) | https://learn.microsoft.com/en-us/cpp/c-runtime-library/stream-i-o |
| Microsoft x64 ABI conventions (register volatility) | updated 2026-06-02 (read 2026-10-11) | https://learn.microsoft.com/en-us/cpp/build/x64-software-conventions |
| Flang "Aliasing in Fortran" (Fortran 2018 15.5.2.13) | read 2026-10-11 | https://flang.llvm.org/docs/Aliasing.html |
| Odin overview | read 2026-10-11 | https://odin-lang.org/docs/overview/ |
| Austral specification | read 2026-10-11 | https://austral-lang.org/spec/spec.html |
| SPARK User's Guide on pointers | read 2026-10-11 | https://docs.adacore.com/spark2014-docs/html/ug/en/source/access.html |

## Papers/proposals/RFCs

| source | date | link |
| --- | --- | --- |
| D. M. Ritchie: The Development of the C Language (HOPL-II) | 1993 (read 2026-10-11) | https://www.nokia.com/bell-labs/about/dennis-m-ritchie/chist.html |
| P.-H. Kamp: The Most Expensive One-byte Mistake (ACM Queue 9(7)) | 2011-07-25 (ACM blocks automated reads so not re-read) | https://queue.acm.org/detail.cfm?id=2010365 |
| B. Stroustrup: Remember the Vasa! (P0977R0) | 2018-03-06 | https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2018/p0977r0.pdf |
| S. Baxter + C. Mazakas: Safe C++ (P3390R0) | 2024-09-11 | https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2024/p3390r0.html |
| RFC 1066: Safe mem::forget | 2015-04-15 | https://rust-lang.github.io/rfcs/1066-safe-mem-forget.html |
| RFC 3550: New range types | 2023-12-18 | https://rust-lang.github.io/rfcs/3550-new-range.html |
| Jung/Jourdan/Krebbers/Dreyer: RustBelt (POPL 2018) | 2017-12-27 online | https://doi.org/10.1145/3158154 |
| Jung/Dang/Kang/Dreyer: Stacked Borrows (POPL 2020) | 2019-12-20 online | https://doi.org/10.1145/3371109 |
| Villani/Hostert/Dreyer/Jung: Tree Borrows (PLDI 2025) | 2025 | https://doi.org/10.1145/3735592 |
| Grossman/Morrisett/Jim/Hicks/Wang/Cheney: Region-Based Memory Management in Cyclone (PLDI 2002) | 2002 | https://doi.org/10.1145/512529.512563 |
| Racordon/Shabalin/Zheng/Abrahams/Saeta: Implementation Strategies for Mutable Value Semantics (JOT 21(2)) | 2022 | https://doi.org/10.5381/jot.2022.21.2.a2 |
| Arvidsson/Castegren/Clebsch/Drossopoulou/Noble/Parkinson/Wrigstad: Reference Capabilities for Flexible Memory Management (OOPSLA 2023) | 2023 | https://doi.org/10.1145/3622846 |

## Posts and reports

| source | date | link |
| --- | --- | --- |
| Rust blog on generic associated types becoming stable in 1.65 | 2022-10-28 | https://blog.rust-lang.org/2022/10/28/gats-stabilization/ |
| Rust blog announcing Rust 1.77.0 | 2024-03-21 | https://blog.rust-lang.org/2024/03/21/Rust-1.77.0/ |
| Rust blog with the compiler performance survey 2025 results | 2025-09-10 | https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results/ |
| withoutboats: Changing the rules of Rust | 2023-09-17 | https://without.boats/blog/changing-the-rules-of-rust/ |
| withoutboats: Pin | 2024-07-19 | https://without.boats/blog/pin/ |
| G. Hoare: The Rust I Wanted Had No Future (404 on 2026-10-11) | June 2023 | https://graydon2.dreamwidth.org/307291.html |
| M. Tsai quoting Hoare's post | 2023-06-08 | https://mjtsai.com/blog/2023/06/08/the-rust-i-wanted-had-no-future/ |
| H. Sutter: trip report from the February 2025 meeting in Hagenberg | 2025-02-17 | https://herbsutter.com/2025/02/17/trip-report-february-2025-iso-c-standards-meeting-hagenberg-austria/ |
| The Register: Safe C++ proposal ditched | 2025-09-16 | https://www.theregister.com/2025/09/16/safe_c_proposal_ditched |
| E. Ovadia: Vale's Memory Safety Strategy (generational references) | updated 2023-07-09 | https://verdagon.dev/blog/generational-references |
| opam 2.2.0 release notes | 2024-07-01 | https://ocamlpro.com/blog/2024_07_01_opam_2_2_0_releases |
| Fil-C at version 0.686 | 2026-10-04 (read 2026-10-11) | https://fil-c.org/ |
| Wikipedia on Jai (closed beta status) | read 2026-10-11 | https://en.wikipedia.org/wiki/Jai_(programming_language) |

## Tools and benchmarks

| source | date | link |
| --- | --- | --- |
| The Computer Language Benchmarks Game (descriptions + the published outputs used as validation files) | outputs fetched 2026-10-11 | https://benchmarksgame-team.pages.debian.net/benchmarksgame/ |
| hyperfine README | 2.0.0 (read 2026-10-11) | https://github.com/sharkdp/hyperfine |
| Cranelift | read 2026-10-11 | https://cranelift.dev/ |
| QBE | read 2026-10-11 | https://c9x.me/compile/ |
| pinned toolchains with checksums | clang 23.1.3/GCC 16.2.0/Rust 1.99.0/Zig 0.17.0 | [bench/toolchains.toml](../bench/toolchains.toml) |
