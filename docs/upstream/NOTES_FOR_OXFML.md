# Notes for OxFml

Status: `active`
Owner lane: `OxFml`
Relationship: current OxFunc-to-OxFml seam note

## 1. Purpose

Capture the current OxFunc reading of what OxFml should preserve or prove next at the seam.

This is a current-state note, not a historical ledger.
It keeps only the distinctions, ownership splits, and bounded asks that still matter for current OxFunc closure work.

## 2. Current Summary

Current OxFunc reading:
1. the admitted `@` slice is real in OxFunc runtime, Lean alignment, native Excel replay, and the current OxFml adapter corpus.
2. the admitted `W038` callable/helper slice is also real in OxFunc runtime, Lean alignment, native Excel replay, and the current OxFml adapter corpus.
3. `HYPERLINK` is no longer a missing OxFunc kernel; it is value semantics plus publication intent, with actual style/click behavior above OxFunc.
4. `IMAGE` remains a real rich-value/publication seam.
5. `CALL` / `REGISTER.ID` remain a real typed registered-external provider/admission/runtime seam.
6. `GROUPBY` and `PIVOTBY` now have real OxFunc callable-backed kernels, but still need stronger adapter-level proving coverage before honest closure promotion.

## 3. Current Seam Floor OxFunc Depends On

OxFunc currently depends on these seam facts remaining true:
1. explicit `@` remains observable and caller-context-sensitive rather than collapsing into generic top-left array picking.
2. callable/helper artifacts remain semantically real at the seam.
3. direct scalar, array-like, omitted, blank, and reference-observable distinctions are not erased prematurely.
4. bind-time helper rejection stays separate from evaluation-time function semantics.
5. result-class distinctions for publication-sensitive functions survive planning and evaluation.

## 4. `@` / `OP_IMPLICIT_INTERSECTION`

Current OxFunc reading:
1. `@` is no longer blocked by a missing OxFunc kernel.
2. the admitted current-baseline slice is already covered by:
   - Rust runtime
   - Lean binding
   - native replay
   - OxFml adapter cases `B01` through `B07`
3. the remaining live work is:
   - compatibility-version and `_xlfn.SINGLE(...)` roundtrip characterization
   - structured-reference/table-context interaction outside the admitted slice

Current OxFml implication:
1. preserve explicit `@` provenance and caller-context scalarization semantics
2. do not normalize `@` into a generic array-top-left shortcut

## 5. Helper / Callable Family (`W038`)

Current OxFunc reading:
1. `LET`, `LAMBDA`, `ISOMITTED`, `MAP`, `REDUCE`, `SCAN`, `BYROW`, `BYCOL`, and `MAKEARRAY` are no longer blocked by missing OxFunc kernels on the admitted slice.
2. the current adapter corpus already proves admitted direct/helper/higher-order lanes through the real OxFml parser/binder/preparation path.
3. the remaining live seam pressure is narrowed to:
   - bind-time rejection parity where Excel rejects before evaluation
   - final callable-carrier tightening

Current OxFml implication:
1. keep helper formation and validation on the bind/admission side where Excel already rejects before evaluation
2. preserve prepared callable/helper distinctions honestly

## 6. `GROUPBY` / `PIVOTBY`

Current OxFunc reading:
1. OxFunc now has real callable-backed runtime kernels for both functions on an admitted current-baseline slice.
2. these rows are no longer blocked on missing OxFunc callable infrastructure.
3. the next proving gap is real OxFml adapter coverage for callable-backed grouped aggregation.

Current OxFml implication:
1. the adapter should be able to carry the same callable/helper seam facts already proven for `W038` into `GROUPBY` and `PIVOTBY`.
2. OxFunc does not need a new generic callable ABI round to proceed.
3. OxFunc does need bounded real adapter cases that exercise these functions through the actual seam.

## 7. `HYPERLINK` / `IMAGE`

### 7.1 `HYPERLINK`

Current OxFunc reading:
1. OxFunc owns worksheet-visible value semantics and publication intent.
2. the current OxFunc return split is:
   - value: visible text payload
   - publication hint: hyperlink-style/clickability intent
3. actual style application and click behavior remain host-owned.

Current OxFml implication:
1. do not collapse `HYPERLINK` to plain text if richer publication metadata can survive the seam
2. do not model `HYPERLINK` as if OxFunc itself performs host UI mutation

### 7.2 `IMAGE`

Current OxFunc reading:
1. `IMAGE` is still an open rich-value/publication seam.
2. OxFunc does not currently want `IMAGE` scalarized into plain text, a URL string, or a fake placeholder scalar.

Current OxFml implication:
1. preserve the semantic class that `IMAGE` is richer than plain text or ordinary reference return
2. keep enough result-class/capability truth for a host-managed rich-value/publication model

## 8. `CALL` / `REGISTER.ID`

Current OxFunc reading:
1. OxFunc already has typed request normalization and worksheet result projection for the admitted slice.
2. the remaining open work is not ordinary function-kernel work.
3. the remaining open work is:
   - host-backed registered-external provider behavior
   - broader admission/omitted-`type_text` matrix
   - final runtime provider/snapshot ownership tightening

Current OxFml implication:
1. keep `RegisteredExternalProvider` distinct from ordinary host-info/query seams
2. preserve typed registration and invocation packets as real runtime objects, not just note-level ideas

## 9. Bounded Adapter Expansion Requested

OxFunc's current bounded ask to OxFml is:
1. add one or more real `GROUPBY` adapter cases:
   - built-in aggregation callable lane such as `SUM`
   - prepared lambda lane if admitted by the current carrier
   - at least one totals/filter/header/sort-sensitive lane
2. add one or more real `PIVOTBY` adapter cases:
   - default callable-backed pivot lane
   - at least one totals/filter/header-band lane
3. add bind-time rejection adapter cases for helper forms that should fail before OxFunc evaluation:
   - duplicate `LET` names
   - duplicate `LAMBDA` parameter names
   - malformed helper lambda declarations already pinned in `W38`

## 10. What OxFunc Is Not Asking For

OxFunc is not currently asking OxFml for:
1. a final callable ABI
2. a generic provenance redesign
3. a generic callable-note round
4. broad one-shot completion of the full `GROUPBY` / `PIVOTBY` option matrix
5. premature scalarization of `IMAGE`

## 11. Current Closing Sequence

Current OxFunc reading of the best next sequence is:
1. use the existing adapter floor to finish narrowing `W038` and `W014`
2. extend the adapter with bounded `GROUPBY` / `PIVOTBY` and helper-bind rejection cases
3. keep `HYPERLINK` / `IMAGE` publication-class distinctions explicit
4. continue `CALL` / `REGISTER.ID` as a typed registered-external seam packet rather than ordinary function work

## 12. Current Summary To OxFml

Current OxFunc position to OxFml:
1. `@` and the admitted helper family are already real end-to-end seam facts, not note-only topics.
2. the next useful adapter work is callable-backed grouped aggregation plus bind-time helper rejection coverage.
3. `HYPERLINK` should preserve publication intent, and `IMAGE` should preserve rich-value/publication classification.
4. `CALL` / `REGISTER.ID` remain a typed registered-external seam, not an ordinary function-family cleanup lane.
