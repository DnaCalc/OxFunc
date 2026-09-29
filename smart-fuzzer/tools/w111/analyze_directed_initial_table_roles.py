"""Test uniform low/middle/high power-table policies, without exponent fitting."""
import decimal
import itertools
import json
import struct
from collections import Counter
from fractions import Fraction as F
from pathlib import Path

from analyze_decimal_parser_scaling import nibble64, rn
from analyze_initial15_directed_precision import power


def main():
    evidence = Path('docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries')
    rows = {}
    for filename in ['directed-initial-precision-rounddown.json', 'initial-discriminators-rounddown.json']:
        raw = json.loads((evidence / filename).read_text())['witnesses']
        for row in raw:
            if int(row['args'][0], 16) >> 63 == 0:
                key = row['args'][0]
                if key in rows:
                    assert rows[key]['expected_bits'] == row['expected_bits']
                rows[key] = row
    models = list(itertools.product(itertools.product(['down', 'even', 'up'], repeat=3),
                                   itertools.product([False, True], repeat=3)))
    differences = Counter()
    residuals = {}
    for row in rows.values():
        number = struct.unpack('>d', bytes.fromhex(row['args'][0][2:]))[0]
        k = 14 - decimal.Decimal.from_float(number).adjusted()
        magnitude = abs(k)
        chunks = [magnitude % 16, magnitude // 16 % 16 * 16, magnitude // 256 * 256]
        chunks = [(-1 if k < 0 else 1) * c for c in chunks]
        outputs = {}
        for modes, divides in models:
            value = F(number)
            for chunk, mode, divide in zip(chunks, modes, divides):
                if not chunk:
                    continue
                factor = power(-chunk if divide else chunk, 64, mode)
                value = rn(value / factor if divide else value * factor)
            q, r = divmod(value.numerator, value.denominator)
            significand = q + (2 * r >= value.denominator)
            if significand not in outputs:
                outputs[significand] = '0x' + struct.pack('>d', float(nibble64(significand, -k))).hex()
            name = f'power_modes_{modes}-divides_{divides}'
            miss = outputs[significand] != row['expected_bits']
            differences[name] += miss
            if miss:
                residuals.setdefault(name, []).append(row['id'])
    ranking = sorted(differences.items(), key=lambda item: item[1])
    report = {'research_only': True, 'normal_positive_inputs': len(rows), 'uniform_models': len(models),
              'model_differences': dict(ranking),
              'best_residual_ids': {name: residuals.get(name, []) for name, _ in ranking[:10]},
              'no_runtime_changes': True}
    (evidence / 'directed-initial-table-role-models.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print(len(rows), ranking[:20])


if __name__ == '__main__':
    main()
