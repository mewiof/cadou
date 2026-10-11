# statistics for benchmark samples (stdlib only)

import math
import random
import statistics


def quantile(values, q):
	# linear interpolation between order statistics (type 7, the numpy default)
	s = sorted(values)
	position = (len(s) - 1) * q
	low = math.floor(position)
	high = math.ceil(position)
	return s[low] + (s[high] - s[low]) * (position - low)


def median_ci(values, confidence=0.95):
	# distribution-free interval from order statistics. With B ~ Binomial(n, 1/2)
	# samples below the true median the interval from x(k) to x(n-k+1) covers it
	# with probability 1 - 2 P(B <= k-1). k is the largest value that keeps this
	# at or above the confidence. exact is False when n is too small for that
	s = sorted(values)
	n = len(s)
	tail = (1 - confidence) / 2
	k = 0
	cumulative = 0.0
	for i in range(n + 1):
		cumulative += math.comb(n, i) / 2**n
		if cumulative > tail:
			break
		k = i + 1
	if k == 0:
		return s[0], s[-1], False
	return s[k - 1], s[n - k], True


def bootstrap_ratio_ci(a, b, confidence=0.95, resamples=10000, seed=0):
	# percentile bootstrap of median(a) / median(b)
	rng = random.Random(seed)
	ratios = sorted(statistics.median(rng.choices(a, k=len(a))) / statistics.median(rng.choices(b, k=len(b))) for _ in range(resamples))
	tail = (1 - confidence) / 2
	return quantile(ratios, tail), quantile(ratios, 1 - tail)


def mann_whitney(a, b):
	# two-sided Mann-Whitney U (normal approximation with tie and continuity corrections)
	n1, n2 = len(a), len(b)
	pooled = sorted([(x, 0) for x in a] + [(x, 1) for x in b])
	n = n1 + n2
	rank_sum = 0.0
	ties = 0
	i = 0
	while i < n:
		j = i
		while j + 1 < n and pooled[j + 1][0] == pooled[i][0]:
			j += 1
		rank = (i + j) / 2 + 1
		rank_sum += rank * sum(1 for k in range(i, j + 1) if pooled[k][1] == 0)
		t = j - i + 1
		ties += t**3 - t
		i = j + 1
	u1 = rank_sum - n1 * (n1 + 1) / 2
	mean = n1 * n2 / 2
	variance = n1 * n2 / 12 * ((n + 1) - ties / (n * (n - 1)))
	if variance <= 0:
		return min(u1, n1 * n2 - u1), 1.0
	z = max(abs(u1 - mean) - 0.5, 0.0) / math.sqrt(variance)
	return min(u1, n1 * n2 - u1), min(1.0, math.erfc(z / math.sqrt(2)))


def holm(pvalues):
	# Holm-Bonferroni step-down adjustment (results in input order)
	order = sorted(range(len(pvalues)), key=lambda i: pvalues[i])
	adjusted = [0.0] * len(pvalues)
	running = 0.0
	for rank, i in enumerate(order):
		running = max(running, min(1.0, (len(pvalues) - rank) * pvalues[i]))
		adjusted[i] = running
	return adjusted


def geomean(values):
	return math.exp(statistics.fmean(math.log(v) for v in values))


def summarize(values, confidence=0.95):
	low, high, exact = median_ci(values, confidence)
	return {
		"n": len(values),
		"median": statistics.median(values),
		"ci_low": low,
		"ci_high": high,
		"ci_exact": exact,
		"min": min(values),
		"max": max(values),
		"mean": statistics.fmean(values),
		"stdev": statistics.stdev(values) if len(values) > 1 else 0.0,
		"iqr": quantile(values, 0.75) - quantile(values, 0.25),
	}
