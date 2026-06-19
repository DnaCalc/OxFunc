# ODR-FN-003: Single OxFunc↔Excel Discrepancy Catalog

- **Status**: accepted
- **Date**: 2026-06-19
- **Context**: <see below>
- **Decision**: <see below>
- **Consequences**: <see below>
- **Cross-repo impact**: none directly; the Category-1 split (ODR-FN-002) still routes
  context-sensitive discrepancies to the downstream-evaluated catalog.

## Context

OxFunc targets bit-exact Excel parity across ~507 surfaces. Tracking which surfaces still
diverge had spread across many places: per-stream `.md` docs, `BUG_STREAM_REGISTER.csv`,
`KNOWN_EXACTNESS_DEVIATIONS.md`, the smart-fuzzer `FUNCTION_STATUS_MAP` (`mixed_or_open`),
and scattered inline notes. The same function appeared in several of these with no single
authoritative view, and known non-matches with no stream were effectively invisible. The
space is too large to navigate without one coherent index.

## Decision

There is **one canonical live tracker** of open OxFunc-vs-Excel discrepancies:

1. **Category 2 (context-free, OxFunc-locally-testable):**
   [`docs/OXFUNC_EXCEL_DISCREPANCY_CATALOG.md`](../OXFUNC_EXCEL_DISCREPANCY_CATALOG.md).
   Every open Category-2 discrepancy is a row, grouped by issue type then function cluster,
   tagged with severity (`STR` / `NUM-L` / `NUM-S`) and maturity (`M0`…`M3` / `HO`). A
   function may appear in multiple rows for distinct discrepancy types.
2. **Category 1 (context-sensitive):**
   `smart-fuzzer/corpus/context_sensitive_catalog/` carries known context-sensitive
   discrepancies as catalog rows with a `status` field, alongside its seam examples.

The catalog holds **open** items only. A signed-off fix is **removed** from the catalog —
durable per-fix history stays in git and the stream register; transferable rules of thumb
go to `docs/OXFUNC_FIX_LEARNING_LOG.md`. We do not accumulate fixed-case detail.

The detailed `docs/bugs/streams/BUG-FUNC-*.md` records remain the root-cause / evidence
surface; the catalog points to them. `BUG_STREAM_REGISTER.csv` remains stream provenance
(including closed history). `KNOWN_EXACTNESS_DEVIATIONS.md` is **superseded for live
tracking** by the catalog and is retained only as evidence detail for the numeric-drift
families.

## Consequences

1. One place to answer "what still diverges, how bad, how far along" — the catalog.
2. The 28 `mixed_or_open` un-streamed surfaces are now visible as catalog rows (`G8`,
   `M0`), turning a blind spot into a triage worklist.
3. Maintenance discipline: update the catalog on discovery and on sign-off (remove the
   row); keep KED/register/streams as supporting detail, not parallel trackers.
4. Severity + maturity tags let work be prioritized (e.g. structural `STR` before fine
   `NUM-S`) and progress be seen without counting steps.

## Cross-repo impact

None directly. Category-1 rows continue to be evaluated downstream per ODR-FN-002; the
catalog only changes how OxFunc-local discrepancy status is organized.
