// spectral-norm: power method on A(i, j) = 1 / ((i + j)(i + j + 1) / 2 + i + 1)

const std = @import("std");

fn a(i: usize, j: usize) f64 {
	return 1.0 / @as(f64, @floatFromInt((i + j) * (i + j + 1) / 2 + i + 1));
}

fn multiplyAv(v: []const f64, out: []f64) void {
	for (out, 0..) |*o, i| {
		var sum: f64 = 0.0;
		for (v, 0..) |x, j| sum += a(i, j) * x;
		o.* = sum;
	}
}

fn multiplyAtv(v: []const f64, out: []f64) void {
	for (out, 0..) |*o, i| {
		var sum: f64 = 0.0;
		for (v, 0..) |x, j| sum += a(j, i) * x;
		o.* = sum;
	}
}

fn multiplyAtAv(v: []const f64, out: []f64, scratch: []f64) void {
	multiplyAv(v, scratch);
	multiplyAtv(scratch, out);
}

pub fn main(init: std.process.Init) !void {
	const gpa = init.gpa;
	const args = try init.minimal.args.toSlice(init.arena.allocator());
	const n = if (args.len > 1) try std.fmt.parseInt(usize, args[1], 10) else 100;

	const u = try gpa.alloc(f64, n);
	defer gpa.free(u);
	const v = try gpa.alloc(f64, n);
	defer gpa.free(v);
	const scratch = try gpa.alloc(f64, n);
	defer gpa.free(scratch);

	@memset(u, 1.0);
	for (0..10) |_| {
		multiplyAtAv(u, v, scratch);
		multiplyAtAv(v, u, scratch);
	}

	var vbv: f64 = 0.0;
	var vv: f64 = 0.0;
	for (u, v) |x, y| {
		vbv += x * y;
		vv += y * y;
	}

	var buffer: [256]u8 = undefined;
	var stdout_writer = std.Io.File.stdout().writer(init.io, &buffer);
	const out = &stdout_writer.interface;
	try out.print("{d:.9}\n", .{@sqrt(vbv / vv)});
	try out.flush();
}
