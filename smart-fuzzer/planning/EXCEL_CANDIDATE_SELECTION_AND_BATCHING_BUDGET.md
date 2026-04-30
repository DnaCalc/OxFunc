# Excel Candidate Selection And Batching Budget

Status: `planning_artifact_ready`

Owning bead: `oxf-1avj.4`

Owning workset:
`docs/worksets/W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md`

This note defines how a later W089 sweep should spend slow Excel evaluations.
It does not run Excel.

## 1. Builder

Default command:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-SweepPlanningArtifacts.ps1
```

Output:

```text
smart-fuzzer/cache/excel-candidate-budget-v0.json
```

## 2. Selection Priority

Excel candidate priority should be:

1. local outcome class not previously seen for that surface and dimension band,
2. known-bug-adjacent or source-risk rows,
3. boundary values and shape edges,
4. low-risk catalog controls sampled sparsely,
5. stale-claim checks requiring fresh confirmation,
6. PMT/PPMT/IPMT known financial drift as bounded reference mismatch controls.

The former POWER/OP_POWER stale-claim check was closed under W078 on
2026-04-29 and should not receive special quota unless a new signal appears.

Provider/cube/RTD/context rows should receive no Excel quota unless a fixture
exists. `LET`, `LAMBDA`, and callable helper formula-binding lanes likewise
receive no Excel quota in the default pure-function run unless a
formula-binding harness or concrete callable fixture exists.

## 3. Batching Policy

Default planning values:

1. formula batch size: `5000`,
2. workbook chunk target: `50000`,
3. candidate manifest first, Excel second,
4. formulas grouped by context fixture and volatility class,
5. exact typed comparison after result extraction.

The W088 benchmark showed Excel batching can be fast, but W089 should treat
that as a planning signal rather than a permanent throughput guarantee.

## 4. Comparison Classes

Excel comparison classes remain:

1. `exact_typed_bit_match`,
2. `known_expected_deviation`,
3. `unexpected_mismatch`,
4. `excel_harness_blocked`,
5. `oxfml_seam_blocked`,
6. `context_provider_blocked`,
7. `invalid_generator_case`,
8. `unstable_or_non_reproducible`.

There is no tolerance pass class.

Pseudo-random functions use aggregate statistical comparison artifacts rather
than per-draw bit-exact pass rows. A statistical profile mismatch is still a
fuzzer finding.

Statistical profile classes are:

1. `statistical_profile_consistent`,
2. `statistical_profile_mismatch`,
3. `statistical_profile_inconclusive`.

## 5. Planned Artifacts

A later Excel run should write:

1. `excel_candidates.jsonl`,
2. `excel_batches.jsonl`,
3. `excel_outcomes.jsonl`,
4. `comparisons.jsonl`,
5. `comparison_rollup.json`,
6. `failure_packets/` only for unexpected or unstable rows.

## 6. Current Status Axes

1. `execution_state`: `excel_budget_ready`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: blocked seam classification, execution approval, mismatch
   triage protocol
