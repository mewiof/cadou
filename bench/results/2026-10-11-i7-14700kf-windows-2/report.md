# Benchmark run i7-14700kf-windows-2

Recorded 2026-10-11T19:16:49+00:00 on Intel(R) Core(TM) i7-14700KF (`-march=native` means clang raptorlake and gcc alderlake) with 28 logical CPUs under Windows 11 build 26300. Harness at `410da5e8d9`.

Protocol: pinned to logical CPU 4 (performance core 4) at high priority. 3 warmup rounds and 30 timed rounds. Each round runs every program once in a new random order (seed 1). Times are whole-process wall clock with process start included (see `startup`). Medians come with 95% order-statistic intervals. "vs fastest" is the ratio of medians with a 95% bootstrap interval. p comes from a two-sided Mann-Whitney U test against the fastest variant (Holm-adjusted within each benchmark). Compile time is the median of 3 clean builds (unpinned). No hardware counters on this host.

| tool | version | pinned |
| --- | --- | --- |
| clang | clang version 23.1.3 (https://github.com/llvm/llvm-project 0d261d1ca552c95a8f007e061c787ac7132fbcbc) | yes |
| clang++ | clang version 23.1.3 (https://github.com/llvm/llvm-project 0d261d1ca552c95a8f007e061c787ac7132fbcbc) | yes |
| llvm-strip | LLVM version 23.1.3 | yes |
| gcc | gcc.exe (MinGW-W64 x86_64-ucrt-posix-seh, built by Brecht Sanders, r2) 16.2.0 | yes |
| g++ | g++.exe (MinGW-W64 x86_64-ucrt-posix-seh, built by Brecht Sanders, r2) 16.2.0 | yes |
| strip | GNU strip (Binutils for MinGW-W64 x86_64, built by Brecht Sanders, r2) 2.47.20260726 | yes |
| klib | 0.2.8 (5fc2090) | yes |
| zig | 0.17.0 | yes |
| rustc | rustc 1.99.0 (b940084d7 2026-09-28) | yes |

## Summary

Median time over the fastest variant's median. **1.00** is the fastest. "tie" means no significant difference from it (Holm p >= 0.05).

| benchmark | input | c-clang | c-gcc | cpp-clang | cpp-gcc | rust | zig |
| --- | --- | --- | --- | --- | --- | --- | --- |
| binary-trees | 19 | 1.69 | 1.67 | 1.88 | 1.70 | 1.72 | **1.00** |
| fannkuch-redux | 11 | 1.16 | **1.00** | 1.54 | 1.30 | 1.17 | 1.49 |
| fasta | 25000000 | 1.12 | 1.02 | 1.12 | **1.00** | 1.22 | 1.09 |
| hash-map | 4000000 | 1.07 | **1.00** | 4.25 | 4.08 | 1.45 | 1.03 |
| k-nucleotide | < fasta 2500000 | **1.00** | 1.06 | 1.46 | 1.04 | 1.41 | 1.07 |
| mandelbrot | 8000 | **1.00** | 1.05 | 1.01 | 1.05 | 1.06 | 1.05 |
| n-body | 50000000 | 1.30 | 1.30 | 1.16 | 1.30 | 1.27 | **1.00** |
| sort | 10000000 | 7.01 | 7.43 | 4.54 | 3.66 | **1.00** | 2.57 |
| spectral-norm | 5500 | 1.45 | 1.01 tie | 1.45 | **1.00** | 1.66 | 1.97 |
| geometric mean | | 1.46 | 1.38 | 1.75 | 1.53 | 1.31 | 1.28 |

## binary-trees

Input `19`. Output sha256 `0bba728e95e9f207`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| zig | 1.863 s | 1.850-1.886 s | fastest |  | 35 MiB | 858 KiB | 858 KiB | 5.526 s |
| c-gcc | 3.119 s | 3.103-3.166 s | +67.4% (+65.2, +69.9) | <0.001 | 68 MiB | 68 KiB | 27 KiB | 0.271 s |
| c-clang | 3.140 s | 3.117-3.179 s | +68.6% (+66.0, +71.1) | <0.001 | 68 MiB | 148 KiB | 148 KiB | 0.107 s |
| cpp-gcc | 3.171 s | 3.140-3.218 s | +70.2% (+67.6, +72.8) | <0.001 | 68 MiB | 3,202 KiB | 1,378 KiB | 2.597 s |
| rust | 3.201 s | 3.175-3.253 s | +71.8% (+69.3, +74.7) | <0.001 | 68 MiB | 118 KiB | 118 KiB | 0.893 s |
| cpp-clang | 3.500 s | 3.468-3.551 s | +87.9% (+85.3, +91.0) | <0.001 | 68 MiB | 418 KiB | 418 KiB | 1.493 s |

## fannkuch-redux

Input `11`. Output sha256 `496853c07b61c354`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| c-gcc | 1.327 s | 1.322-1.339 s | fastest |  | 3.9 MiB | 63 KiB | 22 KiB | 0.196 s |
| c-clang | 1.541 s | 1.526-1.549 s | +16.1% (+14.8, +16.9) | <0.001 | 3.6 MiB | 148 KiB | 148 KiB | 0.121 s |
| rust | 1.551 s | 1.524-1.557 s | +16.8% (+14.8, +17.5) | <0.001 | 4.0 MiB | 120 KiB | 120 KiB | 0.922 s |
| cpp-gcc | 1.731 s | 1.712-1.752 s | +30.4% (+28.7, +32.0) | <0.001 | 4.0 MiB | 3,204 KiB | 1,381 KiB | 2.542 s |
| zig | 1.982 s | 1.967-1.995 s | +49.3% (+47.9, +50.3) | <0.001 | 3.3 MiB | 860 KiB | 860 KiB | 7.807 s |
| cpp-clang | 2.045 s | 2.036-2.079 s | +54.0% (+52.8, +56.3) | <0.001 | 3.7 MiB | 420 KiB | 420 KiB | 1.490 s |

## fasta

Input `25000000`. Output sha256 `3fcf4f78104c8a65`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cpp-gcc | 1.623 s | 1.609-1.670 s | fastest |  | 4.0 MiB | 63 KiB | 22 KiB | 0.347 s |
| c-gcc | 1.661 s | 1.647-1.684 s | +2.4% (-0.6, +3.6) | 0.012 | 4.0 MiB | 63 KiB | 23 KiB | 0.236 s |
| zig | 1.766 s | 1.753-1.791 s | +8.8% (+5.8, +10.7) | <0.001 | 3.4 MiB | 857 KiB | 857 KiB | 5.492 s |
| c-clang | 1.820 s | 1.799-1.828 s | +12.1% (+8.8, +13.3) | <0.001 | 3.6 MiB | 120 KiB | 120 KiB | 0.118 s |
| cpp-clang | 1.825 s | 1.821-1.841 s | +12.4% (+9.4, +14.0) | <0.001 | 3.6 MiB | 120 KiB | 120 KiB | 0.402 s |
| rust | 1.982 s | 1.962-1.994 s | +22.1% (+18.4, +23.4) | <0.001 | 4.0 MiB | 124 KiB | 124 KiB | 0.951 s |

## hash-map

Input `4000000`. Output sha256 `86bad19f2b641539`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| c-gcc | 0.319 s | 0.314-0.321 s | fastest |  | 135 MiB | 64 KiB | 23 KiB | 0.240 s |
| zig | 0.330 s | 0.327-0.344 s | +3.4% (+2.3, +8.1) | <0.001 | 207 MiB | 858 KiB | 858 KiB | 5.384 s |
| c-clang | 0.340 s | 0.332-0.351 s | +6.7% (+4.7, +9.9) | <0.001 | 135 MiB | 148 KiB | 148 KiB | 0.135 s |
| rust | 0.463 s | 0.445-0.473 s | +45.1% (+40.2, +48.7) | <0.001 | 209 MiB | 121 KiB | 121 KiB | 0.962 s |
| cpp-gcc | 1.300 s | 1.232-1.316 s | +307.7% (+288.3, +315.8) | <0.001 | 172 MiB | 3,208 KiB | 1,382 KiB | 2.535 s |
| cpp-clang | 1.357 s | 1.307-1.382 s | +325.4% (+309.9, +335.7) | <0.001 | 252 MiB | 421 KiB | 421 KiB | 1.502 s |

## k-nucleotide

Input `< fasta 2500000`. Output sha256 `6db529a39e08a232`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| c-clang | 0.455 s | 0.454-0.461 s | fastest |  | 20 MiB | 154 KiB | 154 KiB | 0.150 s |
| cpp-gcc | 0.472 s | 0.466-0.476 s | +3.6% (+2.2, +4.7) | <0.001 | 22 MiB | 3,233 KiB | 1,391 KiB | 2.606 s |
| c-gcc | 0.481 s | 0.477-0.483 s | +5.6% (+4.6, +6.3) | <0.001 | 20 MiB | 67 KiB | 25 KiB | 0.271 s |
| zig | 0.485 s | 0.484-0.493 s | +6.6% (+5.8, +8.1) | <0.001 | 22 MiB | 884 KiB | 884 KiB | 5.802 s |
| rust | 0.641 s | 0.629-0.655 s | +40.9% (+38.1, +43.4) | <0.001 | 23 MiB | 156 KiB | 156 KiB | 1.312 s |
| cpp-clang | 0.666 s | 0.657-0.677 s | +46.2% (+44.2, +48.7) | <0.001 | 27 MiB | 461 KiB | 461 KiB | 1.703 s |

## mandelbrot

Input `8000`. Output sha256 `1785fabcd1a79cb8`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| c-clang | 2.335 s | 2.325-2.376 s | fastest |  | 11 MiB | 148 KiB | 148 KiB | 0.110 s |
| cpp-clang | 2.358 s | 2.346-2.390 s | +1.0% (-0.6, +2.4) | 0.028 | 11 MiB | 419 KiB | 419 KiB | 1.393 s |
| zig | 2.440 s | 2.429-2.474 s | +4.5% (+3.0, +6.0) | <0.001 | 11 MiB | 858 KiB | 858 KiB | 5.378 s |
| c-gcc | 2.446 s | 2.431-2.477 s | +4.7% (+3.1, +6.1) | <0.001 | 11 MiB | 62 KiB | 21 KiB | 0.180 s |
| cpp-gcc | 2.458 s | 2.428-2.488 s | +5.3% (+3.5, +6.2) | <0.001 | 11 MiB | 3,204 KiB | 1,381 KiB | 2.485 s |
| rust | 2.466 s | 2.454-2.503 s | +5.6% (+4.1, +6.9) | <0.001 | 11 MiB | 122 KiB | 122 KiB | 0.918 s |

## n-body

Input `50000000`. Output sha256 `3e6c9ef9d26cfe31`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| zig | 1.076 s | 1.067-1.088 s | fastest |  | 3.4 MiB | 874 KiB | 874 KiB | 5.501 s |
| cpp-clang | 1.249 s | 1.240-1.269 s | +16.1% (+14.5, +18.4) | <0.001 | 3.7 MiB | 421 KiB | 421 KiB | 1.424 s |
| rust | 1.369 s | 1.366-1.396 s | +27.2% (+25.9, +30.2) | <0.001 | 4.0 MiB | 144 KiB | 144 KiB | 1.053 s |
| c-clang | 1.396 s | 1.380-1.409 s | +29.7% (+27.9, +31.9) | <0.001 | 3.7 MiB | 150 KiB | 150 KiB | 0.128 s |
| c-gcc | 1.403 s | 1.397-1.429 s | +30.4% (+28.9, +32.3) | <0.001 | 4.0 MiB | 64 KiB | 23 KiB | 0.208 s |
| cpp-gcc | 1.404 s | 1.399-1.423 s | +30.4% (+29.0, +32.6) | <0.001 | 4.1 MiB | 3,204 KiB | 1,381 KiB | 2.488 s |

## sort

Input `10000000`. Output sha256 `efc9893ea3ad68dc`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| rust | 0.143 s | 0.142-0.145 s | fastest |  | 80 MiB | 124 KiB | 124 KiB | 0.985 s |
| zig | 0.367 s | 0.363-0.373 s | +156.8% (+152.4, +161.0) | <0.001 | 80 MiB | 867 KiB | 867 KiB | 5.736 s |
| cpp-gcc | 0.524 s | 0.518-0.529 s | +266.0% (+260.3, +269.8) | <0.001 | 80 MiB | 3,204 KiB | 1,380 KiB | 2.556 s |
| cpp-clang | 0.650 s | 0.644-0.656 s | +354.4% (+348.5, +359.6) | <0.001 | 80 MiB | 422 KiB | 422 KiB | 1.481 s |
| c-clang | 1.002 s | 0.991-1.012 s | +600.5% (+589.2, +608.2) | <0.001 | 80 MiB | 148 KiB | 148 KiB | 0.114 s |
| c-gcc | 1.063 s | 1.057-1.079 s | +642.7% (+633.6, +653.6) | <0.001 | 80 MiB | 63 KiB | 22 KiB | 0.191 s |

## spectral-norm

Input `5500`. Output sha256 `f9d5b5e3eb7657cf`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| cpp-gcc | 0.455 s | 0.453-0.459 s | fastest |  | 4.1 MiB | 3,207 KiB | 1,384 KiB | 2.628 s |
| c-gcc | 0.460 s | 0.454-0.466 s | +1.1% (-0.9, +2.1) | 0.579 | 4.0 MiB | 65 KiB | 24 KiB | 0.238 s |
| c-clang | 0.661 s | 0.659-0.664 s | +45.1% (+43.7, +45.9) | <0.001 | 3.6 MiB | 150 KiB | 150 KiB | 0.135 s |
| cpp-clang | 0.661 s | 0.658-0.664 s | +45.2% (+43.7, +46.2) | <0.001 | 3.7 MiB | 424 KiB | 424 KiB | 1.483 s |
| rust | 0.757 s | 0.755-0.762 s | +66.3% (+64.7, +67.4) | <0.001 | 4.0 MiB | 142 KiB | 142 KiB | 1.080 s |
| zig | 0.897 s | 0.890-0.912 s | +97.0% (+94.0, +100.4) | <0.001 | 3.4 MiB | 874 KiB | 874 KiB | 5.613 s |

## startup

Input `(none)`. Output sha256 `e3b0c44298fc1c14`.

| variant | median | 95% CI | vs fastest | p | peak memory | binary | stripped | compile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| zig | 3.80 ms | 3.58-4.03 ms | fastest |  | 3.0 MiB | 4.5 KiB | 4.5 KiB | 0.102 s |
| c-clang | 4.39 ms | 4.18-4.51 ms | +15.4% (+8.3, +23.2) | <0.001 | 3.6 MiB | 112 KiB | 112 KiB | 77.61 ms |
| cpp-clang | 4.40 ms | 4.28-4.65 ms | +15.7% (+9.2, +24.0) | <0.001 | 3.6 MiB | 112 KiB | 112 KiB | 78.90 ms |
| c-gcc | 4.40 ms | 4.27-4.62 ms | +15.7% (+9.4, +23.2) | <0.001 | 3.9 MiB | 59 KiB | 20 KiB | 0.153 s |
| cpp-gcc | 4.43 ms | 4.31-4.68 ms | +16.6% (+10.5, +25.2) | <0.001 | 3.9 MiB | 59 KiB | 20 KiB | 0.155 s |
| rust | 4.72 ms | 4.54-5.07 ms | +24.2% (+17.1, +35.9) | <0.001 | 3.9 MiB | 104 KiB | 104 KiB | 0.759 s |
