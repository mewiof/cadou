// spectral-norm: power method on A(i, j) = 1 / ((i + j)(i + j + 1) / 2 + i + 1)

#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <print>
#include <span>
#include <vector>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

namespace {
	double A(int i, int j) {
		return 1.0 / ((i + j) * (i + j + 1) / 2 + i + 1);
	}

	void MultiplyAv(std::span<const double> v, std::span<double> out) {
		int n = static_cast<int>(v.size());
		for (int i = 0; i < n; i++) {
			double sum = 0.0;
			for (int j = 0; j < n; j++)
				sum += A(i, j) * v[j];
			out[i] = sum;
		}
	}

	void MultiplyAtv(std::span<const double> v, std::span<double> out) {
		int n = static_cast<int>(v.size());
		for (int i = 0; i < n; i++) {
			double sum = 0.0;
			for (int j = 0; j < n; j++)
				sum += A(j, i) * v[j];
			out[i] = sum;
		}
	}

	void MultiplyAtAv(std::span<const double> v, std::span<double> out, std::span<double> scratch) {
		MultiplyAv(v, scratch);
		MultiplyAtv(scratch, out);
	}
}

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	int n = argc > 1 ? std::atoi(argv[1]) : 100;
	std::vector<double> u(n, 1.0), v(n), scratch(n);
	for (int i = 0; i < 10; i++) {
		MultiplyAtAv(u, v, scratch);
		MultiplyAtAv(v, u, scratch);
	}

	double vBv = 0.0, vv = 0.0;
	for (int i = 0; i < n; i++) {
		vBv += u[i] * v[i];
		vv += v[i] * v[i];
	}
	std::println("{:.9f}", std::sqrt(vBv / vv));
}
