// fasta: one repeated and two random DNA sequences in 60-character lines

const std = @import("std");
const Writer = std.Io.Writer;

const line_length = 60;

const alu =
	"GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGG" ++
	"GAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAGA" ++
	"CCAGCCTGGCCAACATGGTGAAACCCCGTCTCTACTAAAAAT" ++
	"ACAAAAATTAGCCGGGCGTGGTGGCGCGCGCCTGTAATCCCA" ++
	"GCTACTCGGGAGGCTGAGGCAGGAGAATCGCTTGAACCCGGG" ++
	"AGGCGGAGGTTGCAGTGAGCCGAGATCGCGCCACTGCACTCC" ++
	"AGCCTGGGCGACAGAGCGAGACTCCGTCTCAAAAA";

const Acid = struct {
	symbol: u8,
	probability: f64,
};

const Random = struct {
	seed: u32,

	const im = 139968;
	const ia = 3877;
	const ic = 29573;

	fn next(self: *Random, max: f64) f64 {
		self.seed = (self.seed * ia + ic) % im;
		return max * @as(f64, @floatFromInt(self.seed)) / im;
	}
};

fn repeatFasta(out: *Writer, header: []const u8, sequence: []const u8, count: usize) !void {
	try out.writeAll(header);
	var line: [line_length + 1]u8 = undefined;
	var position: usize = 0;
	var n = count;
	while (n > 0) {
		const length = @min(n, line_length);
		for (line[0..length]) |*c| {
			c.* = sequence[position];
			position += 1;
			if (position == sequence.len) position = 0;
		}
		line[length] = '\n';
		try out.writeAll(line[0 .. length + 1]);
		n -= length;
	}
}

fn randomFasta(out: *Writer, header: []const u8, acids: []Acid, count: usize, random: *Random) !void {
	try out.writeAll(header);
	var cumulative: f64 = 0.0;
	for (acids) |*acid| {
		cumulative += acid.probability;
		acid.probability = cumulative;
	}

	var line: [line_length + 1]u8 = undefined;
	var n = count;
	while (n > 0) {
		const length = @min(n, line_length);
		for (line[0..length]) |*c| {
			const r = random.next(1.0);
			var k: usize = 0;
			while (k + 1 < acids.len and r >= acids[k].probability) k += 1;
			c.* = acids[k].symbol;
		}
		line[length] = '\n';
		try out.writeAll(line[0 .. length + 1]);
		n -= length;
	}
}

pub fn main(init: std.process.Init) !void {
	const args = try init.minimal.args.toSlice(init.arena.allocator());
	const n = if (args.len > 1) try std.fmt.parseInt(usize, args[1], 10) else 1000;

	var iub = [_]Acid{
		.{ .symbol = 'a', .probability = 0.27 }, .{ .symbol = 'c', .probability = 0.12 },
		.{ .symbol = 'g', .probability = 0.12 }, .{ .symbol = 't', .probability = 0.27 },
		.{ .symbol = 'B', .probability = 0.02 }, .{ .symbol = 'D', .probability = 0.02 },
		.{ .symbol = 'H', .probability = 0.02 }, .{ .symbol = 'K', .probability = 0.02 },
		.{ .symbol = 'M', .probability = 0.02 }, .{ .symbol = 'N', .probability = 0.02 },
		.{ .symbol = 'R', .probability = 0.02 }, .{ .symbol = 'S', .probability = 0.02 },
		.{ .symbol = 'V', .probability = 0.02 }, .{ .symbol = 'W', .probability = 0.02 },
		.{ .symbol = 'Y', .probability = 0.02 },
	};
	var homo_sapiens = [_]Acid{
		.{ .symbol = 'a', .probability = 0.3029549426680 },
		.{ .symbol = 'c', .probability = 0.1979883004921 },
		.{ .symbol = 'g', .probability = 0.1975473066391 },
		.{ .symbol = 't', .probability = 0.3015094502008 },
	};

	var buffer: [4096]u8 = undefined;
	var stdout_writer = std.Io.File.stdout().writer(init.io, &buffer);
	const out = &stdout_writer.interface;

	var random: Random = .{ .seed = 42 };
	try repeatFasta(out, ">ONE Homo sapiens alu\n", alu, 2 * n);
	try randomFasta(out, ">TWO IUB ambiguity codes\n", &iub, 3 * n, &random);
	try randomFasta(out, ">THREE Homo sapiens frequency\n", &homo_sapiens, 5 * n, &random);
	try out.flush();
}
