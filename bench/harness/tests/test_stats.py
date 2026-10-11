import math
import unittest

import stats


class QuantileTest(unittest.TestCase):
	def test_matches_linear_interpolation(self):
		self.assertEqual(stats.quantile([1, 2, 3, 4], 0.5), 2.5)
		self.assertEqual(stats.quantile([4, 1, 3, 2], 0.25), 1.75)
		self.assertEqual(stats.quantile([7], 0.9), 7)


class MedianIntervalTest(unittest.TestCase):
	def test_thirty_samples_use_ranks_10_and_21(self):
		# P(Binomial(30, 1/2) <= 9) = 0.0214 <= 0.025 < P(<= 10) = 0.0494
		low, high, exact = stats.median_ci(list(range(1, 31)))
		self.assertEqual((low, high, exact), (10, 21, True))

	def test_coverage_is_at_least_the_confidence(self):
		for n in range(6, 60):
			low, high, exact = stats.median_ci(list(range(1, n + 1)))
			k = low
			coverage = 1 - 2 * sum(math.comb(n, i) for i in range(k)) / 2**n
			self.assertTrue(exact)
			self.assertEqual(high, n - k + 1)
			self.assertGreaterEqual(coverage, 0.95)

	def test_five_samples_are_too_few(self):
		# even [min, max] covers only 1 - 2 / 32 = 93.75% of the time
		self.assertEqual(stats.median_ci([5, 1, 4, 2, 3]), (1, 5, False))


class MannWhitneyTest(unittest.TestCase):
	def test_separated_samples(self):
		# z = (4.5 - 0.5) / sqrt(5.25), p = erfc(z / sqrt 2)
		u, p = stats.mann_whitney([1, 2, 3], [4, 5, 6])
		self.assertEqual(u, 0)
		self.assertAlmostEqual(p, 0.0809, places=4)

	def test_identical_samples(self):
		u, p = stats.mann_whitney([1.0] * 10, [1.0] * 10)
		self.assertEqual(p, 1.0)

	def test_symmetric(self):
		a = [10.1, 10.4, 9.8, 10.0, 10.2, 10.3]
		b = [10.6, 10.9, 10.5, 10.4, 11.0, 10.7]
		self.assertAlmostEqual(stats.mann_whitney(a, b)[1], stats.mann_whitney(b, a)[1])

	def test_thirty_against_thirty_fully_separated(self):
		u, p = stats.mann_whitney(list(range(30)), list(range(100, 130)))
		self.assertEqual(u, 0)
		self.assertLess(p, 1e-9)


class HolmTest(unittest.TestCase):
	def test_step_down(self):
		adjusted = stats.holm([0.01, 0.04, 0.03])
		self.assertEqual([round(p, 6) for p in adjusted], [0.03, 0.06, 0.06])

	def test_capped_at_one(self):
		self.assertEqual(stats.holm([0.6, 0.9]), [1.0, 1.0])


class BootstrapTest(unittest.TestCase):
	def test_ratio_interval_contains_the_true_ratio(self):
		a = [2.0 + 0.01 * (i % 7) for i in range(30)]
		b = [1.0 + 0.005 * (i % 5) for i in range(30)]
		low, high = stats.bootstrap_ratio_ci(a, b, resamples=2000)
		self.assertLess(low, 2.03 / 1.01)
		self.assertGreater(high, 2.03 / 1.01)
		self.assertEqual((low, high), stats.bootstrap_ratio_ci(a, b, resamples=2000))


class GeomeanTest(unittest.TestCase):
	def test_geomean(self):
		self.assertAlmostEqual(stats.geomean([1, 4, 16]), 4)


if __name__ == "__main__":
	unittest.main()
