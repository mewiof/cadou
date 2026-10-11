// fannkuch-redux with operation counts: how many flips and rotations of each length
// the benchmark performs. Same loops as bench/programs/fannkuch-redux/c/idiomatic.c
//
//   clang -O2 fannkuch_counts.c -o counts && counts 11

#include <stdio.h>
#include <stdlib.h>

#define MAX_N 16

int main(int argc, char** argv) {
	int n = argc > 1 ? atoi(argv[1]) : 7;
	if (n < 2 || n > MAX_N) {
		fprintf(stderr, "n must be in 2..%d\n", MAX_N);
		return 1;
	}

	int perm[MAX_N], perm1[MAX_N], count[MAX_N];
	unsigned long long permutations = 0, swaps = 0, flipLength[MAX_N + 1] = {0}, rotationLength[MAX_N + 1] = {0};
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
		permutations++;
		for (int k = perm[0]; k != 0; k = perm[0]) {
			flipLength[k + 1]++;
			swaps += (k + 1) / 2;
			for (int i = 0, j = k; i < j; i++, j--) {
				int t = perm[i];
				perm[i] = perm[j];
				perm[j] = t;
			}
		}

		for (;;) {
			if (r == n) {
				unsigned long long flips = 0, rotations = 0;
				printf("length | flips | rotations\n");
				for (int i = 2; i <= n; i++) {
					printf("%d | %llu | %llu\n", i, flipLength[i], rotationLength[i]);
					flips += flipLength[i];
					rotations += rotationLength[i];
				}
				printf("permutations %llu, flips %llu, swaps %llu, rotations %llu\n", permutations, flips, swaps, rotations);
				return 0;
			}
			rotationLength[r + 1]++;
			int first = perm1[0];
			for (int i = 0; i < r; i++)
				perm1[i] = perm1[i + 1];
			perm1[r] = first;
			if (--count[r] > 0)
				break;
			r++;
		}
	}
}
