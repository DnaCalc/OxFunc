# WORKSET - Function Array Support Systematic Sweep (W090)

## 1. Purpose
Own the systematic, bounded family-by-family review of array-valued
scalar-parameter behavior across supported OxFunc functions, continuing the
seed evidence from `W080` without claiming that every function can be swept in
one pass.

## 2. Why This Packet Exists
`W080` found and repaired several text-family array-spill gaps, and `W079`
showed the same class of risk in lookup selection arguments. Those findings are
enough to justify a broad review program, but the supported function invocation
space is too large for each passing case to become a prose artifact. `W090`
therefore owns compact inventory, sampling, replay, mismatch promotion, and
coverage telemetry for the broader array-support sweep.

## 3. Provenance
1. `docs/worksets/W080_FUNCTION_ARRAY_SUPPORT_REVIEW.md`
2. `docs/worksets/W079_LOOKUP_SELECTION_ARRAY_LOOKUP_VALUE_LIFTING.md`
3. `docs/bugs/streams/BUG-FUNC-007_text_slice_array_position_and_count_spill_gap.md`
4. `docs/bugs/streams/BUG-FUNC-008_text_scalar_and_delimiter_array_support_gap.md`
5. `docs/bugs/streams/BUG-FUNC-016_text_search_replace_array_support_gap.md`
6. `docs/function-lane/W66_SCENARIO_MANIFEST_SEED.csv`
7. `docs/worksets/W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md`
8. `docs/worksets/W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md`
9. `smart-fuzzer/planning/ARRAY_SUPPORT_SYSTEMATIC_SWEEP_PLAN.md`
10. `smart-fuzzer/tools/Build-ArraySupportSweepPlan.ps1`

## 4. Scope
In scope:
1. build an inventory of supported functions and arguments where metadata,
   coercion rules, source patterns, or adjacent Excel evidence suggest
   scalar-only behavior might actually lift or spill arrays,
2. classify risk by family and argument role before selecting batches,
3. define a compact replay matrix per family covering one-array arguments,
   multiple-array and broadcast contrasts, reference-vs-array inputs, blanks,
   omitted arguments, errors, optional/fallback positions, and spill shape,
4. run bounded batches through local OxFunc and Excel comparison when a batch is
   explicitly selected,
5. promote confirmed divergences into `BUG-FUNC-*` streams or narrower repair
   beads/worksets,
6. keep passing cases as aggregate coverage telemetry and representative
   anchors rather than heavy per-case prose,
7. reconcile `W051`, contracts, execution records, and bug streams when a
   confirmed issue changes current-surface truth.

Out of scope:
1. completing every supported function in a single pass,
2. broad random smart-fuzzer execution without the `W088` / `W089` artifact and
   batching discipline,
3. locale and alternate Excel-version sweeps,
4. repairing every discovered bug inside `W090` itself when a narrower bead or
   workset is the cleaner owner,
5. treating high-volume passing rows as completion evidence beyond compact
   coverage telemetry.

## 5. Initial Epic Lanes
1. supported-function and argument-role inventory
2. static risk classification from metadata, coercion declarations, and source
   implementation patterns
3. Excel replay matrix design and batch sizing
4. first post-`W080` family-batch selection
5. local-vs-Excel comparison and mismatch minimization
6. bug promotion and truth-surface reconciliation
7. compact roadmap/highlights trace for explored array-support regions

## 6. Closure Condition
`W090` may close only when:
1. a stable candidate inventory exists for the supported current-version
   function surface,
2. at least one non-text successor family tranche has been executed or
   explicitly deferred with a replacement tranche and rationale,
3. confirmed mismatches from executed tranches have been minimized and promoted
   to ordinary bug streams or narrower worksets,
4. pass-heavy exploration is summarized as aggregate coverage telemetry rather
   than per-case documentation,
5. `W051`, contract, workset, and bug-stream truth surfaces no longer overclaim
   the array-support regions actually examined,
