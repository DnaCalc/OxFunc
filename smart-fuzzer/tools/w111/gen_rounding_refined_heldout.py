"""Fresh independent count, carry, normal-bit and finite endpoint validation.

No expected answers are generated and no oracle outcomes select inputs. Source
snapshots are retained before root dispatches the separately serialized oracle.
"""
import hashlib
import json
import math
import random
import shutil
import struct
from collections import Counter
from pathlib import Path

SEED = 202609293033
RUN = Path('smart-fuzzer/runs/w111-rounding-refined-heldout-20260929')
SOURCES = [
    'crates/oxfunc_core/src/functions/round_fn.rs',
    'crates/oxfunc_core/src/functions/rounddown_fn.rs',
    'crates/oxfunc_core/src/functions/roundup_fn.rs',
    'crates/oxfunc_core/src/functions/trunc_fn.rs',
    'crates/oxfunc_core/src/coercion.rs',
    'crates/oxfunc_core/src/coercion_decimal.rs',
    'crates/oxfunc_core/src/coercion_decimal_powers.rs',
    'formal/lean/OxFunc/DecimalRounding.lean',
]


def bits(number):
    return '0x' + struct.pack('>d', number).hex()


def value(raw):
    return struct.unpack('>d', raw.to_bytes(8, 'big'))[0]


def main():
    if (RUN / 'candidate-freeze.json').exists():
        raise SystemExit('An immutable freeze already exists; use a new run path for any new candidate.')
    (RUN / 'batches').mkdir(parents=True, exist_ok=True)
    rng = random.Random(SEED)
    rows = []
    seen = set()

    def add(number, count, cohort):
        if not math.isfinite(number) or abs(number) < 2**-1022:
            return
        if not math.isfinite(count) or (count and abs(count) < 2**-1022):
            return
        key = bits(number), bits(count)
        if key not in seen:
            seen.add(key)
            rows.append((key, cohort))

    def count_value():
        mode = rng.randrange(5)
        if mode == 0:
            return rng.randrange(-400, 401) + rng.choice([0., .125, .75, -.25])
        if mode == 1:
            center = rng.choice([32767, 32768, 65535, 65536, 2**31-1, 2**31, 2**32-1, 2**32])
            return rng.choice([-1, 1]) * (center + rng.randrange(-65540, 65541) + rng.choice([0., .5]))
        if mode == 2:
            return value((rng.randrange(1, 2047) << 52) | rng.getrandbits(52) | (rng.getrandbits(1) << 63))
        if mode == 3:
            return rng.choice([-1, 1]) * math.nextafter(float(rng.choice([32768, 65536, 2**31, 2**32])), rng.choice([0., math.inf]))
        return rng.uniform(-1000., 1000.)

    # Full raw mantissas avoid the low-bit quantization of wide uniform floats.
    for _ in range(4096):
        number = value((rng.randrange(1, 2047) << 52) | rng.getrandbits(52) | (rng.getrandbits(1) << 63))
        add(number, count_value(), 'full_binary64_normals')

    # Fifteen-digit carry and midpoint neighborhoods at independently selected
    # decimal magnitudes, with several differently significant requested counts.
    for _ in range(512):
        exponent = rng.randrange(-321, 294)
        mantissa = rng.choice([999999999999995, 100000000000005, rng.randrange(10**14, 10**15)])
        center = float(f'{mantissa}e{exponent}')
        if not math.isfinite(center) or center < 2**-1022:
            continue
        number = math.nextafter(center, rng.choice([0., math.inf]))
        for count in [float(-exponent + rng.randrange(-17, 4)), count_value()]:
            add(rng.choice([-1, 1]) * number, count, 'decimal_carry_and_midpoint_neighbors')

    # New requested counts distinguish the observed initial lower-endpoint early
    # return from ordinary tiny-result flushing; farther random neighbors sample
    # endpoint neighborhoods beyond the earlier contiguous discovery controls.
    for offset in list(range(16)) + [rng.randrange(257, 2**20) for _ in range(128)]:
        for endpoint in [0x0010000000000000 + offset, 0x7fefffffffffffff - offset]:
            for _ in range(4):
                number = value(endpoint | (rng.getrandbits(1) << 63))
                add(number, count_value(), 'normal_endpoint_neighbors')

    # Width boundaries where signed addition to decimal-point position wraps,
    # with fresh values spanning decimal width and both signs.
    for _ in range(512):
        number = value((rng.randrange(1, 2047) << 52) | rng.getrandbits(52) | (rng.getrandbits(1) << 63))
        count = float(rng.choice([-1, 1]) * (2**31 + rng.randrange(-400, 401)))
        add(number, count, 'signed_count_plus_decimal_point')

    for function in ['TRUNC', 'ROUNDDOWN', 'ROUNDUP', 'ROUND']:
        probes = [{'probe': {'id': f'w111roundheldout-{function}-{i:05d}', 'args': list(args)}}
                  for i, (args, _) in enumerate(rows)]
        path = RUN / 'batches' / f'batch-{function.lower()}.json'
        path.write_text(json.dumps({'function': function, 'probes': probes}, separators=(',', ':')), encoding='utf-8')
    hashes = {}
    for filename in SOURCES:
        source = Path(filename)
        destination = RUN / 'candidate' / source
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
        hashes[filename] = hashlib.sha256(source.read_bytes()).hexdigest()
    design = {
        'seed': SEED, 'phase': 'independent_after_finite_endpoint_refinement',
        'rows_per_function': len(rows), 'cohorts': dict(Counter(c for _, c in rows)),
        'source_sha256': hashes,
        'source_admission': 'All source numbers are normal finite binary64; counts are finite normals or positive zero. Exact hexadecimal arguments require Value2 readback qualification.',
        'comparison_policy': 'Keep every raw output bit. Report finite/error comparison separately from unresolved nonfinite-encoding numeric payloads. Do not recode those payloads as errors.',
        'open_lanes_at_freeze': ['initial15_precision', 'raw_overflow_numeric_publication', 'historical_TRUNC_MAX_capture_conflict', 'contextual_numeric_text', 'evaluator_declaration_review'],
    }
    (RUN / 'candidate-freeze.json').write_text(json.dumps(design, indent=2), encoding='utf-8')
    print(len(rows), 'rows per function', design['cohorts'])


if __name__ == '__main__':
    main()
