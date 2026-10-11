// k-nucleotide: frequencies of 1- and 2-nucleotide sequences and counts of five
// longer ones in fasta's third sequence (read from stdin)

const std = @import("std");
const Allocator = std.mem.Allocator;

// keys pack two bits per nucleotide (A/C/G/T = 0/1/2/3) with the first one highest
const Counts = std.AutoHashMap(u64, u32);

const Entry = struct {
	key: u64,
	count: u32,
};

fn code(c: u8) u8 {
	return switch (c) {
		'A', 'a' => 0,
		'C', 'c' => 1,
		'G', 'g' => 2,
		else => 3,
	};
}

fn readThree(gpa: Allocator, in: *std.Io.Reader) !std.ArrayList(u8) {
	while (try in.takeDelimiter('\n')) |line| {
		if (std.mem.startsWith(u8, line, ">THREE")) break;
	}
	var sequence: std.ArrayList(u8) = .empty;
	while (try in.takeDelimiter('\n')) |line| {
		if (line.len > 0 and line[0] == '>') break;
		try sequence.ensureUnusedCapacity(gpa, line.len);
		for (line) |c| sequence.appendAssumeCapacity(code(c));
	}
	return sequence;
}

fn count(gpa: Allocator, sequence: []const u8, k: usize) !Counts {
	var counts = Counts.init(gpa);
	const mask = (@as(u64, 1) << @intCast(2 * k)) - 1;
	var key: u64 = 0;
	for (sequence, 0..) |c, i| {
		key = ((key << 2) | c) & mask;
		if (i + 1 >= k) {
			const slot = try counts.getOrPut(key);
			if (!slot.found_existing) slot.value_ptr.* = 0;
			slot.value_ptr.* += 1;
		}
	}
	return counts;
}

fn moreFrequent(_: void, a: Entry, b: Entry) bool {
	if (a.count != b.count) return a.count > b.count;
	return a.key < b.key;
}

fn printFrequencies(gpa: Allocator, out: *std.Io.Writer, sequence: []const u8, k: usize) !void {
	var counts = try count(gpa, sequence, k);
	defer counts.deinit();
	// k is 1 or 2 here (at most 16 keys)
	var entries: [16]Entry = undefined;
	var entry_count: usize = 0;
	var iterator = counts.iterator();
	while (iterator.next()) |slot| : (entry_count += 1) {
		entries[entry_count] = .{ .key = slot.key_ptr.*, .count = slot.value_ptr.* };
	}
	std.mem.sortUnstable(Entry, entries[0..entry_count], {}, moreFrequent);
	const total: f64 = @floatFromInt(sequence.len - k + 1);
	for (entries[0..entry_count]) |entry| {
		var name: [2]u8 = undefined;
		for (name[0..k], 0..) |*letter, i| letter.* = "ACGT"[@intCast((entry.key >> @intCast(2 * (k - 1 - i))) & 3)];
		try out.print("{s} {d:.3}\n", .{ name[0..k], 100.0 * @as(f64, @floatFromInt(entry.count)) / total });
	}
	try out.writeAll("\n");
}

fn printCount(gpa: Allocator, out: *std.Io.Writer, sequence: []const u8, pattern: []const u8) !void {
	var counts = try count(gpa, sequence, pattern.len);
	defer counts.deinit();
	var key: u64 = 0;
	for (pattern) |c| key = (key << 2) | code(c);
	try out.print("{d}\t{s}\n", .{ counts.get(key) orelse 0, pattern });
}

pub fn main(init: std.process.Init) !void {
	const gpa = init.gpa;

	var input_buffer: [4096]u8 = undefined;
	var stdin_reader = std.Io.File.stdin().reader(init.io, &input_buffer);
	var sequence = try readThree(gpa, &stdin_reader.interface);
	defer sequence.deinit(gpa);

	var output_buffer: [1024]u8 = undefined;
	var stdout_writer = std.Io.File.stdout().writer(init.io, &output_buffer);
	const out = &stdout_writer.interface;
	try printFrequencies(gpa, out, sequence.items, 1);
	try printFrequencies(gpa, out, sequence.items, 2);
	for ([_][]const u8{ "GGT", "GGTA", "GGTATT", "GGTATTTTAATT", "GGTATTTTAATTTATAGT" }) |pattern| {
		try printCount(gpa, out, sequence.items, pattern);
	}
	try out.flush();
}
