// binary-trees: build/walk/free lots of perfect binary trees with one heap allocation per node

const std = @import("std");
const Allocator = std.mem.Allocator;

const min_depth = 4;

const Node = struct {
	left: ?*Node,
	right: ?*Node,
};

// allocation order is left subtree -> right subtree -> node in every language
fn create(gpa: Allocator, depth: u32) Allocator.Error!*Node {
	if (depth == 0) {
		const leaf = try gpa.create(Node);
		leaf.* = .{ .left = null, .right = null };
		return leaf;
	}
	const left = try create(gpa, depth - 1);
	errdefer destroy(gpa, left);
	const right = try create(gpa, depth - 1);
	errdefer destroy(gpa, right);
	const node = try gpa.create(Node);
	node.* = .{ .left = left, .right = right };
	return node;
}

fn check(node: *const Node) u32 {
	return 1 + if (node.left) |left| check(left) + check(node.right.?) else 0;
}

fn destroy(gpa: Allocator, node: *Node) void {
	if (node.left) |left| destroy(gpa, left);
	if (node.right) |right| destroy(gpa, right);
	gpa.destroy(node);
}

pub fn main(init: std.process.Init) !void {
	const gpa = init.gpa;
	const args = try init.minimal.args.toSlice(init.arena.allocator());
	const n = if (args.len > 1) try std.fmt.parseInt(u32, args[1], 10) else 10;
	const max_depth = @max(n, min_depth + 2);

	var buffer: [4096]u8 = undefined;
	var stdout_writer = std.Io.File.stdout().writer(init.io, &buffer);
	const out = &stdout_writer.interface;

	const stretch_depth = max_depth + 1;
	const stretch = try create(gpa, stretch_depth);
	try out.print("stretch tree of depth {d}\t check: {d}\n", .{ stretch_depth, check(stretch) });
	destroy(gpa, stretch);

	const long_lived = try create(gpa, max_depth);
	defer destroy(gpa, long_lived);
	var depth: u32 = min_depth;
	while (depth <= max_depth) : (depth += 2) {
		const iterations = @as(u32, 1) << @intCast(max_depth - depth + min_depth);
		var total: u32 = 0;
		for (0..iterations) |_| {
			const tree = try create(gpa, depth);
			total += check(tree);
			destroy(gpa, tree);
		}
		try out.print("{d}\t trees of depth {d}\t check: {d}\n", .{ iterations, depth, total });
	}
	try out.print("long lived tree of depth {d}\t check: {d}\n", .{ max_depth, check(long_lived) });
	try out.flush();
}
