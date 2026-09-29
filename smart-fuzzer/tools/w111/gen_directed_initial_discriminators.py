"""Fresh model-discriminating inputs plus local neighborhoods of retained misses.

Discovery only: inputs are selected from disagreement between two uniform
arithmetic graphs, without consulting any new oracle output.
"""
import decimal
import hashlib
import json
import math
import random
import struct
from collections import Counter
from fractions import Fraction as F
from pathlib import Path

from analyze_decimal_parser_scaling import nibble64
from model_rounding_boundaries import pair


def bits(number):
    return '0x' + struct.pack('>d', float(number)).hex()


def decode(raw):
    return struct.unpack('>d', bytes.fromhex(raw[2:]))[0]


def main():
    seed = 202609293036
    rng = random.Random(seed)
    run = Path('smart-fuzzer/runs/w111-directed-initial-discriminators-20260929')
    if (run / 'design.json').exists():
        raise SystemExit('Immutable discovery packet exists; choose a new path.')
    evidence = Path('docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries')
    old = json.loads((evidence / 'directed-initial-precision-rounddown.json').read_text())['witnesses']
    seen = {r['args'][0] for r in old}
    centers = []
    trials = 0
    while len(centers) < 96 and trials < 400000:
        trials += 1
        significand = rng.randrange(10**14, 10**15)
        scale = rng.randrange(-310, 290)
        number = float(F(2 * significand + 1, 2) * F(10)**scale)
        if not math.isfinite(number) or number < 2**-1022 or bits(number) in seen:
            continue
        exponent = decimal.Decimal.from_float(number).adjusted()
        k = 14 - exponent
        value = nibble64(F(number), k)
        quotient, remainder = divmod(value.numerator, value.denominator)
        alternative = quotient + (2 * remainder >= value.denominator)
        current, current_scale = pair(number, True)
        if current == alternative:
            continue
        assert current_scale == -k
        seen.add(bits(number))
        centers.append({'bits': bits(number), 'cohort': 'fresh_model_disagreement',
                        'current_significand': current, 'nibble_significand': alternative,
                        'scale': -k, 'scaled_fraction': str(value - quotient)})
    # This second cohort is explicitly reused discovery, never an independent
    # validation claim. Include both sides of each one-ULP transition.
    for row in old:
        number = decode(row['args'][0])
        if number <= 0:
            continue
        k = 14 - decimal.Decimal.from_float(number).adjusted()
        value = nibble64(F(number), k)
        q, r = divmod(value.numerator, value.denominator)
        alternative = q + (2 * r >= value.denominator)
        if bits(float(nibble64(alternative, -k))) == row['expected_bits']:
            continue
        if any(c['bits'] == bits(number) for c in centers):
            continue
        centers.append({'bits': bits(number), 'cohort': 'retained_nibble_residual',
                        'current_significand': pair(number, True)[0],
                        'nibble_significand': alternative, 'scale': -k,
                        'scaled_fraction': str(value - q)})
    rows, seen_rows = [], set()
    for center in centers:
        raw = int(center['bits'], 16)
        for offset in [-2, -1, 0, 1, 2]:
            number = decode(f'0x{raw + offset:016x}')
            k = 14 - decimal.Decimal.from_float(number).adjusted()
            for sign in [-1, 1]:
                for count in [k, k + 1]:
                    args = (bits(sign * number), bits(float(count)))
                    if args not in seen_rows:
                        seen_rows.add(args)
                        rows.append((args, center['cohort'], offset))
    (run / 'batches').mkdir(parents=True, exist_ok=True)
    for function in ['ROUND', 'ROUNDDOWN', 'ROUNDUP', 'TRUNC']:
        probes = [{'probe': {'id': f'w111initialdiscriminator-{function}-{index:05d}', 'args': list(args)}}
                  for index, (args, _, _) in enumerate(rows)]
        (run / 'batches' / f'batch-{function.lower()}.json').write_text(
            json.dumps({'function': function, 'probes': probes}, separators=(',', ':')), encoding='utf-8')
    sources = ['crates/oxfunc_core/src/functions/round_fn.rs', 'crates/oxfunc_core/src/coercion_decimal.rs']
    design = {'phase': 'initial15_model_discovery', 'seed': seed, 'trials': trials,
              'rows_per_function': len(rows), 'functions': ['ROUND', 'ROUNDDOWN', 'ROUNDUP', 'TRUNC'],
              'cohorts': dict(Counter(cohort for _, cohort, _ in rows)), 'centers': centers,
              'production_unchanged': True,
              'source_sha256': {p: hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in sources},
              'no_new_oracle_output_used_for_selection': True,
              'purpose': 'Distinguish decimal31 initial normalization from staged RN64 nibble scaling; map two adjacent ULPs on both sides, both signs, and two counts without assuming ROUND shares the directed primitive.'}
    (run / 'design.json').write_text(json.dumps(design, indent=2), encoding='utf-8')
    print(trials, 'trials;', len(centers), 'centers;', len(rows), 'per function;', design['cohorts'])


if __name__ == '__main__':
    main()
