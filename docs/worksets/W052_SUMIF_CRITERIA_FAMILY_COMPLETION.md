# WORKSET - SUMIF Criteria-Family Completion (W52)

## 1. Purpose
Close the missing `SUMIF` member on the already-established criteria-family substrate so OxFunc can report the current-baseline `SUMIF` function honestly rather than carrying a catalog-only hole.

Primary intent:
1. pin the `SUMIF`-specific current-baseline target-range behavior in native Excel,
2. align the Rust and Lean criteria-family substrate to include `SUMIF`,
3. promote the contract/evidence/export artifacts so `SUMIF` is no longer a latent gap behind the catalog and snapshot surfaces.

## 2. Position and Dependencies
Program position:
1. focused successor to the closed seven-function criteria-family packet,
2. narrow current-baseline closure packet for the standalone `SUMIF` gap exposed by the OxFml adapter corpus,
3. provenance follows `W016` / `W022` for the shared criteria kernel and `W044` for published snapshot refresh.

Dependencies:
1. `W004` coercion/reference primitives,
2. `W016` criteria-family baseline,
3. `W022` criteria-family shape-hardening packet,
4. `W044` library-context snapshot export baseline.

## 3. Scope
In scope:
1. native Excel replay for `SUMIF`-specific omitted-target, anchored-target, numeric-only target, and target-error propagation lanes,
2. OxFunc runtime/export wiring for `SUMIF`,
3. Lean metadata/alignment update for the criteria-family substrate,
4. contract, conformance, correlation, evidence, and published-export refresh for `SUMIF`.

Out of scope:
1. re-opening the already-closed criteria parser and `*IFS` family substrate,
2. locale/version sweeps,
3. broader adapter-corpus residuals unrelated to `SUMIF`,
4. any change to evaluator-facing clause shapes beyond confirming the existing refs-visible criteria-family admission rule.

## 4. Working Thesis
`SUMIF` should follow the same single-criteria target-range rule as `AVERAGEIF`, not the exact-shape rule of the `*IFS` family:
1. omitted `sum_range` uses the criteria range directly,
2. an explicit mismatched A1-style `sum_range` anchors from its top-left reference across the criteria-range shape,
3. target aggregation remains numeric-only and propagates a reached worksheet error.

## 5. Deliverables
1. `SUMIF` native replay manifest, runtime requirements, runner, and result artifact,
2. `SUMIF` completion execution record,
3. promoted criteria-family contract/conformance/evidence/correlation updates,
4. refreshed generated export and library-context snapshot artifacts.

## 6. Gate Model
### G1 - Empirical Pinning
Pass when:
1. native Excel rows exist for omitted `sum_range`, anchored mismatched `sum_range`, numeric-only target aggregation, and target-error propagation,
2. the replay artifact matches the expected current-baseline rule set.

### G2 - Runtime and Formal Alignment
Pass when:
1. `SUMIF` is admitted through OxFunc runtime dispatch and export tables,
2. Rust tests cover the admitted `SUMIF` lanes,
3. Lean criteria-family metadata/alignment includes `SUMIF`.

### G3 - Artifact Promotion
Pass when:
1. contract/conformance/evidence/correlation artifacts explicitly include `SUMIF`,
2. generated XLL export and library-context snapshot artifacts no longer expose `SUMIF` as catalog-only.

## 7. Status
Execution state:
1. `complete`

Claim confidence:
1. `provisional`

Assurance maturity:
1. `exercised-locally`

## 8. Completeness Axes
1. `scope_completeness`: `scope_complete`
2. `target_completeness`: `target_complete`
3. `integration_completeness`: `integrated`
4. `open_lanes`:
   - broader locale/version sweeps remain orthogonal validation work.
