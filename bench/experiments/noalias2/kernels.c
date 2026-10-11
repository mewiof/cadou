// three loops where a run-time overlap check cannot stand in for a no-alias promise.
// The loops get called through pointers the compiler cannot see through. Direct calls
// let clang specialize a kernel to its only caller (the question then disappears)
//
//   kernels fields|arrays|call plain|restrict

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

// ================================================================
// FIELDS
// ================================================================

#define PIXEL_COUNT 16384
#define FIELD_ROUNDS 100000

typedef struct {
	uint8_t* pixels;
	size_t count;
} Image;

// a byte store may land in *image. pixels and count get reloaded after every
// store and the trip count is unknown when the loop starts
__attribute__((noinline)) static void Brighten(Image* image, uint8_t amount) {
	for (size_t i = 0; i < image->count; i++)
		image->pixels[i] += amount;
}

__attribute__((noinline)) static void BrightenRestrict(Image* restrict image, uint8_t amount) {
	for (size_t i = 0; i < image->count; i++)
		image->pixels[i] += amount;
}

static void (*const volatile brightenKernels[])(Image*, uint8_t) = {Brighten, BrightenRestrict};

static uint64_t RunFields(int useRestrict) {
	void (*brighten)(Image*, uint8_t) = brightenKernels[useRestrict];
	Image image = {malloc(PIXEL_COUNT), PIXEL_COUNT};
	if (!image.pixels)
		exit(1);
	for (size_t i = 0; i < PIXEL_COUNT; i++)
		image.pixels[i] = (uint8_t)(i * 7);
	for (int round = 0; round < FIELD_ROUNDS; round++) {
		brighten(&image, (uint8_t)(round & 15));
	}
	uint64_t sum = 0;
	for (size_t i = 0; i < PIXEL_COUNT; i++)
		sum += image.pixels[i] * (i + 1);
	free(image.pixels);
	return sum;
}

// ================================================================
// ARRAYS
// ================================================================

#define PARTICLE_COUNT 1024
#define ARRAY_ROUNDS 1000000

// three written arrays against five others need twelve overlap checks
__attribute__((noinline)) static void Move(float* x, float* y, float* z, const float* vx, const float* vy, const float* vz, float dt, size_t count) {
	for (size_t i = 0; i < count; i++) {
		x[i] += vx[i] * dt;
		y[i] += vy[i] * dt;
		z[i] += vz[i] * dt;
	}
}

__attribute__((noinline)) static void MoveRestrict(float* restrict x, float* restrict y, float* restrict z, const float* restrict vx, const float* restrict vy, const float* restrict vz, float dt, size_t count) {
	for (size_t i = 0; i < count; i++) {
		x[i] += vx[i] * dt;
		y[i] += vy[i] * dt;
		z[i] += vz[i] * dt;
	}
}

static void (*const volatile moveKernels[])(float*, float*, float*, const float*, const float*, const float*, float, size_t) = {Move, MoveRestrict};

static double RunArrays(int useRestrict) {
	void (*move)(float*, float*, float*, const float*, const float*, const float*, float, size_t) = moveKernels[useRestrict];
	float* data = malloc(6 * PARTICLE_COUNT * sizeof *data);
	if (!data)
		exit(1);
	float* x = data;
	float* y = x + PARTICLE_COUNT;
	float* z = y + PARTICLE_COUNT;
	float* vx = z + PARTICLE_COUNT;
	float* vy = vx + PARTICLE_COUNT;
	float* vz = vy + PARTICLE_COUNT;
	for (size_t i = 0; i < PARTICLE_COUNT; i++) {
		x[i] = (float)i;
		y[i] = (float)(2 * i);
		z[i] = (float)(3 * i);
		vx[i] = (float)(i % 7) - 3.0f;
		vy[i] = (float)(i % 5) - 2.0f;
		vz[i] = (float)(i % 3) - 1.0f;
	}
	for (int round = 0; round < ARRAY_ROUNDS; round++)
		move(x, y, z, vx, vy, vz, 0.001f, PARTICLE_COUNT);
	double sum = 0.0;
	for (size_t i = 0; i < PARTICLE_COUNT; i++)
		sum += (double)x[i] + (double)y[i] + (double)z[i];
	free(data);
	return sum;
}

// ================================================================
// CALL
// ================================================================

#define VALUE_COUNT 4096
#define CALL_ROUNDS 25000

static uint32_t Square(uint32_t value) {
	return value * value;
}

// the callee is unknown. Without the promise it may read or write *total
__attribute__((noinline)) static void Apply(uint64_t* total, const uint32_t* values, size_t count, uint32_t (*transform)(uint32_t)) {
	for (size_t i = 0; i < count; i++)
		*total += transform(values[i]);
}

__attribute__((noinline)) static void ApplyRestrict(uint64_t* restrict total, const uint32_t* restrict values, size_t count, uint32_t (*transform)(uint32_t)) {
	for (size_t i = 0; i < count; i++)
		*total += transform(values[i]);
}

static void (*const volatile applyKernels[])(uint64_t*, const uint32_t*, size_t, uint32_t (*)(uint32_t)) = {Apply, ApplyRestrict};
static uint32_t (*const volatile hiddenSquare)(uint32_t) = Square;

static uint64_t RunCall(int useRestrict) {
	void (*apply)(uint64_t*, const uint32_t*, size_t, uint32_t (*)(uint32_t)) = applyKernels[useRestrict];
	uint32_t* values = malloc(VALUE_COUNT * sizeof *values);
	if (!values)
		exit(1);
	for (size_t i = 0; i < VALUE_COUNT; i++)
		values[i] = (uint32_t)(i * 2654435761u);
	uint32_t (*transform)(uint32_t) = hiddenSquare;
	uint64_t total = 0;
	for (int round = 0; round < CALL_ROUNDS; round++)
		apply(&total, values, VALUE_COUNT, transform);
	free(values);
	return total;
}

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	const char* kernel = argc > 1 ? argv[1] : "";
	const char* mode = argc > 2 ? argv[2] : "";
	int useRestrict = strcmp(mode, "restrict") == 0;
	if (!useRestrict && strcmp(mode, "plain") != 0) {
		fputs("usage: kernels fields|arrays|call plain|restrict\n", stderr);
		return 1;
	}
	if (strcmp(kernel, "fields") == 0)
		printf("%llu\n", (unsigned long long)RunFields(useRestrict));
	else if (strcmp(kernel, "arrays") == 0)
		printf("%.3f\n", RunArrays(useRestrict));
	else if (strcmp(kernel, "call") == 0)
		printf("%llu\n", (unsigned long long)RunCall(useRestrict));
	else {
		fputs("usage: kernels fields|arrays|call plain|restrict\n", stderr);
		return 1;
	}
	return 0;
}
