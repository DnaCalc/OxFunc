# BUG-FUNC-045: NOMINAL fractional-power publication drift

## Summary
- **Bug id**: `BUG-FUNC-045`
- **Opened**: 2026-08-09
- **Status**: `closed_signed_off`
- **Owner workset**: `W109`

## Source Refs
- **Reported against ref**: `e73115a`
- **Reproduced on ref**: `e73115a` against fresh live-Excel build-20228/CV2 probes
- **Introduced in ref**: unknown
- **Fixed in ref**: `876635e`
- **Ref notes**: The repair, exact route/staging pins, contract/formal
  alignment, and evidence record landed together in `876635e`.

## Ownership And Root Cause
- **Ownership class**: OxFunc-owned bug
- **Root cause class**: wrong_composition
- **Root cause summary**: After truncating `npery`, NOMINAL double-rounds the
  stored `1+effect` base and selects one of two legacy x87 power programs:
  periods `<=2` keep FYL2X/F2XM1/FSCALE register-continuous until the completed
  power store; periods `>=3` use the raw stored-LN/product power chain. Both
  routes store the power before evaluating `n*(power-1)`.

## Why Did We Get This Wrong?
- **Spec already correct and code was wrong?**: unknown
- **Spec vague or missing?**: yes
- **Code once correct and later regressed?**: no
- **Explanation**: Typical-value examples collapse multiple power graphs. The prior evidence did not include adjacent-double near-one ladders or the `npery=1/2` dispatch boundary.

## Reproduction
1. Generate the oracle-blind adjacent-family and follow-up batches with `generate_effect_rri_heldout` and `generate_rri_nominal_followup`.
2. Capture them with `Run-W109BulkBatch.ps1 -NoCache` against Excel 16.0 build 20228, x64, Compatibility Version 2.
3. On the 242-row first batch, native `powf` matches only `125/242`; the raw x87 pow-chain candidate matches `242/242`.
4. On the 600-row follow-up, the raw chain matches `588/600`, while
   POWER/native match `542/600`; a register-continuous direct x87 power matches
   `600/600` and localizes its advantage to truncated `npery=1/2`.
5. A two-row same-effect discriminator proves the hybrid boundary: the n=2 row
   selects direct power (`0x4001e9f49f3f60d8`, raw is two ULP away), while the
   n=3 row selects the raw stored chain (`0x3fff342fbb6b38db`, direct is three
   ULP away).
6. Eight targeted wrapper probes identify x87-double-rounded stored base,
   reject an unspilled extended base into FYL2X, and identify the final ordering
   as `n*(power-1)`.

## Similar-Risk Scan
1. `RRI` was opened separately as `BUG-FUNC-044` and is a distinct raw x87-spill graph with no POWER dispatch.
2. `EFFECT` was opened separately as `BUG-FUNC-043` and uses an integer x87-spill binary-exponentiation body.
3. `PDURATION` remains a separate adjacent-family scan; the NOMINAL result does
   not authorize a speculative shared reroute.

## Fix And Landing
1. The identified two-route production graph and exact discriminator/staging
   pins landed in `876635e`.
2. The bug register, open discrepancy catalog, calculation map, ruled-out
   ledger, W109 report/workset surfaces, worklist, and bead state were
   reconciled after landing. Catalog row `G6-14` is retired.
3. Cross-repo impact was assessed. No FEC/F3E boundary, evaluator-facing
   clause, or downstream ownership changed, so no handoff is required.

## Validation
1. Fresh provenance-bearing build-20228/CV2 `-NoCache` adjacent grid:
   production repair `242/242`; former native path `125/242`; SHA256
   `32EB2557553A505E9DB35DCE8045F6B0EC730B3A403DEE73135D14FA38C94233`.
2. Fresh follow-up: production repair `600/600`; raw-only `588/600`;
   POWER/native `542/600`; SHA256
   `39D909BE1E9396E4A75E32C0A77173A5D011445EF024E71D54996FC037261D66`.
3. Fresh same-effect route boundary: `2/2`; SHA256
   `D8E15900936A4EE3B93DCFEDEAD7536F9290E65D10F8BC4A79CAC0B0312A6A47`.
