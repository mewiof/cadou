// n-body: the sun and the four Jovian planets under symplectic Euler (dt = 0.01)

const std = @import("std");

// typed constants make the products below round like run-time f64 math (comptime_float would not)
const pi: f64 = 3.141592653589793;
const solar_mass: f64 = 4.0 * pi * pi;
const days_per_year: f64 = 365.24;

const Body = struct {
	x: f64,
	y: f64,
	z: f64,
	vx: f64,
	vy: f64,
	vz: f64,
	mass: f64,
};

fn offsetMomentum(bodies: *[5]Body) void {
	var px: f64 = 0.0;
	var py: f64 = 0.0;
	var pz: f64 = 0.0;
	for (bodies) |body| {
		px += body.vx * body.mass;
		py += body.vy * body.mass;
		pz += body.vz * body.mass;
	}
	bodies[0].vx = -px / solar_mass;
	bodies[0].vy = -py / solar_mass;
	bodies[0].vz = -pz / solar_mass;
}

fn energy(bodies: *const [5]Body) f64 {
	var e: f64 = 0.0;
	for (bodies, 0..) |a, i| {
		e += 0.5 * a.mass * (a.vx * a.vx + a.vy * a.vy + a.vz * a.vz);
		for (bodies[i + 1 ..]) |b| {
			const dx = a.x - b.x;
			const dy = a.y - b.y;
			const dz = a.z - b.z;
			e -= a.mass * b.mass / @sqrt(dx * dx + dy * dy + dz * dz);
		}
	}
	return e;
}

fn advance(bodies: *[5]Body, dt: f64) void {
	for (bodies, 0..) |*a, i| {
		for (bodies[i + 1 ..]) |*b| {
			const dx = a.x - b.x;
			const dy = a.y - b.y;
			const dz = a.z - b.z;
			const distance_squared = dx * dx + dy * dy + dz * dz;
			const magnitude = dt / (distance_squared * @sqrt(distance_squared));
			a.vx -= dx * b.mass * magnitude;
			a.vy -= dy * b.mass * magnitude;
			a.vz -= dz * b.mass * magnitude;
			b.vx += dx * a.mass * magnitude;
			b.vy += dy * a.mass * magnitude;
			b.vz += dz * a.mass * magnitude;
		}
	}
	for (bodies) |*body| {
		body.x += dt * body.vx;
		body.y += dt * body.vy;
		body.z += dt * body.vz;
	}
}

pub fn main(init: std.process.Init) !void {
	const args = try init.minimal.args.toSlice(init.arena.allocator());
	const steps = if (args.len > 1) try std.fmt.parseInt(u32, args[1], 10) else 1000;

	var bodies = [5]Body{
		// sun
		.{ .x = 0.0, .y = 0.0, .z = 0.0, .vx = 0.0, .vy = 0.0, .vz = 0.0, .mass = solar_mass },
		// jupiter
		.{
			.x = 4.84143144246472090e+00,
			.y = -1.16032004402742839e+00,
			.z = -1.03622044471123109e-01,
			.vx = 1.66007664274403694e-03 * days_per_year,
			.vy = 7.69901118419740425e-03 * days_per_year,
			.vz = -6.90460016972063023e-05 * days_per_year,
			.mass = 9.54791938424326609e-04 * solar_mass,
		},
		// saturn
		.{
			.x = 8.34336671824457987e+00,
			.y = 4.12479856412430479e+00,
			.z = -4.03523417114321381e-01,
			.vx = -2.76742510726862411e-03 * days_per_year,
			.vy = 4.99852801234917238e-03 * days_per_year,
			.vz = 2.30417297573763929e-05 * days_per_year,
			.mass = 2.85885980666130812e-04 * solar_mass,
		},
		// uranus
		.{
			.x = 1.28943695621391310e+01,
			.y = -1.51111514016986312e+01,
			.z = -2.23307578892655734e-01,
			.vx = 2.96460137564761618e-03 * days_per_year,
			.vy = 2.37847173959480950e-03 * days_per_year,
			.vz = -2.96589568540237556e-05 * days_per_year,
			.mass = 4.36624404335156298e-05 * solar_mass,
		},
		// neptune
		.{
			.x = 1.53796971148509165e+01,
			.y = -2.59193146099879641e+01,
			.z = 1.79258772950371181e-01,
			.vx = 2.68067772490389322e-03 * days_per_year,
			.vy = 1.62824170038242295e-03 * days_per_year,
			.vz = -9.51592254519715870e-05 * days_per_year,
			.mass = 5.15138902046611451e-05 * solar_mass,
		},
	};

	var buffer: [256]u8 = undefined;
	var stdout_writer = std.Io.File.stdout().writer(init.io, &buffer);
	const out = &stdout_writer.interface;

	offsetMomentum(&bodies);
	try out.print("{d:.9}\n", .{energy(&bodies)});
	for (0..steps) |_| advance(&bodies, 0.01);
	try out.print("{d:.9}\n", .{energy(&bodies)});
	try out.flush();
}
