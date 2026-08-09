"""W109 PMT hidden-low-word and small-coefficient discriminator.

This is a prediction-blind offline analysis over the already banked PMT
intermediates.  It asks two deliberately narrow questions:

1. Can a compensated low word from ``n * log1p(rate)`` explain the private
   PMT ``expm1`` result?
2. Can a smooth degree-0..8 correction satisfy the exact binary64 output
   intervals, either at the Kahan quotient or at its captured ``LN(u)``
   denominator?

The power-of-two-rate 234-row corpus is the recovery cohort.  The independent
90-row general-rate corpus is reported only after each coefficient vector is
frozen.  Oracle artifacts are read and validated by
``analyze_pmt_exactness_predicate``; this script performs no oracle calls and
does not mutate evidence.

SciPy is used only as a deterministic linear-programming engine.  Every
reported candidate is replayed through binary64 before its exact-hit count is
printed.  The one-coefficient low-word envelope itself is solved with exact
``Fraction`` endpoints, independent of SciPy.
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from fractions import Fraction
from itertools import combinations
from typing import Callable, Sequence

import numpy as np
from scipy.optimize import linprog

import analyze_pmt_exactness_predicate as audit


@dataclass(frozen=True)
class ModelRow:
    row: audit.AuditRow
    log_anchor: float
    low_exact: Fraction
    low: float


def rounded(value: Fraction) -> float:
    """Correctly round an exact rational to binary64."""

    return float(value)


def output_interval(raw_bits: int) -> tuple[Fraction, Fraction]:
    """Return the exact half-ULP preimage hull for an expected finite f64."""

    value = audit.from_bits(f"{raw_bits:016x}")
    lower_neighbor = math.nextafter(value, -math.inf)
    upper_neighbor = math.nextafter(value, math.inf)
    return (
        (Fraction(lower_neighbor) + Fraction(value)) / 2,
        (Fraction(value) + Fraction(upper_neighbor)) / 2,
    )


def build_rows() -> list[ModelRow]:
    source = audit.load_power_of_two_rows() + audit.load_general_rate_rows()
    anchors: dict[tuple[str, str], float] = {}
    for row in source:
        if row.n == 1:
            anchors[(row.family, row.rate_key)] = abs(row.tau)

    result: list[ModelRow] = []
    for row in source:
        key = (row.family, row.rate_key)
        if key not in anchors:
            raise RuntimeError(f"missing n=1 log anchor for {key}")
        log_anchor = anchors[key]
        exact_tau = -Fraction(row.n) * Fraction(log_anchor)
        low_exact = exact_tau - Fraction(row.tau)
        low = rounded(low_exact)
        if Fraction(low) != low_exact:
            raise RuntimeError(
                f"TwoProduct residual is not exactly binary64 for {key}, n={row.n}"
            )
        if (low == 0.0) != row.exact_tau:
            raise RuntimeError(
                f"exact-product classification mismatch for {key}, n={row.n}"
            )
        result.append(ModelRow(row, log_anchor, low_exact, low))
    return result


def plain_kahan(item: ModelRow) -> float:
    row = item.row
    return (row.u - 1.0) * row.tau / row.lnu


def x87_kahan(item: ModelRow) -> float:
    row = item.row
    numerator = audit.x87_double_round_mul(row.u - 1.0, row.tau)
    return audit.x87_double_round_div(numerator, row.lnu)


def plain_low_correction(item: ModelRow) -> float:
    row = item.row
    return (row.u - 1.0) * item.low / row.lnu


def x87_low_correction(item: ModelRow) -> float:
    row = item.row
    numerator = audit.x87_double_round_mul(row.u - 1.0, item.low)
    return audit.x87_double_round_div(numerator, row.lnu)


def x87_add(left: float, right: float) -> float:
    return rounded(audit.round_fraction_binary(Fraction(left) + Fraction(right), 64))


def plain_low_predivide(item: ModelRow) -> float:
    row = item.row
    high = (row.u - 1.0) * row.tau
    low = (row.u - 1.0) * item.low
    return (high + low) / row.lnu


def x87_spill_low_predivide(item: ModelRow) -> float:
    row = item.row
    high = audit.x87_double_round_mul(row.u - 1.0, row.tau)
    low = audit.x87_double_round_mul(row.u - 1.0, item.low)
    numerator = x87_add(high, low)
    return audit.x87_double_round_div(numerator, row.lnu)


def x87_continuous_low_predivide(item: ModelRow) -> float:
    row = item.row
    a = Fraction(row.u - 1.0)
    high = audit.round_fraction_binary(a * Fraction(row.tau), 64)
    low = audit.round_fraction_binary(a * item.low_exact, 64)
    numerator = audit.round_fraction_binary(high + low, 64)
    quotient = audit.round_fraction_binary(numerator / Fraction(row.lnu), 64)
    return rounded(quotient)


def plain_low_both(item: ModelRow) -> float:
    row = item.row
    numerator = (row.u - 1.0) * row.tau + (row.u - 1.0) * item.low
    return numerator / (row.lnu + item.low)


def x87_spill_low_both(item: ModelRow) -> float:
    row = item.row
    high = audit.x87_double_round_mul(row.u - 1.0, row.tau)
    low = audit.x87_double_round_mul(row.u - 1.0, item.low)
    numerator = x87_add(high, low)
    denominator = x87_add(row.lnu, item.low)
    return audit.x87_double_round_div(numerator, denominator)


def plain_low_factor(item: ModelRow) -> float:
    factor = 1.0 + item.low / item.row.tau
    return plain_kahan(item) * factor


def x87_spill_low_factor(item: ModelRow) -> float:
    ratio = audit.x87_double_round_div(item.low, item.row.tau)
    factor = x87_add(1.0, ratio)
    return audit.x87_double_round_mul(x87_kahan(item), factor)


def partition_score(rows: Sequence[ModelRow], model: Callable[[ModelRow], float]) -> str:
    buckets: dict[str, list[int]] = {
        "all": [0, 0],
        "power": [0, 0],
        "general": [0, 0],
        "exact": [0, 0],
        "rounded": [0, 0],
    }
    for item in rows:
        hit = audit.bits(model(item)) == item.row.expected_bits
        keys = [
            "all",
            "power" if item.row.family == "power-of-two-rate" else "general",
            "exact" if item.row.exact_tau else "rounded",
        ]
        for key in keys:
            buckets[key][0] += int(hit)
            buckets[key][1] += 1
    return " ".join(f"{key}={hit}/{total}" for key, (hit, total) in buckets.items())


def ideal_low_terms(item: ModelRow) -> tuple[Fraction, Fraction]:
    """One-round Kahan quotient ``q0 + alpha*slope`` over exact operands."""

    row = item.row
    a = Fraction(row.u - 1.0)
    denominator = Fraction(row.lnu)
    q0 = a * Fraction(row.tau) / denominator
    slope = a * item.low_exact / denominator
    return q0, slope


def staged_low_terms(
    base: Callable[[ModelRow], float],
    correction: Callable[[ModelRow], float],
) -> Callable[[ModelRow], tuple[Fraction, Fraction]]:
    def terms(item: ModelRow) -> tuple[Fraction, Fraction]:
        return Fraction(base(item)), Fraction(correction(item))

    return terms


def exact_linear_score(
    rows: Sequence[ModelRow],
    terms: Callable[[ModelRow], tuple[Fraction, Fraction]],
    alpha: Fraction,
) -> int:
    hits = 0
    for item in rows:
        q0, slope = terms(item)
        hits += int(audit.bits(rounded(q0 + alpha * slope)) == item.row.expected_bits)
    return hits


def recover_one_coefficient(
    train: Sequence[ModelRow],
    terms: Callable[[ModelRow], tuple[Fraction, Fraction]],
) -> tuple[Fraction, int, int, int]:
    """Find an exact-rational alpha maximizing satisfied RN intervals.

    The maximum is found over every interval endpoint and every open cell
    between adjacent endpoints.  Rows with zero slope are classified as
    immutable hits or immutable misses.
    """

    intervals: list[tuple[Fraction, Fraction]] = []
    immutable_hits = 0
    immutable_misses = 0
    for item in train:
        q0, slope = terms(item)
        lower, upper = output_interval(item.row.expected_bits)
        if slope == 0:
            if audit.bits(rounded(q0)) == item.row.expected_bits:
                immutable_hits += 1
            else:
                immutable_misses += 1
            continue
        left = (lower - q0) / slope
        right = (upper - q0) / slope
        if left > right:
            left, right = right, left
        intervals.append((left, right))

    endpoints = sorted({value for interval in intervals for value in interval})
    candidates = {Fraction(0), Fraction(1), Fraction(-1)}
    candidates.update(endpoints)
    candidates.update(
        (left + right) / 2 for left, right in zip(endpoints, endpoints[1:], strict=False)
    )

    def relaxed_count(alpha: Fraction) -> int:
        return immutable_hits + sum(left <= alpha <= right for left, right in intervals)

    best = min(
        candidates,
        key=lambda alpha: (-relaxed_count(alpha), abs(alpha), alpha.denominator, alpha.numerator),
    )
    exact_hits = exact_linear_score(train, terms, best)
    return best, exact_hits, immutable_hits, immutable_misses


def describe_alpha(alpha: Fraction) -> str:
    return f"{float(alpha):+.17g} ({alpha.numerator}/{alpha.denominator})"


def chebyshev_values(x: float, degree: int) -> list[float]:
    values = [1.0]
    if degree == 0:
        return values
    values.append(x)
    for _ in range(2, degree + 1):
        values.append(2.0 * x * values[-1] - values[-2])
    return values


def coordinate(item: ModelRow, mode: str, log_min: float, log_max: float) -> float:
    magnitude = abs(item.row.tau)
    if mode == "linear-tau":
        return 2.0 * magnitude - 1.0
    if mode == "log2-tau":
        value = math.log2(magnitude)
        return 2.0 * (value - log_min) / (log_max - log_min) - 1.0
    raise ValueError(mode)


@dataclass(frozen=True)
class LinearFamily:
    name: str
    center: Callable[[ModelRow], Fraction]
    interval: Callable[[ModelRow], tuple[Fraction, Fraction]]
    replay: Callable[[ModelRow, float], float]
    scale: Callable[[ModelRow], float]


@dataclass(frozen=True)
class IntervalFit:
    coefficients: np.ndarray
    slack: float
    feasible: bool
    exact_certificate_margin: Fraction | None
    least_squares_coefficients: np.ndarray


def quotient_family(base: Callable[[ModelRow], float]) -> LinearFamily:
    def center(item: ModelRow) -> Fraction:
        return Fraction(base(item))

    def interval(item: ModelRow) -> tuple[Fraction, Fraction]:
        lower, upper = output_interval(item.row.expected_bits)
        q0 = center(item)
        return lower - q0, upper - q0

    def replay(item: ModelRow, correction: float) -> float:
        return base(item) + correction

    def scale(item: ModelRow) -> float:
        # A relative-epsilon smooth error curve.  This remains well-scaled
        # across the eight-decade tau span while retaining physical units.
        return abs(audit.from_bits(f"{item.row.expected_bits:016x}")) * 2.0**-52

    return LinearFamily("quotient correction", center, interval, replay, scale)


def denominator_family(item: ModelRow) -> tuple[Fraction, Fraction]:
    """Required delta from captured LN(u) for plain staged numerator/divide."""

    row = item.row
    numerator = Fraction((row.u - 1.0) * row.tau)
    q_lower, q_upper = output_interval(row.expected_bits)
    # numerator is positive and q is negative, hence inversion reverses order.
    d_lower = numerator / q_upper
    d_upper = numerator / q_lower
    if d_lower > d_upper:
        d_lower, d_upper = d_upper, d_lower
    return d_lower - Fraction(row.lnu), d_upper - Fraction(row.lnu)


def denominator_model() -> LinearFamily:
    def center(item: ModelRow) -> Fraction:
        return Fraction(item.row.lnu)

    def replay(item: ModelRow, correction: float) -> float:
        row = item.row
        return ((row.u - 1.0) * row.tau) / (row.lnu + correction)

    def scale(item: ModelRow) -> float:
        return abs(item.row.lnu) * 2.0**-52

    return LinearFamily(
        "denominator correction",
        center,
        denominator_family,
        replay,
        scale,
    )


def solve_fraction_system(
    matrix: Sequence[Sequence[Fraction]], vector: Sequence[Fraction]
) -> list[Fraction] | None:
    """Solve a small square rational system by exact Gaussian elimination."""

    size = len(vector)
    work = [list(matrix[row]) + [vector[row]] for row in range(size)]
    for column in range(size):
        pivot = next((row for row in range(column, size) if work[row][column]), None)
        if pivot is None:
            return None
        work[column], work[pivot] = work[pivot], work[column]
        divisor = work[column][column]
        for item in range(column, size + 1):
            work[column][item] /= divisor
        for row in range(size):
            if row == column:
                continue
            factor = work[row][column]
            if factor == 0:
                continue
            for item in range(column, size + 1):
                work[row][item] -= factor * work[column][item]
    return [work[row][size] for row in range(size)]


def exact_farkas_margin(
    constraints: Sequence[Sequence[Fraction]],
    bounds: Sequence[Fraction],
    dual_weights: Sequence[float],
) -> Fraction | None:
    """Recover and verify an exact Farkas certificate from the HiGHS basis.

    For ``C*x <= b``, nonnegative weights with ``lambda*C == 0`` and
    ``lambda*b < 0`` prove infeasibility over the rationals.  A basic dual
    solution normally has ``dimension + 1`` nonzero weights.  We use the
    numerical dual only to locate that tiny support; all subsequent solving
    and verification is exact ``Fraction`` arithmetic.
    """

    width = len(constraints[0])
    def verify_supports(ranked: Sequence[int]) -> Fraction | None:
        ranked = ranked[: min(len(ranked), width + 8)]
        if len(ranked) < width + 1:
            return None
        for support in combinations(ranked, width + 1):
            equations: list[list[Fraction]] = []
            for column in range(width):
                equations.append([constraints[index][column] for index in support])
            equations.append([Fraction(1)] * len(support))
            rhs = [Fraction(0)] * width + [Fraction(1)]
            weights = solve_fraction_system(equations, rhs)
            if weights is None or any(weight < 0 for weight in weights):
                continue
            for column in range(width):
                if sum(
                    weight * constraints[index][column]
                    for weight, index in zip(weights, support, strict=True)
                ) != 0:
                    raise RuntimeError("internal error: invalid exact Farkas null vector")
            weighted_bound = sum(
                weight * bounds[index]
                for weight, index in zip(weights, support, strict=True)
            )
            if weighted_bound < 0:
                return -weighted_bound
        return None

    ranked = sorted(
        (index for index, weight in enumerate(dual_weights) if weight > 1e-10),
        key=lambda index: dual_weights[index],
        reverse=True,
    )
    margin = verify_supports(ranked)
    if margin is not None:
        return margin

    # Degenerate min-slack bases sometimes expose a non-basic dual vector.
    # Solve the normalized Farkas alternative directly to obtain a sparse
    # basic support, then rationalize and verify that support exactly.
    c = np.asarray([float(value) for value in bounds], dtype=np.float64)
    constraint_float = np.asarray(
        [[float(value) for value in row] for row in constraints], dtype=np.float64
    )
    a_eq = np.vstack([constraint_float.T, np.ones(len(constraints))])
    b_eq = np.concatenate([np.zeros(width), np.ones(1)])
    certificate_lp = linprog(
        c,
        A_eq=a_eq,
        b_eq=b_eq,
        bounds=[(0.0, None)] * len(constraints),
        method="highs",
        options={"presolve": True},
    )
    if certificate_lp.success and certificate_lp.fun < -1e-8:
        ranked = sorted(
            (
                index
                for index, weight in enumerate(certificate_lp.x)
                if weight > 1e-10
            ),
            key=lambda index: certificate_lp.x[index],
            reverse=True,
        )
        margin = verify_supports(ranked)
        if margin is not None:
            return margin
    return None


def fit_interval_family(
    train: Sequence[ModelRow],
    family: LinearFamily,
    mode: str,
    degree: int,
    log_min: float,
    log_max: float,
) -> IntervalFit:
    matrix_exact: list[list[Fraction]] = []
    for item in train:
        basis = chebyshev_values(coordinate(item, mode, log_min, log_max), degree)
        matrix_exact.append([Fraction(value) for value in basis])
    return fit_interval_matrix(train, family, matrix_exact)


def fit_interval_matrix(
    train: Sequence[ModelRow],
    family: LinearFamily,
    matrix_exact: Sequence[Sequence[Fraction]],
) -> IntervalFit:
    """Minimize uniform interval violation for an explicit linear basis."""

    lower_exact: list[Fraction] = []
    upper_exact: list[Fraction] = []
    for item in train:
        scale = Fraction(family.scale(item))
        lo_exact, hi_exact = family.interval(item)
        lower_exact.append(lo_exact / scale)
        upper_exact.append(hi_exact / scale)

    a = np.asarray(
        [[float(value) for value in row] for row in matrix_exact], dtype=np.float64
    )
    lo = np.asarray([float(value) for value in lower_exact], dtype=np.float64)
    hi = np.asarray([float(value) for value in upper_exact], dtype=np.float64)
    width = len(matrix_exact[0])

    # [A*c - slack <= hi, -A*c - slack <= -lo], slack >= 0.
    a_ub = np.vstack(
        [
            np.column_stack([a, -np.ones(len(train))]),
            np.column_stack([-a, -np.ones(len(train))]),
        ]
    )
    b_ub = np.concatenate([hi, -lo])
    objective = np.zeros(width + 1)
    objective[-1] = 1.0
    result = linprog(
        objective,
        A_ub=a_ub,
        b_ub=b_ub,
        bounds=[(None, None)] * width + [(0.0, None)],
        method="highs",
        options={"presolve": True},
    )
    if not result.success:
        raise RuntimeError(f"LP failed for {family.name}: {result.message}")
    coefficients = result.x[:-1]
    slack = float(result.x[-1])
    dual_weights = [-float(value) for value in result.ineqlin.marginals]
    constraints_exact = matrix_exact + [
        [-value for value in row] for row in matrix_exact
    ]
    bounds_exact = upper_exact + [-value for value in lower_exact]
    certificate = None
    if slack > 1e-8:
        certificate = exact_farkas_margin(
            constraints_exact, bounds_exact, dual_weights
        )
    targets = (lo + hi) / 2.0
    least_squares, *_ = np.linalg.lstsq(a, targets, rcond=None)
    return IntervalFit(
        coefficients=coefficients,
        slack=slack,
        feasible=slack <= 1e-8,
        exact_certificate_margin=certificate,
        least_squares_coefficients=least_squares,
    )


def smooth_low_features(
    item: ModelRow,
    mode: str,
    smooth_degree: int,
    low_degree: int,
    log_min: float,
    log_max: float,
    family: LinearFamily,
) -> list[float]:
    """Smooth tau correction plus a tau-modulated TwoProduct correction."""

    x = coordinate(item, mode, log_min, log_max)
    smooth = chebyshev_values(x, smooth_degree)
    low_basis = chebyshev_values(x, low_degree)
    normalized_low = x87_low_correction(item) / family.scale(item)
    return smooth + [normalized_low * value for value in low_basis]


def fit_smooth_low_family(
    train: Sequence[ModelRow],
    family: LinearFamily,
    mode: str,
    smooth_degree: int,
    low_degree: int,
    log_min: float,
    log_max: float,
) -> IntervalFit:
    matrix = [
        [Fraction(value) for value in smooth_low_features(
            item,
            mode,
            smooth_degree,
            low_degree,
            log_min,
            log_max,
            family,
        )]
        for item in train
    ]
    return fit_interval_matrix(train, family, matrix)


def replay_polynomial(
    rows: Sequence[ModelRow],
    family: LinearFamily,
    mode: str,
    coefficients: Sequence[float],
    log_min: float,
    log_max: float,
) -> int:
    hits = 0
    degree = len(coefficients) - 1
    for item in rows:
        basis = chebyshev_values(coordinate(item, mode, log_min, log_max), degree)
        normalized = sum(c * value for c, value in zip(coefficients, basis, strict=True))
        correction = family.scale(item) * normalized
        candidate = family.replay(item, correction)
        hits += int(audit.bits(candidate) == item.row.expected_bits)
    return hits


def ideal_interval_hits(
    rows: Sequence[ModelRow],
    family: LinearFamily,
    mode: str,
    coefficients: Sequence[float],
    log_min: float,
    log_max: float,
) -> int:
    hits = 0
    degree = len(coefficients) - 1
    for item in rows:
        basis = chebyshev_values(coordinate(item, mode, log_min, log_max), degree)
        normalized = sum(
            Fraction(coefficient) * Fraction(value)
            for coefficient, value in zip(coefficients, basis, strict=True)
        )
        correction = Fraction(family.scale(item)) * normalized
        lower, upper = family.interval(item)
        hits += int(lower <= correction <= upper)
    return hits


def replay_smooth_low(
    rows: Sequence[ModelRow],
    family: LinearFamily,
    mode: str,
    smooth_degree: int,
    low_degree: int,
    coefficients: Sequence[float],
    log_min: float,
    log_max: float,
) -> int:
    hits = 0
    for item in rows:
        features = smooth_low_features(
            item,
            mode,
            smooth_degree,
            low_degree,
            log_min,
            log_max,
            family,
        )
        normalized = sum(
            coefficient * value
            for coefficient, value in zip(coefficients, features, strict=True)
        )
        candidate = family.replay(item, family.scale(item) * normalized)
        hits += int(audit.bits(candidate) == item.row.expected_bits)
    return hits


def report_low_word(rows: Sequence[ModelRow]) -> None:
    power = [item for item in rows if item.row.family == "power-of-two-rate"]
    general = [item for item in rows if item.row.family == "general-rate"]
    exact = [item for item in rows if item.row.exact_tau]
    rounded_rows = [item for item in rows if not item.row.exact_tau]
    print("=== hidden TwoProduct low word ===")
    print(
        f"rows={len(rows)} power={len(power)} general={len(general)} "
        f"low=0/exact={len(exact)} low!=0/rounded={len(rounded_rows)}"
    )
    print("plain baseline:", partition_score(rows, plain_kahan))
    print("x87-spill baseline:", partition_score(rows, x87_kahan))
    print("\nConcrete alpha=1 delivery graphs")
    for name, model in [
        ("plain numerator-hi+lo before divide", plain_low_predivide),
        ("x87-spill numerator-hi+lo before divide", x87_spill_low_predivide),
        ("x87-resident numerator-hi+lo before divide", x87_continuous_low_predivide),
        ("plain low added to numerator and denominator", plain_low_both),
        ("x87-spill low added to numerator and denominator", x87_spill_low_both),
        ("plain quotient*(1+low/tau)", plain_low_factor),
        ("x87-spill quotient*(1+low/tau)", x87_spill_low_factor),
    ]:
        print(f"  {name}: {partition_score(rows, model)}")

    models = [
        ("ideal one-round", ideal_low_terms),
        (
            "plain staged q + alpha*corr",
            staged_low_terms(plain_kahan, plain_low_correction),
        ),
        (
            "x87 staged q + alpha*corr",
            staged_low_terms(x87_kahan, x87_low_correction),
        ),
    ]
    for name, terms in models:
        alpha, train_hits, immutable_hits, immutable_misses = recover_one_coefficient(
            power, terms
        )
        general_hits = exact_linear_score(general, terms, alpha)
        all_hits = exact_linear_score(rows, terms, alpha)
        zero_hits = exact_linear_score(power, terms, Fraction(0))
        one_hits = exact_linear_score(power, terms, Fraction(1))
        combined_immutable_hits = 0
        combined_immutable_misses = 0
        for item in rows:
            q0, slope = terms(item)
            if slope != 0:
                continue
            if audit.bits(rounded(q0)) == item.row.expected_bits:
                combined_immutable_hits += 1
            else:
                combined_immutable_misses += 1
        print(f"\n{name}")
        print(
            f"  immutable low=0 on recovery: hits={immutable_hits} misses={immutable_misses}; "
            f"combined hits={combined_immutable_hits} misses={combined_immutable_misses}; "
            f"all-row ceiling={len(rows) - combined_immutable_misses}"
        )
        print(f"  alpha=0 power={zero_hits}/{len(power)}; alpha=1 power={one_hits}/{len(power)}")
        print(
            f"  recovered alpha={describe_alpha(alpha)}; "
            f"power={train_hits}/{len(power)} general={general_hits}/{len(general)} "
            f"combined={all_hits}/{len(rows)}"
        )


def report_polynomials(rows: Sequence[ModelRow]) -> None:
    power = [item for item in rows if item.row.family == "power-of-two-rate"]
    general = [item for item in rows if item.row.family == "general-rate"]
    log_values = [math.log2(abs(item.row.tau)) for item in power]
    log_min, log_max = min(log_values), max(log_values)
    print("\n=== exact-output-interval coefficient recovery ===")
    print(
        "Recovery cohort=234 power rows; coefficient vectors are frozen before "
        "the 90 general rows are replayed.  slack=0 means the ideal linear "
        "interval system is feasible; positive slack is the minimum uniform "
        "interval widening in relative-epsilon scale units."
    )
    families = [quotient_family(plain_kahan), denominator_model()]
    for family in families:
        for mode in ("linear-tau", "log2-tau"):
            print(f"\n{family.name}; coordinate={mode}")
            print(
                " degree   min-slack exact-cert  LP-ideal  LP-replay "
                "LSQ-replay  general-LSQ"
            )
            for degree in range(0, 9):
                fit = fit_interval_family(
                    power, family, mode, degree, log_min, log_max
                )
                ideal_hits = ideal_interval_hits(
                    power, family, mode, fit.coefficients, log_min, log_max
                )
                power_hits = replay_polynomial(
                    power, family, mode, fit.coefficients, log_min, log_max
                )
                lsq_power = replay_polynomial(
                    power,
                    family,
                    mode,
                    fit.least_squares_coefficients,
                    log_min,
                    log_max,
                )
                lsq_general = replay_polynomial(
                    general,
                    family,
                    mode,
                    fit.least_squares_coefficients,
                    log_min,
                    log_max,
                )
                certificate = "yes" if fit.exact_certificate_margin is not None else "NO"
                print(
                    f" {degree:>6} {fit.slack:>11.6g} {certificate:>10} "
                    f"{ideal_hits:>5}/{len(power):<3} {power_hits:>5}/{len(power):<3} "
                    f"{lsq_power:>5}/{len(power):<3} {lsq_general:>7}/{len(general)}"
                )

    print("\n=== joint smooth + hidden-low coefficient family ===")
    family = quotient_family(x87_kahan)
    print(
        "The established x87-spill Kahan output is corrected by a smooth "
        "Chebyshev series plus low_corr*Chebyshev.  All coefficients are "
        "recovered on power rows and frozen before general replay."
    )
    for mode in ("linear-tau", "log2-tau"):
        print(f"\ncoordinate={mode}")
        print(
            " smooth low params   min-slack exact-cert  LP-replay "
            "LSQ-replay general-LSQ"
        )
        for smooth_degree in (2, 4, 6, 8):
            for low_degree in (0, 1, 2):
                fit = fit_smooth_low_family(
                    power,
                    family,
                    mode,
                    smooth_degree,
                    low_degree,
                    log_min,
                    log_max,
                )
                lp_power = replay_smooth_low(
                    power,
                    family,
                    mode,
                    smooth_degree,
                    low_degree,
                    fit.coefficients,
                    log_min,
                    log_max,
                )
                lsq_power = replay_smooth_low(
                    power,
                    family,
                    mode,
                    smooth_degree,
                    low_degree,
                    fit.least_squares_coefficients,
                    log_min,
                    log_max,
                )
                lsq_general = replay_smooth_low(
                    general,
                    family,
                    mode,
                    smooth_degree,
                    low_degree,
                    fit.least_squares_coefficients,
                    log_min,
                    log_max,
                )
                certificate = "yes" if fit.exact_certificate_margin is not None else "NO"
                print(
                    f" {smooth_degree:>6} {low_degree:>3} "
                    f"{len(fit.coefficients):>6} {fit.slack:>11.6g} {certificate:>10} "
                    f"{lp_power:>5}/{len(power):<3} {lsq_power:>5}/{len(power):<3} "
                    f"{lsq_general:>7}/{len(general)}"
                )


def main() -> None:
    rows = build_rows()
    report_low_word(rows)
    report_polynomials(rows)


if __name__ == "__main__":
    main()
