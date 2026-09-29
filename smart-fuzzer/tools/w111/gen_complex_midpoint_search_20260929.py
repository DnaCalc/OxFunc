"""Search general decimal conversion hypotheses for distinguishing normal inputs.

Arithmetic is a portable exact-rational simulation. RN64 means round to a
64-bit binary significand, nearest-even; it is not a claim about Excel internals.
Candidate: RN64(input * RN64(10**k)), followed by integer half-away rounding.
Competing candidates are exact decimal rounding and a 19-digit decimal pre-round.
Only disagreements and their adjacent binary64 controls consume oracle probes.
"""
from fractions import Fraction as F
from functools import lru_cache
import json
from pathlib import Path
import random
import struct
import sys

SEED = 2026092903

def round_binary(x, precision):
    exponent = x.numerator.bit_length() - x.denominator.bit_length()
    if x < F(2)**exponent:
        exponent -= 1
    shift = precision - 1 - exponent
    y = x * F(2)**shift
    q, r = divmod(y.numerator, y.denominator)
    if 2*r > y.denominator or (2*r == y.denominator and q % 2):
        q += 1
    return F(q) * F(2)**(-shift)

@lru_cache(None)
def exact_power(k):
    return F(10)**k

@lru_cache(None)
def extended_power(k):
    return round_binary(exact_power(k), 64)

def half_away(x):
    q, r = divmod(x.numerator, x.denominator)
    return q + (2*r >= x.denominator)

def hypotheses(x):
    k = 14 - int(format(x, '.30e').split('e')[1])
    exact = half_away(F(x)*exact_power(k))
    binary64_significand = half_away(round_binary(F(x)*extended_power(k), 64))
    decimal19 = half_away(F(format(x, '.18e'))*exact_power(k))
    return k, exact, binary64_significand, decimal19

def number(bits):
    return struct.unpack('<d', struct.pack('<Q', bits))[0]

def main(out):
    rng = random.Random(SEED)
    selected = []
    draws = 0
    while len(selected) < 100 and draws < 300000:
        draws += 1
        bits = (rng.randint(1, 2046) << 52) | rng.getrandbits(52)
        x = number(bits)
        h = hypotheses(x)
        if len(set(h[1:])) > 1:
            selected.append((bits, h))
    # Original heldout witness plus unrelated machine-neighbor controls.
    selected.append((0x38cf805c3cd702b2, hypotheses(number(0x38cf805c3cd702b2))))
    unique = {}
    for bits, _ in selected:
        for offset in range(-2, 3):
            positive = bits + offset
            if not 0x0010000000000000 <= positive < 0x7fefffffffffffff:
                continue
            h = hypotheses(number(positive))
            for sign in [0, 1 << 63]:
                encoded = f'0x{positive | sign:016x}'
                for args in [(encoded, '0x0000000000000000'), ('0x0000000000000000', encoded)]:
                    unique.setdefault(args, h)
    probes, predictions = [], []
    for i, (args, h) in enumerate(unique.items()):
        id = f'complex-midpoint-{SEED}-{i:04d}'
        probes.append({'probe': {'id': id, 'args': list(args)}})
        predictions.append({'id':id,'args':list(args),'scale':-h[0],
                            'exact_significand':h[1],'binary_rn64_significand':h[2],
                            'decimal19_significand':h[3]})
    out.mkdir(parents=True, exist_ok=True)
    (out/'batch-complex.json').write_text(json.dumps({'function':'COMPLEX','probes':probes},indent=2),encoding='utf-8')
    (out/'hypotheses.json').write_text(json.dumps({'authority':'non_semantic_research_hypotheses',
        'generator':Path(__file__).name,'seed':SEED,'draws':draws,'selected_centers':len(selected),
        'predictions':predictions},indent=2),encoding='utf-8')
    print(f'{draws} random draws, {len(selected)} centers, {len(probes)} probes -> {out}')

if __name__ == '__main__':
    main(Path(sys.argv[1]))
