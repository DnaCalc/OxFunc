"""Confound audit for the W109 PMT exact-tau predicate.

The 234-row power-of-two-rate oracle shows that misses of the published
all-binary64 Kahan reconstruction cluster on rows where ``n * log1p(r)`` is
exactly representable.  That observation is useful only if it survives the
two obvious confounds: the magnitude/binade of tau and the distance of the
exact final quotient from a binary64 rounding midpoint.

This script uses only Python's standard library.  It fits nested logistic
models with flexible one-hot controls for tau binade, midpoint-distance
quantile, and corpus family, then reports the one-degree-of-freedom
likelihood-ratio test for the exact-tau indicator.  It audits the original
power-of-two-rate corpus, the independently captured general-rate corpus, and
their union.  It reads the gitignored W109 evidence and does not mutate it.
"""

from __future__ import annotations

import csv
import hashlib
import json
import math
import struct
from dataclasses import dataclass
from fractions import Fraction
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
SOLVERS = ROOT / "smart-fuzzer/work/w109/G6-solvers"
CORPUS = SOLVERS / "expm1_intermediates.csv"
GENERAL_META = SOLVERS / "meta-pmt-general-intermediates-20260809.csv"
GENERAL_EXP_BATCH = SOLVERS / "batch-pmt-general-exp-20260809.json"
GENERAL_EXP_ANSWERS = SOLVERS / "answers-pmt-general-exp-20260809.json"
GENERAL_LN_BATCH = SOLVERS / "batch-pmt-general-ln-20260809.json"
GENERAL_LN_ANSWERS = SOLVERS / "answers-pmt-general-ln-20260809.json"


@dataclass(frozen=True)
class AuditRow:
    family: str
    rate_key: str
    n: int
    tau: float
    u: float
    lnu: float
    expected_bits: int
    exact_tau: bool
    recorded_kahan_bits: int | None = None


def from_bits(text: str) -> float:
    return struct.unpack("<d", struct.pack("<Q", int(text, 16)))[0]


def bits(value: float) -> int:
    return struct.unpack("<Q", struct.pack("<d", value))[0]


def ordered_bits(raw: int) -> int:
    return (~raw & ((1 << 64) - 1)) if raw >> 63 else raw | (1 << 63)


def ulp_delta(actual_bits: int, candidate_bits: int) -> int:
    return ordered_bits(actual_bits) - ordered_bits(candidate_bits)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def require_bits(text: Any, context: str) -> str:
    if not isinstance(text, str) or len(text) != 18 or not text.startswith("0x"):
        raise RuntimeError(f"{context}: expected 0x + 16 hex digits, got {text!r}")
    try:
        int(text[2:], 16)
    except ValueError as error:
        raise RuntimeError(f"{context}: invalid f64 bits {text!r}") from error
    return text.lower()


