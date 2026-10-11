// hash-map: insert n random 64-bit keys. Look up each of them plus n absent keys
// from a second stream

const std = @import("std");

// splitmix64 (the same streams in every language)
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
	const n = if (args.len > 1) try std.fmt.parseInt(u64, args[1], 10) else 1000;

	var map = std.AutoHashMap(u64, u64).init(init.gpa);
	defer map.deinit();
	var keys: SplitMix64 = .{ .state = 0 };
	for (0..n) |i| try map.put(keys.next(), i);

	var again: SplitMix64 = .{ .state = 0 };
	var others: SplitMix64 = .{ .state = 1 };
	var hits: u64 = 0;
	var sum: u64 = 0;
	for (0..n) |_| {
		if (map.get(again.next())) |value| {
			hits += 1;
			sum += value;
		}
		if (map.get(others.next())) |value| {
			hits += 1;
			sum += value;
		}
	}

	var buffer: [64]u8 = undefined;
	var stdout_writer = std.Io.File.stdout().writer(init.io, &buffer);
	const out = &stdout_writer.interface;
	try out.print("{d} {d}\n", .{ hits, sum });
	try out.flush();
}
