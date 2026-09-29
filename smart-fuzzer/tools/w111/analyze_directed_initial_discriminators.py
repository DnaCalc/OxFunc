"""Judge the frozen initial15 discriminator packet against uniform hypotheses."""
import decimal
import hashlib
import json
import shutil
from pathlib import Path

import analyze_rounding_endpoints as endpoints
from analyze_decimal_parser_scaling import F, nibble64


def alternative_pair(number, away):
    k = 14 - decimal.Decimal.from_float(abs(number)).adjusted()
    value = nibble64(F(abs(number)), k)
    quotient, remainder = divmod(value.numerator, value.denominator)
    return quotient + (2 * remainder >= value.denominator if away else 2 * remainder > value.denominator), -k


def main():
    run = Path('smart-fuzzer/runs/w111-directed-initial-discriminators-20260929')
    evidence = Path('docs/function-lane/evidence/w111-broad-20260929/rounding-boundaries')
    original_pair = endpoints.pair
    report = {'research_only': True, 'production_unchanged': True, 'functions': {}}
    for function in ['ROUND', 'ROUNDDOWN', 'ROUNDUP', 'TRUNC']:
        source = run / f'answers-{function.lower()}.json'
        data = json.loads(source.read_text(encoding='utf-8-sig'))
        shutil.copyfile(source, evidence / f'initial-discriminators-{function.lower()}.json')
        models = {}
        for name, candidate in [('decimal31_current', original_pair), ('nibble64_initial', alternative_pair)]:
            endpoints.pair = candidate
            misses = []
            for row in data['witnesses']:
                actual = endpoints.predict(function, row['args'], difference_policy='top16')
                if actual != row['expected_bits']:
                    misses.append(dict(row, actual=actual))
            models[name] = {'rows': len(data['witnesses']), 'mismatches': len(misses), 'misses': misses}
        report['functions'][function] = {'models': models, 'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest()}
    endpoints.pair = original_pair
    (evidence / 'initial-discriminator-model-judgement.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    print({function: {name: model['mismatches'] for name, model in item['models'].items()}
           for function, item in report['functions'].items()})


if __name__ == '__main__':
    main()
