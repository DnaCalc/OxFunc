# Spark Long-Run Smart-Fuzzer Guide

Status: `run_guide_active`

Owning workset:
`docs/worksets/W092_SPARK_GUIDED_SMART_FUZZER_LONG_RUN.md`

This is the controlling run guide for W092 beads. Every W092 bead should name
this file explicitly and should follow it unless a later W092 update narrows the
scope.

## 1. Runner Profile

The intended runner is `gpt-5.3-codex-spark`.

Operational consequences:

1. use short, explicit cycles rather than one vague autonomous pass,
2. keep each command bounded and artifact-producing,
3. prefer local evidence and compact rollups over prose,
4. do not run broad expensive Excel sweeps until a bounded tranche is named,
5. continue for many cycles when useful signal is still appearing,
6. stop only on the explicit gates in this guide.

Use `/goal` when launching the runner. Recommended objective:

```text
/goal Create and execute the W092 Spark-guided long-run smart-fuzzer lane in C:\Work\DnaCalc\OxFunc. Build the controlling guide and bead rollout, then keep running feedback-guided fuzzer cycles for as long as useful signal is being produced. Stop only on catalog-axis saturation, sustained no-new-signal plateau, excessive untriaged finding pressure, all-path blockers, resource-safety failure, or direct user instruction.
```

Do not set a token budget unless the launcher explicitly supplies one. The goal
is intentionally suitable for a very long run.

## 2. Loop Skeleton

Repeat this cycle while no stop gate is met:

1. Load current run state, known deviations, active bug streams, and previous
   smart-fuzzer rollups.
2. Select a small tranche of functions and axes.
3. Generate or select cases from the highest-value unexplored or underexplored
   buckets.
4. Run cheap local OxFunc/OxFml evaluation first where available.
5. Select high-value candidates for Excel comparison.
6. Compare typed outcomes using the current smart-fuzzer comparator policy.
7. Classify each interesting row.
8. Minimize durable unexpected mismatches and unstable outcomes.
9. Promote durable findings through `docs/bugs/`, handoffs, or follow-up beads.
10. Update compact run rollups and coverage feedback.
11. Re-score the catalog and continue.

## 2.1 Initial Runnable Tranche

Audit snapshot: `2026-05-04`.

The first W092 execution cycle should use the bounded axis-witness tranche as a
harness-freshness and scheduler smoke run. It is small, already shaped for the
generic runner, covers `19` runnable invocation-space axes over `38` cases, and
exercises both local OxFunc value evaluation and Excel `Formula2` comparison.

Build or refresh the case set:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-AxisWitnessCaseSet.ps1
```

Run the first W092 cycle:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId w092-axis-witness-cycle-001 `
  -CaseSetPath smart-fuzzer\cache\axis-witness-case-set-v0.json `
  -CaseSetTrancheId w089-axis-witness-sweep-v0
```

Required post-run inspection:

```powershell
Get-Content -Raw smart-fuzzer\runs\w092-axis-witness-cycle-001\rollup.json
Get-ChildItem smart-fuzzer\runs\w092-axis-witness-cycle-001\failure_packets
```

Feedback branch:

1. If the run reports `unexpected_mismatch`, `excel_harness_blocked`,
   `local_harness_blocked`, `generator_invalid`, or `unstable` rows, minimize
   and promote before running more generation.
2. If the run reports `adapter_or_seam_mismatch`, confirm it maps to an open
   seam handoff or follow-up bead before running more generation.
3. If the run reports `known_residual`, confirm it maps to an open bug stream
   or repair bead before running more generation.
4. If the run is all exact typed matches or only already-promoted seam
   mismatches / known residuals and Excel is available, proceed to one
   bounded scenario-seed category tranche rather than a full-catalog replay.
5. Prefer low-pressure categories first while BUG-FUNC-021, BUG-FUNC-024, and
   BUG-FUNC-025 remain open, so W092 does not flood the existing statistical
   and matrix exactness lanes with duplicate known residuals.

Initial low-pressure scenario-seed tranche order:

1. `w089-scenario-seed-text-functions`
2. `w089-scenario-seed-date-and-time-functions`
3. `w089-scenario-seed-logical-functions`
4. `w089-scenario-seed-lookup-and-reference-functions`
5. `w089-scenario-seed-engineering-functions`

