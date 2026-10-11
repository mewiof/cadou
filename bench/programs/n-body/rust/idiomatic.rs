// n-body: the sun and the four Jovian planets under symplectic Euler (dt = 0.01)

use std::f64::consts::PI;

const SOLAR_MASS: f64 = 4.0 * PI * PI;
const DAYS_PER_YEAR: f64 = 365.24;

struct Body {
	x: f64,
	y: f64,
	z: f64,
	vx: f64,
	vy: f64,
	vz: f64,
	mass: f64,
}

fn offset_momentum(bodies: &mut [Body; 5]) {
	let (mut px, mut py, mut pz) = (0.0, 0.0, 0.0);
	for body in bodies.iter() {
		px += body.vx * body.mass;
		py += body.vy * body.mass;
		pz += body.vz * body.mass;
	}
	let sun = &mut bodies[0];
	sun.vx = -px / SOLAR_MASS;
	sun.vy = -py / SOLAR_MASS;
	sun.vz = -pz / SOLAR_MASS;
}

fn energy(bodies: &[Body; 5]) -> f64 {
	let mut e = 0.0;
	for (i, a) in bodies.iter().enumerate() {
		e += 0.5 * a.mass * (a.vx * a.vx + a.vy * a.vy + a.vz * a.vz);
		for b in &bodies[i + 1..] {
			let dx = a.x - b.x;
			let dy = a.y - b.y;
			let dz = a.z - b.z;
			e -= a.mass * b.mass / (dx * dx + dy * dy + dz * dz).sqrt();
		}
	}
	e
}

fn advance(bodies: &mut [Body; 5], dt: f64) {
	for i in 0..bodies.len() {
		let (head, tail) = bodies.split_at_mut(i + 1);
		let a = &mut head[i];
		for b in tail.iter_mut() {
			let dx = a.x - b.x;
			let dy = a.y - b.y;
			let dz = a.z - b.z;
			let distance_squared = dx * dx + dy * dy + dz * dz;
			let magnitude = dt / (distance_squared * distance_squared.sqrt());
			a.vx -= dx * b.mass * magnitude;
			a.vy -= dy * b.mass * magnitude;
			a.vz -= dz * b.mass * magnitude;
			b.vx += dx * a.mass * magnitude;
			b.vy += dy * a.mass * magnitude;
			b.vz += dz * a.mass * magnitude;
		}
	}
	for body in bodies.iter_mut() {
		body.x += dt * body.vx;
		body.y += dt * body.vy;
		body.z += dt * body.vz;
	}
}

fn main() {
	let steps: u32 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1000);
	let mut bodies = [
		// sun
		Body { x: 0.0, y: 0.0, z: 0.0, vx: 0.0, vy: 0.0, vz: 0.0, mass: SOLAR_MASS },
		// jupiter
		Body {
			x: 4.84143144246472090e+00,
			y: -1.16032004402742839e+00,
			z: -1.03622044471123109e-01,
			vx: 1.66007664274403694e-03 * DAYS_PER_YEAR,
			vy: 7.69901118419740425e-03 * DAYS_PER_YEAR,
			vz: -6.90460016972063023e-05 * DAYS_PER_YEAR,
			mass: 9.54791938424326609e-04 * SOLAR_MASS,
		},
		// saturn
		Body {
			x: 8.34336671824457987e+00,
			y: 4.12479856412430479e+00,
			z: -4.03523417114321381e-01,
			vx: -2.76742510726862411e-03 * DAYS_PER_YEAR,
			vy: 4.99852801234917238e-03 * DAYS_PER_YEAR,
			vz: 2.30417297573763929e-05 * DAYS_PER_YEAR,
			mass: 2.85885980666130812e-04 * SOLAR_MASS,
		},
		// uranus
		Body {
			x: 1.28943695621391310e+01,
			y: -1.51111514016986312e+01,
			z: -2.23307578892655734e-01,
			vx: 2.96460137564761618e-03 * DAYS_PER_YEAR,
			vy: 2.37847173959480950e-03 * DAYS_PER_YEAR,
			vz: -2.96589568540237556e-05 * DAYS_PER_YEAR,
			mass: 4.36624404335156298e-05 * SOLAR_MASS,
		},
		// neptune
		Body {
			x: 1.53796971148509165e+01,
			y: -2.59193146099879641e+01,
			z: 1.79258772950371181e-01,
			vx: 2.68067772490389322e-03 * DAYS_PER_YEAR,
			vy: 1.62824170038242295e-03 * DAYS_PER_YEAR,
			vz: -9.51592254519715870e-05 * DAYS_PER_YEAR,
			mass: 5.15138902046611451e-05 * SOLAR_MASS,
		},
	];

	offset_momentum(&mut bodies);
	println!("{:.9}", energy(&bodies));
	for _ in 0..steps {
		advance(&mut bodies, 0.01);
	}
	println!("{:.9}", energy(&bodies));
}
