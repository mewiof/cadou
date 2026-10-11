// spectral-norm: power method on A(i, j) = 1 / ((i + j)(i + j + 1) / 2 + i + 1)

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

static double A(int i, int j) {
	return 1.0 / ((i + j) * (i + j + 1) / 2 + i + 1);
}

static void MultiplyAv(int n, const double* v, double* out) {
	for (int i = 0; i < n; i++) {
		double sum = 0.0;
		for (int j = 0; j < n; j++)
			sum += A(i, j) * v[j];
		out[i] = sum;
	}
}

static void MultiplyAtv(int n, const double* v, double* out) {
	for (int i = 0; i < n; i++) {
		double sum = 0.0;
		for (int j = 0; j < n; j++)
			sum += A(j, i) * v[j];
		out[i] = sum;
	}
}

static void MultiplyAtAv(int n, const double* v, double* out, double* scratch) {
	MultiplyAv(n, v, scratch);
	MultiplyAtv(n, scratch, out);
}

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	int n = argc > 1 ? atoi(argv[1]) : 100;
	double* u = malloc((size_t)n * sizeof *u);
	double* v = malloc((size_t)n * sizeof *v);
	double* scratch = malloc((size_t)n * sizeof *scratch);
	if (!u || !v || !scratch) {
		fputs("out of memory\n", stderr);
		return 1;
	}

	for (int i = 0; i < n; i++)
		u[i] = 1.0;
	for (int i = 0; i < 10; i++) {
		MultiplyAtAv(n, u, v, scratch);
		MultiplyAtAv(n, v, u, scratch);
	}

	double vBv = 0.0, vv = 0.0;
	for (int i = 0; i < n; i++) {
		vBv += u[i] * v[i];
		vv += v[i] * v[i];
	}
	printf("%.9f\n", sqrt(vBv / vv));

	free(u);
	free(v);
	free(scratch);
	return 0;
}
