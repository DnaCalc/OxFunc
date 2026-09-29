"""Uniform finite-precision normalization hypotheses, including directed stages.

This is offline research, not a source patch or an input-dependent model switch.
"""
import itertools
import json
import struct
import sys
import decimal
from collections import Counter
from fractions import Fraction as F
from functools import lru_cache
from pathlib import Path

from analyze_decimal_parser_scaling import nibble64
from analyze_numeric_text_precision import half_down


def rounded(q, precision, mode):
    if not q:
        return q
    exponent = q.numerator.bit_length() - q.denominator.bit_length()
    if q < F(2)**exponent:
        exponent -= 1
    shift = precision - 1 - exponent
    scaled = q * F(2)**shift
    n, r = divmod(scaled.numerator, scaled.denominator)
    if mode == 'up' and r or mode == 'even' and (2*r > scaled.denominator or 2*r == scaled.denominator and n & 1):
        n += 1
    return F(n) * F(2)**(-shift)


@lru_cache(None)
def power(exponent, precision, mode):
    return rounded(F(10)**exponent, precision, mode)


def main():
    run = Path('smart-fuzzer/runs/w111-numeric-text-rendering-precision-typed-20260929')
    cases = [json.loads(line) for line in (run/'cases/cases.jsonl').read_text(encoding='utf-8-sig').splitlines()]
    answers = {r['case_id']:r['outcome'].get('bits_hex') for r in map(json.loads, (run/'outcomes/excel.jsonl').read_text(encoding='utf-8-sig').splitlines())}
    directed = '--directed' in sys.argv
    if directed:
        raw = json.loads(Path('smart-fuzzer/runs/w111-directed-initial-precision-20260929/answers-rounddown.json').read_text())['witnesses']
        unique = {}
        for row in raw:
            number = struct.unpack('>d', bytes.fromhex(row['args'][0][2:]))[0]
            if number > 0:
                unique.setdefault(number, row)
        cases = []
        answers = {}
        for number, row in unique.items():
            count = 14 - decimal.Decimal.from_float(number).adjusted()
            cases.append({'canonical_surface_name':'ROUND', 'case_id':row['id'],
                          'args':[{'value':number},{'value':count}]})
            answers[row['id']] = row['expected_bits']
    models = list(itertools.product([64,80,96], ['down','even','up'], ['down','even','up'],
                                   ['full','binary','nibble','normalize'], [False,True], [False,True]))
    counts, residuals = Counter(), {}
    rows = 0
    for case in cases:
        if case['canonical_surface_name'] != 'ROUND' or case['args'][0]['value'] < 0:
            continue
        x = F(case['args'][0]['value'])
        k = int(case['args'][1]['value'])
        rows += 1
        outputs = {}
        for precision, power_mode, stage_mode, decomposition, divide, reverse in models:
            if decomposition == 'full':
                chunks = [k]
            elif decomposition == 'normalize':
                chunks = [k-14,14]
            else:
                ex = abs(k)
                chunks = [ex % 16, ex//16%16*16, ex//256*256] if decomposition == 'nibble' else [ex & (1<<j) for j in range(10)]
                chunks = [chunk * (-1 if k < 0 else 1) for chunk in chunks]
            if reverse:
                chunks.reverse()
            value = x
            for chunk in chunks:
                if not chunk:
                    continue
                factor = power(-chunk if divide else chunk, precision, power_mode)
                value = rounded(value/factor if divide else value*factor, precision, stage_mode)
            if directed:
                quotient, remainder = divmod(value.numerator, value.denominator)
                significand = quotient + (2 * remainder >= value.denominator)
            else:
                significand = half_down(value)
            if significand not in outputs:
                outputs[significand] = '0x' + struct.pack('>d', float(nibble64(significand,-k))).hex()
            label = f'p{precision}-powers{power_mode}-stages{stage_mode}-{decomposition}-divide{divide}-reverse{reverse}'
            counts[label] += outputs[significand] != answers[case['case_id']]
            if outputs[significand] != answers[case['case_id']]:
                residuals.setdefault(label, []).append(case['case_id'])
    ranking = sorted(counts.items(), key=lambda item:item[1])
    report = {'research_only':True, 'rows':rows, 'uniform_models':len(models),
              'model_differences':dict(ranking), 'best_residual_ids':{name:residuals.get(name,[]) for name,_ in ranking[:20]},
              'no_runtime_changes':True}
    destination = 'rounding-boundaries/directed-initial-stage-models.json' if directed else 'numeric-to-text/directed-stage-precision-models.json'
    Path('docs/function-lane/evidence/w111-broad-20260929', destination).write_text(json.dumps(report,indent=2),encoding='utf-8')
    print(rows, ranking[:20])


if __name__ == '__main__':
    main()
