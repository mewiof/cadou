// k-nucleotide: frequencies of 1- and 2-nucleotide sequences and counts of five
// longer ones in fasta's third sequence (read from stdin)

#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <iostream>
#include <print>
#include <string>
#include <string_view>
#include <unordered_map>
#include <utility>
#include <vector>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

namespace {
	// keys pack two bits per nucleotide (A/C/G/T = 0/1/2/3) with the first one highest
	using Counts = std::unordered_map<std::uint64_t, std::uint32_t>;

	std::uint8_t Code(char c) {
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

	std::vector<std::uint8_t> ReadThree() {
		std::string line;
		while (std::getline(std::cin, line) && !line.starts_with(">THREE")) {
		}
		std::vector<std::uint8_t> sequence;
		while (std::getline(std::cin, line) && !line.starts_with('>')) {
			for (char c : line)
				sequence.push_back(Code(c));
		}
		return sequence;
	}

	Counts Count(const std::vector<std::uint8_t>& sequence, std::size_t k) {
		Counts counts;
		const std::uint64_t mask = (std::uint64_t{1} << (2 * k)) - 1;
		std::uint64_t key = 0;
		for (std::size_t i = 0; i < sequence.size(); i++) {
			key = ((key << 2) | sequence[i]) & mask;
			if (i + 1 >= k)
				counts[key]++;
		}
		return counts;
	}

	std::string Decode(std::uint64_t key, std::size_t k) {
		std::string name(k, ' ');
		for (std::size_t i = 0; i < k; i++)
			name[i] = "ACGT"[(key >> (2 * (k - 1 - i))) & 3];
		return name;
	}

	void PrintFrequencies(const std::vector<std::uint8_t>& sequence, std::size_t k) {
		const Counts counts = Count(sequence, k);
		std::vector<std::pair<std::uint64_t, std::uint32_t>> entries(counts.begin(), counts.end());
		std::sort(entries.begin(), entries.end(), [](const auto& a, const auto& b) {
			return a.second != b.second ? a.second > b.second : a.first < b.first;
		});
		const double total = static_cast<double>(sequence.size() - k + 1);
		for (const auto& [key, count] : entries)
			std::println("{} {:.3f}", Decode(key, k), 100.0 * count / total);
		std::println("");
	}

	void PrintCount(const std::vector<std::uint8_t>& sequence, std::string_view pattern) {
		const Counts counts = Count(sequence, pattern.size());
		std::uint64_t key = 0;
		for (char c : pattern)
			key = (key << 2) | Code(c);
		const auto found = counts.find(key);
		std::println("{}\t{}", found == counts.end() ? 0u : found->second, pattern);
	}
}

int main() {
#ifdef _WIN32
	_setmode(_fileno(stdin), _O_BINARY);
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	std::ios::sync_with_stdio(false);
	const std::vector<std::uint8_t> sequence = ReadThree();
	PrintFrequencies(sequence, 1);
	PrintFrequencies(sequence, 2);
	for (std::string_view pattern : {"GGT", "GGTA", "GGTATT", "GGTATTTTAATT", "GGTATTTTAATTTATAGT"})
		PrintCount(sequence, pattern);
}
