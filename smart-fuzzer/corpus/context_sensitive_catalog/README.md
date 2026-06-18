# Context-Sensitive Invocation Catalog (Category 1)

Status: `published_catalog`

Owning workset: `docs/worksets/W104_INVOCATION_TEST_CATEGORY_SPLIT_AND_CONTEXT_SENSITIVE_CATALOG.md`
Owning decision: `docs/decisions/ODR-FN-002-invocation-test-category-split.md`

## What this is

This directory is the **published Category-1 catalog** under
[ODR-FN-002](../../../docs/decisions/ODR-FN-002-invocation-test-category-split.md):
function invocations whose result depends on references, implicit intersection,
spill neighborhood, reference transforms, caller location, host/provider state, or
formula-binding scope.

These rows are **context-sensitive**. They cannot be evaluated honestly from OxFunc
without a faithful OxFml/OxCalc context, and standing up local scaffolding that fakes
that context would just re-implement downstream work and prove the mock instead of the
product. So this catalog follows one rule:

> **Publish here. Do not fake context here. Evaluate through a downstream-driven runner
> over OxCalc → OxFml → OxFunc.**

This catalog **is part of the smart-fuzzer's testing scope** — it is not an out-of-scope
register. It is the seed corpus for a Category-1 smart-fuzzer runner that drives the
downstream **OxCalc → OxFml → OxFunc** stack as its evaluation engine (with its own
oracle/comparison), so references, implicit intersection, spill, host/provider, locale,
and formula-binding context are real. **That runner does not exist yet** — it is the next
infrastructure lane after the Category-2 local runner. Until it lands, these rows are
published and grown but not executed, and the `expected_behavior` field is **prose for
that downstream runner**, not a machine-checked oracle value evaluated here.

The complement — context-free invocations OxFunc *can* drive directly against Excel —
is Category 2, handled today by the smart-fuzzer's existing local-Rust + Excel-COM
harness. See `smart-fuzzer/planning/CATEGORY_B_EVALUATION_CLASS_PROBE_PLAN.md`. As the
smart-fuzzer grows into AFL-style feedback-guided exploration, both categories are in
scope: the same mutation/coverage/comparison techniques apply, with Category 1 mutated
and explored by the future downstream-driven runner.

## Files

1. `catalog-v0.json` — the catalog data (see schema below).
2. this `README.md` — index, schema, and policy.

## Schema

`catalog-v0.json` is a JSON object:

```json
{
  "schema_version": "oxfunc.context_sensitive_catalog.v0",
  "authority": "published_for_downstream_evaluation",
  "evaluation_policy": "not_evaluated_in_oxfunc",
  "downstream_path": "OxCalc->OxFml->OxFunc",
  "entries": [ /* CatalogEntry[] */ ]
}
```

Each `CatalogEntry`:

| Field | Meaning |
|-------|---------|
| `catalog_id` | Stable id, `CSC-NNNN`. |
| `seam_class` | One of the seam classes below — why it is context-sensitive. |
| `surface_name` | Canonical function/operator surface (e.g. `INDEX`, `INDIRECT`, `OP_IMPLICIT_INTERSECTION`). |
| `formula` | The formula text to evaluate downstream. |
| `caller` | Caller cell `{sheet,row,col}` when the result depends on caller location; else `null`. |
| `cell_fixture` | Worksheet cells/ranges/tables the formula reads, as `{target,value}` rows. Describes the workbook the downstream evaluator must build. |
| `why_context_sensitive` | Prose: the exact dependency that blocks local OxFunc evaluation. |
| `expected_behavior` | Prose: the expected Excel behavior for the downstream evaluator to assert against. Not a machine oracle value here. |
| `source_ref` | Originating bug/handoff/workset, when the row came from one (e.g. `HO-FN-018`). |

## Seam classes

1. `implicit_intersection` — explicit `@` and value-context coercion of array/range operands.
2. `reference_transform` — reference-returning `INDEX`/`OFFSET`, `ADDRESS`, `AREAS`, range composition.
3. `host_context` — `INDIRECT`, `CELL`, `INFO`, `FORMULATEXT`, `SHEET`/`SHEETS`, time providers, RTD/cube/web.
4. `structured_reference` — table/structured references such as `Table1[Col]`.
5. `cross_sheet_reference` — `Sheet2!A1`, 3-D `Sheet1:Sheet2!A1`.
6. `spill_anchor` — spill-range references such as `A1#`.
7. `formula_binding` — `LET`, `LAMBDA`, `BYROW`, `BYCOL`, `MAP`, `REDUCE`, `SCAN`, `MAKEARRAY`, `ISOMITTED`.
8. `locale_context` — functions whose text↔number parsing/formatting depends on a
   locale-format capability bundle OxFunc does not own locally (W082), e.g. `VALUE`,
   `TEXT`, `DATEVALUE`, `TIMEVALUE`, `NUMBERVALUE`.

## How downstream consumes this

The OxCalc→OxFml→OxFunc path builds the `cell_fixture` workbook, evaluates `formula`
from `caller`, and checks the observed result against `expected_behavior` (and, where
that path has an Excel oracle, against Excel). Mismatches found downstream are routed
through that path's bug intake, not OxFunc's local `docs/bugs/` unless the root cause is
proven to be OxFunc-owned.

## Graduation rule

A row leaves Category 1 only when it can be reduced to a genuinely context-free form
(literals / typed fixtures / array literals, single `Formula2`) **without faking
context**. At that point it moves to a Category-2 probe set and the catalog entry is
retired with a pointer to its new home.
