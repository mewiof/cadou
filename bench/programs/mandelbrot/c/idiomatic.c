// mandelbrot: up to 50 iterations per point for re in -1.5..0.5 and im in -1..1 (binary PBM out)

#include <stdio.h>
#include <stdlib.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

#define ITERATIONS 50
#define LIMIT_SQUARED 4.0

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	int size = argc > 1 ? atoi(argv[1]) : 200;
	int rowBytes = (size + 7) / 8;
	unsigned char* bitmap = calloc((size_t)rowBytes * size, 1);
	if (!bitmap) {
		fputs("out of memory\n", stderr);
		return 1;
	}

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
				bitmap[(size_t)y * rowBytes + x / 8] |= (unsigned char)(0x80 >> (x % 8));
		}
	}

	printf("P4\n%d %d\n", size, size);
	fwrite(bitmap, 1, (size_t)rowBytes * size, stdout);
	free(bitmap);
	return 0;
}
