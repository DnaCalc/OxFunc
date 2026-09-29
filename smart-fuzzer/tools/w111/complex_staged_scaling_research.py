"""Compare general staged-power arithmetic graphs against retained COMPLEX captures.

The stages reuse the publicly characterized decimal-parser exponent decomposition:
units, sixteens, and 256s. Every power is generated mathematically and rounded to
64 binary significand bits; each arithmetic operation uses nearest-even RN64.
This is black-box hypothesis comparison, not an assertion about Excel internals.
All failures are retained. No input-dependent candidate selection is permitted.
"""
from functools import lru_cache
from fractions import Fraction as F
import json
from pathlib import Path
import sys

import complex_binary_scaling_candidate as original
from gen_complex_midpoint_search_20260929 import extended_power, half_away, round_binary


@lru_cache(None)
def pair(value, order, operation, combine, decomposition='nibble'):
    k = 14 - int(format(abs(value), '.30e').split('e')[1])
    chunks = ([abs(k) & 15, abs(k) & 240, abs(k) & 3840] if decomposition == 'nibble'
              else [abs(k) & (1 << bit) for bit in range(9)])
    if order == 'descending':
        chunks.reverse()
    # signed: multiply signed powers. split: multiply for k>=0, divide for k<0.
    # divide: divide by powers of the opposite sign. inverse-split: reverse split.
    divide = operation == 'divide' or (operation == 'split' and k < 0) or (
        operation == 'inverse-split' and k >= 0)
    sign = (-1 if k < 0 else 1) * (-1 if divide else 1)
    powers = [extended_power(sign * chunk) for chunk in chunks if chunk]
    if combine:
        product = F(1)
        for power in powers:
            product = round_binary(product * power, 64)
        powers = [product]
    scaled = F(abs(value))
    for power in powers:
        scaled = round_binary(scaled / power if divide else scaled * power, 64)
    return half_away(scaled), -k


@lru_cache(None)
def normalize_pair(value, digits, order, scaling):
    exponent = int(format(abs(value), '.30e').split('e')[1])
    # Normalize to [1,10) or [.1,1), then scale to fifteen significant digits.
    normalize_power = exponent + (digits - 14)
    normalize = ('scale', -normalize_power)
    significant = ('scale', digits)
    stages = [normalize, significant] if order == 'normalize-first' else [significant, normalize]
    x = F(abs(value))
    for _, power in stages:
        if scaling == 'split' and power < 0:
            x = round_binary(x / extended_power(-power), 64)
        else:
            x = round_binary(x * extended_power(power), 64)
    return half_away(x), exponent - 14


def main(root, output):
    names = ['format-complex', 'format-imconjugate', 'format-improduct', 'format-imsum',
             'extremes-complex', 'ties-complex', 'heldout-complex', 'heldout-imconjugate',
             'heldout-improduct', 'heldout-imsum', 'heldout2-complex', 'heldout2-imconjugate',
             'heldout2-improduct', 'heldout2-imsum', 'midpoints-complex', 'scaling-heldout-complex']
    captures = [(name, json.loads((root / (name + '.json')).read_text(encoding='utf-8-sig')))
                for name in names]
    reports = []
    baseline = original.decimal_pair
    variants = [('single-full-power', baseline)]
    for decomposition in ['nibble', 'binary']:
        for order in ['ascending', 'descending']:
            for operation in ['signed', 'split', 'divide', 'inverse-split']:
                for combine in [False, True]:
                    label = ('' if decomposition == 'nibble' else 'binary-') + f'{order}-{operation}-' + ('combined-powers' if combine else 'sequential')
                    variants.append((label, lambda n, o=order, p=operation, c=combine, d=decomposition: pair(n, o, p, c, d)))
    for digits in [14, 15]:
        for order in ['normalize-first', 'digits-first']:
            for scaling in ['signed', 'split']:
                label = f'{order}-{digits}-{scaling}'
                variants.append((label, lambda n, d=digits, o=order, s=scaling: normalize_pair(n, d, o, s)))
    for label, candidate in variants:
        original.decimal_pair = candidate
        rows = 0
        misses = []
        by_capture = {}
        for name, capture in captures:
            local_misses = 0
            for row in capture['witnesses']:
                actual = original.predict(capture['function'], row['args'])
                rows += 1
                if actual != row['expected_bits']:
                    local_misses += 1
                    misses.append(dict(capture=name, id=row['id'], args=row['args'], actual=actual,
                                       expected=row['expected_bits']))
            by_capture[name] = dict(rows=len(capture['witnesses']), misses=local_misses)
        reports.append(dict(candidate=label, rows=rows, matches=rows-len(misses),
                            by_capture=by_capture, misses=misses))
        print(label, rows-len(misses), '/', rows, flush=True)
    output.write_text(json.dumps(dict(authority='non_semantic_research_hypotheses',
                                     reports=reports), indent=2), encoding='utf-8')


if __name__ == '__main__':
    main(Path(sys.argv[1]), Path(sys.argv[2]))
