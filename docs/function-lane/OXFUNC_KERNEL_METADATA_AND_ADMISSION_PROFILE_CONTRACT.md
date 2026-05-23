# OxFunc Kernel Metadata And Admission Profile Contract

Status: `canonical_contract_seed`
Owner: OxFunc
Last updated: 2026-05-23

## 1. Purpose

This contract records OxFunc-owned metadata and kernel-admission decisions
needed by downstream formula and calculation engines.

Current scope:
1. numerical-reduction and worksheet-error collapse selector ownership from
   `HANDOFF-CALC-003`,
2. rich/sparse argument-admission metadata reservations from
   `HANDOFF-CALC-004`,
3. prepared-package invalidation signals when bind-visible metadata changes.

This file is a contract and ownership surface. It only claims current Rust
behavior where the referenced code and tests exercise the path.

## 2. CALC-003 Correctness-Floor Selectors

OxFunc accepts ownership of function/kernel metadata for:
1. reduction-sensitive kernels,
2. error-collapse-sensitive kernels.

OxFunc accepts ownership of the exact kernel-side semantics for:
1. `NumericalReductionPolicy`,
2. `ErrorAlgebra`.

OxFml owns context carriage through semantic plans, sessions, prepared calls,
and replay projections. OxCalc owns coordinator/profile selection and
prepared-package invalidation policy consumption.

### 2.1 NumericalReductionPolicy

Canonical initial selector keys:
1. `SequentialLeftFold`
2. `PairwiseTree`
3. `KahanCompensated`

Required semantics:
1. `SequentialLeftFold` reduces numeric inputs in recorded logical input
   order, left to right, with each admitted numeric operand applied once.
2. `PairwiseTree` reduces over the same recorded logical input order using a
   deterministic tree whose shape identity is replay-visible.
3. `KahanCompensated` treats compensation state as semantic algorithm state,
   not as an optional optimization.

Current OxFunc decision:
1. accept the selector vocabulary and semantic ownership reservation,
2. publish registry-level `semantic_kernel_metadata` and
   `semantic_kernel_metadata_version` fields for current first-family
   classification,
3. implement the current runtime boundary in
   `crates/oxfunc_core/src/semantic_kernel.rs`,
4. exercise `SequentialLeftFold` through `SUM` prepared aggregate evaluation,
5. defer `PairwiseTree`, `KahanCompensated`, replay tree-shape fields, and
   compensation-state replay fields until a successor implementation lane.

### 2.2 ErrorAlgebra

Canonical initial selector key:
1. `CanonicalExcelLegacy`

Required worksheet-error precedence for `CanonicalExcelLegacy`:
1. `#NULL!`
2. `#DIV/0!`
3. `#VALUE!`
4. `#REF!`
5. `#NAME?`
6. `#NUM!`
7. `#N/A`

Rules:
1. Any non-canonical error algebra must use a new selector key and
   `profile_version`.
2. A non-canonical algebra must define a total precedence order over every
   admitted worksheet-error code and explicit placement for newly admitted
   error codes.
3. Replay under one error algebra is invalid under another unless an explicit
   migration proof exists.

Current OxFunc decision:
1. accept the precedence definition as the first canonical OxFunc-owned
   worksheet-error collapse algebra,
2. publish registry-level `semantic_kernel_metadata` and
   `semantic_kernel_metadata_version` fields for current first-family
   classification,
3. implement the current helper boundary as
   `collapse_worksheet_errors(ErrorAlgebra::CanonicalExcelLegacy, ...)`,
4. keep broad per-kernel conversion in progress until each affected family has
   function-specific evidence for its worksheet-error collapse path.

### 2.3 Metadata And Version Signal

OxFunc reserves a bind-visible metadata signal for selector behavior changes:

```text
semantic_kernel_metadata_version
```

Minimum interpretation:
1. The signal is part of OxFunc-published function metadata or a linked
   registry/profile metadata object.
2. Any change that can alter reduction selector handling, error-algebra
   handling, or affected-function classification must advance the signal.
