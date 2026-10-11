// k-nucleotide: frequencies of 1- and 2-nucleotide sequences and counts of five
// longer ones in fasta's third sequence (read from stdin)

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

#include "khash.h"

// keys pack two bits per nucleotide (A/C/G/T = 0/1/2/3) with the first one highest
KHASH_MAP_INIT_INT64(Counts, uint32_t)

typedef struct {
	uint8_t* codes;
	size_t length;
	size_t capacity;
} Sequence;

typedef struct {
	uint64_t key;
	uint32_t count;
} Entry;

static uint8_t Code(char c) {
	switch (c) {
	case 'A':
	case 'a':
		return 0;
	case 'C':
	case 'c':
		return 1;
	case 'G':
	case 'g':
		return 2;
	default:
		return 3;
	}
}

static Sequence ReadThree(void) {
	char line[4096];
	while (fgets(line, sizeof line, stdin) && strncmp(line, ">THREE", 6) != 0) {
	}
	Sequence sequence = {0};
	while (fgets(line, sizeof line, stdin) && line[0] != '>') {
		size_t length = strcspn(line, "\n");
		if (sequence.length + length > sequence.capacity) {
			sequence.capacity = 2 * (sequence.length + length);
			sequence.codes = realloc(sequence.codes, sequence.capacity);
			if (!sequence.codes)
				exit(1);
		}
		for (size_t i = 0; i < length; i++)
			sequence.codes[sequence.length++] = Code(line[i]);
	}
	return sequence;
}

static khash_t(Counts)* Count(const Sequence* sequence, int k) {
	khash_t(Counts)* counts = kh_init(Counts);
	uint64_t mask = (UINT64_C(1) << (2 * k)) - 1;
	uint64_t key = 0;
	for (size_t i = 0; i < sequence->length; i++) {
		key = ((key << 2) | sequence->codes[i]) & mask;
		if (i + 1 >= (size_t)k) {
			int absent;
			khiter_t slot = kh_put(Counts, counts, key, &absent);
			if (absent)
				kh_value(counts, slot) = 0;
			kh_value(counts, slot)++;
		}
	}
	return counts;
}

static int MoreFrequent(const void* left, const void* right) {
	const Entry* a = left;
	const Entry* b = right;
	if (a->count != b->count)
		return a->count < b->count ? 1 : -1;
	return (a->key > b->key) - (a->key < b->key);
}

static void PrintFrequencies(const Sequence* sequence, int k) {
	khash_t(Counts)* counts = Count(sequence, k);
	// k is 1 or 2 here (at most 16 keys)
	Entry entries[16];
	size_t entryCount = 0;
	for (khiter_t slot = kh_begin(counts); slot != kh_end(counts); slot++) {
		if (kh_exist(counts, slot))
			entries[entryCount++] = (Entry){kh_key(counts, slot), kh_value(counts, slot)};
	}
	qsort(entries, entryCount, sizeof *entries, MoreFrequent);
	double total = (double)(sequence->length - k + 1);
	for (size_t i = 0; i < entryCount; i++) {
		char name[3];
		for (int j = 0; j < k; j++)
			name[j] = "ACGT"[(entries[i].key >> (2 * (k - 1 - j))) & 3];
		name[k] = '\0';
		printf("%s %.3f\n", name, 100.0 * entries[i].count / total);
	}
	printf("\n");
	kh_destroy(Counts, counts);
}

static void PrintCount(const Sequence* sequence, const char* pattern) {
	int k = (int)strlen(pattern);
	khash_t(Counts)* counts = Count(sequence, k);
	uint64_t key = 0;
	for (int i = 0; i < k; i++)
		key = (key << 2) | Code(pattern[i]);
	khiter_t slot = kh_get(Counts, counts, key);
	printf("%u\t%s\n", slot == kh_end(counts) ? 0 : kh_value(counts, slot), pattern);
	kh_destroy(Counts, counts);
}

int main(void) {
#ifdef _WIN32
	_setmode(_fileno(stdin), _O_BINARY);
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	Sequence sequence = ReadThree();
	PrintFrequencies(&sequence, 1);
	PrintFrequencies(&sequence, 2);
	static const char* patterns[] = {"GGT", "GGTA", "GGTATT", "GGTATTTTAATT", "GGTATTTTAATTTATAGT"};
	for (size_t i = 0; i < sizeof patterns / sizeof patterns[0]; i++)
		PrintCount(&sequence, patterns[i]);
	free(sequence.codes);
	return 0;
}
