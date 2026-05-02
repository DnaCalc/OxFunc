# Axis Witness Sweep Run Plan

Status: `oxfunc_accessible_run_exercised`

Owning workset:
`docs/worksets/W089_SMART_FUZZER_SWEEPING_INVOCATION_SPACE_EXPLORATION.md`

Owning bead: `oxf-vkg8`

## Purpose

This run shape turns the W089 invocation-space analysis into paired witness
cases. Each pair changes one declared axis while keeping the call small enough
to evaluate through the direct OxFunc value surface and Excel `Formula2`
surface. A pair is useful only when Excel observes different outcomes for the
control and variant calls; each individual call is still compared against
OxFunc with exact typed equality.

The batch is exploration telemetry and mismatch discovery. It is not function
closure evidence.

## Scope Boundary

This batch targets the OxFunc-accessible region of the invocation space. In
this document, `runnable` means the axis can be exercised with the current
direct OxFunc value-call helper, simple typed fixtures, and the Excel `Formula2`
comparison runner.

`blocked` or `deferred` means the axis belongs to the broader DNA Calc system
and needs a larger fixture or seam-specific harness before comparison evidence
would be meaningful. It does not mean the axis is unimportant, and it does not
mean OxFunc has a function-semantic mismatch.

Default work should stay in the OxFunc-accessible region until a later run
explicitly declares a larger DNA Calc harness, such as an OxFml prepared-call
runner, an XLL bridge runner, a provider host, or workbook/locale/reference
fixtures beyond the current single-workbook value comparator.

## Builder

Default command:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Build-AxisWitnessCaseSet.ps1
```

Default output:

```text
smart-fuzzer/cache/axis-witness-case-set-v0.json
```

Default execution command:

```powershell
powershell -ExecutionPolicy Bypass -File smart-fuzzer\tools\Run-ArraySupportTranche.ps1 `
  -RunId w089-axis-witness-local `
  -CaseSetPath smart-fuzzer\cache\axis-witness-case-set-v0.json
```

The runner writes the usual compact run artifacts under
`smart-fuzzer/runs/<run_id>/`. It also records `axis_witness_pairs` in
`rollup.json`, including whether each control/variant pair differentiated on
Excel.

## Latest Smoke Run

Run ID:

```text
w089-axis-witness-take-repair-20260502-001
```

Observed on Excel `16.0` build `19929`, workbook Compatibility Version `2`.

Rollup:

1. total cases: 38,
2. runnable axis witness pairs: 19,
3. Excel-differentiated pairs: 19,
4. exact typed OxFunc/Excel matches: 38,
5. unexpected mismatches: 0,
6. harness blockers: 0.

The earlier smoke run `w089-axis-witness-20260501-002` found one TAKE
scalar-publication mismatch:

```text
=TAKE({1,2;3,4},1,1)
```

That lane is now tracked as `BUG-FUNC-026` and bead `oxf-vkg8.1`, but the
function-level repair was later reclassified as mislocalized. Nested Excel
probes show `TYPE(TAKE({1,2;3,4},1,1))=64`; the remaining direct-comparator
shape difference belongs in the publication/comparator seam above OxFunc.

## Runnable Axis Witnesses

The default case set covers these runnable axes:

1. function identity,
2. optional argument present versus omitted,
3. explicit missing optional argument,
4. variadic argument count,
5. scalar text coercion success versus error,
6. logical `TRUE` versus `FALSE`,
7. error value versus ordinary number,
8. blank cell versus empty text,
9. numeric zero-edge behavior,
10. tiny negative numeric band versus zero,
11. empty string versus whitespace,
12. case-sensitive text comparison,
13. scalar control versus array literal,
14. row-vector versus column-vector orientation,
15. reference versus array-literal substitute,
16. single-cell versus rectangular-area reference,
17. same-sheet multi-area reference,
18. caller-location context,
19. error-code comparison.

Known financial exactness deviations can be included only with
`-IncludeKnownDeviationReferencePairs`; they remain expected reference
mismatch lanes, not repair targets for this run.

## Broader DNA Calc Axis Witnesses

The builder keeps blocked axis witnesses visible in the case-set metadata
rather than silently dropping them. These are not part of the current
OxFunc-accessible run target. They currently need additional fixtures or
separate harnesses in the bigger DNA Calc system:

1. workbook compatibility version,
2. date system,
3. locale profile,
4. volatile recalc / stochastic statistical comparator,
5. host/provider capability,
6. OxFml prepared-call seam,
7. XLL bridge seam,
8. provider host seam,
9. cross-sheet references,
10. structured references,
11. spill anchors,
12. callable/helper values,
13. presentation/rich values,
14. provider-context values,
15. aggregate statistical profile comparison classes.

Blocked axes must not be counted as function mismatches. They become runnable
only when the run manifest names the required fixture or seam harness and the
comparison policy for that axis.

## Comparison Policy

The pass class remains `exact_typed_bit_match`. Numeric tolerance buckets are
not passes. Error outcomes compare by typed Excel/OxFunc error code. Array
outcomes compare by shape plus row-major cell digests.

## Status Axes

1. `execution_state`: `oxfunc_accessible_run_exercised`
2. `scope_completeness`: `scope_partial`
3. `target_completeness`: `target_partial`
4. `integration_completeness`: `partial`
5. `open_lanes`: stay focused on OxFunc-accessible axes for default W089 work;
   add separate DNA Calc fixtures for broader context/seam/reference axes only
   when explicitly scoped.
