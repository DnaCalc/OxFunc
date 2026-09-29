"""Retain all oracle outputs compactly with deterministic input regeneration.

Usage: retain_numeric_campaign.py RUN EVIDENCE
       retain_numeric_campaign.py --replay EVIDENCE OUT
The input batches are reproducible from a hashed, tracked generator and seed.
No answer is reconstructed from OxFunc or inferred from passing counts.
"""
import hashlib
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

root=Path(__file__).resolve().parents[3]
default_generator=Path(__file__).with_name('gen_w111_broad_20260929.py')
prefixes={'gen_w111_broad_20260929.py':'w111broad',
          'gen_extended_numeric_20260929.py':'w111extended'}
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()

if sys.argv[1]=='--replay':
    evidence,out=map(Path,sys.argv[2:4])
    bank=json.loads((evidence/'numeric-outcomes.json').read_text(encoding='utf-8'))
    generator=root/bank.get('generator',str(default_generator.relative_to(root)))
    if sha(generator)!=bank['generator_sha256']:
        raise SystemExit('Generator changed: restore the recorded source revision before replay')
    spec=importlib.util.spec_from_file_location('broad_generator',generator)
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    rows=module.make(bank['seed'],bank['random_rows_per_function']).batches
    out.mkdir(parents=True,exist_ok=True)
    for fn,record in bank['functions'].items():
        prefix=bank.get('id_prefix','w111broad')
        probes=[{'probe':{'id':f'{prefix}-{bank["seed"]}-{fn}-{i:05d}',
                'args':[module.bits(v) for v in row]}} for i,row in enumerate(rows[fn])]
        original=json.dumps({'function':fn,'probes':probes},separators=(',',':')).encode()
        if hashlib.sha256(original).hexdigest()!=record['batch_sha256']:
            raise SystemExit(f'Input regeneration hash mismatch: {fn}')
        assert len(probes)==len(record['expected_bits']),fn
        witnesses=[dict(p['probe'],expected_bits=b) for p,b in zip(probes,record['expected_bits'])]
        (out/('answers-'+fn.lower()+'.json')).write_text(json.dumps(dict(function=fn,
            witnesses=witnesses,capture_provenance=record['capture_provenance']),separators=(',',':')),encoding='utf-8')
    print(f'Regenerated {len(bank["functions"])} fully banked WitnessSets in {out}')
else:
    run,evidence=map(Path,sys.argv[1:3]);evidence.mkdir(parents=True,exist_ok=True)
    manifest=json.loads((run/'capture_manifest.json').read_text(encoding='utf-8'))
    partial='finished_utc' not in manifest
    if partial and '--allow-partial' not in sys.argv:
        raise SystemExit('Capture has not finished; --allow-partial banks only recorded successful captures')
    generator=Path(__file__).with_name(manifest.get('generator',default_generator.name))
    if generator.name not in prefixes: raise SystemExit('Unsupported generator format')
    if manifest.get('generator_sha256',sha(generator))!=sha(generator):
        raise SystemExit('Generator source differs from capture manifest')
    admitted={entry['function'] for entry in manifest['captures']}
    bank=dict(schema_version='w111.numeric_bank.v2',generator=str(generator.relative_to(root)).replace('\\','/'),
              generator_sha256=sha(generator),seed=manifest['seed'],
              id_prefix=prefixes[generator.name],capture_partial=partial,
              uncaptured_functions=[b['function'] for b in manifest['batches'] if b['function'] not in admitted],
              random_rows_per_function=manifest['random_rows_per_function'],
              source_commit=manifest['source_commit'],source_status=manifest['source_status'],
              comparison_policy=manifest.get('comparison_policy','exact_typed_bits_no_tolerance'),functions={})
    for batch in manifest['batches']:
        fn=batch['function'];path=run/'answers'/('answers-'+fn.lower()+'.json')
        if fn not in admitted: continue
        answer=json.loads(path.read_text(encoding='utf-8-sig'))
        expected=[w['expected_bits'] for w in answer['witnesses']]
        assert len(expected)==batch['rows'],fn
        bank['functions'][fn]=dict(batch_sha256=batch['sha256'],answer_sha256=sha(path),
            expected_bits=expected,capture_provenance=answer['capture_provenance'])
    (evidence/'numeric-outcomes.json').write_text(json.dumps(bank,separators=(',',':')),encoding='utf-8')
    print(f'Retained {len(bank["functions"])} functions; {sum(len(x["expected_bits"]) for x in bank["functions"].values())} answers')
