# HANDOFF-CALC-004 OxFunc Receiving Review

## Purpose

Record the OxFunc-side receiving review of:

1. `../OxCalc/docs/handoffs/HANDOFF_CALC_004_OXFUNC_RICH_ARG_ACCEPTED_NOTE.md`
2. `../OxCalc/docs/handoffs/HANDOFF_CALC_004_OXFML_CAPABILITY_SET_HOLE_ADMISSION.md`
3. `../OxFml/docs/handoffs/HANDOFF_CALC_004_OXFML_RECEIPT.md`

This is an identity and metadata reservation. It does not claim rich/sparse
execution support in current OxFunc kernels.

## Decision Summary

Decision: `accept_identity_reservation_defer_activation`.

OxFunc accepts an argument-admission metadata shape equivalent to:

```text
RichArgAccepted(required_capability_set)
```

OxFunc accepts sparse-reader admission metadata as a successor lane, with final
Rust naming deferred. The reserved conceptual shape is:

```text
SparseRangeAccepted(extent_class, cardinality_class)
```

## Canonical OxFunc Update

Canonical local contract:
1. `docs/function-lane/OXFUNC_KERNEL_METADATA_AND_ADMISSION_PROFILE_CONTRACT.md`

Accepted capability selectors:
1. `Indexable`
2. `Enumerable`
3. `Shaped`
4. `Materialisable`

Accepted admission rule:
1. a producer is admissible only when its stable-key capability set is a
   superset of the required stable-key capability set.

Identity rule:
1. the required capability set is the admission identity,
2. producer class and full producer capability set do not replace that
   identity.

## Producer Capability Publication

Producer capability publication should be represented as typed metadata on the
producer or returned rich/sparse carrier, with reserved fields:
1. `producer_capability_set_keys`
2. `exercised_capability_keys`

Current non-`IMAGE` empty output means the producer/exercised capability facts
are not yet emitted; it is not a support claim. `IMAGE` now publishes
registry-level `_webimage` producer capability keys only.

## Version Signal

OxFunc reserves:

```text
arg_admission_metadata_version
```

This signal should invalidate prepared packages when a function's
argument-preparation/admission profile changes, including later switches to
rich or sparse admission. Conservative all-function invalidation is acceptable
until a narrower affected-function fingerprint exists.

## First Activation Lane

Preferred first concrete rich producer lane:
1. `IMAGE` / `_webimage` producer capability publication, because OxFunc
   already has a pinned rich-value return carrier and OxFml has local
   `TypedContextQueryFamily::Image` evidence.

Deferred sparse-reader lane:
1. aggregate reducers over sparse range readers, after a concrete sparse
   reader API and replay-visible sparse iteration semantics are specified.

## Replacement Plan

This packet is replaced by an identity-first split:
1. OxFml owns template-hole identity, capability-set identity, mismatch
   surfaces, and replay columns.
2. OxFunc owns producer/kernel admission metadata, producer capability
   publication, versioning, and activation.
3. OxCalc keeps local empty/reserved evidence until receiving repos emit
   canonical producer and exercised capability facts.

## Non-Claims

1. No `ArgPreparationProfile::RichArgAccepted` Rust variant exists in the
   current OxFunc source.
2. No sparse-reader admission profile exists in the current OxFunc source.
3. No current kernel consumes rich or sparse producers through this metadata.
4. No generic rich producer protocol or rich-kernel admission is currently
   emitted by OxFunc; only `IMAGE` registry metadata publishes `_webimage`
   producer capability keys.

## Status

- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - Rust metadata model for rich/sparse admission
  - producer capability publication fields
  - OxFml/OxCalc consumption of `arg_admission_metadata_version`
  - first `IMAGE` / `_webimage` producer capability activation and tests
  - later sparse range reader API and kernel activation