Example follow-up command:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId w092-scenario-text-cycle-001 `
  -CaseSetPath smart-fuzzer\cache\scenario-seed-executable-cases-v0.json `
  -CaseSetTrancheId w089-scenario-seed-text-functions
```

The W090 successor compatibility/statistical tranches remain runnable but
should be used only for targeted repair confirmation or minimization while the
open statistical exactness bead is still carrying unclosed residual pressure.

## 2.2 W092 Cycle Ledger

Current ledger snapshot: `2026-05-04`.

Executed W092 run artifacts:

1. `w092-axis-witness-cycle-001`: first axis-witness freshness run; surfaced
   the already-promoted TAKE `1x1` publication seam as `unexpected_mismatch`.
2. `w092-axis-witness-cycle-002`: same `38` cases after comparator narrowing;
   `37` exact typed matches and `1` `adapter_or_seam_mismatch`.
3. Default scenario-seed category tranches:
   - text: `35` exact typed matches,
   - date/time: `17` exact typed matches,
   - logical: `11` exact typed matches,
   - lookup/reference: `34` exact typed matches,
   - engineering: `55` exact typed matches and `1` `known_residual`
     (`BUG-FUNC-024`),
   - information: `7` exact typed matches,
   - financial: `9` exact typed matches,
   - featured: `9` exact typed matches,
   - math/trig: `49` exact typed matches, `1` `known_residual`
     (`BUG-FUNC-025`), and `2` `adapter_or_seam_mismatch` (`HO-FN-010`),
   - statistical: `34` exact typed matches and `24` `known_residual`
     (`BUG-FUNC-021`),
   - compatibility: `20` exact typed matches and `13` `known_residual`
     (`BUG-FUNC-021`).
4. `w092-w090-successor-all-cycle-001`: `102` exact typed matches and `37`
   `known_residual` rows over the W090 successor aggregate (`BUG-FUNC-021`).
5. `w092-scenario-seed-max10-v0.json` widened scenario seeds from `321` to
   `345` runnable cases. Delta tranches:
   - lookup/reference: `43` exact typed matches,
   - math/trig: `57` exact typed matches, `1` `known_residual`, and `2`
     `adapter_or_seam_mismatch`,
   - date/time: `22` exact typed matches,
   - logical: `12` exact typed matches,
   - text: `36` exact typed matches.
6. `w092-scenario-seed-max20-v0.json` widened scenario seeds from `345` to
   `366` runnable cases. Delta tranches:
   - date/time: `32` exact typed matches,
   - lookup/reference: `53` exact typed matches,
   - math/trig: `58` exact typed matches, `1` `known_residual`, and `2`
     `adapter_or_seam_mismatch`.
7. `w092-scenario-seed-max50-v0.json` widened scenario seeds from `366` to
   `375` runnable cases. Delta tranches:
   - date/time: `40` exact typed matches,
   - lookup/reference: `54` exact typed matches.
8. `w092-scenario-seed-max100-v0.json` produced `375` runnable cases again;
   this is the current scenario-seed literal-argument plateau.
9. `w092-axis-known-reference-cycle-001`: `37` exact typed matches, `1`
   `adapter_or_seam_mismatch`, and `2` `known_expected_deviation` rows for the
   PMT reference pair.
10. `w092-scenario-seed-known-deviations-v0.json` with
    `-IncludeKnownDeviations` also produced `375` runnable cases; no additional
    scenario-seed tranche was opened by including known-deviation rows.
11. `w092-array-successor-max2x6-v0.json` widened the array-successor cache
    from `139` to `194` runnable cases. Aggregate replay
    `w092-array-successor-max2x6-all-cycle-001` produced `147` exact typed
    matches, `40` `known_residual` rows, and `7` fresh
    `unexpected_mismatch` rows. The fresh rows are local `#VALUE!` vs Excel
    spill-array mismatches and were routed to reopened `BUG-FUNC-018` /
    `oxf-b39r`.
12. `w092-array-successor-max3x9-v0.json` widened the same cache to `219`
    runnable cases. Aggregate replay
    `w092-array-successor-max3x9-all-cycle-001` produced `167` exact typed
    matches, `42` `known_residual` rows, and `10` fresh
    `unexpected_mismatch` rows, all in the reopened `BUG-FUNC-018` scalar
    parameter array-lift class.
13. `w092-array-successor-max4x12-v0.json` widened the same cache to `233`
    runnable cases. Aggregate replay
    `w092-array-successor-max4x12-all-cycle-001` produced `175` exact typed
    matches, `44` `known_residual` rows, and `14` fresh
    `unexpected_mismatch` rows, all still in the reopened `BUG-FUNC-018`
    scalar parameter array-lift class.
