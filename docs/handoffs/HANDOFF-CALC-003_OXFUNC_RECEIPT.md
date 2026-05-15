# HANDOFF-CALC-003 OxFunc Receiving Review

## Purpose

Record the OxFunc-side receiving review of:

1. `../OxCalc/docs/handoffs/HANDOFF_CALC_003_OXFUNC_NUMERICAL_REDUCTION_AND_ERROR_ALGEBRA_NOTE.md`
2. `../OxCalc/docs/handoffs/HANDOFF_CALC_003_OXFML_NUMERICAL_REDUCTION_AND_ERROR_ALGEBRA.md`
3. `../OxFml/docs/handoffs/HANDOFF_CALC_003_OXFML_RECEIPT.md`

This is a receiving acknowledgement and replacement-plan note. It does not
claim selector enforcement in current OxFunc Rust kernels.

## Decision Summary

Decision: `accept_metadata_and_semantics_reservation_defer_enforcement`.

OxFunc accepts ownership of:
1. reduction-sensitive kernel metadata,
2. error-collapse-sensitive kernel metadata,
3. exact `NumericalReductionPolicy` semantics,
4. exact `ErrorAlgebra` / worksheet-error precedence semantics.

OxFml remains owner of formula-plan/session/prepared-call context carriage and
replay projection. OxCalc remains owner of coordinator/profile policy and
prepared-package invalidation consumption.

## Canonical OxFunc Update

Canonical local contract:
1. `docs/function-lane/OXFUNC_KERNEL_METADATA_AND_ADMISSION_PROFILE_CONTRACT.md`

Accepted initial `NumericalReductionPolicy` keys:
1. `SequentialLeftFold`
2. `PairwiseTree`
3. `KahanCompensated`

Accepted initial `ErrorAlgebra` key:
1. `CanonicalExcelLegacy`

Accepted canonical error precedence:
1. `#NULL!`
2. `#DIV/0!`
3. `#VALUE!`
4. `#REF!`
5. `#NAME?`
6. `#NUM!`
7. `#N/A`

## Metadata Version Signal

OxFunc reserves:

```text
semantic_kernel_metadata_version
```

This signal should invalidate prepared packages when selector behavior,
affected-function classification, or reduction/error-collapse metadata changes.
Conservative all-function invalidation is acceptable until a narrower
per-function or per-family fingerprint exists.

## First Affected Families

First review-required families:
1. aggregate numeric reducers: `SUM`, `SUMSQ`, `PRODUCT`, `AVERAGE`, `MIN`,
   `MAX`, `SUBTOTAL`, `AGGREGATE`,
2. conditional/database reducers: `SUMIF`, `SUMIFS`, `AVERAGEIF`,
   `AVERAGEIFS`, `COUNTIF`, `COUNTIFS`, database aggregate functions,
3. array and matrix numeric reducers: `SUMPRODUCT`, `MMULT`, `MDETERM`,
   `MINVERSE`, `SUMX2MY2`, `SUMX2PY2`, `SUMXMY2`,
4. helper/callable reducers: `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, `MAP`,
   `GROUPBY`, `PIVOTBY`,
5. criteria/selection families where multiple candidate worksheet errors can
   collapse into one result,
6. shared range/array aggregation and error-selection helpers.

Explicitly deferred:
1. provider/cube functions under W041,
2. RTD/external-provider lifecycle lanes under W043,
3. locale and alternate Excel-version sweeps,
4. functions without numeric reduction or multi-error collapse until concrete
   review proves otherwise.

## Replacement Plan

This packet is replaced by a two-owner plan:
1. OxFml: `CorrectnessFloorContext` carriage and replay identity.
2. OxFunc: kernel metadata, selector semantics, affected-family metadata, and
   invalidation version signals.

OxCalc may keep local W050 selector artifacts as compatibility evidence until
receiving repos emit canonical runtime/replay fields.

## Non-Claims

1. No current OxFunc Rust kernel is claimed to enforce
   `NumericalReductionPolicy`.
2. No current OxFunc Rust kernel is claimed to enforce configurable
   `ErrorAlgebra`.
3. No current OxFunc prepared-call invalidation implementation is claimed.
4. This receipt is not function-phase completion for any affected function.

## Status

- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - Rust metadata fields and registry/export publication
  - shared helper or per-kernel conversion to canonical error algebra
  - replay field names for pairwise tree shape and Kahan compensation state
  - OxFml/OxCalc consumption of `semantic_kernel_metadata_version`
  - exercised tests for affected function families
