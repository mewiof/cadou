import unittest

import check_text


def problems(text):
	return check_text.prose_problems(text)


class ProseTest(unittest.TestCase):
	def test_plain_sentences_pass(self):
		self.assertEqual(problems("The harness pins itself to one CPU. Children inherit the mask."), [])
		self.assertEqual(problems("If there is none, safe is the default."), [])

	def test_comma_and_clause(self):
		self.assertTrue(problems("The checker runs locally, and CI runs it too."))
		self.assertTrue(problems("Keep it short, so it gets read."))

	def test_semicolon(self):
		self.assertTrue(problems("full-line comments only; a block is a run of lines"))

	def test_two_commas(self):
		self.assertTrue(problems("What gets easier, what gets harder, what is ruled out."))

	def test_exempt_places(self):
		# quoted/code/parenthesized text and numbers keep their punctuation
		self.assertEqual(problems('Ritchie: "less convenient than using a terminator; really, truly".'), [])
		self.assertEqual(problems("Use `a, b, c; d` in code."), [])
		self.assertEqual(problems("Ranks 10 and 21 (of 30, no distribution, assumed)."), [])
		self.assertEqual(problems("It ran 50,000,000 steps."), [])

	def test_dash_aside(self):
		self.assertTrue(problems("The harness - which is small - runs fast."))


class MessageTest(unittest.TestCase):
	def test_wrapped_paragraph_is_one_text(self):
		message = "Add the harness\n\nIt times every program\nand checks every output.\n# a git comment, ignored, really\n"
		self.assertEqual(check_text.check_message(message), [])

	def test_bad_message(self):
		self.assertTrue(check_text.check_message("Add it\n\nIt works, and it is fast.\n"))


if __name__ == "__main__":
	unittest.main()
