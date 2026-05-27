# OxFunc Smart Fuzzer

Status: `planning_sandbox`

This directory is the planning and experiment area for the OxFunc smart-fuzzer.

## Mission

The OxFunc parity target is **bit-exact emulation of Excel** for every in-scope function and operator (`517` rows: `494` functions plus `23` operators, after the `17` deferred service/cube/pivot rows in `docs/function-lane/W50_DEFERRED_CURRENT_VERSION_INVENTORY.csv`). All in-scope rows are equally active.

Any OxFunc-vs-Excel discrepancy is a bug. Bit-exact Excel parity is the goal, **including matching Excel where Excel itself is imprecise** — the default repair direction is to match Excel, not to be analytically correct. Severity is graded, not flat:

1. **Structural mismatch — top priority, fix on discovery.** Wrong kind, wrong error code, wrong shape, wrong array/spill behavior, wrong handling of error/blank/missing/reference inputs, unexpected crash or rejection. Root cause may be in the kernel, the function metadata, or the harness itself; all three are bugs.
2. **Numeric drift — continuous-triage class.** Float drift in numeric kernels. `> 1` ULP drift is more serious than `1` ULP drift, but both are bugs. Each gets a witness and a `BUG-FUNC-*` stream. A row where OxFunc returns the analytic exact value and Excel is `±1` ULP off is tagged `excel_imprecision_witness` to make the repair direction explicit, but it is still in the numeric-drift bug count.

The smart-fuzzer is built to find the **structural and elusive** class ahead of piling up more LSB witnesses in known-drifting numeric kernels. Pass-heavy mass agreement is exploration telemetry, not closure evidence.

Full design and severity-aware classification scheme: `smart-fuzzer/planning/SMART_FUZZER_DESIGN.md`. The companion CHARTER section is `CHARTER.md` §4.1. The canonical severity vocabulary is implemented in `smart-fuzzer/tools/CellRefBatch.psm1` and consumed by every comparator runner — see `smart-fuzzer/tools/README.md`.

## Owning Worksets

1. `docs/worksets/W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md` — pilot execution substrate.
2. `docs/worksets/W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md` — invocation-space sweep plan.
3. `docs/worksets/W090_FUNCTION_ARRAY_SUPPORT_SYSTEMATIC_SWEEP.md` — array-valued scalar-parameter sweep.
4. `docs/worksets/W092_SPARK_GUIDED_SMART_FUZZER_LONG_RUN.md` — feedback-guided long-run loop.
5. `docs/worksets/W097_BIT_EXACT_RESWEEP_OF_KNOWN_MISMATCHES.md` — cell-ref-plumbing re-measurement.

The smart-fuzzer is an evidence-generation and regression-discovery system. It does not define OxFunc semantics, does not replace function contracts, and does not by itself promote any function status. Any durable mismatch found here must be promoted through the ordinary OxFunc bug intake, evidence, workset, and bead surfaces.

The fuzzer is expected to produce many more passes than failures. Passing case records are exploration telemetry, not individually sacred evidence artifacts. Keep them compact, aggregatable, and cheap to discard or regenerate. Reserve detailed narrative and promotion effort for failures, minimized mismatches, and small representative pass samples that explain coverage.

## Current Scope Boundary

The current OxFunc smart-fuzzer focus is the OxFunc-accessible region of the
invocation space. A case is OxFunc-accessible when both sides can be described
by:

1. direct OxFunc value-surface inputs,
2. simple typed cell/reference fixtures supported by the local resolver,
3. a single Excel `Formula2` evaluation with matching workbook setup,
4. an exact typed comparison policy that does not depend on host services,
   parser/binder state, or external providers.

Those cases are eligible for the default local Rust plus Excel comparison
runner. Mismatches from this region can become ordinary OxFunc bug streams
after minimization.

Some invocation-space axes are real Excel/DNA Calc axes but are outside this
default OxFunc-accessible region. Examples include workbook compatibility
sweeps, alternate date systems, locale profiles, OxFml prepared-call behavior,
XLL bridge behavior, provider/cube/web host state, cross-sheet or structured
references, spill-neighborhood behavior, inline callable formation, and rich
values. These axes need bigger DNA Calc fixtures or seam-specific harnesses
before their results can be compared honestly.

Blocked/deferred rows for those axes are coverage facts, not OxFunc function
failures. They should remain visible in rollups so the unexplored space is not
forgotten, but they must not be counted as mismatches or repair targets for the
current OxFunc-accessible fuzzer work.

## Definition

In OxFunc, a smart-fuzzer is a typed, metadata-aware, feedback-guided explorer
over Excel function invocation space. It generates candidate worksheet function
calls and related context fixtures, evaluates them cheaply through local Rust
and adapter paths, spends slower Excel evaluations on high-value candidates, and
turns confirmed mismatches into minimized replayable artifacts.

The "smart" part is not only random generation. It combines:

1. function metadata from the library-context snapshot and function contracts,
2. value-universe and prepared-argument distinctions,
3. existing bug streams and scenario manifests,
4. static source-code risk signals,
5. fast local outcome diversity and coverage feedback,
6. batched Excel comparison,
7. agent-assisted review of mismatch clusters and generator blind spots.

## Directory Layout

1. `planning/`
   - tracked design notes, schemas, artifact contracts, rollout sketches, and
     decision inputs.
2. `prompts/`
   - tracked prompt packets for external model review.
3. `corpus/`
   - candidate minimized cases that are not yet promoted into canonical
     function-lane, bug, or test surfaces.
4. `tools/`
   - tracked reproducible helpers for generating ignored run and cache
     artifacts.
5. `runs/`
   - local generated run outputs. Ignored by default.
6. `work/`
   - local scratch and transient experiment state. Ignored by default.
7. `cache/`
   - local derived indexes and generated helper artifacts. Ignored by default.

## Excel Comparator Plumbing Rule

Comparator runs that compare against Excel under a bit-exact policy must
pass numeric inputs to Excel through cell `Range.Value2`, not through
formula literal text. Excel's formula parser is not guaranteed to map a
17-digit decimal back to the original `f64`. The full rule, an empirical
witness, and the runner inventory are in
`smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`. New runners should
default to cell-ref plumbing; existing runners that still use literal-text
input cannot be the basis for closing a `BUG-FUNC-*` exactness stream.

## Authority Rules

1. Source-of-truth semantics remain in `docs/function-lane/*`, Rust code,
   Lean/formal artifacts, and promoted evidence records.
2. Live execution state remains in `.beads/` through `br`; this directory is
   not an execution-state tracker.
3. Generated run outputs must carry Excel version/channel, workbook
   compatibility metadata, runner version, manifest hash, and git revision
   before they can be considered for promotion.
4. Confirmed mismatches must be reduced, classified as function-semantic or
   seam/harness status, and routed through `docs/bugs/`.
5. Clean-room rules apply: only public documentation, published research, and
   reproducible black-box Excel observations may inform conclusions.

## First Practical Goal

The first fuzzer pass should measure the actual Excel batch evaluation rate and
prove the artifact loop on a narrow pilot surface before broad catalog rollout:

1. generate typed candidate invocations,
2. run fast local evaluation,
3. rank candidates for Excel,
4. batch Excel evaluation,
5. compare typed outcomes,
6. minimize any mismatch,
7. promote only durable reduced cases into canonical surfaces.

The first implementation should therefore optimize for compact machine-readable
run data, rollup statistics, and failure packets rather than one document per
case.
