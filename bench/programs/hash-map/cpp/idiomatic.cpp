// hash-map: insert n random 64-bit keys. Look up each of them plus n absent keys
// from a second stream

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <print>
#include <unordered_map>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

namespace {
	// splitmix64 (the same streams in every language)
	class SplitMix64 {
	public:
		explicit SplitMix64(std::uint64_t seed) : _state(seed) {}

		std::uint64_t Next() {
			std::uint64_t z = (_state += 0x9E3779B97F4A7C15u);
			z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9u;
			z = (z ^ (z >> 27)) * 0x94D049BB133111EBu;
			return z ^ (z >> 31);
		}

	private:
		std::uint64_t _state;
	};
}

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	const std::uint64_t n = argc > 1 ? std::strtoull(argv[1], nullptr, 10) : 1000;
	std::unordered_map<std::uint64_t, std::uint64_t> map;
	SplitMix64 keys(0);
	for (std::uint64_t i = 0; i < n; i++)
		map[keys.Next()] = i;

	SplitMix64 again(0), others(1);
	std::uint64_t hits = 0, sum = 0;
	for (std::uint64_t i = 0; i < n; i++) {
		if (const auto found = map.find(again.Next()); found != map.end()) {
			hits++;
			sum += found->second;
		}
		if (const auto found = map.find(others.Next()); found != map.end()) {
			hits++;
			sum += found->second;
		}
	}
	std::println("{} {}", hits, sum);
}
