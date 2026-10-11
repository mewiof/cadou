// sort: n random 64-bit integers with the standard library sort. Prints the
// smallest/median/largest value and a checksum that depends on the order

const std = @import("std");

// splitmix64 (the same stream in every language)
const SplitMix64 = struct {
	state: u64,

	fn next(self: *SplitMix64) u64 {
		self.state +%= 0x9E3779B97F4A7C15;
		var z = self.state;
		z = (z ^ (z >> 30)) *% 0xBF58476D1CE4E5B9;
		z = (z ^ (z >> 27)) *% 0x94D049BB133111EB;
		return z ^ (z >> 31);
	}
};

pub fn main(init: std.process.Init) !void {
	const args = try init.minimal.args.toSlice(init.arena.allocator());
	const n = if (args.len > 1) try std.fmt.parseInt(usize, args[1], 10) else 1000;
	if (n == 0) std.process.exit(1);

	const values = try init.gpa.alloc(u64, n);
	defer init.gpa.free(values);
	var random: SplitMix64 = .{ .state = 0 };
	for (values) |*value| value.* = random.next();

	std.mem.sortUnstable(u64, values, {}, std.sort.asc(u64));

	var checksum: u64 = 0;
	for (values, 1..) |value, i| checksum +%= value *% @as(u64, i);

	var buffer: [128]u8 = undefined;
	var stdout_writer = std.Io.File.stdout().writer(init.io, &buffer);
	const out = &stdout_writer.interface;
	try out.print("{d} {d} {d} {d}\n", .{ values[0], values[n / 2], values[n - 1], checksum });
	try out.flush();
}
