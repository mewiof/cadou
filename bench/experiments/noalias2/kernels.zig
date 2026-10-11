// three loops where a run-time overlap check cannot stand in for a no-alias promise.
// Zig pointers may alias unless the parameter is marked noalias. The loops get called
// through pointers read back as volatile (direct calls let LLVM specialize a kernel
// to its only caller)
//
//   kernels fields|arrays|call plain|noalias

const std = @import("std");

fn hidden(comptime T: type, value: T) T {
	var copy = value;
	return @as(*volatile T, &copy).*;
}

// ================================================================
// FIELDS
// ================================================================

const pixel_count = 16384;
const field_rounds = 100000;

const Image = struct {
	pixels: []u8,
};

// a byte store may land in image.* (the slice's pointer and length)
noinline fn brighten(image: *Image, amount: u8) void {
	var i: usize = 0;
	while (i < image.pixels.len) : (i += 1) image.pixels[i] +%= amount;
}

noinline fn brightenNoalias(noalias image: *Image, amount: u8) void {
	var i: usize = 0;
	while (i < image.pixels.len) : (i += 1) image.pixels[i] +%= amount;
}

fn runFields(gpa: std.mem.Allocator, use_noalias: bool) !u64 {
	var image: Image = .{ .pixels = try gpa.alloc(u8, pixel_count) };
	defer gpa.free(image.pixels);
	for (image.pixels, 0..) |*pixel, i| pixel.* = @truncate(i * 7);
	const kernel = hidden(*const fn (*Image, u8) void, &brighten);
	const kernel_noalias = hidden(*const fn (noalias *Image, u8) void, &brightenNoalias);
	for (0..field_rounds) |round| {
		const amount: u8 = @intCast(round & 15);
		if (use_noalias) kernel_noalias(&image, amount) else kernel(&image, amount);
	}
	var sum: u64 = 0;
	for (image.pixels, 0..) |pixel, i| sum += @as(u64, pixel) * @as(u64, i + 1);
	return sum;
}

// ================================================================
// ARRAYS
// ================================================================

const particle_count = 1024;
const array_rounds = 1000000;

noinline fn advance(x: []f32, y: []f32, z: []f32, vx: []const f32, vy: []const f32, vz: []const f32, dt: f32) void {
	for (0..x.len) |i| {
		x[i] += vx[i] * dt;
		y[i] += vy[i] * dt;
		z[i] += vz[i] * dt;
	}
}

noinline fn advanceNoalias(noalias x: []f32, noalias y: []f32, noalias z: []f32, noalias vx: []const f32, noalias vy: []const f32, noalias vz: []const f32, dt: f32) void {
	for (0..x.len) |i| {
		x[i] += vx[i] * dt;
		y[i] += vy[i] * dt;
		z[i] += vz[i] * dt;
	}
}

fn runArrays(gpa: std.mem.Allocator, use_noalias: bool) !f64 {
	const data = try gpa.alloc(f32, 6 * particle_count);
	defer gpa.free(data);
	const n = particle_count;
	const x = data[0..n];
	const y = data[n .. 2 * n];
	const z = data[2 * n .. 3 * n];
	const vx = data[3 * n .. 4 * n];
	const vy = data[4 * n .. 5 * n];
	const vz = data[5 * n .. 6 * n];
	for (0..n) |i| {
		x[i] = @floatFromInt(i);
		y[i] = @floatFromInt(2 * i);
		z[i] = @floatFromInt(3 * i);
		vx[i] = @as(f32, @floatFromInt(i % 7)) - 3.0;
		vy[i] = @as(f32, @floatFromInt(i % 5)) - 2.0;
		vz[i] = @as(f32, @floatFromInt(i % 3)) - 1.0;
	}
	const kernel = hidden(*const fn ([]f32, []f32, []f32, []const f32, []const f32, []const f32, f32) void, &advance);
	const kernel_noalias = hidden(*const fn (noalias []f32, noalias []f32, noalias []f32, noalias []const f32, noalias []const f32, noalias []const f32, f32) void, &advanceNoalias);
	for (0..array_rounds) |_| {
		if (use_noalias) kernel_noalias(x, y, z, vx, vy, vz, 0.001) else kernel(x, y, z, vx, vy, vz, 0.001);
	}
	var sum: f64 = 0.0;
	for (0..n) |i| sum += @as(f64, x[i]) + @as(f64, y[i]) + @as(f64, z[i]);
	return sum;
}

// ================================================================
// CALL
// ================================================================

const value_count = 4096;
const call_rounds = 25000;

fn square(value: u32) u32 {
	return value *% value;
}

// the callee is unknown. Without noalias it may read or write total.*
noinline fn apply(total: *u64, values: []const u32, transform: *const fn (u32) u32) void {
	for (values) |value| total.* += transform(value);
}

noinline fn applyNoalias(noalias total: *u64, noalias values: []const u32, transform: *const fn (u32) u32) void {
	for (values) |value| total.* += transform(value);
}

fn runCall(gpa: std.mem.Allocator, use_noalias: bool) !u64 {
	const values = try gpa.alloc(u32, value_count);
	defer gpa.free(values);
	for (values, 0..) |*value, i| value.* = @as(u32, @truncate(i)) *% 2654435761;
	const transform = hidden(*const fn (u32) u32, &square);
	const kernel = hidden(*const fn (*u64, []const u32, *const fn (u32) u32) void, &apply);
	const kernel_noalias = hidden(*const fn (noalias *u64, noalias []const u32, *const fn (u32) u32) void, &applyNoalias);
	var total: u64 = 0;
	for (0..call_rounds) |_| {
		if (use_noalias) kernel_noalias(&total, values, transform) else kernel(&total, values, transform);
	}
	return total;
}

pub fn main(init: std.process.Init) !void {
	const gpa = init.gpa;
	const args = try init.minimal.args.toSlice(init.arena.allocator());
	const kernel = if (args.len > 1) args[1] else "";
	const mode = if (args.len > 2) args[2] else "";
	const use_noalias = std.mem.eql(u8, mode, "noalias");
	if (!use_noalias and !std.mem.eql(u8, mode, "plain")) std.process.fatal("usage: kernels fields|arrays|call plain|noalias", .{});

	var buffer: [64]u8 = undefined;
	var stdout_writer = std.Io.File.stdout().writer(init.io, &buffer);
	const out = &stdout_writer.interface;
	if (std.mem.eql(u8, kernel, "fields")) {
		try out.print("{d}\n", .{try runFields(gpa, use_noalias)});
	} else if (std.mem.eql(u8, kernel, "arrays")) {
		try out.print("{d:.3}\n", .{try runArrays(gpa, use_noalias)});
	} else if (std.mem.eql(u8, kernel, "call")) {
		try out.print("{d}\n", .{try runCall(gpa, use_noalias)});
	} else {
		std.process.fatal("usage: kernels fields|arrays|call plain|noalias", .{});
	}
	try out.flush();
}
