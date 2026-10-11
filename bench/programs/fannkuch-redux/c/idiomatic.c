// fannkuch-redux: count prefix reversals over all permutations of 0..n-1

#include <stdio.h>
#include <stdlib.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

#define MAX_N 16

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	int n = argc > 1 ? atoi(argv[1]) : 7;
	if (n < 1 || n > MAX_N) {
		fprintf(stderr, "n must be in 1..%d\n", MAX_N);
		return 1;
	}

	int perm[MAX_N], perm1[MAX_N], count[MAX_N];
	int maxFlips = 0, checksum = 0, permIndex = 0;
	for (int i = 0; i < n; i++)
		perm1[i] = i;

	int r = n;
	for (;;) {
		while (r != 1) {
			count[r - 1] = r;
			r--;
		}

		for (int i = 0; i < n; i++)
			perm[i] = perm1[i];
		int flips = 0;
		for (int k = perm[0]; k != 0; k = perm[0]) {
			for (int i = 0, j = k; i < j; i++, j--) {
				int t = perm[i];
				perm[i] = perm[j];
				perm[j] = t;
			}
			flips++;
		}
		if (flips > maxFlips)
			maxFlips = flips;
		checksum += permIndex % 2 == 0 ? flips : -flips;

		// next permutation in counting order (rotates the first r + 1 elements)
		for (;;) {
			if (r == n) {
				printf("%d\nPfannkuchen(%d) = %d\n", checksum, n, maxFlips);
				return 0;
			}
			int first = perm1[0];
			for (int i = 0; i < r; i++)
				perm1[i] = perm1[i + 1];
			perm1[r] = first;
			if (--count[r] > 0)
				break;
			r++;
		}
		permIndex++;
	}
}
