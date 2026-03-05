# Value Universe Preliminary Spec (W3)

Status: `active`
Workset: `W3`

## 1. Purpose
Define a boundary-scoped value universe for OxFunc/F3E so function contracts, coercion rules, and formal/runtime artifacts use one shared tag vocabulary.

## 2. Boundary Sets
The universe is decomposed into boundary-specific sets:
1. `CellContentValue`
2. `EvalValue`
3. `CallArgValue`
4. `ReferenceLike`
5. `ExtendedValue`

Interpretation rule:
1. these sets are boundary views over one common tag algebra,
2. not every tag is admitted at every boundary.

## 3. Tag Algebra
Canonical tag list for W3 baseline:
1. `number`
2. `text`
3. `logical`
4. `error`
5. `array`
6. `reference_like`
7. `missing_arg`
8. `empty_cell`
9. `lambda_value`
10. `extended_wrapper`
11. `null_like` (reserved disputed category; not admitted in baseline boundaries)

Machine-readable table:
1. `VALUE_UNIVERSE_TAG_TABLE.csv`

## 4. Baseline Boundary Admission Policy
1. `CellContentValue` admits:
   - `number`, `text`, `logical`, `error`, `empty_cell`
2. `EvalValue` admits:
   - `number`, `text`, `logical`, `error`, `array`, `reference_like`, `lambda_value`
   - and explicitly does **not** admit `missing_arg`, `empty_cell`, `null_like`
3. `CallArgValue` admits:
   - all `EvalValue` tags plus `missing_arg` and `empty_cell`
4. `ReferenceLike` boundary admits:
   - `reference_like` only
5. `ExtendedValue` boundary admits:
   - `extended_wrapper` plus pass-through of core evaluable tags

## 5. Disputed Categories
### 5.1 Missing
1. represented as `missing_arg`,
2. treated as call-boundary specific, not eval-result specific.

### 5.2 Empty
1. represented as `empty_cell`,
2. treated as cell/call-boundary representable, not eval-result specific.

### 5.3 Null
1. represented only as reserved `null_like` tag in baseline algebra,
2. not admitted in any baseline boundary until direct evidence exists.
3. `#NULL!` is modeled as `error`, not `null_like`.

## 6. Error Taxonomy and Versioning
Error values remain scalar `error` tags with code-level metadata.

Code-level registry split (provisional):
1. legacy transferable family (`#NULL!`, `#DIV/0!`, `#VALUE!`, `#REF!`, `#NAME?`, `#NUM!`, `#N/A`)
2. extended worksheet-era family (for example `#SPILL!`, `#CALC!`, `#FIELD!`, `#BLOCKED!`, `#CONNECT!`)

Versioning rule:
1. code family membership is version-scoped by Excel build/channel and compatibility mode.

## 7. Arrays, Lambda, and References
1. arrays are first-class `EvalValue` tags; materialization policy is downstream (W4/W5/W6).
2. lambda values are intermediate-eval tags, not baseline cell content tags.
3. 3D references are modeled as a `reference_like` subtype (`reference_kind=three_d`) and require resolver-policy handling in W4.

## 8. Lean/Rust Mirror Rule
W3 baseline requires shared tag vocabulary in:
1. Rust: `crates/oxfunc_core/src/value.rs`
2. Lean: `formal/lean/OxFunc/ValueUniverse.lean`

Invariant objective for baseline:
1. `EvalValue` excludes `missing_arg`, `empty_cell`, `null_like`.

## 9. Integration Hooks
1. W4 consumes tag/boundary policy for coercion and `Ref -> EvalValue` seam typing.
2. W5 consumes numeric/error/array policy for `ABS`.
3. W6 consumes text/reference/error policy for `XMATCH`.

## 10. Open Points
1. explicit handling for modern dynamic-array error transfer across UDF/XLL boundaries,
2. empirical evidence for any first-class `null_like` behavior,
3. final W7-informed text/collation constraints before W3 validation closure.