6. remaining unswept regions have an explicit next-owner or next-tranche plan,
7. no claim is made that the full supported array-support surface has been
   reviewed unless the telemetry proves that exact scope.

## 7. Current Reading
1. execution_state: `first_cycle_closed_successor_sweep_ready`
2. scope_completeness: `scope_complete`
3. target_completeness: `target_partial`
4. integration_completeness: `integrated`
5. open_lanes:
   - successor array-support tranches beyond the first math scalar-numeric
     array-lift tranche
   - locale and alternate Excel-version sweeps, out of W090 scope
6. ready artifacts:
   - `smart-fuzzer/planning/ARRAY_SUPPORT_SYSTEMATIC_SWEEP_PLAN.md`
   - `smart-fuzzer/tools/Build-ArraySupportSweepPlan.ps1`
   - `smart-fuzzer/tools/Run-ArraySupportTranche.ps1`
   - generated local cache, reproducible but ignored by default:
     `array-support-candidate-inventory-v0.json`,
     `array-support-first-tranche-v0.json`,
     `array-support-replay-matrix-v0.json`, and
     `array-support-highlights-v0.md`

## 8. Execution Notes

### 2026-04-29 Array Inventory And First-Tranche Plan

Bead: `oxf-19lo`

Added the W090 array-support planning surface and builder:

1. `smart-fuzzer/planning/ARRAY_SUPPORT_SYSTEMATIC_SWEEP_PLAN.md`
2. `smart-fuzzer/tools/Build-ArraySupportSweepPlan.ps1`

The builder derives the array-support candidate inventory from the W089
dimension inventory, source-code scalar-coercion/lift signals, W079/W080 seeded
array fixes, blocked/deferred lanes, and known-deviation tags. It generated
local ignored cache artifacts with these counts on 2026-04-29:

1. surfaces inventoried: 534,
2. medium-or-higher candidates: 379,
3. seeded/prior reconciled surfaces: 15,
4. first-tranche surfaces: 14.

First selected tranche:
`w090-tranche-a-math-scalar-numeric-array-lift`

Selected surfaces:
`ROUND`, `ROUNDDOWN`, `ROUNDUP`, `TRUNC`, `CEILING`, `CEILING.MATH`,
`CEILING.PRECISE`, `FLOOR`, `FLOOR.MATH`, `FLOOR.PRECISE`, `ISO.CEILING`,
`ATAN2`, `BASE`, and `MROUND`.

This bead did not run local-vs-Excel comparison and did not claim any function
array-support lane is complete. Passing rows from later execution remain
coverage telemetry only; unexpected mismatches must be promoted through
ordinary bug streams or narrower repair beads.

Status axes after this bead:

1. `execution_state`: `inventory_and_first_tranche_plan_ready`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: first-tranche local-vs-Excel execution, mismatch promotion,
   and later-tranche selection

### 2026-04-30 First-Tranche Execution, Repair, And Closure Gate

Bead: `oxf-k7ux`

Added the executable W090 tranche runner and local evaluator:

1. `smart-fuzzer/tools/Run-ArraySupportTranche.ps1`
2. `smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/array_tranche_local_eval.rs`

The runner captures full Excel spill ranges, serializes local and Excel
outcomes into typed scalar/array digests, compares numeric cells bit-exactly,
and writes full packets only for non-pass classifications.

Initial run:

1. run id: `w090-array-tranche-a-local-010`
2. Excel: `16.0` build `19929`
3. workbook Compatibility Version: `2`
4. cases: `34`
5. exact typed bit matches: `2`
6. unexpected mismatches: `32`
7. mismatch class: local `#VALUE!` for scalar-coercion sites where Excel
   spilled arrays
8. matched function: `MROUND`
9. mismatching functions: `ROUND`, `ROUNDDOWN`, `ROUNDUP`, `TRUNC`,
   `CEILING`, `CEILING.MATH`, `CEILING.PRECISE`, `FLOOR`, `FLOOR.MATH`,
   `FLOOR.PRECISE`, `ISO.CEILING`, `ATAN2`, and `BASE`

