"""Independent distinguishing search after the binary-scaling candidate was frozen.

Seed 2026092905, unrelated to discovery seeds. Only exact arithmetic models
select inputs; no Excel outcomes are consulted. Includes both signs, both
component positions, machine neighbors and exact maximum-magnitude controls.
"""
from fractions import Fraction as F
import hashlib
import json
from pathlib import Path
import random
import sys

from complex_binary_scaling_candidate import decimal_pair
from gen_complex_midpoint_search_20260929 import (
    exact_power, extended_power, half_away, number, round_binary,
)

SEED = 2026092905


def hypotheses(x):
    k = 14 - int(format(x, '.30e').split('e')[1])
    value = F(x)
    return dict(scale=-k, candidate=decimal_pair(x)[0],
                exact=half_away(value*exact_power(k)),
                multiply_reciprocal=half_away(round_binary(value*extended_power(k), 64)),
                divide_reciprocal=half_away(round_binary(value/extended_power(-k), 64)),
                decimal19=half_away(F(format(x, '.18e'))*exact_power(k)))


def main(out):
    rng = random.Random(SEED)
    centers = []
    draws = 600000
    for _ in range(draws):
        bits = (rng.randint(1, 2046) << 52) | rng.getrandbits(52)
        h = hypotheses(number(bits))
        if len(set(v for k, v in h.items() if k != 'scale')) > 1:
            centers.append(bits)
    # Independently selected maximum and minimum magnitude controls, plus exact boundaries.
    centers += [0x7fefffffffffffff, 0x0010000000000000]
    centers += [0x7fefffffffffffff-rng.randint(3, 1000000) for _ in range(8)]
    centers += [0x0010000000000000+rng.randint(3, 1000000) for _ in range(8)]
    selected = {}
    for center in centers:
        for offset in range(-2, 3):
            bits = center + offset
            if not 0x0010000000000000 <= bits <= 0x7fefffffffffffff:
                continue
            h = hypotheses(number(bits))
            for sign in [0, 1 << 63]:
                value = f'0x{bits | sign:016x}'
                for args in [(value,'0x0000000000000000'),('0x0000000000000000',value)]:
                    selected.setdefault(args,h)
    probes, predictions = [], []
    for i, (args, h) in enumerate(selected.items()):
        id = f'complex-scaling-heldout-{SEED}-{i:04d}'
        probes.append({'probe':{'id':id,'args':list(args)}})
        predictions.append(dict(id=id,args=list(args),**h))
    batches = out/'batches'; batches.mkdir(parents=True,exist_ok=True)
    path = batches/'batch-complex.json'
    path.write_text(json.dumps({'function':'COMPLEX','probes':probes},indent=2),encoding='utf-8')
    (batches/'manifest.json').write_text(json.dumps(dict(generator=Path(__file__).name,seed=SEED,
        draws=draws,selected_centers=len(centers),batches=[dict(function='COMPLEX',rows=len(probes),
        path=path.name,sha256=hashlib.sha256(path.read_bytes()).hexdigest())]),indent=2),encoding='utf-8')
    (out/'hypotheses.json').write_text(json.dumps(dict(authority='non_semantic_research_hypotheses',
        seed=SEED,draws=draws,selected_centers=len(centers),predictions=predictions),indent=2),encoding='utf-8')
    print(f'{draws} draws, {len(centers)} centers, {len(probes)} probes -> {out}',flush=True)


if __name__ == '__main__':
    main(Path(sys.argv[1]))
