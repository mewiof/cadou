# Benchmark run i7-14700kf-windows

Recorded 2026-10-11T15:39:26+00:00 on Intel(R) Core(TM) i7-14700KF (`-march=native` means clang raptorlake and gcc alderlake) with 28 logical CPUs under Windows 11 build 26300. Harness at `b194b5f44e`.

Protocol: pinned to logical CPU 4 (performance core 4) at high priority. 3 warmup rounds and 30 timed rounds. Each round runs every program once in a new random order (seed 1). Times are whole-process wall clock with process start included (see `startup`). Medians come with 95% order-statistic intervals. "vs fastest" is the ratio of medians with a 95% bootstrap interval. p comes from a two-sided Mann-Whitney U test against the fastest variant (Holm-adjusted within each benchmark). Compile time is the median of 3 clean builds (unpinned). No hardware counters on this host.

| tool | version | pinned |
| --- | --- | --- |
| clang | clang version 23.1.3 (https://github.com/llvm/llvm-project 0d261d1ca552c95a8f007e061c787ac7132fbcbc) | yes |
| clang++ | clang version 23.1.3 (https://github.com/llvm/llvm-project 0d261d1ca552c95a8f007e061c787ac7132fbcbc) | yes |
| llvm-strip | LLVM version 23.1.3 | yes |
| gcc | gcc.exe (MinGW-W64 x86_64-ucrt-posix-seh, built by Brecht Sanders, r2) 16.2.0 | yes |
| g++ | g++.exe (MinGW-W64 x86_64-ucrt-posix-seh, built by Brecht Sanders, r2) 16.2.0 | yes |
| strip | GNU strip (Binutils for MinGW-W64 x86_64, built by Brecht Sanders, r2) 2.47.20260726 | yes |
| zig | 0.17.0 | yes |
| rustc | rustc 1.99.0 (b940084d7 2026-09-28) | yes |

## Summary

Median time over the fastest variant's median. **1.00** is the fastest. "tie" means no significant difference from it (Holm p >= 0.05).

| benchmark | input | c-clang | c-gcc | cpp-clang | cpp-gcc | rust | zig |
| --- | --- | --- | --- | --- | --- | --- | --- |
| binary-trees | 19 | 1.71 | 1.67 | 1.90 | 1.72 | 1.74 | **1.00** |
| fannkuch-redux | 11 | 1.16 | **1.00** | 1.52 | 1.30 | 1.15 | 1.48 |
| fasta | 25000000 | 1.36 | 1.27 | 1.35 | 1.26 | 1.11 | **1.00** |
| mandelbrot | 8000 | **1.00** | 1.04 | 1.01 | 1.04 | 1.06 | 1.04 |
| n-body | 50000000 | 1.30 | 1.31 | 1.17 | 1.31 | 1.28 | **1.00** |
| spectral-norm | 5500 | 1.46 | 1.01 tie | 1.45 | **1.00** | 1.66 | 1.97 |
| geometric mean | | 1.31 | 1.20 | 1.37 | 1.25 | 1.31 | 1.20 |

## binary-trees

Input `19`. Output sha256 `0bba728e95e9f207`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| zig | 1.877 s | 1.855-1.895 s | fastest |  | 35 MiB | 858 KiB | 858 KiB | 7.926 s |
| c-gcc | 3.139 s | 3.123-3.212 s | +67.3% (+65.4, +71.3) | <0.001 | 68 MiB | 68 KiB | 27 KiB | 0.267 s |
| c-clang | 3.210 s | 3.184-3.274 s | +71.0% (+68.8, +74.4) | <0.001 | 68 MiB | 148 KiB | 148 KiB | 0.109 s |
| cpp-gcc | 3.226 s | 3.150-3.258 s | +71.9% (+67.4, +74.5) | <0.001 | 68 MiB | 3,202 KiB | 1,378 KiB | 2.534 s |
| rust | 3.266 s | 3.207-3.304 s | +74.0% (+70.6, +76.9) | <0.001 | 68 MiB | 118 KiB | 118 KiB | 0.923 s |
| cpp-clang | 3.561 s | 3.513-3.600 s | +89.7% (+86.5, +92.4) | <0.001 | 68 MiB | 418 KiB | 418 KiB | 1.491 s |

## fannkuch-redux

Input `11`. Output sha256 `496853c07b61c354`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| c-gcc | 1.350 s | 1.340-1.366 s | fastest |  | 3.9 MiB | 63 KiB | 22 KiB | 0.209 s |
| rust | 1.554 s | 1.529-1.572 s | +15.1% (+12.7, +16.7) | <0.001 | 4.0 MiB | 120 KiB | 120 KiB | 0.931 s |
| c-clang | 1.560 s | 1.534-1.571 s | +15.5% (+13.4, +16.9) | <0.001 | 3.6 MiB | 148 KiB | 148 KiB | 0.130 s |
| cpp-gcc | 1.754 s | 1.735-1.783 s | +29.9% (+27.7, +32.5) | <0.001 | 4.0 MiB | 3,204 KiB | 1,381 KiB | 2.639 s |
| zig | 1.998 s | 1.966-2.020 s | +48.0% (+45.7, +49.9) | <0.001 | 3.3 MiB | 860 KiB | 860 KiB | 7.703 s |
| cpp-clang | 2.057 s | 2.050-2.072 s | +52.4% (+50.4, +53.8) | <0.001 | 3.7 MiB | 420 KiB | 420 KiB | 1.540 s |

## fasta

Input `25000000`. Output sha256 `3fcf4f78104c8a65`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| zig | 1.811 s | 1.777-1.816 s | fastest |  | 3.4 MiB | 857 KiB | 857 KiB | 5.508 s |
| rust | 2.011 s | 1.995-2.028 s | +11.1% (+10.3, +13.7) | <0.001 | 4.0 MiB | 124 KiB | 124 KiB | 0.987 s |
| cpp-gcc | 2.288 s | 2.274-2.309 s | +26.3% (+25.5, +29.0) | <0.001 | 3.9 MiB | 63 KiB | 22 KiB | 0.361 s |
| c-gcc | 2.298 s | 2.265-2.320 s | +26.9% (+25.5, +30.2) | <0.001 | 3.9 MiB | 63 KiB | 23 KiB | 0.252 s |
| cpp-clang | 2.437 s | 2.409-2.461 s | +34.6% (+33.0, +37.5) | <0.001 | 3.6 MiB | 119 KiB | 119 KiB | 0.411 s |
| c-clang | 2.455 s | 2.426-2.468 s | +35.6% (+34.0, +38.5) | <0.001 | 3.6 MiB | 119 KiB | 119 KiB | 0.124 s |

## mandelbrot

Input `8000`. Output sha256 `1785fabcd1a79cb8`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| c-clang | 2.360 s | 2.344-2.377 s | fastest |  | 11 MiB | 148 KiB | 148 KiB | 0.112 s |
| cpp-clang | 2.378 s | 2.356-2.396 s | +0.7% (-0.4, +1.7) | 0.029 | 11 MiB | 419 KiB | 419 KiB | 1.440 s |
| zig | 2.457 s | 2.446-2.489 s | +4.1% (+3.4, +5.7) | <0.001 | 11 MiB | 858 KiB | 858 KiB | 5.631 s |
| c-gcc | 2.458 s | 2.444-2.482 s | +4.1% (+3.3, +5.2) | <0.001 | 11 MiB | 62 KiB | 21 KiB | 0.180 s |
| cpp-gcc | 2.466 s | 2.440-2.485 s | +4.5% (+3.3, +5.5) | <0.001 | 11 MiB | 3,204 KiB | 1,381 KiB | 2.566 s |
| rust | 2.506 s | 2.481-2.521 s | +6.2% (+4.9, +7.1) | <0.001 | 11 MiB | 122 KiB | 122 KiB | 0.944 s |

## n-body

Input `50000000`. Output sha256 `3e6c9ef9d26cfe31`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| zig | 1.081 s | 1.075-1.099 s | fastest |  | 3.4 MiB | 874 KiB | 874 KiB | 7.953 s |
| cpp-clang | 1.262 s | 1.252-1.267 s | +16.7% (+15.0, +17.5) | <0.001 | 3.7 MiB | 421 KiB | 421 KiB | 1.515 s |
| rust | 1.383 s | 1.374-1.394 s | +27.9% (+26.2, +29.2) | <0.001 | 4.0 MiB | 144 KiB | 144 KiB | 1.107 s |
| c-clang | 1.401 s | 1.388-1.423 s | +29.5% (+27.6, +31.7) | <0.001 | 3.7 MiB | 150 KiB | 150 KiB | 0.132 s |
| c-gcc | 1.417 s | 1.411-1.442 s | +31.1% (+29.4, +33.4) | <0.001 | 4.0 MiB | 64 KiB | 23 KiB | 0.210 s |
| cpp-gcc | 1.422 s | 1.417-1.437 s | +31.5% (+29.8, +33.1) | <0.001 | 4.1 MiB | 3,204 KiB | 1,381 KiB | 2.680 s |

## spectral-norm

Input `5500`. Output sha256 `f9d5b5e3eb7657cf`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cpp-gcc | 0.462 s | 0.458-0.468 s | fastest |  | 4.1 MiB | 3,207 KiB | 1,384 KiB | 2.705 s |
| c-gcc | 0.465 s | 0.460-0.471 s | +0.5% (-1.1, +2.1) | 0.318 | 4.0 MiB | 65 KiB | 24 KiB | 0.255 s |
| cpp-clang | 0.671 s | 0.665-0.677 s | +45.1% (+42.9, +47.1) | <0.001 | 3.7 MiB | 424 KiB | 424 KiB | 1.524 s |
| c-clang | 0.673 s | 0.662-0.679 s | +45.6% (+43.1, +47.3) | <0.001 | 3.6 MiB | 150 KiB | 150 KiB | 0.142 s |
| rust | 0.769 s | 0.765-0.780 s | +66.3% (+64.4, +68.8) | <0.001 | 4.0 MiB | 142 KiB | 142 KiB | 1.109 s |
| zig | 0.912 s | 0.904-0.918 s | +97.3% (+94.2, +99.3) | <0.001 | 3.4 MiB | 874 KiB | 874 KiB | 8.203 s |

## startup

Input `(none)`. Output sha256 `e3b0c44298fc1c14`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| zig | 4.07 ms | 3.80-4.69 ms | fastest |  | 3.0 MiB | 4.5 KiB | 4.5 KiB | 0.103 s |
| c-gcc | 4.87 ms | 4.32-5.82 ms | +19.6% (+0.7, +41.1) | 0.018 | 3.9 MiB | 59 KiB | 20 KiB | 0.169 s |
| c-clang | 5.27 ms | 4.53-7.18 ms | +29.4% (+4.4, +70.3) | 0.007 | 3.6 MiB | 112 KiB | 112 KiB | 86.16 ms |
| cpp-clang | 5.27 ms | 4.61-6.02 ms | +29.5% (+8.5, +42.9) | 0.003 | 3.6 MiB | 112 KiB | 112 KiB | 84.60 ms |
| rust | 5.48 ms | 4.98-5.92 ms | +34.5% (+12.9, +47.7) | <0.001 | 3.9 MiB | 104 KiB | 104 KiB | 0.797 s |
| cpp-gcc | 5.68 ms | 4.57-6.91 ms | +39.5% (+10.6, +65.0) | 0.002 | 3.9 MiB | 59 KiB | 20 KiB | 0.171 s |
