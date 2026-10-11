// add every element into total.*: Zig pointers may alias unless the parameter
// is marked noalias

const std = @import("std");

const count = 1 << 20;
const rounds = 1000;

noinline fn accumulate(total: *u32, values: []const u32) void {
	for (values) |value| total.* +%= value;
}

noinline fn accumulateNoalias(noalias total: *u32, noalias values: []const u32) void {
	for (values) |value| total.* +%= value;
}

pub fn main(init: std.process.Init) !void {
	const gpa = init.gpa;
	const args = try init.minimal.args.toSlice(init.arena.allocator());
	const mode = if (args.len > 1) args[1] else "";
	const use_noalias = std.mem.eql(u8, mode, "noalias");
	if (!use_noalias and !std.mem.eql(u8, mode, "plain")) std.process.fatal("usage: accumulate plain|noalias", .{});

	const values = try gpa.alloc(u32, count);
	defer gpa.free(values);
	for (values, 0..) |*value, i| value.* = @intCast(i & 7);

	var total: u32 = 0;
	for (0..rounds) |_| {
		if (use_noalias) accumulateNoalias(&total, values) else accumulate(&total, values);
	}

	var buffer: [64]u8 = undefined;
	var stdout_writer = std.Io.File.stdout().writer(init.io, &buffer);
	const out = &stdout_writer.interface;
	try out.print("{d}\n", .{total});
	try out.flush();
}
