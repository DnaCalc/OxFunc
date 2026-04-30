# Comprehensive Smart-Fuzzer Run Notes - 2026-04-30

Status: `run_artifacts_recorded`

Owning scope: W089 smart-fuzzer invocation-space exploration.

This note records how the 2026-04-30 broad smart-fuzzer exploration was
conducted, what dimensions were exercised, and which observations were promoted
for repair or future review. Pass-heavy telemetry remains in generated
`smart-fuzzer/runs/` artifacts and is not duplicated here case-by-case.

## Target Boundary

The run excludes the currently deferred pure-function target lanes:

1. `RTD` async subscription lifecycle behavior,
2. `LET`, `LAMBDA`, and formula-binding/callable helper formation or invocation
   without a concrete callable fixture,
3. live provider, cube, web, external-link, and host-context lanes without
   fixtures,
4. locale/version sweeps outside the default profile.

Pseudo-random functions are not treated as per-draw bit-exact cases. Any
stochastic work in this pass must use aggregate statistical or invariant
classification.

## Conduct Log

1. Used sub-agents for bounded grunt work:
   - tooling inventory: runnable smart-fuzzer tools and the generic case-set
     path,
   - bead/bug inventory: current blocked/open bug lanes,
   - local evaluator interface: admissible typed argument forms for generated
     cases.
2. Rebuilt W089 dimension and planning artifacts after confirming the current
   target boundary:
   - `534` inventoried surfaces,
   - `3` known-deviation surfaces after repair: `PMT`, `PPMT`, `IPMT`,
   - `193` blocked/deferred surfaces,
   - arity bands: `255` exact, `94` optional suffix range, `128` unknown,
     `57` variadic known minimum.
3. Added `Build-ScenarioSeedExecutableCases.ps1` to derive executable
   literal-argument cases from existing function-lane scenario manifests.
4. Reused the existing `array_tranche_local_eval` local evaluator and
   `Run-ArraySupportTranche.ps1` Excel comparison harness for typed exact
   comparison.
5. Ran a broad W089 seed replay, repaired clear tooling/local bugs, and reran
   as `w089-comprehensive-seed-20260430-004`.
6. Replayed the W090 successor case set as a control lane for the existing
   array-support/statistical-drift profile.
7. Ran the expanded finance reference generator and Excel throughput benchmark.

## Exploration Axes

1. Function surface: inventory spans `534` catalog/runtime surfaces. The
   executable broad seed admitted `339` literal cases over `11` category
   tranches.
2. Function categories exercised: math/trig, statistical, engineering,
   lookup/reference, text, compatibility, date/time, logical, financial,
   featured seed functions, and information.
3. Argument arity and omission: exact-arity seeds, optional-argument seeds, and
   zero-argument functions. A generator repair fixed false `missing_arg`
   injection for true zero-argument calls. Later W089 housekeeping clarified
   that below-minimum and above-maximum arity calls are outside the default
   pure OxFunc comparison universe and belong to OxFml admission-negative
   testing instead.
4. Value kinds: numbers, strings, logicals, error literals, missing arguments,
   empty cells where admitted by existing case sets, and inline arrays.
5. Array shape: scalar, row vector, column vector, rectangular matrix, dynamic
   array spill, and 1x1 scalar-vs-array publication shape.
6. Numeric exactness: all ordinary comparisons use exact typed equality and
   bit-exact numeric digest matching. No tolerance is applied.
7. Harness axes: local Rust value evaluation, Excel formula text through COM,
   error-code classification, spill capture, and scalar non-spill fallback.
8. Known deviation/reference lane: PMT/PPMT/IPMT are expected drift and remain
   blocked under `oxf-fckb`; they are useful reference mismatches, not repair
   targets in this run.
9. Stale POWER lane: POWER/OP_POWER are no longer known-deviation tags. The
   inventory now treats them as fresh-confirmation sentinels in risk telemetry;
   POWER was skipped in the executable seed only because no parseable target
   call was found in the admitted manifest rows.

