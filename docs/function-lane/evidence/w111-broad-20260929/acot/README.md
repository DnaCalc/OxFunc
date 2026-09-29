# ACOT reciprocal and angle graph

Status: `scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: broader coercion/context and host
integration, and alternate arithmetic platforms.

The retained extended numeric discovery has 1,224 observations. The previous
platform atan2 kernel differs on 206 rows. Output subnormal flushing resolves
one of those; direct platform atan2 or direct public FPATAN2 still differs on
205. General reciprocal-and-angle graphs distinguish the arithmetic stages:

| Graph | Exact rows |
| --- | ---: |
| Binary64 reciprocal, platform atan, binary64 PI adjustment | 1,199 / 1,224 |
| Binary64 reciprocal, FPATAN, binary64 PI adjustment | 1,222 / 1,224 |
| Binary64 publication after RN64 reciprocal, FPATAN, binary64 PI adjustment | 1,224 / 1,224 |
| Binary64 PI/2 minus FPATAN(input) | 747 / 1,224 |

The selected candidate returns PI/2 for zero. Otherwise it computes
`reciprocal = RN53(RN64(1/x))`, then FPATAN of the published reciprocal, then
adds binary64 PI only for negative inputs. Subnormal results publish +0.
`discovery-graph-comparison.jsonl` preserves every alternative's differences.
This is a black-box arithmetic model using public ISA operations, without Excel
binary inspection or a claim of shared binary identity. In particular, separate
ATAN work already has rare residuals against the FPATAN primitive; its universal
parity is not asserted here.

The candidate sources are frozen in `candidate-freeze.json` before an independent
7,946-case packet. It combines 7,500 fresh seeded draws with signed neighbors of
24 mathematically selected reciprocal double-rounding centers and explicit
boundaries. Inverse-neighbors of an earlier ATAN residual are tagged as diagnostic
controls, not independent random draws. That capture matches 7,942/7,946. Every
one of its 7,924 fresh random/reciprocal/boundary rows matches; four of the 22
prior-ATAN diagnostic controls differ by one ULP. Their inputs are both signs of
`0x3fdfe545a90d332b` and `0x3fdfe545a90d332c`, which lead to the previously observed
arctangent discrepancy. All four remain in `heldout-candidate-judge.json`; no
input-specific adjustment has been applied. The original frozen arithmetic graph
and source remain unchanged.

Subsequent shared ATAN research distinguishes a general reciprocal reduction
for arctangent arguments with magnitude above one: subtract the retained extended
FPATAN reciprocal angle from extended PI/2, then publish binary64 and restore
sign. Switching ACOT's arctangent binding to that helper resolves all four
residuals and preserves the other observations: 9,170/9,170 exact. The original
four failures remain immutable in their candidate report. New source hashes
are in `reduced-candidate-freeze.json`, before a fresh 7,976-case capture using
seed 2026092919. That refined independent capture matches 7,976/7,976 without source changes.
The retained ACOT numeric total is now 17,146/17,146 exact.

`w111_acot_live_replay.rs` passes all 1,224 discovery rows through public dispatch
with exact numeric bits/error tags, plus all 7,946 first-heldout rows under the
refined arctangent binding, plus all 7,976 fresh second-heldout rows (3 tests).
The executable Lean `acotWithKernels` graph
exposes reciprocal and arctangent binding parameters, zero/quadrant/underflow
examples and the existing coercion adapter. Its focused build passes (7 jobs).
The x86-64 helper binding follows the existing saved/restored control-word
convention; the non-x86 fallback remains a separate platform qualification.

Reproduction tools: `gen_acot_holdout.py` and the `acot_probe` binary in
`smart-fuzzer/tools/w111/complex_kernel_probe`. No metadata or evaluator API
declaration changes are made by this numeric kernel candidate.
