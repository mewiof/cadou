// binary-trees: build/walk/free lots of perfect binary trees with one heap allocation per node

#include <stdio.h>
#include <stdlib.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
#endif

#define MIN_DEPTH 4

typedef struct Node {
	struct Node* left;
	struct Node* right;
} Node;

// allocation order is left subtree -> right subtree -> node in every language
static Node* Create(int depth) {
	Node* left = NULL;
	Node* right = NULL;
	if (depth > 0) {
		left = Create(depth - 1);
		right = Create(depth - 1);
	}
	Node* node = malloc(sizeof *node);
	if (!node) {
		fputs("out of memory\n", stderr);
		exit(1);
	}
	node->left = left;
	node->right = right;
	return node;
}

static int Check(const Node* node) {
	return 1 + (node->left ? Check(node->left) + Check(node->right) : 0);
}

static void Destroy(Node* node) {
	if (node->left) {
		Destroy(node->left);
		Destroy(node->right);
	}
	free(node);
}

int main(int argc, char** argv) {
#ifdef _WIN32
	_setmode(_fileno(stdout), _O_BINARY);
#endif
	int n = argc > 1 ? atoi(argv[1]) : 10;
	int maxDepth = n > MIN_DEPTH + 2 ? n : MIN_DEPTH + 2;

	int stretchDepth = maxDepth + 1;
	Node* stretch = Create(stretchDepth);
	printf("stretch tree of depth %d\t check: %d\n", stretchDepth, Check(stretch));
	Destroy(stretch);

	Node* longLived = Create(maxDepth);
	for (int depth = MIN_DEPTH; depth <= maxDepth; depth += 2) {
		int iterations = 1 << (maxDepth - depth + MIN_DEPTH);
		int check = 0;
		for (int i = 0; i < iterations; i++) {
			Node* tree = Create(depth);
			check += Check(tree);
			Destroy(tree);
		}
		printf("%d\t trees of depth %d\t check: %d\n", iterations, depth, check);
	}
	printf("long lived tree of depth %d\t check: %d\n", maxDepth, Check(longLived));
	Destroy(longLived);
	return 0;
}