## Run Artifacts

Ignored generated run/cache artifacts:

1. `smart-fuzzer/cache/dimension-inventory-v0.json`
   - known-deviation surfaces: `3`,
   - blocked/deferred surfaces: `193`.
2. `smart-fuzzer/cache/generator-matrix-v0.json`
3. `smart-fuzzer/cache/local-dry-run-budget-v0.json`
   - planned local case budget after POWER classification repair: `263,875`.
4. `smart-fuzzer/cache/excel-candidate-budget-v0.json`
   - planned Excel candidate quota after POWER classification repair: `2,479`.
5. `smart-fuzzer/cache/scenario-seed-executable-cases-v0.json`
   - cases: `339`,
   - tranches: `11`,
   - skipped: `347`,
   - skipped reasons: `193` blocked/deferred, `95` no manifest formula,
     `32` no target call, `27` no parseable literal seed.
6. `smart-fuzzer/runs/w089-comprehensive-seed-20260430-001/`
   - first broad seed replay before repairs.
7. `smart-fuzzer/runs/w089-comprehensive-seed-20260430-004/`
   - final broad seed replay after repairs.
8. `smart-fuzzer/runs/w090-successor-all-20260430-smart-wide-001/`
   - W090 successor replay control.
9. `smart-fuzzer/runs/w089-finance-reference-20260430-001/`
   - expanded finance reference run.
10. `smart-fuzzer/runs/w089-excel-throughput-20260430-001/`
    - Excel throughput benchmark.

Tracked repair/tooling artifacts:

1. `smart-fuzzer/tools/Build-ScenarioSeedExecutableCases.ps1`
2. `smart-fuzzer/tools/Build-DimensionInventory.ps1`
3. `smart-fuzzer/tools/Run-ArraySupportTranche.ps1`
4. `crates/oxfunc_core/src/functions/surface_dispatch.rs`

## Observations

### Broad Seed Replay

Final run `w089-comprehensive-seed-20260430-004`:

1. total cases: `339`,
2. exact typed bit matches: `288`,
3. unexpected mismatches: `48`,
4. Excel harness blockers: `3`,
5. Excel environment: Excel `16.0`, build `19929`, workbook Compatibility
   Version `2`.

The three harness blockers are invalid formula-assignment seeds for `ABS()`,
`SIN()`, and `XMATCH()` rather than confirmed function divergences. Under the
post-run W089 boundary clarification, these rows are invalid-generator /
OxFml-admission-negative cases and should be excluded from future default
comparison case sets.

### W090 Successor Control

Run `w090-successor-all-20260430-smart-wide-001`:

1. total cases: `139`,
2. exact typed bit matches: `98`,
3. unexpected mismatches: `41`,
4. harness blockers: `0`.

This reconfirmed the existing `BUG-FUNC-021` / `oxf-simj` statistical numeric
exactness lane after array admission succeeds.

### Finance Reference Run

Run `w089-finance-reference-20260430-001`:

1. generated/local cases: `1,000,000`,
2. local throughput: `157,219.90` cases/sec,
3. Excel sampled cases: `256`,
4. exact matches: `197`,
5. expected known deviations: `59`,
6. unexpected mismatches: `0`.

This supports keeping PMT/PPMT/IPMT as the current blocked known-deviation
reference lane without attempting repair in this pass.

### Excel Throughput

Run `w089-excel-throughput-20260430-001` measured `44,430.04` cases/sec for the
simple benchmark formula mix on this host. That is high enough to support
larger batched comparison passes, but more complex formulas and spill-heavy
cases should still budget conservatively.

### Static Risk

The refreshed static risk index is dominated by known blocked/deferred lanes:
`PPMT`, `PMT`, `RATE`, `TEXT`, `BAHTTEXT`, `RANDBETWEEN`, `RTD`, `RANDARRAY`,
`IMAGE`, and `TRANSLATE`. This is useful prioritization telemetry, not failure
evidence.

