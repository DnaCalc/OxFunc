"""Uniform two-stage scaling hypotheses for directed initial normalization.

This script reads retained observations; no Excel access or production edits.
Each model uses one fixed intermediate decimal magnitude across every input.
"""
import decimal
import itertools
import json
import struct
from collections import Counter
from fractions import Fraction as F
from functools import lru_cache
from pathlib import Path

from analyze_decimal_parser_scaling import nibble64, power, rn


def bits(value):
    return '0x' + struct.pack('>d', float(value)).hex()


@lru_cache(None)
def chunks(exponent, decomposition, reverse):
    sign = -1 if exponent < 0 else 1
    magnitude = abs(exponent)
    if decomposition == 'full':
        result = [magnitude]
    elif decomposition == 'binary':
        result = [magnitude & (1 << j) for j in range(10)]
    else:
        result = [magnitude % 16, (magnitude // 16 % 16) * 16, (magnitude // 256) * 256]
    result = [sign * item for item in result if item]
    return tuple(reversed(result)) if reverse else tuple(result)


def scale(value, exponent, decomposition, reverse, divide, precision=64):
    for chunk in chunks(exponent, decomposition, reverse):
        if divide:
            value = rn(value / power(-chunk, precision), precision)
        else:
            value = rn(value * power(chunk, precision), precision)
    return value


def main():
    evidence = Path('docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries')
    raw = json.loads((evidence / 'directed-initial-precision-rounddown.json').read_text())['witnesses']
    unique = {}
    for row in raw:
        number = struct.unpack('>d', bytes.fromhex(row['args'][0][2:]))[0]
        if number > 0:
            unique.setdefault(number, row)
    models = list(itertools.product(range(-2, 18), ['full', 'binary', 'nibble'],
                                   [False, True], [False, True], [53, 64]))
    differences = Counter()
    residuals = {}
    for number, row in unique.items():
        exponent = decimal.Decimal.from_float(number).adjusted()
        cached = {}
        for intermediate, decomposition, reverse, divide, storage in models:
            value = scale(F(number), intermediate - exponent, decomposition, reverse, divide)
            if storage == 53:
                value = rn(value, 53)
            value = scale(value, 14 - intermediate, decomposition, reverse, divide)
            quotient, remainder = divmod(value.numerator, value.denominator)
            significand = quotient + (2 * remainder >= value.denominator)
            if significand not in cached:
                cached[significand] = bits(nibble64(significand, exponent - 14))
            label = f'intermediate{intermediate}-{decomposition}-reverse{reverse}-divide{divide}-storage{storage}'
            miss = cached[significand] != row['expected_bits']
            differences[label] += miss
            if miss:
                residuals.setdefault(label, []).append(row['id'])
    ranking = sorted(differences.items(), key=lambda item: item[1])
    report = {'research_only': True, 'normal_positive_centers': len(unique), 'uniform_models': len(models),
              'model_differences': dict(ranking),
              'best_residual_ids': {name: residuals.get(name, []) for name, _ in ranking[:20]},
              'no_runtime_changes': True}
    (evidence / 'directed-initial-split-scaling-models.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(len(unique), ranking[:20])


if __name__ == '__main__':
    main()
