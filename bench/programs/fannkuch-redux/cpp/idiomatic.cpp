// fannkuch-redux: count prefix reversals over all permutations of 0..n-1

#include <algorithm>
#include <array>
#include <cstdio>
#include <cstdlib>
#include <print>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

constexpr int MAX_N = 16;

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	int n = argc > 1 ? std::atoi(argv[1]) : 7;
	if (n < 1 || n > MAX_N) {
		std::println(stderr, "n must be in 1..{}", MAX_N);
		return 1;
	}

	std::array<int, MAX_N> perm{}, perm1{}, count{};
	int maxFlips = 0, checksum = 0, permIndex = 0;
	for (int i = 0; i < n; i++)
		perm1[i] = i;

	int r = n;
	for (;;) {
		while (r != 1) {
			count[r - 1] = r;
			r--;
		}

		perm = perm1;
		int flips = 0;
		for (int k = perm[0]; k != 0; k = perm[0]) {
			std::reverse(perm.begin(), perm.begin() + k + 1);
			flips++;
		}
		maxFlips = std::max(maxFlips, flips);
		checksum += permIndex % 2 == 0 ? flips : -flips;

		// next permutation in counting order (rotates the first r + 1 elements)
		for (;;) {
			if (r == n) {
				std::println("{}\nPfannkuchen({}) = {}", checksum, n, maxFlips);
				return 0;
			}
			std::rotate(perm1.begin(), perm1.begin() + 1, perm1.begin() + r + 1);
			if (--count[r] > 0)
				break;
			r++;
		}
		permIndex++;
	}
}