## Repairs

1. `ABS` unary array-lift repair:
   - bug stream: `BUG-FUNC-022`,
   - bead: `oxf-xmhu`,
   - fixed ref: `add56eeb6a0fdc49055fcab4222bb680a30c05ff`,
   - focused test: `observed_scalar_array_lift_handles_abs_arrays`.
2. Scenario seed zero-argument repair:
   - `PI()` and `NA()` no longer emit a spurious `missing_arg` value.
3. Excel dynamic error mapping repair:
   - `#SPILL!` now maps through text and `ERROR.TYPE=9`,
   - `#CALC!` now maps through text and `ERROR.TYPE=14`,
   - verified manually with `=SEQUENCE(0)` and a real spill conflict.
4. POWER stale classification repair:
   - `Build-DimensionInventory.ps1` no longer classifies POWER/OP_POWER as
     known deviations,
   - the sentinel remains as risk/selection telemetry,
   - rebuilt inventory now reports only `PMT`, `PPMT`, and `IPMT` as known
     deviations.
5. Invalid-arity universe-boundary repair:
   - default scenario-seed generation now checks candidate argument count
     against published `arity_min` / `arity_max`,
   - invalid arity rows are skipped before Excel COM comparison,
   - such rows remain available only as OxFml admission-negative evidence when
     a dedicated seam test asks for them.

## Residual Open Lanes

1. `BUG-FUNC-021` / `oxf-simj`: statistical numeric exactness drift. W089 adds
   broad-manifest evidence to the W090 successor evidence.
2. `BUG-FUNC-023` / `oxf-i45e`: non-statistical residual exactness and matrix
   shape drift:
   - `BESSELY(2.5,1)` low-bit numeric drift,
   - `VDB(2400,300,120,6,18)` one-bit numeric drift,
   - `MINVERSE({1,2;3,4})` matrix numeric low-bit drift,
   - `MINVERSE(5)` and `MMULT(5,2)` local `array:1x1` vs Excel scalar shape.
3. `oxf-fckb`: PMT/PPMT/IPMT expected exactness drift remains blocked and is
   not a repair target for this run.
4. Deferred/out-of-current-target invocation dimensions:
   - `RTD` async subscription lifecycle,
   - `LET`, `LAMBDA`, formula-binding, and callable helper formation without a
     fixture,
   - provider/cube/web/external-link and host-context lanes without fixtures,
   - locale and alternate Excel-version sweeps,
   - stochastic functions needing statistical rather than per-draw bit-exact
     comparators,
   - rich reference/workbook/neighborhood cases not admitted by the pure value
     comparator.

## Status Axes

1. `execution_state`: `comprehensive_seed_run_recorded`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: `BUG-FUNC-021`, `BUG-FUNC-023`, blocked PMT/PPMT/IPMT
   reference mismatch lane, and the deferred/unexecuted invocation-space
   dimensions listed above

## Follow-Up: BUG-FUNC-023 Repair And Split

Follow-up run `oxf-i45e-w089-repair-20260430-001` repaired three rows from
the mixed non-statistical stream and replayed the same `339` scenario-seed
cases against Excel:

1. exact typed bit matches improved to `297`,
2. unexpected mismatches dropped to `39`,
3. Excel harness blockers remained `3`,
4. `VDB(2400,300,120,6,18)`, `MINVERSE(5)`, and `MMULT(5,2)` now match Excel,
5. `BESSELY(2.5,1)` moved to `BUG-FUNC-024` / `oxf-xp6p`,
6. `MINVERSE({1,2;3,4})` moved to `BUG-FUNC-025` / `oxf-dzfk`.

Updated open lanes after that repair are `BUG-FUNC-021`, `BUG-FUNC-024`,
`BUG-FUNC-025`, the blocked PMT/PPMT/IPMT reference lane, and deferred or
unexecuted invocation-space dimensions.
