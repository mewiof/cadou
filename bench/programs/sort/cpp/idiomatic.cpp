// sort: n random 64-bit integers with the standard library sort. Prints the
// smallest/median/largest value and a checksum that depends on the order

#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <print>
#include <vector>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

namespace {
	// splitmix64 (the same stream in every language)
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
	const std::size_t n = argc > 1 ? std::strtoull(argv[1], nullptr, 10) : 1000;
	if (n == 0)
		return 1;
	std::vector<std::uint64_t> values(n);
	SplitMix64 random(0);
	for (std::uint64_t& value : values)
		value = random.Next();

	std::sort(values.begin(), values.end());

	std::uint64_t checksum = 0;
	for (std::size_t i = 0; i < n; i++)
		checksum += values[i] * (i + 1);
	std::println("{} {} {} {}", values[0], values[n / 2], values[n - 1], checksum);
}