14. `w092-array-successor-max5x15-v0.json` widened only the logical tranche to
    `25` cases; replay `w092-array-successor-max5x15-logical-cycle-001`
    produced `22` exact typed matches and `3` `BUG-FUNC-018` repeats.
15. `w092-array-successor-max10x30-v0.json` widened only the logical tranche to
    `40` cases; replay `w092-array-successor-max10x30-logical-cycle-001`
    produced `34` exact typed matches and `6` `BUG-FUNC-018` rows, including
    distinct `SWITCH` no-default, text-match, and logical-match variants.
16. `w092-array-successor-max20x60-v0.json` widened only the logical tranche to
    `42` cases; replay `w092-array-successor-max20x60-logical-cycle-001`
    produced `36` exact typed matches and `6` `BUG-FUNC-018` repeats, with no
    new mismatch formula beyond the max10 logical replay.
17. `w092-array-successor-max100x300-v0.json` produced `253` runnable cases,
    the same count as the `max20x60` build. This is the current
    array-successor scalar-argument generator plateau for the admitted manifest
    corpus.

Aggregate W092 executed cases in the ledger above: `1704`, with `1444` exact
typed matches, `204` `known_residual` rows, `8` `adapter_or_seam_mismatch`
rows, `2` `known_expected_deviation` rows, and `46` fresh
`unexpected_mismatch` rows routed to reopened `BUG-FUNC-018` after triage.

## 2.2.1 Broad Scalar Explorer Cycles (`2026-05-09`)

The plateau in Section 2.2 is bounded by the manifest-seed and
array-successor universes; the catalog-axis space outside those generators
is still under-probed. A new local Rust explorer
`smart-fuzzer/tools/pmt_ppmt_local_eval/src/bin/broad_scalar_explorer.rs`
plus driver `smart-fuzzer/tools/Run-BroadScalarExploration.ps1` walks `~50`
single-arg/two-arg numeric scalar functions across per-family numeric
bands (subnormals and non-finites are excluded because the Excel formula
literal parser rejects them).

Cycle ledger (Excel `16.0` build `19929`, workbook compatibility `2`):

1. `broad-scalar-cycle-003`: `1,000,000` cases, `600` Excel candidates,
   `338` matches, `214` literal-encoding drift, `48` unexpected.
2. `broad-scalar-cycle-004`: `1,500,000` cases, `600` candidates,
   `340` matches, `208` drift, `52` unexpected.
3. `broad-scalar-cycle-005`: `1,500,000` cases, `600` candidates,
   `328` matches, `223` drift, `49` unexpected.
4. `broad-scalar-cycle-006`: `2,000,000` cases, `600` candidates,
   `336` matches, `214` drift, `50` unexpected.
5. `broad-scalar-cycle-007`: `2,000,000` cases, `600` candidates,
   `319` matches, `233` drift, `48` unexpected.
6. `broad-scalar-cycle-008`: `2,000,000` cases, `600` candidates,
   `334` matches, `213` drift, `53` unexpected.
7. `broad-scalar-cycle-009`: `1,500,000` cases, `600` candidates,
   `337` matches, `216` drift, `47` unexpected.

Aggregate: `11,500,000` local evaluations, `4,200` Excel comparisons,
`2,332` exact bit matches, `1,521` literal-encoding-drift rows,
`347` unexpected mismatches, `0` Excel harness blockers.

The seven cycles agree on which mismatch classes recur. Findings are
classified, witnessed, and routed through new bug stream
`BUG-FUNC-027` and the run summary
`smart-fuzzer/planning/BROAD_SCALAR_EXPLORATION_2026-05-09.md`.

These cycles are W092 generation work, not a re-claim against the
Section 2.3 plateau gates: the broad-scalar explorer is a different
generator universe from the manifest-seed and array-successor cycles.

## 2.3 Stop-Gate And Promotion Audit

Audit snapshot: `2026-05-04`.

Objective-to-artifact checklist:

1. Create the W092 workset: satisfied by
   `docs/worksets/W092_SPARK_GUIDED_SMART_FUZZER_LONG_RUN.md` and
   `docs/WORKSET_REGISTER.md`.
