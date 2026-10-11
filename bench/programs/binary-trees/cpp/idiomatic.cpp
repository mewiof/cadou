// binary-trees: build/walk/free lots of perfect binary trees with one heap allocation per node

#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <memory>
#include <print>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

namespace {
	constexpr int MIN_DEPTH = 4;

	struct Node {
		std::unique_ptr<Node> left;
		std::unique_ptr<Node> right;
	};

	// allocation order is left subtree -> right subtree -> node in every language
	std::unique_ptr<Node> Create(int depth) {
		std::unique_ptr<Node> left, right;
		if (depth > 0) {
			left = Create(depth - 1);
			right = Create(depth - 1);
		}
		return std::make_unique<Node>(std::move(left), std::move(right));
	}

	int Check(const Node& node) {
		return 1 + (node.left ? Check(*node.left) + Check(*node.right) : 0);
	}
}

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	int n = argc > 1 ? std::atoi(argv[1]) : 10;
	int maxDepth = std::max(n, MIN_DEPTH + 2);

	int stretchDepth = maxDepth + 1;
	std::println("stretch tree of depth {}\t check: {}", stretchDepth, Check(*Create(stretchDepth)));

	auto longLived = Create(maxDepth);
	for (int depth = MIN_DEPTH; depth <= maxDepth; depth += 2) {
		int iterations = 1 << (maxDepth - depth + MIN_DEPTH);
		int check = 0;
		for (int i = 0; i < iterations; i++)
			check += Check(*Create(depth));
		std::println("{}\t trees of depth {}\t check: {}", iterations, depth, check);
	}
	std::println("long lived tree of depth {}\t check: {}", maxDepth, Check(*longLived));
}
