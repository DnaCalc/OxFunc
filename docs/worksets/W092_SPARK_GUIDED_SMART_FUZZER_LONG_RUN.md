# W092 Spark-Guided Long-Run Smart-Fuzzer Exploration

Status: `stopped_at_no_new_signal_plateau`

## 1. Purpose

Create and operate a long-running, feedback-guided smart-fuzzer lane that a
less intelligent but very fast Codex runner can pursue for many cycles without
losing OxFunc doctrine, artifact discipline, or bug-promotion hygiene.

This workset is a successor to `W088` and `W089`. It does not replace the
existing smart-fuzzer design. It adds a Spark-suitable run guide and bead path
for repeated exploration across function identity, arity, value type, coercion,
array, reference, context, and comparison-policy axes.

The controlling run guide is:

`smart-fuzzer/planning/SPARK_LONG_RUN_SMART_FUZZER_GUIDE.md`

## 2. Depends On

1. `W088` for the smart-fuzzer pilot substrate.
2. `W089` for sweeping invocation-space dimension planning.
3. `W070` for bead-based execution discipline.
4. `W072` for bug intake, root-cause, and regression-stream protocol.
5. `W091` for canonical runtime function registry direction.

## 3. Parent Doctrine And Spec Surfaces

1. `CHARTER.md`
2. `OPERATIONS.md`
3. `docs/BEADS.md`
4. `docs/worksets/W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md`
5. `docs/worksets/W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md`
6. `smart-fuzzer/README.md`
7. `smart-fuzzer/planning/SMART_FUZZER_DESIGN.md`
8. `smart-fuzzer/planning/SWEEPING_INVOCATION_SPACE_RUN_PLAN.md`
9. `smart-fuzzer/planning/RUN_ARTIFACT_CONTRACT.md`
10. `smart-fuzzer/planning/SPARK_LONG_RUN_SMART_FUZZER_GUIDE.md`

## 4. Scope

In scope:

1. define the Spark-suitable long-run guide,
2. create an explicit bead set for guide, register, audit, execution, and
   promotion lanes,
3. run repeated bounded cycles when an execution bead is active,
4. use feedback from coverage, typed outcomes, mismatches, blockers, and
   minimization improvements to steer future cycles,
5. promote durable findings through ordinary bug streams and follow-up beads.

Out of scope:

1. function semantic closure claims from sampled fuzzer passes,
2. a parallel comprehensive function registry,
3. broad provider/cube/RTD/live external-data parity without fixtures,
4. hiding generated pass cases in heavyweight prose artifacts,
5. treating stale historical deviations as current bugs without fresh replay.

## 5. Long-Run Stop Gates

The runner should continue for as long as useful signal is being produced. It
should stop only when one of these ambitious gates is reached:

1. `catalog-axis-saturation`: every OxFunc-accessible built-in has retained
   coverage over all applicable high-value axis classes.
2. `no-new-signal-plateau`: repeated full scheduler passes over the eligible
   catalog produce no new coverage bucket, typed local outcome, Excel mismatch,
   blocker class, or minimization improvement.
3. `open-finding-pressure-limit`: durable unexpected findings are accumulating
   faster than they can be minimized and responsibly promoted.
4. `all-active-paths-blocked`: every remaining high-value path requires an
   unavailable fixture, sibling-repo change, Excel automation capability, or
   human decision, and blockers are recorded in beads.
5. `resource-safety-stop`: disk pressure, corrupt artifacts, repeated tool
   failures, or Excel automation instability make continued looping unreliable.
6. `user-stop`: the human changes or stops the goal.

Non-stop conditions:

1. one function family was explored,
2. one cycle succeeded,
3. one mismatch was found,
4. known residuals were observed,
5. ordinary pass telemetry became large,
6. the run has been active for a long time.

## 6. Artifact And Promotion Policy

1. Ordinary passing rows remain compact telemetry and rollups.
2. Heavy artifacts are reserved for mismatches, unstable rows, generator-invalid
   rows, harness blockers, minimized reproducers, and curated replay samples.
3. Every durable unexpected mismatch needs Excel version/build/channel,
   workbook compatibility, run id, case id, and replay/minimization status
   before promotion.
4. Known residuals remain distinct from new mismatches.
5. OxFml-facing seam observations require handoff or sibling notes rather than
   OxFunc-only silent fixes.

## 7. Initial Bead Lanes

1. guide lane,
2. register lane,
3. cycle-harness audit lane,
4. long-run execution lane,
5. promotion lane.

Each W092 bead must name
`smart-fuzzer/planning/SPARK_LONG_RUN_SMART_FUZZER_GUIDE.md` as the controlling
run guide.

## 8. Reporting Contract

All W092 reports must include:

1. `execution_state`,
2. `scope_completeness`,
3. `target_completeness`,
4. `integration_completeness`,
5. explicit `open_lanes` while any axis remains partial.

Use `in_progress` when uncertain.

Initial status axes:

1. `scope_completeness`: `scope_partial`
2. `target_completeness`: `target_complete` for the W092 long-run objective
   through the current stop gate
3. `integration_completeness`: `integrated` for the W092 guide, bead rollout,
   run ledger, and promotion records
4. `open_lanes`: broader catalog-axis saturation, richer generator design,
   provider/context/locale/version fixtures, and promoted follow-up lanes
   `BUG-FUNC-018`, `BUG-FUNC-021`, `BUG-FUNC-024`, `BUG-FUNC-025`,
   `BUG-FUNC-015`, and `HO-FN-010`.

## 9. Stop-Gate Snapshot

W092 execution reached the current `no-new-signal-plateau` gate on
`2026-05-04` for available nonblocked generators:

1. scenario-seed generation plateaued at `375` runnable cases,
2. array-successor generation plateaued at `253` runnable cases,
3. final logical replays added no mismatch formula outside the promoted
   `BUG-FUNC-018` class,
4. promotion records are mapped in
   `smart-fuzzer/planning/SPARK_LONG_RUN_SMART_FUZZER_GUIDE.md` Section 2.3.
