// n-body: the sun and the four Jovian planets under symplectic Euler (dt = 0.01)

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

#define PI 3.141592653589793
#define SOLAR_MASS (4.0 * PI * PI)
#define DAYS_PER_YEAR 365.24
#define BODY_COUNT 5

typedef struct {
	double x, y, z;
	double vx, vy, vz;
	double mass;
} Body;

static void OffsetMomentum(Body* bodies) {
	double px = 0.0, py = 0.0, pz = 0.0;
	for (int i = 0; i < BODY_COUNT; i++) {
		px += bodies[i].vx * bodies[i].mass;
		py += bodies[i].vy * bodies[i].mass;
		pz += bodies[i].vz * bodies[i].mass;
	}
	bodies[0].vx = -px / SOLAR_MASS;
	bodies[0].vy = -py / SOLAR_MASS;
	bodies[0].vz = -pz / SOLAR_MASS;
}

static double Energy(const Body* bodies) {
	double e = 0.0;
	for (int i = 0; i < BODY_COUNT; i++) {
		const Body* a = &bodies[i];
		e += 0.5 * a->mass * (a->vx * a->vx + a->vy * a->vy + a->vz * a->vz);
		for (int j = i + 1; j < BODY_COUNT; j++) {
			const Body* b = &bodies[j];
			double dx = a->x - b->x;
			double dy = a->y - b->y;
			double dz = a->z - b->z;
			e -= a->mass * b->mass / sqrt(dx * dx + dy * dy + dz * dz);
		}
	}
	return e;
}

static void Advance(Body* bodies, double dt) {
	for (int i = 0; i < BODY_COUNT; i++) {
		Body* a = &bodies[i];
		for (int j = i + 1; j < BODY_COUNT; j++) {
			Body* b = &bodies[j];
			double dx = a->x - b->x;
			double dy = a->y - b->y;
			double dz = a->z - b->z;
			double distanceSquared = dx * dx + dy * dy + dz * dz;
			double magnitude = dt / (distanceSquared * sqrt(distanceSquared));
			a->vx -= dx * b->mass * magnitude;
			a->vy -= dy * b->mass * magnitude;
			a->vz -= dz * b->mass * magnitude;
			b->vx += dx * a->mass * magnitude;
			b->vy += dy * a->mass * magnitude;
			b->vz += dz * a->mass * magnitude;
		}
	}
	for (int i = 0; i < BODY_COUNT; i++) {
		bodies[i].x += dt * bodies[i].vx;
		bodies[i].y += dt * bodies[i].vy;
		bodies[i].z += dt * bodies[i].vz;
	}
}

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	int steps = argc > 1 ? atoi(argv[1]) : 1000;
	Body bodies[BODY_COUNT] = {
		// sun
		{0.0, 0.0, 0.0, 0.0, 0.0, 0.0, SOLAR_MASS},
		// jupiter
		{4.84143144246472090e+00, -1.16032004402742839e+00, -1.03622044471123109e-01,
			1.66007664274403694e-03 * DAYS_PER_YEAR, 7.69901118419740425e-03 * DAYS_PER_YEAR, -6.90460016972063023e-05 * DAYS_PER_YEAR,
			9.54791938424326609e-04 * SOLAR_MASS},
		// saturn
		{8.34336671824457987e+00, 4.12479856412430479e+00, -4.03523417114321381e-01,
			-2.76742510726862411e-03 * DAYS_PER_YEAR, 4.99852801234917238e-03 * DAYS_PER_YEAR, 2.30417297573763929e-05 * DAYS_PER_YEAR,
			2.85885980666130812e-04 * SOLAR_MASS},
		// uranus
		{1.28943695621391310e+01, -1.51111514016986312e+01, -2.23307578892655734e-01,
			2.96460137564761618e-03 * DAYS_PER_YEAR, 2.37847173959480950e-03 * DAYS_PER_YEAR, -2.96589568540237556e-05 * DAYS_PER_YEAR,
			4.36624404335156298e-05 * SOLAR_MASS},
		// neptune
		{1.53796971148509165e+01, -2.59193146099879641e+01, 1.79258772950371181e-01,
			2.68067772490389322e-03 * DAYS_PER_YEAR, 1.62824170038242295e-03 * DAYS_PER_YEAR, -9.51592254519715870e-05 * DAYS_PER_YEAR,
			5.15138902046611451e-05 * SOLAR_MASS},
	};

	OffsetMomentum(bodies);
	printf("%.9f\n", Energy(bodies));
	for (int i = 0; i < steps; i++)
		Advance(bodies, 0.01);
	printf("%.9f\n", Energy(bodies));
	return 0;
}