4. Targeted wrapper staging: `8/8` exact.
5. Deterministic unit test:
   `nominal_uses_direct_x87_then_raw_pow_routes_with_stored_power` passes.
6. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml`: passed;
   library `1518 passed, 0 failed, 4 ignored`, with all integration and doc-test
   executables passing.

## Evidence
1. `smart-fuzzer/work/w109/G6-solvers/batch-nominal-adjacent-20260809.json`
2. `smart-fuzzer/work/w109/G6-solvers/answers-nominal-adjacent-20260809.json`
3. `smart-fuzzer/work/w109/G6-solvers/batch-nominal-followup-20260809.json`
4. `smart-fuzzer/work/w109/G6-solvers/answers-nominal-followup-20260809.json`
5. `smart-fuzzer/work/w109/G6-solvers/batch-nominal-direct-branch-pair-scratch.json`
6. `smart-fuzzer/work/w109/G6-solvers/answers-nominal-direct-branch-pair-scratch.json`
7. `docs/function-lane/W109_EFFECT_RRI_NOMINAL_IDENTIFICATION_20260809.md`

## Closure Checklist
- [x] fix landed or non-OxFunc ownership recorded (`876635e`)
- [x] validation recorded
- [x] root cause class recorded
- [x] similar-risk scan recorded
- [x] spec/matrix/contract updated if required
- [x] cross-repo impact assessed; handoff not required because no FEC/F3E or evaluator-facing clause changed
- [x] linked reports updated

## Closure Verification (2026-08-09)

Status axes for the declared `BUG-FUNC-045` discrepancy slice:

- `execution_state: complete`
- `scope_completeness: scope_complete`
- `target_completeness: target_complete`
- `integration_completeness: integrated`
- `open_lanes: []` within this bug slice. Wider W109 financial/campaign rows and
  orthogonal alternate-version, alternate-channel, and locale validation are
  outside this bug closure; no function-phase, family, or global claim is made.

Pre-Closure Verification Checklist (`OPERATIONS.md` Section 12):

1. contract rows complete/promoted for the slice: yes; the exact NOMINAL route
   boundary and publication staging are aligned at the required
   `provisional_w109_aligned` state. The wider family contract remains
   provisional and is not claimed here.
2. Lean/formal alignment satisfied: yes; the route classifier landed and the
   `492`-job Lean build passed.
3. Rust implementation and required tests pass: yes; the full core suite passed
   with `1518` library tests, `0` failures, and `4` ignored tests, and all shown
   integration/doc-test targets passed.
4. deterministic replay artifact exists: yes; the `852/852` combined replay
   covers adjacent, follow-up, same-effect boundary, and wrapper corpora.
5. evidence links complete and reproducible: yes; hashes and generator/racer
   paths are recorded above and in the linked W109 report.
6. both version axes explicit: yes; Excel 16.0 build 20228 x64 and workbook
   Compatibility Version 2 are the declared reference profile.
7. public-doc/empirical divergence handled in favor of Excel: yes; the landed
   graph follows the reproducible black-box observations.
8. XLL seam limitation documented where material: yes; it is not material to
   this direct worksheet-oracle/core-kernel slice.
9. cross-repo impact assessed: yes; no FEC/F3E boundary or evaluator-facing
   change was made, so no handoff is required.
10. no known semantic gap remains in the declared discrepancy slice: yes.
11. completion-language audit passed: yes; closure is limited to this bug and
    does not claim wider NOMINAL function-phase or campaign closure.
12. in-progress worklist updated: yes; it records the landed repair while the
    wider W109 lane remains partial.
13. execution-state surface updated: yes; bead `oxf-jwh5.4` is closed with the
    landed ref and validation evidence.

Completion Claim Self-Audit (`OPERATIONS.md` Section 14):

1. scope re-read: pass; only the identified NOMINAL fractional-power
   publication bug is claimed.
2. gate criteria re-read: pass; implementation, exact replay, full tests,
   formal alignment, evidence, and landed ref are present.
3. silent scope reduction: pass; no declared route or staging boundary was
   dropped.
4. looks-done-but-is-not patterns: pass; no stub, compile-only path,
   unsupported proof claim, or unacknowledged handoff supports closure.
5. result: pass for the declared `BUG-FUNC-045` slice.