3. Until narrower affected-function metadata exists, OxFml and OxCalc may
   conservatively invalidate all prepared packages that rely on OxFunc
   function metadata.
4. A later narrower plan may add per-function or per-family fingerprints, but
   must preserve conservative invalidation safety.
5. Current Rust publishes this through `RegistryFunctionMeta` and
   `render_registry_metadata_csv(...)`.
6. OxFml should carry this field in prepared identity, runtime artifacts, and
   replay artifacts; any changed value is a prepared-package invalidation
   signal.

## 3. First CALC-003 Affected Families

First affected or review-required families:
1. aggregate numeric reducers: `SUM`, `SUMSQ`, `PRODUCT`, `AVERAGE`, `MIN`,
   `MAX`, `SUBTOTAL`, `AGGREGATE`,
2. conditional/database reducers: `SUMIF`, `SUMIFS`, `AVERAGEIF`,
   `AVERAGEIFS`, `COUNTIF`, `COUNTIFS`, database aggregate functions,
3. array and matrix numeric reducers: `SUMPRODUCT`, `MMULT`, `MDETERM`,
   `MINVERSE`, `SUMX2MY2`, `SUMX2PY2`, `SUMXMY2`,
4. helper/callable reducers: `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, `MAP`,
   `GROUPBY`, `PIVOTBY`,
5. criteria and selection families where multiple candidate errors can be
   observed and collapsed,
6. shared helpers that aggregate range/array cells or choose one worksheet
   error from several candidates.

Explicitly deferred:
1. provider/cube functions under W041,
2. RTD and external-provider lifecycle lanes under W043,
3. locale and alternate Excel-version sweeps unless reopened by a concrete
   workset,
4. functions with no numeric reduction and no multi-error collapse path until
   a specific kernel review proves otherwise.

## 4. CALC-004 Rich And Sparse Admission

OxFunc accepts the identity reservation for an OxFunc-owned metadata profile
equivalent to:

```text
RichArgAccepted(required_capability_set)
```

This is accepted as a metadata/profile shape plus a deterministic capability
set mismatch helper. No current built-in consumes a rich producer as an
argument.

### 4.1 Capability Set

Initial rich-value capability selector families:
1. `Indexable`
2. `Enumerable`
3. `Shaped`
4. `Materialisable`

Rules:
1. Each capability must have a stable typed key.
2. Required capability-set identity is the sorted, deduplicated set of stable
   keys.
3. A producer is admissible only when its published stable-key set is a
   superset of the required set.
4. The required capability set remains the identity of the hole/admission
   requirement; it is not replaced by producer class or by the producer's full
   capability set.
5. Current Rust exposes this through
   `ArgAdmissionMetadata::RichArgAccepted { required_capability_set_keys }`
   and validates the requirement with deterministic missing-key reporting.
6. Current mismatch timing is admission/preparation-time when a downstream
   caller asks OxFunc to validate a producer capability set against the
   metadata requirement. Runtime and replay mismatch fields remain downstream
   integration work because no current built-in rich-argument consumer is
   active.

### 4.2 Sparse Reader Admission

OxFunc accepts sparse-reader admission as a successor metadata lane. The first
runtime reader API exists as `ReferenceResolver::resolve_reference_values(...)`.
It is now exercised for the first aggregate group plus selected shared
range-taking adapters and reference-visible lanes listed below; broader
metadata admission names and replay fields remain successor work.

Reserved conceptual profile:

```text
SparseRangeAccepted(extent_class, cardinality_class)
```

Current decision:
1. accept the sparse-reader identity/admission lane,
2. adapt the exact name later to match Rust metadata design,
3. use `ResolvedReferenceValues` as the current concrete sparse reader carrier
   for admitted current Rust lanes,
4. defer full metadata/replay promotion and whole-surface family activation.

Minimum sparse-reader semantics for the later API:
1. `Defined` includes assigned cell values, including empty-string text.
2. `Blank` covers never-assigned and assigned-then-cleared cell values at the
   value layer.
3. Structural state that survives clear operations is not owned by the sparse
   value reader.

Current activation:
1. `SUM`, `COUNT`, `COUNTA`, and `COUNTBLANK` call
   `resolve_reference_values(...)` before dense reference materialization for
   `CallArgValue::Reference` and `EvalValue::Reference` inputs.
2. Shared aggregate/range adapters now also try sparse readers for ordinary
   aggregate/statistical/logical representatives using `expand_aggregate_arg`,
   text range-scan representatives using `expand_arg_values_only`, and
   lookup/match representatives using `expand_lookup_vector_arg`.
3. Criteria aggregate representatives use the same generic sparse materialized
   value lane where the resolver provides aligned extents.
4. `ROWS` and `COLUMNS` can read sparse declared extents; `INDEX` can validate
   bounds from the sparse extent and return an opaque projected
   `ReferenceLike`.
5. `SUBTOTAL` and `AGGREGATE` can combine sparse values with host-provided
   `AggregateReferenceContext`.
6. `ReferenceKind::Structured` table carriers are admitted as opaque targets
   when `allow_structured_refs` is enabled; OxFunc does not parse table names,
   sections, column ids, current-row selectors, or TreeCalc table ids.
7. Bracketed structured-reference targets are not treated as external workbook
   references by OxFunc's generic external-reference guard; table/external
   availability is resolver/provider-owned for structured carriers.
8. The exercised W056 guardrail covers table data column, whole data section,
   header section, totals section, and current-row carrier forms through
   opaque sparse readers.
9. `COUNTBLANK` uses the sparse declared extent plus defined cells to count
   blanks without requiring dense materialization; missing cells in the sparse
   reader are blank at the value layer, and defined empty-string text is also
   counted as blank for `COUNTBLANK`.
10. If structured references are denied, rejection is the generic
   `CapabilityDenied { kind: Structured, capability: "allow_structured_refs" }`
   path before provider calls.

Current W056 range-taking inventory:
1. `supported_sparse_defined-cell_aggregate_group`: `SUM`, `COUNT`, `COUNTA`,
   `COUNTBLANK`, and shared `expand_aggregate_arg` users such as `AVERAGE`,
   `MAX`, ordinary statistical reducers, `AND`, and `OR`. The guardrail
   exercises representative functions; functions on the shared adapter inherit
   the same reader mechanics but still need function-specific semantic evidence
   before broader Excel parity claims.
2. `supported_sparse_materialized_value_group`: `TEXTJOIN` as a text
   range-scan representative, `MATCH` / `XLOOKUP` lookup-vector and return
   representatives, and criteria aggregate representatives including
   `COUNTIF` / `SUMIFS` where resolver-provided extents are aligned.
3. `supported_sparse_extent_or_projection`: `ROWS`, `COLUMNS`, and `INDEX`.
   `INDEX` preserves an opaque projected `ReferenceLike` target and does not
   parse structured selectors.
4. `host_context_supported`: `SUBTOTAL` / `AGGREGATE` require
   `AggregateReferenceContext`; `AREAS` counts opaque reference areas;
   `FORMULATEXT` and host-owned `CELL` info types pass the original
   `ReferenceLike` to `HostInfoProvider`.
5. `generic_dense_resolver_supported`: functions whose current implementation
   first dereferences references into `EvalValue`/`EvalArray` through
   `resolve_eval_value(...)`. These can consume structured-table carriers only
   when the resolver materializes them; this is not sparse-reader support and
   does not prove table-specific context semantics.
6. `typed_exclusion_or_downstream_context_owned`: `ROW` / `COLUMN` caller
   context lanes, `CELL("address"|"row"|"col")`, `OFFSET`, dynamic-array
   selector/reshape functions, volatile reference construction, `CALL`,
   `OP_IMPLICIT_INTERSECTION`, `OP_SPILL_REF`, and reference range/intersection
   operators do not gain TreeCalc/table selector semantics in OxFunc. They are
   either host-context lanes, A1/operator lanes, or opaque carrier lanes until a
   generic resolver/context API supplies the needed facts.
7. `downstream_context_owned`: structured-table name precedence, table context
   identity, selected section/region availability, header/totals/current-row
   existence, TreeCalc table selectors, and table dependency/invalidation facts
   are OxFml/OxCalc inputs, not OxFunc branches.

### 4.3 Producer Capability Publication

Producer capability publication should be represented as metadata on the
producer or returned rich/sparse carrier, not as a function-name-specific
exception.

Reserved publication fields:
1. `producer_capability_set_keys`
2. `exercised_capability_keys`

Interpretation:
1. producer keys describe what the value/carrier can provide,
2. exercised keys describe what a kernel actually invoked,
3. empty keys in current V1 traces mean no producer/exercised capability was
   emitted, not support for all capabilities.
4. Current Rust publishes registry-level producer keys through
   `RegistryFunctionMeta` and `render_registry_metadata_csv(...)`.
5. Current Rust also exposes adjacent runtime facts for `IMAGE` through
   `eval_image_surface_extended_with_capabilities(...)`; successful `_webimage`
   results copy the producer keys into `exercised_capability_keys`, while
   provider failure/capability-denied results leave both runtime lists empty.

### 4.4 Rich/Sparse Metadata Version Signal

OxFunc reserves a bind-visible metadata signal for rich/sparse admission
changes:

```text
arg_admission_metadata_version
```

Rules:
1. Adding the metadata vocabulary is an identity reservation.
2. Switching an existing function argument from `ValuesOnlyPreAdapter` or
   `RefsVisibleInAdapter` to rich/sparse admission changes bind-visible
   metadata and must advance the signal.
3. Until narrower affected-function metadata exists, OxFml and OxCalc may
   conservatively invalidate all prepared packages that rely on OxFunc
   argument-preparation metadata.
4. Current Rust exposes this as registry-level `arg_admission_metadata` and
   `arg_admission_metadata_version`, including through
   `render_registry_metadata_csv(...)`; current built-ins remain on existing
   ordinary argument-preparation profiles.
5. OxFml should carry this field in prepared identity, runtime artifacts, and
   replay artifacts; any changed value is a prepared-package invalidation
   signal.

## 5. First CALC-004 Activation Lane

First concrete lanes:
1. `IMAGE` / rich-value producer capability publication for `_webimage`.
2. W056 structured-table sparse `ReferenceLike` consumption for `SUM`,
   `COUNT`, `COUNTA`, `COUNTBLANK`, selected shared aggregate/statistical/
   logical/text/lookup/criteria representatives, `ROWS`, `COLUMNS`, `INDEX`,
   and host-context `SUBTOTAL` / `AGGREGATE` through opaque
   `ReferenceResolver::resolve_reference_values(...)` plus host context APIs.
3. Current code keeps registry-level keys and adjacent runtime facts separate:
   registry metadata describes what the producer can publish; the runtime
   wrapper describes what a successful run actually exercised.

Deferred alternative:
1. sparse range-reader metadata/replay promotion for the whole function
   surface, because each family still needs function-specific evidence and
   replay-visible sparse iteration semantics before promotion.

No current claim:
1. no current OxFunc kernel admits `RichArgAccepted`,
2. sparse-reader behavior is currently exercised only for the scoped
   representatives and shared adapter paths listed above; full table-context
   semantics and whole-surface sparse admission remain open,
3. only the `IMAGE` registry/runtime path currently publishes `_webimage`
   producer capability facts; no generic rich producer protocol or rich-kernel
   admission is claimed.

## 6. Status

- execution_state: in_progress
- scope_completeness: scope_partial
- target_completeness: target_partial
- integration_completeness: partial
- open_lanes:
  - downstream consumer migration to `render_registry_metadata_csv(...)` or the
    equivalent direct registry API
  - OxFml/OxCalc migration to canonical metadata/version signals and IMAGE
    runtime capability facts
  - replay field names for pairwise tree shape and Kahan compensation state
  - per-family `ErrorAlgebra` wiring after function-specific evidence
  - sparse range reader metadata/replay promotion beyond the current W056
    representative and shared-adapter evidence
  - downstream table-context semantics and host-context facts that OxFunc does
    not own
