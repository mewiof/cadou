// sort: n random 64-bit integers with the standard library sort. Prints the
// smallest/median/largest value and a checksum that depends on the order

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

// splitmix64 (the same stream in every language)
static uint64_t Next(uint64_t* state) {
	uint64_t z = (*state += 0x9E3779B97F4A7C15u);
	z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9u;
	z = (z ^ (z >> 27)) * 0x94D049BB133111EBu;
	return z ^ (z >> 31);
}

static int CompareValues(const void* left, const void* right) {
	uint64_t a = *(const uint64_t*)left;
	uint64_t b = *(const uint64_t*)right;
	return (a > b) - (a < b);
}

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	size_t n = argc > 1 ? (size_t)strtoull(argv[1], NULL, 10) : 1000;
	if (n == 0)
		return 1;
	uint64_t* values = malloc(n * sizeof *values);
	if (!values)
		return 1;
	uint64_t state = 0;
	for (size_t i = 0; i < n; i++)
		values[i] = Next(&state);

	qsort(values, n, sizeof *values, CompareValues);

	uint64_t checksum = 0;
	for (size_t i = 0; i < n; i++)
		checksum += values[i] * (i + 1);
	printf("%llu %llu %llu %llu\n", (unsigned long long)values[0], (unsigned long long)values[n / 2], (unsigned long long)values[n - 1], (unsigned long long)checksum);
	free(values);
	return 0;
}