2. Create the controlling Spark guide: satisfied by this file.
3. Create the bead rollout: satisfied by epic `oxf-6jbh` and child beads
   `oxf-6jbh.1` through `oxf-6jbh.5`.
4. Identify the first safe runnable tranche: satisfied by Section 2.1 and
   closed bead `oxf-6jbh.3`.
5. Execute feedback-guided cycles while useful signal appears: satisfied by
   the W092 run ledger in Section 2.2.
6. Classify known residuals and seams before continuing: satisfied by
   `Run-ArraySupportTranche.ps1` comparator classifications and the mappings
   below.
7. Promote durable findings through ordinary records: satisfied by the bug,
   handoff, and bead mappings below.
8. Stop only on an allowed gate: satisfied by the current
   `no-new-signal-plateau` gate for available nonblocked generators.

Promotion mapping:

1. `BUG-FUNC-018` / `oxf-b39r`: W092 reopened the scalar-parameter array-lift
   lane for widened successor seeds.
2. `BUG-FUNC-021` / `oxf-simj`: statistical numeric exactness rows remain
   known residuals.
3. `BUG-FUNC-024` / `oxf-xp6p`: `BESSELY(2.5,1)` remains a known residual.
4. `BUG-FUNC-025` / `oxf-dzfk`: `MINVERSE({1,2;3,4})` remains a known
   residual.
5. `BUG-FUNC-026` / `HO-FN-010` / `oxf-vkg8.1`: `TAKE` and adjacent `1x1`
   final-publication rows are classified as adapter or seam mismatches.
6. `BUG-FUNC-015` / `oxf-fckb`: PMT reference rows remain known expected
   deviations on the blocked financial exactness lane.

Stop-gate evidence:

1. Scenario-seed generation plateaued at `375` runnable cases at `max100`;
   `-IncludeKnownDeviations` did not add runnable cases.
2. Array-successor generation plateaued at `253` runnable cases:
   `w092-array-successor-max20x60-v0.json` and
   `w092-array-successor-max100x300-v0.json` produced the same case count.
3. The final logical replays produced no mismatch formula outside the already
   promoted `BUG-FUNC-018` class.
4. No resource-safety or Excel automation failure was observed.

Status axes after this stop gate:

1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_complete` for the W092 long-run objective
   through the current stop gate
3. `integration_completeness`: `integrated` for W092 guide, bead, run-ledger,
   and promotion records
4. `open_lanes`: `BUG-FUNC-018` landed-ref promotion, `BUG-FUNC-021`,
   `BUG-FUNC-024`, `BUG-FUNC-025`, `BUG-FUNC-015`, `HO-FN-010`, richer
   generator design, and provider/context/locale/version fixtures

Post-stop repair note:

1. `BUG-FUNC-018` reopened scalar-parameter array-lift rows were repaired in
   the W092 working tree after the stop-gate audit.
2. `w092-bug-func-018-repair-max20x60-all-001` replayed the current `253`-case
   successor plateau as `211` exact typed bit matches plus `42` known residual
   rows in existing exactness lanes, with `0` unexpected mismatches.
3. The repair evidence is local working-tree evidence pending a landed commit
   ref.

## 3. Function Selection Heuristics

Prioritize functions with one or more of these traits:

1. recent bug-stream activity,
2. known exactness deviation adjacency,
3. array lifting or spill sensitivity,
4. reference-preserving vs value-materialized behavior,
5. optional/default/omitted argument behavior,
6. variadic or repeated-parameter behavior,
7. coercion-heavy arguments,
8. volatile or host-context sensitivity,
9. source-risk or recently touched implementation hotspots,
10. low prior fuzzer coverage,
11. stale claims that need fresh confirmation.

Diversify across families. A noisy family may receive more budget, but it must
not starve text, lookup/reference, math, statistical, financial, date/time,
logical, compatibility, and dynamic-array lanes.

## 4. Axis Selection Heuristics

Prefer cases that increase one of these signals:

1. new arity shape,
2. optional/default omission pattern,
3. explicit missing-vs-blank-vs-empty distinction,
4. new scalar value-kind signature,
5. new error code path,
6. new numeric band or boundary neighborhood,
7. new text band, wildcard, delimiter, Unicode, or long-string neighborhood,
8. new array shape, orientation, or mixed-cell payload,
9. new reference-vs-array contrast,
10. new coercion path,
11. new spill shape or 1x1 scalar/array contrast,
12. new metamorphic relation check,
13. new local outcome class.

Use pairwise or strength-3 combinations across high-risk dimensions. Avoid full
Cartesian products unless the tranche is intentionally tiny.

## 5. Feedback Scoring

After each cycle, increase priority for buckets that produced:

1. unexpected mismatches,
2. unstable outcomes,
3. generator-invalid cases that expose harness assumptions,
4. new blocker classes,
5. new typed local outcomes,
6. new Excel error codes,
7. new array shapes,
8. successful minimization improvements.

Decrease priority for buckets that repeatedly produce only duplicate known
residuals or duplicate pass telemetry, unless the bucket is needed for catalog
coverage saturation.

## 6. Excel Spend Policy

Local evaluation is cheap. Excel comparison is selective.

Spend Excel budget on:

1. local outcomes not previously seen for the function and axis bucket,
2. high-risk functions or recent bug adjacency,
3. boundary and shape-edge cases,
4. metamorphic relation failures,
5. representative low-risk controls for drift detection,
6. stale-claim confirmation probes.

Do not spend Excel budget on large duplicate pass classes, known residual
duplicates, or provider/cube/RTD/live external-data lanes without a fixture.

## 7. Classification Buckets

Every interesting row must be classified as one of:

1. `match`,
2. `known_residual`,
3. `unexpected_function_semantic_mismatch`,
4. `adapter_or_seam_mismatch`,
5. `excel_harness_blocked`,
6. `generator_invalid`,
7. `unstable_or_nondeterministic`,
8. `needs_triage`.

Known residuals are not passes and are not new bugs. Unexpected mismatches are
not durable until replay metadata and reduction status are recorded.

## 8. Artifact Rules

Use the existing run artifact contract.

Required per promoted run:

1. `manifest.json`,
2. `rollup.json`,
3. retained telemetry or aggregate coverage sufficient to explain selection,
4. failure packets for durable mismatches, blockers, unstable rows, or
   generator-invalid rows,
5. minimized reproducers where feasible.

Ordinary passing cases should remain compact telemetry. Do not write per-case
prose for pass noise.

## 9. Promotion Rules

Promote a durable unexpected finding only after recording:

1. function id and surface name,
2. generated formula or invocation payload,
3. local typed outcome,
4. Excel typed outcome,
5. Excel version/build/channel,
6. workbook compatibility and locale profile,
7. run id and case id,
8. comparison kind,
9. minimization status,
10. classification bucket.

Destinations:

1. `docs/bugs/` for OxFunc-owned function semantics,
2. handoff packet or upstream note for OxFml-facing seams,
3. follow-up bead for repair/minimization/exploration work,
4. smart-fuzzer corpus only for curated replay seeds.

## 10. Ambitious Stop Gates

The runner should continue for days if useful signal remains. Stop only on one
of these gates:

1. `catalog-axis-saturation`: every OxFunc-accessible built-in in the canonical
   registry has meaningful retained coverage for every applicable high-value
   axis class: arity edge, optional/default omission, scalar value types,
   error/blank/missing behavior, array shape contrast, reference-vs-array
   contrast where admitted, numeric/text boundary bands where relevant, and
   known-risk adjacency.
2. `no-new-signal-plateau`: repeated full scheduler passes over the eligible
   catalog produce no new coverage bucket, no new typed local outcome, no new
   Excel mismatch, no new blocker class, and no new minimization improvement.
3. `open-finding-pressure-limit`: durable unexpected findings are accumulating
   faster than they can be minimized and responsibly promoted, so continued
   generation would reduce signal quality.
4. `all-active-paths-blocked`: every remaining high-value lane needs an
   unavailable fixture, unavailable Excel automation capability, sibling-repo
   change, or human decision, and blockers are recorded in beads.
5. `resource-safety-stop`: disk pressure, corrupt artifacts, repeated tool
   failures, or Excel automation instability make continued looping unreliable.
6. `user-stop`: the human changes or stops the goal.

Do not stop merely because:

1. one family was explored,
2. one cycle succeeded,
3. one mismatch was found,
4. known residuals were observed,
5. ordinary pass telemetry grew large,
6. the run has lasted a long time.

## 11. Status Reporting

Every checkpoint or final report must include:

1. `execution_state`,
2. `scope_completeness`,
3. `target_completeness`,
4. `integration_completeness`,
5. `open_lanes`.

Default long-run state is `in_progress` unless a stop gate is actually reached.
Sampled passing cases do not support function-phase-complete claims.
