// add every element into *total. Without restrict the compiler must assume total
// may point into values (the running sum then has to stay in memory)

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

#define COUNT (1 << 20)
#define ROUNDS 1000

__attribute__((noinline)) static void Accumulate(uint32_t* total, const uint32_t* values, size_t count) {
	for (size_t i = 0; i < count; i++)
		*total += values[i];
}

__attribute__((noinline)) static void AccumulateRestrict(uint32_t* restrict total, const uint32_t* restrict values, size_t count) {
	for (size_t i = 0; i < count; i++)
		*total += values[i];
}

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	if (argc < 2 || (strcmp(argv[1], "plain") != 0 && strcmp(argv[1], "restrict") != 0)) {
		fputs("usage: accumulate plain|restrict\n", stderr);
		return 1;
	}
	int useRestrict = strcmp(argv[1], "restrict") == 0;
	uint32_t* values = malloc(COUNT * sizeof *values);
	if (!values)
		return 1;
	for (size_t i = 0; i < COUNT; i++)
		values[i] = (uint32_t)(i & 7);

	uint32_t total = 0;
	for (int round = 0; round < ROUNDS; round++) {
		if (useRestrict)
			AccumulateRestrict(&total, values, COUNT);
		else
			Accumulate(&total, values, COUNT);
	}
	printf("%u\n", total);
	free(values);
	return 0;
}
