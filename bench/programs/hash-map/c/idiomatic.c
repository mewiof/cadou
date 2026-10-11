// hash-map: insert n random 64-bit keys. Look up each of them plus n absent keys
// from a second stream

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

#include "khash.h"

KHASH_MAP_INIT_INT64(Map, uint64_t)

// splitmix64 (the same streams in every language)
static uint64_t Next(uint64_t* state) {
	uint64_t z = (*state += 0x9E3779B97F4A7C15u);
	z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9u;
	z = (z ^ (z >> 27)) * 0x94D049BB133111EBu;
	return z ^ (z >> 31);
}

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	uint64_t n = argc > 1 ? strtoull(argv[1], NULL, 10) : 1000;
	khash_t(Map)* map = kh_init(Map);
	uint64_t keys = 0;
	for (uint64_t i = 0; i < n; i++) {
		int absent;
		khiter_t slot = kh_put(Map, map, Next(&keys), &absent);
		kh_value(map, slot) = i;
	}

	uint64_t again = 0, others = 1, hits = 0, sum = 0;
	for (uint64_t i = 0; i < n; i++) {
		khiter_t slot = kh_get(Map, map, Next(&again));
		if (slot != kh_end(map)) {
			hits++;
			sum += kh_value(map, slot);
		}
		slot = kh_get(Map, map, Next(&others));
		if (slot != kh_end(map)) {
			hits++;
			sum += kh_value(map, slot);
		}
	}
	printf("%llu %llu\n", (unsigned long long)hits, (unsigned long long)sum);
	kh_destroy(Map, map);
	return 0;
}
