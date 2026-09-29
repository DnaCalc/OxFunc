"""Fixed decimal digit-extraction graphs for the retained initial15 packet.

The extra digit count and extraction policy apply uniformly to every row.
"""
import decimal
import itertools
import json
import struct
from collections import Counter
from fractions import Fraction as F
from pathlib import Path

from analyze_decimal_parser_scaling import nibble64, power, rn


def integer(value, policy):
    q, r = divmod(value.numerator, value.denominator)
    return q + (policy == 'away' and 2*r >= value.denominator or
                policy == 'even' and (2*r > value.denominator or 2*r == value.denominator and q & 1))


def main():
    evidence = Path('docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries')
    raw = json.loads((evidence / 'directed-initial-precision-rounddown.json').read_text())['witnesses']
    rows = {r['args'][0]: r for r in raw if int(r['args'][0], 16) >> 63 == 0}
    models = list(itertools.product(range(0, 22), ['nibble', 'full_multiply', 'full_divide'],
                                   ['truncate', 'away', 'even']))
    differences = Counter()
    residuals = {}
    for row in rows.values():
        number = struct.unpack('>d', bytes.fromhex(row['args'][0][2:]))[0]
        k = 14 - decimal.Decimal.from_float(number).adjusted()
        outputs = {}
        for extra, graph, extraction in models:
            if graph == 'nibble':
                value = nibble64(F(number), k + extra)
            elif graph == 'full_multiply':
                value = rn(F(number) * power(k + extra, 64))
            else:
                value = rn(F(number) / power(-k - extra, 64))
            significand = integer(F(integer(value, extraction), 10**extra), 'away')
            if significand not in outputs:
                outputs[significand] = '0x' + struct.pack('>d', float(nibble64(significand, -k))).hex()
            name = f'extra{extra}-{graph}-{extraction}'
            miss = outputs[significand] != row['expected_bits']
            differences[name] += miss
            if miss:
                residuals.setdefault(name, []).append(row['id'])
    ranking = sorted(differences.items(), key=lambda item: item[1])
    report = {'research_only': True, 'normal_positive_centers': len(rows), 'uniform_models': len(models),
              'model_differences': dict(ranking),
              'best_residual_ids': {name: residuals.get(name, []) for name, _ in ranking[:10]},
              'no_runtime_changes': True}
    (evidence / 'directed-initial-digit-extraction-models.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(len(rows), ranking[:20])


if __name__ == '__main__':
    main()
