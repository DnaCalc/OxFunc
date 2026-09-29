"""Compare fixed, general power-table construction rules against black-box rows.

No entry is selected to fit a particular exponent or observation.
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


@lru_cache(None)
def table(exponent, construction, negative):
    magnitude = abs(exponent)
    sign = -1 if exponent < 0 else 1
    if negative != 'signed':
        sign = 1
    base = rn(F(10)**sign)
    if construction == 'direct':
        value = power(sign * magnitude, 64)
    elif construction == 'iterative':
        value = F(1)
        for _ in range(magnitude):
            value = rn(value * base)
    elif construction == 'binary':
        value = F(1)
        while magnitude:
            if magnitude & 1:
                value = rn(value * base)
            magnitude >>= 1
            base = rn(base * base)
    elif construction == 'sixteen':
        low, high = magnitude % 16, magnitude // 16
        value = power(sign * low, 64)
        base = power(sign * 16, 64)
        for _ in range(high):
            value = rn(value * base)
    elif construction == 'sixteen_binary':
        low, high = magnitude % 16, magnitude // 16
        value = power(sign * low, 64)
        base = power(sign * 16, 64)
        while high:
            if high & 1:
                value = rn(value * base)
            high >>= 1
            base = rn(base * base)
    if exponent < 0 and negative == 'reciprocal':
        value = rn(1 / value)
    return value


def main():
    evidence = Path('docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries')
    raw = json.loads((evidence / 'directed-initial-precision-rounddown.json').read_text())['witnesses']
    unique = {}
    for row in raw:
        number = struct.unpack('>d', bytes.fromhex(row['args'][0][2:]))[0]
        if number > 0:
            unique.setdefault(number, row)
    models = list(itertools.product(['direct', 'iterative', 'binary', 'sixteen', 'sixteen_binary'],
                                   ['signed', 'reciprocal', 'divide'], [False, True], [53, 64]))
    differences = Counter()
    residuals = {}
    for number, row in unique.items():
        k = 14 - decimal.Decimal.from_float(number).adjusted()
        magnitude = abs(k)
        chunks = [magnitude % 16, magnitude // 16 % 16 * 16, magnitude // 256 * 256]
        chunks = [(-1 if k < 0 else 1) * item for item in chunks if item]
        cached = {}
        for construction, negative, reverse, precision in models:
            value = F(number)
            for chunk in reversed(chunks) if reverse else chunks:
                factor = table(chunk, construction, negative)
                value = rn(value / factor if chunk < 0 and negative == 'divide' else value * factor, precision)
            quotient, remainder = divmod(value.numerator, value.denominator)
            significand = quotient + (2 * remainder >= value.denominator)
            if significand not in cached:
                cached[significand] = '0x' + struct.pack('>d', float(nibble64(significand, -k))).hex()
            label = f'{construction}-{negative}-reverse{reverse}-precision{precision}'
            miss = cached[significand] != row['expected_bits']
            differences[label] += miss
            if miss:
                residuals.setdefault(label, []).append(row['id'])
    ranking = sorted(differences.items(), key=lambda item: item[1])
    report = {'research_only': True, 'normal_positive_centers': len(unique), 'uniform_models': len(models),
              'model_differences': dict(ranking),
              'best_residual_ids': {name: residuals.get(name, []) for name, _ in ranking[:10]},
              'no_runtime_changes': True}
    (evidence / 'directed-initial-power-construction-models.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(len(unique), ranking[:20])


if __name__ == '__main__':
    main()
