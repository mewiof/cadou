// mandelbrot: up to 50 iterations per point for re in -1.5..0.5 and im in -1..1 (binary PBM out)

#include <cstdio>
#include <cstdlib>
#include <print>
#include <vector>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

constexpr int ITERATIONS = 50;
constexpr double LIMIT_SQUARED = 4.0;

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	int size = argc > 1 ? std::atoi(argv[1]) : 200;
	std::size_t rowBytes = (size + 7) / 8;
	std::vector<unsigned char> bitmap(rowBytes * size);

	for (int y = 0; y < size; y++) {
		double ci = 2.0 * y / size - 1.0;
		for (int x = 0; x < size; x++) {
			double cr = 2.0 * x / size - 1.5;
			double zr = 0.0, zi = 0.0, tr = 0.0, ti = 0.0;
			for (int i = 0; i < ITERATIONS && tr + ti <= LIMIT_SQUARED; i++) {
				zi = 2.0 * zr * zi + ci;
				zr = tr - ti + cr;
				tr = zr * zr;
				ti = zi * zi;
			}
			if (tr + ti <= LIMIT_SQUARED)
				bitmap[y * rowBytes + x / 8] |= static_cast<unsigned char>(0x80 >> (x % 8));
		}
	}

	std::print("P4\n{} {}\n", size, size);
	std::fwrite(bitmap.data(), 1, bitmap.size(), stdout);
}