Promotion and repair:

1. promoted as `BUG-FUNC-017`
2. repair bead: `oxf-k7ux`
3. fixed ref: `0b966d0ee7c8ce4a327b0b3090f9a108248c37fd`
4. repair surface:
   - generic prepared-argument broadcast helper,
   - binary numeric broadcast for `ROUND`, `ROUNDDOWN`, `ROUNDUP`, and
     `ATAN2`,
   - optional-argument array lift for `TRUNC`,
   - exact/optional two- and three-argument array lift for the ceiling/floor
     family,
   - text-returning array lift for `BASE`

Validation:

1. `cargo check --manifest-path crates/oxfunc_core/Cargo.toml`
2. `cargo test --manifest-path crates/oxfunc_core/Cargo.toml --lib -- --nocapture`
   - `1249` passed, `0` failed, `1` ignored
3. rerun id: `w090-array-tranche-a-local-011`
4. rerun result: `34/34` exact typed bit matches, `0` mismatches, `0`
   failure packets

Successor plan for remaining unswept regions:

1. no full supported array-support surface claim is made,
2. future work should regenerate the W090 candidate inventory,
3. future tranches should exclude W079/W080 and BUG-FUNC-017 landed lanes
   unless a regression is suspected,
4. the next tranche should select the highest-risk remaining non-text
   scalar-coercion category spread and include reference-vs-inline-array plus
   blank/error contrast rows,
5. pass rows remain compact telemetry only.

Pre-Closure Verification Checklist from `OPERATIONS.md` Section 12:

| # | Check | Yes/No |
|---|-------|--------|
| 1 | Function contract rows complete and promoted for all in-scope functions? | yes; no new contract promotion required beyond `BUG-FUNC-017` and W51/W090 records |
| 2 | Lean obligations for each slice class satisfied or explicitly aligned per formalization strategy? | yes; no new Lean substrate obligation introduced by this value-surface lift helper |
| 3 | Rust implementation and required tests pass for all in-scope functions? | yes |
| 4 | At least one deterministic replay artifact exists per in-scope function behavior? | yes; W090 runs `010` and `011` |
| 5 | Evidence links complete and reproducible? | yes |
| 6 | Version scope explicit on both axes? | yes; Excel `16.0` build `19929`, workbook Compatibility Version `2` |
| 7 | Public-doc vs empirical discrepancies recorded and resolved in favor of empirical Excel behavior? | yes; empirical spill behavior is recorded in `BUG-FUNC-017` |
| 8 | XLL verification-seam limitations documented where material? | yes; not material to this COM/value-surface replay |
| 9 | Cross-repo impact assessed and handoff filed if affected? | yes; no FEC/F3E or evaluator-facing seam change |
| 10 | No known semantic gap remains in declared scope? | yes; tranche A rerun is exact over the declared W090 first-cycle replay matrix |
| 11 | Completion language audit passed? | yes |
| 12 | `docs/IN_PROGRESS_FEATURE_WORKLIST.md` updated? | yes |
| 13 | execution-state blocker surface updated? | yes; bead `oxf-k7ux` records the repair lane |

Completion Claim Self-Audit from `OPERATIONS.md` Section 14:

1. Scope re-read: pass; W090 first-cycle scope was inventory, one non-text
   tranche, compact replay, mismatch promotion/repair, and successor plan.
2. Gate criteria re-read: pass; each W090 closure condition is satisfied for
   the declared first-cycle scope.
3. Silent scope reduction check: pass; the full supported array-support surface
   remains explicitly target-partial and is not claimed reviewed.
4. "Looks done but is not" pattern check: pass; run artifacts and Rust tests
   exercise the implemented paths, and no handoff is pending.
5. Result included: pass.

Status axes after this gate:

1. `scope_completeness`: `scope_complete`
2. `target_completeness`: `target_partial`
3. `integration_completeness`: `integrated`
4. `open_lanes`: successor unswept array-support tranches outside the closed
   W090 first-cycle scope
