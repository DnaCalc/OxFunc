# CUMPRINC exact-graph lane partial report — 2026-08-09

## Status

- `scope_completeness: scope_partial`
- `target_completeness: target_partial`
- `integration_completeness: partial`
- No production implementation, formal route, or bead is changed by this
  checkpoint. Root reconciliation records the bounded evidence in the shared
  catalog, calculation map, ruled-out ledger, and W109 workset without retiring
  G6-07.
- The fresh 60-row PMT and 540-row CUMPRINC captures are discovery evidence, not heldout evidence.

The exact CUMPRINC graph is still unidentified.  The lane does, however,
separate the published PMT dependency from principal generation and range
accumulation, reject several broad graph families, and leave deterministic
generators and scorers for continuation.

## Reproduced controls and graph separation

The shipping kernel reproduces the catalog controls:

- `CUMPRINC(0.1,12,1000,1,12,0)` is exact at
  `0xc08f400000000001`.
- `CUMPRINC(0.1,12,1000,1,6,0)` is one ULP high locally:
  Excel `0xc0768ceb86d1d5a0`, shipping `0xc0768ceb86d1d5a1`.

The evidence supports treating these as distinct graph layers:

1. payment generation/publication;
2. hidden per-period principal generation and possible store boundary;
3. hidden range accumulation and final publication.

The frozen W108 decompositions already show that CUMPRINC single-period
publication differs from PPMT and from published `PMT-IPMT` by one raw bit in
the canonical case.  Fresh range decompositions likewise do not always add
back under binary64 rounding, which rules out merely summing the published
singleton values.

## Fresh deterministic discriminator

The generator uses five fresh full-mantissa loan contexts, both timings, nine
range shapes, and six exactly related PV variants (`PV`, three adjacent raw-bit
successors, `PV/2`, and `2*PV`).  It aborts on collisions against frozen and
banked CUMPRINC/PMT inputs and emits 540 CUMPRINC rows plus 60 paired PMT rows.
Two independent generations were byte-identical before capture.

The serialized capture satisfied: fresh Excel preflight count zero; Excel
16.0 build 20228 x64, Compatibility Version 2; Value2/cell-reference plumbing;
`NoCache`; per-file function/count/unique-ID/argument-bit/result-kind
assertions; unchanged batch hashes; and bounded teardown with Excel count zero.

## Oracle-blind discovery scores

These families were fixed before their answers were captured.  A paired
published PMT is an explicit input only where the family name says so.

| Candidate family | Best axes (abridged) | Exact | Max ULP | Sum ULP |
|---|---|---:|---:|---:|
| Shipping kernel | current private PMT + FV recurrence | 90/540 | 34 | 2011 |
| Published-PMT recurrence | strict balance/add-principal, continuous-x87 fold | 150/540 | 60 | 2804 |
| Published-PMT discount/geometric | x87-period, portable delta, `PvDivEmMulVMulR`, exp/log1p growth, continuous-x87 fold | 190/540 | 8 | 836 |
| PMT-free direct boundary | x87-period, exp/log1p, ratio-then-multiply | 78/540 | 785 | 23143 |
| PMT-free stable boundary | strict, portable delta, exp/log1p, multiply-then-divide | 176/540 | 5 | 553 |
| Published-PMT continuous recurrence | balance/add-principal | 157/540 | 29 | 1276 |
| Published-PMT closed FV | x87-period, exp/log1p factor, `exp-1` delta, reciprocal multiply, continuous-x87 fold | 156/540 | 32 | 1478 |
| Public `loan.fs` expression | internal-expm1/portable-log1p factor, separate `exp-1` delta, end-PMT, `pv*(factor*rate)`, reciprocal due correction, continuous-x87 fold | 172/540 | 34 | 1468 |

The public `loan.fs` race implements
`ipmt_end = -(pv*F_(per-1)*r + pmt_end*(F_(per-1)-1))`, divides interest by
`1+r` for timing 1, computes `ppmt = pmt_timing - ipmt`, and folds PPMT.  The
race includes end-PMT versus timing-PMT, ten factor providers (including
strict and stored-x87 binary exponentiation), five delta providers, three
associations, strict/stored/continuous-x87 bodies, divide versus reciprocal
timing correction, and strict versus continuous-x87 folds.  Binary-power
variants do not enter its top 30.  This is a useful negative result, not an
authority claim about Excel.

On the cached W108 combined range/singleton set, the strongest tested families
remain weak: optimized recurrence 18/62 exact (answer-fitted per-loan PMT raw
offset), discount/geometric 20/62, stable boundary 19/62, and optimized closed
FV 20/62.  Per-loan PMT offsets of only -1, 0, or +1 improve some cached rows
but do not identify the graph.

