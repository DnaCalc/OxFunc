# OxFunc Smart Fuzzer

Status: `planning_sandbox`

This directory is a non-production planning and experiment area for a future
OxFunc smart-fuzzer.

Owning workset: `docs/worksets/W088_SMART_FUZZER_DIFFERENTIAL_EXPLORATION.md`.

The smart-fuzzer is an evidence-generation and regression-discovery system. It
does not define OxFunc semantics, does not replace function contracts, and does
not by itself promote any function status. Any durable mismatch found here must
be promoted through the ordinary OxFunc bug intake, evidence, workset, and bead
surfaces.

The fuzzer is expected to produce many more passes than failures. Passing case
records are exploration telemetry, not individually sacred evidence artifacts.
Keep them compact, aggregatable, and cheap to discard or regenerate. Preserve
detailed narrative and promotion effort for failures, minimized mismatches, and
small representative pass samples that explain coverage.

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
   - tracked design notes, schemas, rollout sketches, and decision inputs.
2. `prompts/`
   - tracked prompt packets for external model review.
3. `corpus/`
   - candidate minimized cases that are not yet promoted into canonical
     function-lane, bug, or test surfaces.
4. `runs/`
   - local generated run outputs. Ignored by default.
5. `work/`
   - local scratch and transient experiment state. Ignored by default.
6. `cache/`
   - local derived indexes and generated helper artifacts. Ignored by default.

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
