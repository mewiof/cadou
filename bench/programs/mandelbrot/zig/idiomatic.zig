// mandelbrot: up to 50 iterations per point for re in -1.5..0.5 and im in -1..1 (binary PBM out)

const std = @import("std");

const iterations = 50;
const limit_squared: f64 = 4.0;

pub fn main(init: std.process.Init) !void {
	const gpa = init.gpa;
	const args = try init.minimal.args.toSlice(init.arena.allocator());
	const size = if (args.len > 1) try std.fmt.parseInt(usize, args[1], 10) else 200;
	const row_bytes = (size + 7) / 8;
	const bitmap = try gpa.alloc(u8, row_bytes * size);
	defer gpa.free(bitmap);
	@memset(bitmap, 0);

	const size_f: f64 = @floatFromInt(size);
	for (0..size) |y| {
		const ci = 2.0 * @as(f64, @floatFromInt(y)) / size_f - 1.0;
		const row = bitmap[y * row_bytes ..][0..row_bytes];
		for (0..size) |x| {
			const cr = 2.0 * @as(f64, @floatFromInt(x)) / size_f - 1.5;
			var zr: f64 = 0.0;
			var zi: f64 = 0.0;
			var tr: f64 = 0.0;
			var ti: f64 = 0.0;
			var i: u32 = 0;
			while (i < iterations and tr + ti <= limit_squared) : (i += 1) {
				zi = 2.0 * zr * zi + ci;
				zr = tr - ti + cr;
				tr = zr * zr;
				ti = zi * zi;
			}
			if (tr + ti <= limit_squared) row[x / 8] |= @as(u8, 0x80) >> @intCast(x % 8);
		}
	}

	var buffer: [4096]u8 = undefined;
	var stdout_writer = std.Io.File.stdout().writer(init.io, &buffer);
	const out = &stdout_writer.interface;
	try out.print("P4\n{d} {d}\n", .{ size, size });
	try out.writeAll(bitmap);
	try out.flush();
}