def load_json(path: Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as stream:
        value = json.load(stream)
    if not isinstance(value, dict):
        raise RuntimeError(f"{path}: expected a JSON object")
    return value


def validate_capture_provenance(value: dict[str, Any], path: Path) -> None:
    provenance = value.get("capture_provenance")
    if not isinstance(provenance, dict):
        raise RuntimeError(f"{path}: missing capture_provenance")
    environment = provenance.get("environment")
    cache = provenance.get("oracle_cache")
    runner = provenance.get("runner")
    expected_environment = {
        "excel_version": "16.0",
        "excel_build": "20228",
        "excel_bitness": "64-bit",
        "workbook_compatibility": "2",
        "excel_input_plumbing": "cell_value2_bulk",
    }
    if not isinstance(environment, dict):
        raise RuntimeError(f"{path}: invalid capture environment")
    for key, expected in expected_environment.items():
        if environment.get(key) != expected:
            raise RuntimeError(
                f"{path}: provenance {key}={environment.get(key)!r}, expected {expected!r}"
            )
    if not isinstance(cache, dict) or cache.get("mode") != "no_cache":
        raise RuntimeError(f"{path}: capture was not NoCache")
    if cache.get("hits") != 0 or cache.get("misses") != 0:
        raise RuntimeError(f"{path}: unexpected cache counters {cache!r}")
    if not isinstance(runner, dict) or runner.get("version") != "w109-bulk-batch-v2":
        raise RuntimeError(f"{path}: unexpected runner metadata {runner!r}")


def next_down(value: float) -> float:
    raw = bits(value)
    if value == 0.0:
        return -float.fromhex("0x0.0000000000001p-1022")
    return from_bits(f"{raw + 1 if value < 0.0 else raw - 1:016x}")


def next_up(value: float) -> float:
    raw = bits(value)
    if value == 0.0:
        return float.fromhex("0x0.0000000000001p-1022")
    return from_bits(f"{raw - 1 if value < 0.0 else raw + 1:016x}")


def midpoint_distance_in_ulps(exact: Fraction, rounded: float) -> float:
    """Distance to the nearer RN boundary, normalized by that side's ULP."""

    center = Fraction(rounded)
    lower = Fraction(next_down(rounded))
    upper = Fraction(next_up(rounded))
    lower_spacing = center - lower
    upper_spacing = upper - center
    lower_midpoint = (lower + center) / 2
    upper_midpoint = (center + upper) / 2
    lower_distance = abs(exact - lower_midpoint) / lower_spacing
    upper_distance = abs(exact - upper_midpoint) / upper_spacing
    return float(min(lower_distance, upper_distance))


def round_fraction_binary(value: Fraction, precision: int) -> Fraction:
    """Round a finite rational to a binary significand using RN ties-to-even."""

    if value == 0:
        return value
    sign = -1 if value < 0 else 1
    magnitude = abs(value)
    numerator = magnitude.numerator
    denominator = magnitude.denominator
    exponent = numerator.bit_length() - denominator.bit_length()
    if exponent >= 0:
        if numerator < denominator << exponent:
            exponent -= 1
    elif numerator << (-exponent) < denominator:
        exponent -= 1

    shift = exponent - precision + 1
    if shift >= 0:
        scaled_numerator = numerator
        scaled_denominator = denominator << shift
    else:
        scaled_numerator = numerator << (-shift)
        scaled_denominator = denominator
    quotient, remainder = divmod(scaled_numerator, scaled_denominator)
    twice = remainder * 2
    if twice > scaled_denominator or (twice == scaled_denominator and quotient & 1):
        quotient += 1
    if quotient == 1 << precision:
        quotient >>= 1
        shift += 1
    rounded = Fraction(quotient)
    if shift >= 0:
        rounded *= 1 << shift
    else:
        rounded /= 1 << (-shift)
    return sign * rounded


def x87_double_round_mul(left: float, right: float) -> float:
    return float(round_fraction_binary(Fraction(left) * Fraction(right), 64))


def x87_double_round_div(left: float, right: float) -> float:
    return float(round_fraction_binary(Fraction(left) / Fraction(right), 64))


def kahan_plain(row: AuditRow) -> float:
    if row.u == 1.0:
        return row.tau
    if abs(row.tau) < 1.0:
        return (row.u - 1.0) * row.tau / row.lnu
    return row.u - 1.0


def kahan_x87_spill(row: AuditRow) -> float:
    if row.u == 1.0:
        return row.tau
    if abs(row.tau) < 1.0:
        numerator = x87_double_round_mul(row.u - 1.0, row.tau)
        return x87_double_round_div(numerator, row.lnu)
    return row.u - 1.0


def load_power_of_two_rows() -> list[AuditRow]:
    with CORPUS.open(newline="", encoding="utf-8") as stream:
        raw_rows = list(csv.DictReader(stream))
    logs: dict[int, float] = {}
    for row in raw_rows:
        if int(row["n"]) == 1:
            logs[int(row["k"])] = abs(from_bits(row["tau_bits"]))

    rows: list[AuditRow] = []
    for row in raw_rows:
        key = int(row["k"])
        n = int(row["n"])
        tau = from_bits(row["tau_bits"])
        exact = Fraction(n) * Fraction(logs[key]) == Fraction(abs(tau))
        rows.append(
            AuditRow(
                family="power-of-two-rate",
                rate_key=f"2^{key}",
                n=n,
                tau=tau,
                u=from_bits(row["u_bits"]),
                lnu=from_bits(row["lnu_bits"]),
                expected_bits=int(row["em_pinned"], 16),
                exact_tau=exact,
            )
        )
    return rows


def load_general_rate_rows() -> list[AuditRow]:
    with GENERAL_META.open(newline="", encoding="utf-8") as stream:
        metadata = list(csv.DictReader(stream))
    exp_batch = load_json(GENERAL_EXP_BATCH)
    exp_answers = load_json(GENERAL_EXP_ANSWERS)
    ln_batch = load_json(GENERAL_LN_BATCH)
    ln_answers = load_json(GENERAL_LN_ANSWERS)
    validate_capture_provenance(exp_answers, GENERAL_EXP_ANSWERS)
    validate_capture_provenance(ln_answers, GENERAL_LN_ANSWERS)
    if exp_batch.get("function") != "EXP" or exp_answers.get("function") != "EXP":
        raise RuntimeError("general-rate EXP function tag mismatch")
    if ln_batch.get("function") != "LN" or ln_answers.get("function") != "LN":
        raise RuntimeError("general-rate LN function tag mismatch")

    exp_probes = exp_batch.get("probes")
    ln_probes = ln_batch.get("probes")
    exp_witnesses = exp_answers.get("witnesses")
    ln_witnesses = ln_answers.get("witnesses")
    cohorts = (metadata, exp_probes, exp_witnesses, ln_probes, ln_witnesses)
    if any(not isinstance(cohort, list) for cohort in cohorts):
        raise RuntimeError("general-rate artifact has a non-array cohort")
    counts = [len(cohort) for cohort in cohorts]
    if counts != [90, 90, 90, 90, 90]:
        raise RuntimeError(f"general-rate cohort counts are not all 90: {counts}")

    logs: dict[str, float] = {}
    for meta in metadata:
        if int(meta["n"]) == 1:
            logs[require_bits(meta["r_bits"], "metadata r_bits")] = abs(
                from_bits(require_bits(meta["tau_bits"], "metadata tau_bits"))
            )

    rows: list[AuditRow] = []
    for index, meta in enumerate(metadata):
        exp_probe = exp_probes[index].get("probe")
        ln_probe = ln_probes[index].get("probe")
        exp_witness = exp_witnesses[index]
        ln_witness = ln_witnesses[index]
        if not all(
            isinstance(item, dict)
            for item in (exp_probe, ln_probe, exp_witness, ln_witness)
        ):
            raise RuntimeError(f"general-rate row {index}: malformed probe/witness")

        exp_id = meta["exp_id"]
        ln_id = meta["ln_id"]
        tau_bits = require_bits(meta["tau_bits"], f"row {index} tau")
        if exp_probe.get("id") != exp_id or exp_witness.get("id") != exp_id:
            raise RuntimeError(f"general-rate row {index}: EXP id mismatch")
        if ln_probe.get("id") != ln_id or ln_witness.get("id") != ln_id:
            raise RuntimeError(f"general-rate row {index}: LN id mismatch")
        if exp_probe.get("args") != [tau_bits] or exp_witness.get("args") != [tau_bits]:
            raise RuntimeError(f"general-rate row {index}: EXP argument mismatch")
        u_bits = require_bits(exp_witness.get("expected_bits"), f"row {index} EXP answer")
        if ln_probe.get("args") != [u_bits] or ln_witness.get("args") != [u_bits]:
            raise RuntimeError(f"general-rate row {index}: LN argument mismatch")
        lnu_bits = require_bits(ln_witness.get("expected_bits"), f"row {index} LN answer")

        rate_key = require_bits(meta["r_bits"], f"row {index} r_bits")
        if rate_key not in logs:
            raise RuntimeError(f"general-rate row {index}: rate lacks an n=1 anchor")
        n = int(meta["n"])
        tau = from_bits(tau_bits)
        rows.append(
            AuditRow(
                family="general-rate",
                rate_key=rate_key,
                n=n,
                tau=tau,
                u=from_bits(u_bits),
                lnu=from_bits(lnu_bits),
                expected_bits=int(require_bits(meta["em_pinned"], "em_pinned"), 16),
                exact_tau=Fraction(n) * Fraction(logs[rate_key]) == Fraction(abs(tau)),
                recorded_kahan_bits=int(require_bits(meta["kahan"], "kahan"), 16),
            )
        )
    return rows


def solve(matrix: list[list[float]], vector: list[float]) -> list[float]:
    """Gaussian elimination with partial pivoting for a small dense system."""

    n = len(vector)
    augmented = [matrix[row][:] + [vector[row]] for row in range(n)]
    for column in range(n):
        pivot = max(range(column, n), key=lambda row: abs(augmented[row][column]))
        if abs(augmented[pivot][column]) < 1e-14:
            raise RuntimeError("singular logistic normal matrix")
        augmented[column], augmented[pivot] = augmented[pivot], augmented[column]
        scale = augmented[column][column]
        for item in range(column, n + 1):
            augmented[column][item] /= scale
        for row in range(n):
            if row == column:
                continue
            factor = augmented[row][column]
            if factor == 0.0:
                continue
            for item in range(column, n + 1):
                augmented[row][item] -= factor * augmented[column][item]
    return [augmented[row][n] for row in range(n)]


def logistic_fit(design: list[list[float]], outcome: list[int]) -> tuple[list[float], float]:
    """Fit an unpenalized logistic model by damped Newton/IRLS."""

    width = len(design[0])
    beta = [0.0] * width

    def log_likelihood(candidate: list[float]) -> float:
        total = 0.0
        for row, target in zip(design, outcome, strict=True):
            eta = sum(value * coefficient for value, coefficient in zip(row, candidate, strict=True))
            if eta >= 0.0:
                total += target * eta - eta - math.log1p(math.exp(-eta))
            else:
                total += target * eta - math.log1p(math.exp(eta))
        return total

    likelihood = log_likelihood(beta)
    for _ in range(100):
        gradient = [0.0] * width
        information = [[0.0] * width for _ in range(width)]
        for row, target in zip(design, outcome, strict=True):
            eta = sum(value * coefficient for value, coefficient in zip(row, beta, strict=True))
            probability = 1.0 / (1.0 + math.exp(-max(-40.0, min(40.0, eta))))
            residual = target - probability
            weight = max(probability * (1.0 - probability), 1e-12)
            for left in range(width):
                gradient[left] += row[left] * residual
                for right in range(width):
                    information[left][right] += weight * row[left] * row[right]

        step = solve(information, gradient)
        scale = 1.0
        while scale > 2.0**-20:
            proposal = [value + scale * delta for value, delta in zip(beta, step, strict=True)]
            proposal_likelihood = log_likelihood(proposal)
            if proposal_likelihood >= likelihood:
                beta = proposal
                likelihood = proposal_likelihood
                break
            scale *= 0.5
        if max(abs(scale * delta) for delta in step) < 1e-9:
            break
    return beta, likelihood


def quantile_buckets(values: list[float], count: int) -> list[int]:
    ordered = sorted(range(len(values)), key=values.__getitem__)
    buckets = [0] * len(values)
    for rank, index in enumerate(ordered):
        buckets[index] = min(count - 1, rank * count // len(values))
    return buckets


def categorical_design(
    binades: list[int],
    distance_buckets: list[int],
    families: list[str],
    exact_flags: list[bool] | None,
) -> list[list[float]]:
    unique_binades = sorted(set(binades))
    unique_distance = sorted(set(distance_buckets))
    unique_families = sorted(set(families))
    rows: list[list[float]] = []
    for index, (binade, distance) in enumerate(zip(binades, distance_buckets, strict=True)):
        row = [1.0]
        row.extend(1.0 if binade == value else 0.0 for value in unique_binades[1:])
        row.extend(1.0 if distance == value else 0.0 for value in unique_distance[1:])
        row.extend(1.0 if families[index] == value else 0.0 for value in unique_families[1:])
        if exact_flags is not None:
            row.append(1.0 if exact_flags[index] else 0.0)
        rows.append(row)
    return rows


def audit(name: str, rows: list[AuditRow]) -> None:
    misses: list[int] = []
    exact_flags: list[bool] = []
    binades: list[int] = []
    midpoint_distances: list[float] = []

    for row in rows:
        quotient = kahan_plain(row)
        if row.u == 1.0:
            exact_quotient = Fraction(row.tau)
        elif abs(row.tau) < 1.0:
            numerator = (row.u - 1.0) * row.tau
            exact_quotient = Fraction(numerator) / Fraction(row.lnu)
        else:
            exact_quotient = Fraction(row.u - 1.0)

        misses.append(1 if bits(quotient) != row.expected_bits else 0)
        exact_flags.append(row.exact_tau)
        binades.append(math.frexp(abs(row.tau))[1] - 1)
        midpoint_distances.append(midpoint_distance_in_ulps(exact_quotient, quotient))

    distance_buckets = quantile_buckets(midpoint_distances, 8)
    families = [row.family for row in rows]
    reduced = categorical_design(binades, distance_buckets, families, None)
    expanded = categorical_design(binades, distance_buckets, families, exact_flags)

    exact_miss = sum(miss for miss, exact in zip(misses, exact_flags, strict=True) if exact)
    exact_total = sum(exact_flags)
    rounded_miss = sum(miss for miss, exact in zip(misses, exact_flags, strict=True) if not exact)
    rounded_total = len(exact_flags) - exact_total
    plain_hits = len(rows) - sum(misses)
    spill_hits = sum(bits(kahan_x87_spill(row)) == row.expected_bits for row in rows)
    recorded = [row for row in rows if row.recorded_kahan_bits is not None]
    recorded_mismatches = sum(
        bits(kahan_plain(row)) != row.recorded_kahan_bits for row in recorded
    )
    if recorded_mismatches:
        raise RuntimeError(
            f"{name}: {recorded_mismatches} rows disagree with recorded Kahan bits"
        )

    print(f"\n[{name}]")
    print(
        f"rows={len(rows)} plain-kahan={plain_hits}/{len(rows)} "
        f"x87-spill-kahan={spill_hits}/{len(rows)}"
    )
    for model_name, model in (
        ("plain", kahan_plain),
        ("x87-spill", kahan_x87_spill),
    ):
        for partition_name, wanted_exact in (("exact", True), ("rounded", False)):
            deltas = [
                ulp_delta(row.expected_bits, bits(model(row)))
                for row in rows
                if row.exact_tau == wanted_exact
            ]
            histogram: dict[int, int] = {}
            for delta in deltas:
                histogram[delta] = histogram.get(delta, 0) + 1
            compact = ", ".join(
                f"{delta:+d}:{count}"
                for delta, count in sorted(histogram.items(), key=lambda item: (-item[1], item[0]))[:7]
            )
            print(f"{model_name} {partition_name} delta histogram={compact}")
    print(
        "unadjusted exact-tau table: "
        f"exact {exact_miss}/{exact_total} misses; "
        f"rounded {rounded_miss}/{rounded_total} misses"
    )
    print("period strata (n: misses/rows, exact rows):")
    for period in sorted({row.n for row in rows}):
        indices = [index for index, row in enumerate(rows) if row.n == period]
        period_misses = sum(misses[index] for index in indices)
        period_exact = sum(exact_flags[index] for index in indices)
        period_exact_misses = sum(
            misses[index] for index in indices if exact_flags[index]
        )
        period_rounded_misses = sum(
            misses[index] for index in indices if not exact_flags[index]
        )
        print(
            f"  {period:>4}: {period_misses:>3}/{len(indices):<3} misses, "
            f"exact {period_exact_misses:>2}/{period_exact:<2}, "
            f"rounded {period_rounded_misses:>2}/{len(indices) - period_exact:<2}"
        )
    if exact_total == 0 or rounded_total == 0:
        print("adjusted test=not estimable because the exact-tau indicator is constant")
        return

    try:
        _, reduced_ll = logistic_fit(reduced, misses)
        expanded_beta, expanded_ll = logistic_fit(expanded, misses)
    except RuntimeError as error:
        print(f"adjusted test=not estimable ({error})")
        return

    statistic = max(0.0, 2.0 * (expanded_ll - reduced_ll))
    p_value = math.erfc(math.sqrt(statistic / 2.0))
    exact_coefficient = expanded_beta[-1]
    odds_ratio = (
        math.exp(exact_coefficient) if exact_coefficient < 700.0 else float("inf")
    )
    print(
        "controls: tau-binade one-hot + midpoint-distance octile one-hot + "
        "corpus-family one-hot "
        f"({len(reduced[0])} reduced parameters)"
    )
    print(f"reduced log-likelihood={reduced_ll:.9f}")
    print(f"expanded log-likelihood={expanded_ll:.9f}")
    print(
        "exact-tau adjusted coefficient="
        f"{exact_coefficient:+.9f} odds-ratio={odds_ratio:.6g}"
    )
    print(f"likelihood-ratio chi2(1)={statistic:.9f} p={p_value:.9g}")
    print(
        "interpretation="
        + (
            "exact-tau remains independently predictive; retain the branch/early-out hypothesis"
            if p_value < 0.01 and exact_coefficient > 0.0
            else "exact-tau is not independently predictive in this cohort after the stated controls"
        )
    )


def main() -> None:
    power_rows = load_power_of_two_rows()
    general_rows = load_general_rate_rows()
    print(f"power-of-two corpus={CORPUS}")
    print(
        "general-rate artifacts="
        + ", ".join(
            f"{path.name}:{sha256(path)}"
            for path in (
                GENERAL_META,
                GENERAL_EXP_BATCH,
                GENERAL_EXP_ANSWERS,
                GENERAL_LN_BATCH,
                GENERAL_LN_ANSWERS,
            )
        )
    )
    audit("power-of-two-rate bank", power_rows)
    audit("general-rate live intermediates", general_rows)
    audit("combined", power_rows + general_rows)


if __name__ == "__main__":
    main()
