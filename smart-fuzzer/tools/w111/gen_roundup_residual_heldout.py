"""Independent signed residual validation after the uniform upper16 hypothesis."""
import hashlib
import json
import math
import random
import shutil
import struct
from pathlib import Path


def bits(number):
    return '0x' + struct.pack('>d', number).hex()


def decode(raw):
    return struct.unpack('>d', raw.to_bytes(8, 'big'))[0]


def main():
    seed = 202609293035
    rng = random.Random(seed)
    run = Path('smart-fuzzer/runs/w111-roundup-residual-heldout-20260929')
    if (run / 'candidate-freeze.json').exists():
        raise SystemExit('An immutable freeze exists; use a new run path for another candidate.')
    (run / 'batches').mkdir(parents=True, exist_ok=True)
    rows, seen = [], set()
    def add(number, count, cohort):
        if not math.isfinite(number) or abs(number) < 2**-1022:
            return
        pair = bits(number), bits(float(count))
        if pair not in seen:
            seen.add(pair)
            rows.append((pair, cohort))
    for _ in range(3072):
        number = decode((rng.randrange(1, 2047) << 52) | rng.getrandbits(52) | (rng.getrandbits(1) << 63))
        count = rng.choice([rng.randrange(-400, 401) + rng.choice([0., .125, -.75]),
                            rng.choice([-1, 1]) * rng.randrange(32760, 131080),
                            rng.choice([-1, 1]) * 2**rng.randrange(31, 900)])
        add(number, count, 'fresh_full_bit_normals')
    cutoff = math.ldexp(1., -1026)
    for _ in range(512):
        scale = rng.randrange(-307, -297)
        significand = rng.randrange(1, 100)
        base = float(f'{significand}e{scale}')
        factor = rng.choice([rng.uniform(.25, 4.), 1., math.nextafter(1., 0.), math.nextafter(1., math.inf)])
        center = base + cutoff * factor
        for number in [math.nextafter(center, 0.), center, math.nextafter(center, math.inf)]:
            count = rng.randrange(-scale, 308)
            for sign in [-1., 1.]:
                add(sign * number, count, 'fresh_signed_residual_neighborhoods')
    # Coefficient/scale-equivalent exact decimal multiples check that a zero
    # decimal remainder remains zero even when staged assembly can differ.
    for _ in range(256):
        significand = rng.randrange(1, 10**8)
        scale = rng.randrange(-307, 301)
        number = float(f'{significand}e{scale}')
        for sign in [-1., 1.]:
            add(sign * number, -scale, 'exact_decimal_multiple_controls')
    for function in ['ROUNDUP', 'ROUNDDOWN']:
        selected = rows if function == 'ROUNDUP' else [r for r in rows if r[1] != 'fresh_full_bit_normals']
        probes = [{'probe': {'id': f'w111roundresidualheldout-{function}-{i:05d}', 'args': list(args)}}
                  for i, (args, _) in enumerate(selected)]
        (run / 'batches' / f'batch-{function.lower()}.json').write_text(json.dumps({'function': function, 'probes': probes}, separators=(',', ':')), encoding='utf-8')
    sources = ['crates/oxfunc_core/src/functions/' + p for p in ['round_fn.rs', 'roundup_fn.rs', 'rounddown_fn.rs', 'trunc_fn.rs']]
    sources += ['crates/oxfunc_core/src/coercion.rs', 'crates/oxfunc_core/src/coercion_decimal.rs', 'formal/lean/OxFunc/DecimalRounding.lean']
    hashes = {}
    for source in sources:
        path = Path(source)
        destination = run / 'candidate' / path
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, destination)
        hashes[source] = hashlib.sha256(path.read_bytes()).hexdigest()
    design = {'phase': 'independent_after_signed_residual_refinement', 'seed': seed,
              'roundup_rows': len(rows), 'rounddown_rows': sum(c != 'fresh_full_bit_normals' for _, c in rows),
              'source_sha256': hashes,
              'open_lanes': ['raw_overflow_numeric_publication', 'initial15_precision', 'contextual_numeric_text', 'receiving_acknowledgement']}
    (run / 'candidate-freeze.json').write_text(json.dumps(design, indent=2), encoding='utf-8')
    print(design['roundup_rows'], 'ROUNDUP;', design['rounddown_rows'], 'ROUNDDOWN')


if __name__ == '__main__':
    main()