## Metamorphic and diagnostic results

Fresh discovery invariants:

- Full schedule versus `-PV`: 24/60 exact; all other cases differ by one ULP.
- Rounded prefix-plus-suffix versus full: 19/60 exact, maximum two ULP.
- Rounded prefix-early-plus-interior versus prefix-middle: 20/60 exact,
  maximum two ULP.
- `2*CUMPRINC(PV/2) == CUMPRINC(PV)`: 90/90 exact.
- `2*CUMPRINC(PV) == CUMPRINC(2*PV)`: 90/90 exact.
- Timing-1 singleton period 1 equals paired published PMT: 30/30 exact.

Using the observed timing-0 singleton period 1 as a CUMPRINC oracle anchor
removes PMT from a diagnostic recurrence.  Its best iterative model is only
176/540 exact, maximum three ULP, sum 533.  Independently optimizing all tested
low Ext80 bits of a hidden first principal for every one of 30 timing-0 loans
still yields only 135/270 exact and zero of 30 nine-range groups wholly exact.
Therefore a hidden PMT/first-principal low word alone cannot explain the
range graph.

The apparent hidden-Ext80 effective-coefficient score is intentionally kept
as an overfitting diagnostic.  It fits **90 independent coefficients**, one
for every `(loan, timing, range)` context, and uses all six PV answers in each
context.  It has no context-heldout prediction: 498/540 training rows exact,
maximum two ULP, with 66/90 groups wholly exact.  The corresponding stored-f64
fit is 389/540 with 24/90 groups wholly exact.

PV-metamer holdout audits choose each coefficient using training answers only
and do not use validation answers for either ranking or tie-breaking:

| Per-query fit (still 90 parameters) | Held-aside score | Exact groups | Max ULP | Sum ULP |
|---|---:|---:|---:|---:|
| Train `v00`; validate `v01-v05` | 329/450 | 25/90 | 2 | 130 |
| Train `v00-v01`; validate `v02-v05` | 286/360 | 43/90 | 2 | 78 |
| Train adjacent `v00-v03`; validate half/double | 162/180 | 81/90 | 1 | 18 |

The final split is weak evidence because exact power-of-two homogeneity is
already observed.  None of these coefficient fits is a candidate for
promotion: parameters are not shared across contexts or ranges.

## Durable paths and hashes

Source:

- `smart-fuzzer/tools/calc_graph_racer/src/bin/race_cumprinc_exact.rs`
  SHA256 `3C5079695B1131D9EC76421DA00C0D6C18740A1C0113BA89034777920FA09AA8`
- `smart-fuzzer/tools/calc_graph_racer/src/bin/generate_cumprinc_exact_discriminator.rs`
  SHA256 `D4E82055FE15AE0CECFC6BCD1D551BD29C705362D0A6A7D11EA365683D327369`
- `smart-fuzzer/tools/calc_graph_racer/src/bin/analyze_cumprinc_anchor_graph.rs`
  SHA256 `8730E17A538B90D3B4D0B54339E59D3AD369AD4BB24E09F13B082BC931B8BF41`

Ignored discovery artifacts under `smart-fuzzer/work/w109/G6-cumprinc/`:

- `batch-cumprinc-exact-discriminator-20260809.json`
  SHA256 `D9D333A1CF8CF8C4957BB86E7D402A30F03A5AB8A1F44F4FE8D21754FBA0A9D1`
- `batch-pmt-cumprinc-companion-20260809.json`
  SHA256 `F99B2830425A32D11BD67DE74F2F5A0D987A31028F59857724C4D20B6967399E`
- `meta-cumprinc-exact-discriminator-20260809.csv`
  SHA256 `F0B1075F95B9142FD38BCE4470FD4BEB717A13C5194B59391D5AE6F8584399C5`
- `answers-cumprinc-exact-discriminator-20260809.json`
  SHA256 `989A0ECD6330736AEDC28060BF390804D7AE0F543074B294CE735D2668E33CA0`
- `answers-pmt-cumprinc-companion-20260809.json`
  SHA256 `7A63003EA6B9320064FF1FD7ED8E33B07B08F688B6F6C3D00AF54B373878D973`

All three binaries build together with the isolated target directory under
the same ignored work root.

## Open lanes

1. Identify the hidden per-period principal publication/store graph without
   fitting one parameter per query or loan.
2. Identify the range fold/boundary graph, including whether hidden addends
   remain extended across the range.
3. Explain the adjacent-PV intermediate-rounding pattern while preserving the
   exact half/double homogeneity.
4. Freeze an oracle-blind discriminator for any new coherent low-parameter
   graph and obtain genuinely heldout validation before production changes.
5. Only after the graph is deterministic: assess production integration and
   run the required closure checklists.
