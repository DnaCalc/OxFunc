# Unexpected Mismatch Triage And Minimization Protocol

Status: `planning_artifact_ready`

Owning bead: `oxf-1avj.7`

Owning workset:
`docs/worksets/W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md`

This note defines how W089 should handle mismatches after a later execution
run. It does not run minimization.

## 1. Triage Classes

Every non-match must be classified as one of:

1. `known_expected_deviation`,
2. `unexpected_mismatch`,
3. `excel_harness_blocked`,
4. `oxfml_seam_blocked`,
5. `context_provider_blocked`,
6. `invalid_generator_case`,
7. `unstable_or_non_reproducible`.

Approximate numeric agreement is not a pass.

Calls outside published `arity_min` / `arity_max` are not valid OxFunc
smart-fuzzer comparison cases. If such a row reaches triage, classify it as
`invalid_generator_case` and, when useful, preserve it only as OxFml
admission-negative evidence. Do not promote formula-entry rejection for invalid
arity as an `excel_harness_blocked` function-semantic lane.

## 2. Known Deviations

PMT/PPMT/IPMT financial exactness drift is currently expected and blocked from
repair work. W089 may use it as a reference mismatch lane, but it should not
create repair work unless the existing blocked bug lane is explicitly reopened.

The former POWER/OP_POWER stale-claim check was freshly confirmed and closed
under W078 on 2026-04-29. Future POWER mismatches should be triaged as new
signals rather than assumed continuations of BUG-FUNC-005.

## 2.1 Encoding-Drift Pre-Check

Before any numeric mismatch is classified as a kernel/algorithm drift, the
runner that produced it must confirm that numeric inputs were passed to
Excel via cell `Range.Value2` and not via formula literal text. Excel's
formula parser is not always correctly-rounded for long decimal literals,
so a `~1e-12 * scale` magnitude difference may be entirely caused by the
parser landing on a neighbouring `f64`. See
`smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`.

A row produced under literal-text plumbing must be re-run under cell-ref
plumbing before its kernel-drift classification is taken as durable.

## 3. Minimization Order

Unexpected mismatches should be reduced in this order:

1. preserve function identity and comparison class,
2. remove irrelevant context fixtures,
3. shrink arrays by row, column, and element,
4. shrink references toward single-cell or smallest equivalent area,
5. reduce numeric values toward nearby critical bands,
6. shorten strings while preserving the mismatch,
7. reduce optional arguments toward the smallest reproducing arity,
8. rerun local and Excel comparison for the minimized candidate.

## 4. Promotion Rules

Promote only durable mismatches:

1. reproducible under captured Excel version/channel and workbook
   compatibility,
2. reproducible locally through the declared OxFunc/OxFml seam,
3. classified as function-semantic or seam/harness,
4. minimized or preserved with a clear reason why minimization would destroy
   the witness,
5. linked to `docs/bugs/` and regression follow-up.

## 5. Failure Packet Shape

Failure packets should contain:

1. `packet_id`,
2. `source_run_id`,
3. `surface_id`,
4. `canonical_surface_name`,
5. `formula_or_call`,
6. `fixture_cells`,
7. `local_outcome`,
8. `excel_outcome`,
9. `comparison_class`,
10. `minimization_lineage`,
11. `known_deviation_or_bug_refs`,
12. `promotion_decision`.

## 6. Current Status Axes

1. `execution_state`: `triage_protocol_ready`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: execution approval
