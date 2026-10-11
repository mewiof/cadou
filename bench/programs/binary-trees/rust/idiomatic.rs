// binary-trees: build/walk/free lots of perfect binary trees with one heap allocation per node

const MIN_DEPTH: u32 = 4;

struct Node {
	left: Option<Box<Node>>,
	right: Option<Box<Node>>,
}

// allocation order is left subtree -> right subtree -> node in every language
fn create(depth: u32) -> Box<Node> {
	let (left, right) = if depth > 0 { (Some(create(depth - 1)), Some(create(depth - 1))) } else { (None, None) };
	Box::new(Node { left, right })
}

fn check(node: &Node) -> u32 {
	match (&node.left, &node.right) {
		(Some(left), Some(right)) => 1 + check(left) + check(right),
		_ => 1,
	}
}

fn main() {
	let n: u32 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(10);
	let max_depth = n.max(MIN_DEPTH + 2);

	let stretch_depth = max_depth + 1;
	println!("stretch tree of depth {stretch_depth}\t check: {}", check(&create(stretch_depth)));

	let long_lived = create(max_depth);
	for depth in (MIN_DEPTH..=max_depth).step_by(2) {
		let iterations = 1u32 << (max_depth - depth + MIN_DEPTH);
		let mut total = 0;
		for _ in 0..iterations {
			total += check(&create(depth));
		}
		println!("{iterations}\t trees of depth {depth}\t check: {total}");
	}
	println!("long lived tree of depth {max_depth}\t check: {}", check(&long_lived));
}
