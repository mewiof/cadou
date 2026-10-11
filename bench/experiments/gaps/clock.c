// core clock probe: a chain of dependent register adds retires one add per cycle.
// Adds of an immediate would not do (Raptor Cove folds such chains at rename)

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char** argv) {
	uint64_t iterations = argc > 1 ? strtoull(argv[1], NULL, 10) : 100000000;
	uint64_t x = 0;
	for (uint64_t i = 0; i < iterations; i++) {
		__asm__ volatile(
			"add %0, %0\n\tadd %0, %0\n\tadd %0, %0\n\tadd %0, %0\n\tadd %0, %0\n\t"
			"add %0, %0\n\tadd %0, %0\n\tadd %0, %0\n\tadd %0, %0\n\tadd %0, %0\n\t"
			"add %0, %0\n\tadd %0, %0\n\tadd %0, %0\n\tadd %0, %0\n\tadd %0, %0\n\t"
			"add %0, %0\n\tadd %0, %0\n\tadd %0, %0\n\tadd %0, %0\n\tadd %0, %0"
			: "+r"(x));
	}
	printf("%llu\n", (unsigned long long)x);
	return 0;
}
