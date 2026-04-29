# W090 Array Support Systematic Sweep Plan

Status: `first_cycle_closed_successor_sweep_ready`

## 1. Purpose

Define the W090 array-support sweep as a reproducible, pass-light exploration
program rather than a prose-heavy test ledger. This plan narrows the catalog-
wide W089 dimensions to the specific question: where can an array-valued value
argument reach a function position that local OxFunc still treats as a scalar
coercion site, while current Excel may spill elementwise or by broadcast.

## 2. Derived Inventory

The tracked builder is:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-ArraySupportSweepPlan.ps1 -RefreshInventory
```

Default generated cache outputs are ignored and reproducible:

```text
smart-fuzzer/cache/array-support-candidate-inventory-v0.json
smart-fuzzer/cache/array-support-first-tranche-v0.json
smart-fuzzer/cache/array-support-replay-matrix-v0.json
smart-fuzzer/cache/array-support-highlights-v0.md
```

The candidate inventory combines:

1. W089 `dimension-inventory-v0.json`,
2. `OXFUNC_LIBRARY_CONTEXT_SNAPSHOT_EXPORT_V1.csv`,
3. source-file signals from `crates/oxfunc_core/src/functions/*.rs`,
4. W079/W080 seeded array-support fixes,
5. blocked/deferred and known-deviation tags.

The generated inventory is not semantic authority. It is an exploration queue
and must not be used to claim a function surface has been reviewed unless a
replay artifact exists for that specific lane.

## 3. Risk Signals

The first W090 risk classifier gives priority to:

1. ordinary `ValuesOnlyPreAdapter` functions,
2. `Custom` or scalar-only coercion/lift metadata,
3. source files with `coerce_prepared_to_number` or
   `coerce_prepared_to_text`,
4. source files using `run_values_only_prepared` or
   `prepare_args_values_only` without an obvious array-map, broadcast, or
   binary-numeric helper,
5. source files that explicitly reject array values,
6. non-text families, because W080 already seeded the first text-family
   corrections.

The classifier demotes:

1. text and lookup surfaces already reconciled by W079/W080,
2. provider/cube/RTD and other context-blocked surfaces,
3. known-deviation lanes such as PMT/PPMT/IPMT,
4. stale POWER concerns unless they are freshly replayed.

## 4. First Tranche

Tranche id: `w090-tranche-a-math-scalar-numeric-array-lift`

Selected surfaces:

1. `ROUND`
2. `ROUNDDOWN`
3. `ROUNDUP`
4. `TRUNC`
5. `CEILING`
6. `CEILING.MATH`
7. `CEILING.PRECISE`
8. `FLOOR`
9. `FLOOR.MATH`
10. `FLOOR.PRECISE`
11. `ISO.CEILING`
12. `ATAN2`
13. `BASE`
14. `MROUND`

Rationale:

1. this is a non-text successor tranche after W080,
2. the family is bounded and mostly ordinary value-only evaluation,
3. local source patterns show scalar coercion sites without obvious
   shape-preserving lift in several selected functions,
4. the formulas are cheap for Excel and local replay,
5. the selected functions exercise required scalar args, optional scalar args,
   multi-argument same-shape arrays, and array-with-omitted-optional controls.

Non-goals:

1. no full math-family claim,
2. no text-family replay,
3. no lookup-family replay,
4. no POWER bug assumption without fresh confirmation,
5. no repair inside the planning bead.

## 5. Replay Matrix

Each selected surface gets compact formula seeds along these axes:

1. scalar control,
2. single array argument in each scalar position,
3. same-shape multiple arrays where the formula seed exists,
4. omitted optional controls with array-valued required arguments where
   applicable,
5. arrays containing worksheet errors,
6. arrays containing blanks or empty cells,
7. reference-area versus inline-array contrast,
8. shape mismatch or broadcast probes.

The first generated replay matrix stores Excel formula seeds and telemetry
keys; it does not run Excel and does not create pass/fail evidence.

## 6. Telemetry Economy

Passing rows are summarized by counters only:

1. function,
2. argument position,
3. array shape,
4. array cell band,
5. optional-state band,
6. reference-vs-inline source,
7. local outcome class,
8. Excel outcome class,
9. comparison class.

Detailed packets are reserved for:

1. unexpected mismatches,
2. unstable or non-reproducible outcomes,
3. harness blockers,
4. minimized reproducers promoted into `BUG-FUNC-*` streams.

## 7. Next Execution Step

The first execution step was a small Excel/local replay of
`w090-tranche-a-math-scalar-numeric-array-lift`, followed by immediate
classification:

1. exact typed bit match: count only,
2. Excel spills and local rejects: promote a bug stream and create a repair
   bead,
3. both reject: record a representative anchor and move on,
4. harness/context blocked: record blocked telemetry, not a function mismatch.

No tolerance is allowed for numeric pass classification; all passes must be
exact typed bit matches.

## 8. First-Cycle Result

Run `w090-array-tranche-a-local-010` found `32` unexpected mismatches and `2`
exact matches. The mismatches were promoted as `BUG-FUNC-017` and repaired on
landed ref `0b966d0ee7c8ce4a327b0b3090f9a108248c37fd`.

Rerun `w090-array-tranche-a-local-011` produced `34/34` exact typed bit matches
against Excel `16.0` build `19929`, workbook Compatibility Version `2`.

## 9. Successor Plan

The remaining unswept array-support surface stays target-partial by design.
Future work should continue from the generated candidate inventory rather than
from prose test lists. The next owner should:

1. regenerate `array-support-candidate-inventory-v0.json`,
2. exclude W079/W080 and BUG-FUNC-017 landed lanes unless a regression is
   suspected,
3. pick the next highest-risk non-text tranche by source scalar-coercion signal
   and category spread,
4. include reference-vs-inline-array and blank/error contrast rows for the
   chosen tranche,
5. preserve the pass-light rule: aggregate pass telemetry only, full packets
   only for mismatches or harness blockers.
