// fasta: one repeated and two random DNA sequences in 60-character lines

#include <algorithm>
#include <array>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <span>
#include <string_view>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

namespace {
	constexpr std::size_t LINE_LENGTH = 60;

	constexpr std::string_view ALU =
		"GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGG"
		"GAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGA"
		"CCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAAT"
		"ACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCA"
		"GCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGG"
		"AGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCC"
		"AGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAA";

	struct Acid {
		char symbol;
		double probability;
	};

	class Random {
	public:
		double Next(double max) {
			_seed = (_seed * IA + IC) % IM;
			return max * _seed / IM;
		}

	private:
		static constexpr std::uint32_t IM = 139968;
		static constexpr std::uint32_t IA = 3877;
		static constexpr std::uint32_t IC = 29573;
		std::uint32_t _seed = 42;
	};

	void RepeatFasta(std::string_view header, std::string_view sequence, std::size_t n) {
		std::fwrite(header.data(), 1, header.size(), stdout);
		std::array<char, LINE_LENGTH + 1> line;
		std::size_t position = 0;
		while (n > 0) {
			std::size_t lineLength = std::min(n, LINE_LENGTH);
			for (std::size_t i = 0; i < lineLength; i++) {
				line[i] = sequence[position];
				if (++position == sequence.size())
					position = 0;
			}
			line[lineLength] = '\n';
			std::fwrite(line.data(), 1, lineLength + 1, stdout);
			n -= lineLength;
		}
	}

	void RandomFasta(std::string_view header, std::span<Acid> acids, std::size_t n, Random& random) {
		std::fwrite(header.data(), 1, header.size(), stdout);
		double cumulative = 0.0;
		for (Acid& acid : acids) {
			cumulative += acid.probability;
			acid.probability = cumulative;
		}

		std::array<char, LINE_LENGTH + 1> line;
		while (n > 0) {
			std::size_t lineLength = std::min(n, LINE_LENGTH);
			for (std::size_t i = 0; i < lineLength; i++) {
				double r = random.Next(1.0);
				std::size_t k = 0;
				while (k + 1 < acids.size() && r >= acids[k].probability)
					k++;
				line[i] = acids[k].symbol;
			}
			line[lineLength] = '\n';
			std::fwrite(line.data(), 1, lineLength + 1, stdout);
			n -= lineLength;
		}
	}
}

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	// NOTE: the UCRT flushes stdout after every call when stdout is a character
	// device (NUL e.g.). Every fasta writes through a 64 KiB buffer
	std::setvbuf(stdout, nullptr, _IOFBF, 1 << 16);
	std::size_t n = argc > 1 ? std::strtoull(argv[1], nullptr, 10) : 1000;
	std::array<Acid, 15> iub{{
		{'a', 0.27}, {'c', 0.12}, {'g', 0.12}, {'t', 0.27},
		{'B', 0.02}, {'D', 0.02}, {'H', 0.02}, {'K', 0.02},
		{'M', 0.02}, {'N', 0.02}, {'R', 0.02}, {'S', 0.02},
		{'V', 0.02}, {'W', 0.02}, {'Y', 0.02},
	}};
	std::array<Acid, 4> homoSapiens{{
		{'a', 0.3029549426680},
		{'c', 0.1979883004921},
		{'g', 0.1975473066391},
		{'t', 0.3015094502008},
	}};

	Random random;
	RepeatFasta(">ONE Homo sapiens alu\n", ALU, 2 * n);
	RandomFasta(">TWO IUB ambiguity codes\n", iub, 3 * n, random);
	RandomFasta(">THREE Homo sapiens frequency\n", homoSapiens, 5 * n, random);
}
