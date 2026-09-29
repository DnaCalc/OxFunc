"""Uniform half-unit-before-scaling hypotheses for initial decimal precision."""
import decimal
import itertools
import json
import struct
from collections import Counter
from fractions import Fraction as F
from pathlib import Path

from analyze_decimal_parser_scaling import nibble64, power, rn


def main():
    evidence = Path('docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries')
    rows = {}
    for filename in ['directed-initial-precision-rounddown.json', 'initial-discriminators-rounddown.json']:
        for row in json.loads((evidence / filename).read_text())['witnesses']:
            if int(row['args'][0], 16) >> 63 == 0:
                rows[row['args'][0]] = row
    models = list(itertools.product(['exact', 'full64', 'nibble64'], [53, 64, 80, 96],
                                   ['exact', 'full64', 'nibble64'], ['truncate', 'nearest']))
    differences = Counter()
    for row in rows.values():
        number = struct.unpack('>d', bytes.fromhex(row['args'][0][2:]))[0]
        k = 14 - decimal.Decimal.from_float(number).adjusted()
        outputs = {}
        for half_rule, precision, scale_rule, extraction in models:
            if half_rule == 'exact':
                half = F(1, 2) * F(10)**(-k)
            elif half_rule == 'full64':
                half = power(-k, 64) / 2
            else:
                half = nibble64(F(1, 2), -k)
            added = rn(F(number) + half, precision)
            if scale_rule == 'exact':
                value = added * F(10)**k
            elif scale_rule == 'full64':
                value = rn(added * power(k, 64))
            else:
                value = nibble64(added, k)
            if extraction == 'truncate':
                significand = int(value)
            else:
                # Round to a much finer integer lattice before truncating,
                # corresponding to a fixed four guard-decimal-digit buffer.
                significand = round(value * 10000) // 10000
            if significand not in outputs:
                outputs[significand] = '0x' + struct.pack('>d', float(nibble64(significand, -k))).hex()
            differences[str((half_rule, precision, scale_rule, extraction))] += outputs[significand] != row['expected_bits']
    ranking = sorted(differences.items(), key=lambda item: item[1])
    report = {'research_only': True, 'normal_positive_inputs': len(rows), 'uniform_models': len(models),
              'model_differences': dict(ranking), 'no_runtime_changes': True}
    (evidence / 'directed-initial-preaddition-models.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(len(rows), ranking[:20])


if __name__ == '__main__':
    main()
