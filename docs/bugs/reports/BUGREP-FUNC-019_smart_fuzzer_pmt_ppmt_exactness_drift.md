# BUGREP-FUNC-019: Smart-fuzzer PMT/PPMT exactness drift versus Excel

## Summary
- **Report id**: `BUGREP-FUNC-019`
- **Filed**: `2026-04-28`
- **Status**: `triaged`
- **Canonical bug**: `BUG-FUNC-015`

## Intake
- **Source channel**: `local smart-fuzzer pilot`
- **Reported against ref**: `d864c1bf0c1ba29e20f8858f0b5851f94352d88f`
- **Reported against kind**: `commit`
- **Report owner workset**: `W088`

## Prompt / Observation
1. The W088 PMT/PPMT pilot generated 28 bounded cases across arity,
   zero-rate, monthly-rate, small-rate, negative-rate, type, future-value,
   period, and invalid-period lanes.
2. Direct OxFunc value-surface evaluation and live Excel COM `Value2`
   evaluation both ran for all 28 cases.
3. The run found 21 numeric bit-level mismatches and 7 exact matches:
   - `PMT`: 11 mismatches, 3 matches
   - `PPMT`: 10 mismatches, 4 matches
4. The known witness rows still reproduce:
   - `PMT(0.05/12,360,200000)`:
     local `-1073.6432460242763` (`0xc090c692af15f632`),
     Excel `-1073.6432460242781` (`0xc090c692af15f63a`)
   - `PPMT(0.05/12,1,360,200000)`:
     local `-240.30991269094295` (`0xc06e09eace0506e4`),
     Excel `-240.30991269094474` (`0xc06e09eace050723`)
5. The zero-rate and invalid-period lanes in the pilot matched exactly, so the
   reopened lane is concentrated on non-zero-rate publication/exactness.

## Initial Classification
- **Ownership guess**: `OxFunc-owned bug`
- **Duplicate of existing report?**: `no`
- **Needs canonical stream?**: `yes`

## Notes
1. The full per-case failure packets are in the ignored local run directory
   `smart-fuzzer/runs/w088-pmt-ppmt-pilot/`.
2. The promoted durable summary is intentionally compact; the fuzzer should not
   turn every passing sample into heavy documentation.
3. The pilot used the public `oxfunc_core` value surface through
   `smart-fuzzer/tools/pmt_ppmt_local_eval/`, not a duplicate formula
   implementation.
