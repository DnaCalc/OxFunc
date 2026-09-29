"""Transfer retained midpoint discriminators to directed initial normalization.

These inputs are discovery controls reused from a different function lane, not
an independent validation claim. The normal source bits are preserved exactly.
"""
import hashlib
import json
import struct
from pathlib import Path


def bits(number):
    return '0x' + struct.pack('>d', float(number)).hex()


def main():
    source = Path('smart-fuzzer/runs/w111-numeric-text-rendering-precision-typed-20260929/cases/cases.jsonl')
    cases = [json.loads(line) for line in source.read_text(encoding='utf-8-sig').splitlines()]
    rows = set()
    for case in cases:
        if case['canonical_surface_name'] == 'ROUND':
            number, count = [arg['value'] for arg in case['args']]
            for extra in [0, 1]:
                rows.add((bits(number), bits(count + extra)))
    ordered = sorted(rows)
    run = Path('smart-fuzzer/runs/w111-directed-initial-precision-20260929')
    (run / 'batches').mkdir(parents=True, exist_ok=True)
    for function in ['ROUNDDOWN', 'ROUNDUP', 'TRUNC']:
        probes = [{'probe': {'id': f'w111directedprecision-{function}-{index:05d}', 'args': list(args)}}
                  for index, args in enumerate(ordered)]
        (run / 'batches' / f'batch-{function.lower()}.json').write_text(json.dumps({'function': function, 'probes': probes}, separators=(',', ':')), encoding='utf-8')
    (run / 'design.json').write_text(json.dumps({'phase': 'crossfunction_initial15_discovery',
        'rows_per_function': len(ordered), 'input_source': source.as_posix(),
        'input_source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
        'purpose': 'Distinguish directed initial15 ties and intermediate precision from generic text/ROUND policy on the already retained close rational midpoints. No new primitive is presumed.'}, indent=2), encoding='utf-8')
    print(len(ordered), 'per function')


if __name__ == '__main__':
    main()
