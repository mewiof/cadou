// n-body: the sun and the four Jovian planets under symplectic Euler (dt = 0.01)

#include <array>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <print>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

namespace {
	constexpr double PI = 3.141592653589793;
	constexpr double SOLAR_MASS = 4.0 * PI * PI;
	constexpr double DAYS_PER_YEAR = 365.24;

	struct Body {
		double x, y, z;
		double vx, vy, vz;
		double mass;
	};

	using Bodies = std::array<Body, 5>;

	void OffsetMomentum(Bodies& bodies) {
		double px = 0.0, py = 0.0, pz = 0.0;
		for (const Body& body : bodies) {
			px += body.vx * body.mass;
			py += body.vy * body.mass;
			pz += body.vz * body.mass;
		}
		bodies[0].vx = -px / SOLAR_MASS;
		bodies[0].vy = -py / SOLAR_MASS;
		bodies[0].vz = -pz / SOLAR_MASS;
	}

	double Energy(const Bodies& bodies) {
		double e = 0.0;
		for (std::size_t i = 0; i < bodies.size(); i++) {
			const Body& a = bodies[i];
			e += 0.5 * a.mass * (a.vx * a.vx + a.vy * a.vy + a.vz * a.vz);
			for (std::size_t j = i + 1; j < bodies.size(); j++) {
				const Body& b = bodies[j];
				double dx = a.x - b.x;
				double dy = a.y - b.y;
				double dz = a.z - b.z;
				e -= a.mass * b.mass / std::sqrt(dx * dx + dy * dy + dz * dz);
			}
		}
		return e;
	}

	void Advance(Bodies& bodies, double dt) {
		for (std::size_t i = 0; i < bodies.size(); i++) {
			Body& a = bodies[i];
			for (std::size_t j = i + 1; j < bodies.size(); j++) {
				Body& b = bodies[j];
				double dx = a.x - b.x;
				double dy = a.y - b.y;
				double dz = a.z - b.z;
				double distanceSquared = dx * dx + dy * dy + dz * dz;
				double magnitude = dt / (distanceSquared * std::sqrt(distanceSquared));
				a.vx -= dx * b.mass * magnitude;
				a.vy -= dy * b.mass * magnitude;
				a.vz -= dz * b.mass * magnitude;
				b.vx += dx * a.mass * magnitude;
				b.vy += dy * a.mass * magnitude;
				b.vz += dz * a.mass * magnitude;
			}
		}
		for (Body& body : bodies) {
			body.x += dt * body.vx;
			body.y += dt * body.vy;
			body.z += dt * body.vz;
		}
	}
}

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	int steps = argc > 1 ? std::atoi(argv[1]) : 1000;
	Bodies bodies{{
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
	}};

	OffsetMomentum(bodies);
	std::println("{:.9f}", Energy(bodies));
	for (int i = 0; i < steps; i++)
		Advance(bodies, 0.01);
	std::println("{:.9f}", Energy(bodies));
}
