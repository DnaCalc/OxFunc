"""Build a reproducible research-only variant of the retained public GRATIO port.

Only the conservative y>=700 early-exit test is disabled. No production source
is changed, and no claim about the remaining numerical graph is made.
"""
from pathlib import Path
import hashlib
import json

root = Path(__file__).resolve().parents[3]
source = root / 'crates/oxfunc_core/src/functions/special_math_common.rs'
text = source.read_text()
text = text.split('// BRATIO:')[0].rsplit('// ---------------------------------------------------------------------------', 1)[0]
old = 'if z >= 700.0 / a {'
assert text.count(old) == 1
text = text.replace(old, 'if false && z >= 700.0 / a {')
out = root / 'docs/function-lane/evidence/w111-broad-20260929/distribution-surface/poisson-publication/research'
out.mkdir(exist_ok=True)
(out / 'no-cutoff-gratio.rs').write_text(text)
(out / 'variant-provenance.json').write_text(json.dumps({
    'source': str(source.relative_to(root)),
    'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
    'mutation': 'disable only the inherited conservative z>=700/a early exit',
    'authority': 'research-only public GRATIO port variant; no oracle-fitted constants',
}, indent=2))
