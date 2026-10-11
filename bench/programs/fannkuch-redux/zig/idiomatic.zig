// fannkuch-redux: count prefix reversals over all permutations of 0..n-1

const std = @import("std");

const max_n = 16;

pub fn main(init: std.process.Init) !void {
	const args = try init.minimal.args.toSlice(init.arena.allocator());
	const n = if (args.len > 1) try std.fmt.parseInt(usize, args[1], 10) else 7;
	if (n < 1 or n > max_n) std.process.fatal("n must be in 1..{d}", .{max_n});

	var buffer: [256]u8 = undefined;
	var stdout_writer = std.Io.File.stdout().writer(init.io, &buffer);
	const out = &stdout_writer.interface;

	var perm1: [max_n]usize = undefined;
	var count: [max_n]usize = undefined;
	var max_flips: i32 = 0;
	var checksum: i32 = 0;
	var perm_index: u64 = 0;
	for (perm1[0..n], 0..) |*p, i| p.* = i;

	var r = n;
	while (true) {
		while (r != 1) : (r -= 1) count[r - 1] = r;

		var perm = perm1;
		var flips: i32 = 0;
		var k = perm[0];
		while (k != 0) : (k = perm[0]) {
			std.mem.reverse(usize, perm[0 .. k + 1]);
			flips += 1;
		}
		max_flips = @max(max_flips, flips);
		checksum += if (perm_index % 2 == 0) flips else -flips;

		// next permutation in counting order (rotates the first r + 1 elements)
		while (true) {
			if (r == n) {
				try out.print("{d}\nPfannkuchen({d}) = {d}\n", .{ checksum, n, max_flips });
				try out.flush();
				return;
			}
			std.mem.rotate(usize, perm1[0 .. r + 1], 1);
			count[r] -= 1;
			if (count[r] > 0) break;
			r += 1;
		}
		perm_index += 1;
	}
}
