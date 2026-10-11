// fasta: one repeated and two random DNA sequences in 60-character lines

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

#define LINE_LENGTH 60
#define IM 139968
#define IA 3877
#define IC 29573

typedef struct {
	char symbol;
	double probability;
} Acid;

static const char ALU[] =
	"GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGG"
	"GAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGA"
	"CCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAAT"
	"ACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCA"
	"GCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGG"
	"AGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCC"
	"AGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAA";

static Acid iub[] = {
	{'a', 0.27}, {'c', 0.12}, {'g', 0.12}, {'t', 0.27},
	{'B', 0.02}, {'D', 0.02}, {'H', 0.02}, {'K', 0.02},
	{'M', 0.02}, {'N', 0.02}, {'R', 0.02}, {'S', 0.02},
	{'V', 0.02}, {'W', 0.02}, {'Y', 0.02},
};

static Acid homoSapiens[] = {
	{'a', 0.3029549426680},
	{'c', 0.1979883004921},
	{'g', 0.1975473066391},
	{'t', 0.3015094502008},
};

static double Random(unsigned* seed, double max) {
	*seed = (*seed * IA + IC) % IM;
	return max * *seed / IM;
}

static void RepeatFasta(const char* header, const char* sequence, size_t n) {
	fputs(header, stdout);
	size_t length = strlen(sequence);
	size_t position = 0;
	char line[LINE_LENGTH + 1];
	while (n > 0) {
		size_t lineLength = n < LINE_LENGTH ? n : LINE_LENGTH;
		for (size_t i = 0; i < lineLength; i++) {
			line[i] = sequence[position];
			if (++position == length)
				position = 0;
		}
		line[lineLength] = '\n';
		fwrite(line, 1, lineLength + 1, stdout);
		n -= lineLength;
	}
}

static void RandomFasta(const char* header, Acid* acids, size_t acidCount, size_t n, unsigned* seed) {
	fputs(header, stdout);
	double cumulative = 0.0;
	for (size_t i = 0; i < acidCount; i++) {
		cumulative += acids[i].probability;
		acids[i].probability = cumulative;
	}

	char line[LINE_LENGTH + 1];
	while (n > 0) {
		size_t lineLength = n < LINE_LENGTH ? n : LINE_LENGTH;
		for (size_t i = 0; i < lineLength; i++) {
			double r = Random(seed, 1.0);
			size_t k = 0;
			while (k + 1 < acidCount && r >= acids[k].probability)
				k++;
			line[i] = acids[k].symbol;
		}
		line[lineLength] = '\n';
		fwrite(line, 1, lineLength + 1, stdout);
		n -= lineLength;
	}
}

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	// NOTE: the UCRT flushes stdout after every call when stdout is a character
	// device (NUL e.g.). Every fasta writes through a 64 KiB buffer
	setvbuf(stdout, NULL, _IOFBF, 1 << 16);
	size_t n = argc > 1 ? (size_t)atol(argv[1]) : 1000;
	unsigned seed = 42;
	RepeatFasta(">ONE Homo sapiens alu\n", ALU, 2 * n);
	RandomFasta(">TWO IUB ambiguity codes\n", iub, sizeof iub / sizeof iub[0], 3 * n, &seed);
	RandomFasta(">THREE Homo sapiens frequency\n", homoSapiens, sizeof homoSapiens / sizeof homoSapiens[0], 5 * n, &seed);
	return 0;
}
