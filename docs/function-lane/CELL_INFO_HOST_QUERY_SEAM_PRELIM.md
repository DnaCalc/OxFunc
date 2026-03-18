# CELL and INFO Host-Query Seam (Prelim)

Status: `active`
Owner lane: `OxFunc`

## 1. Purpose
Define the working OxFunc/FEC split for `CELL` and `INFO`.

The question is not whether these functions need host facts. They do.
The question is where to split:
1. query-language normalization and Excel-specific semantics,
2. acquisition of workbook/application/environment facts.

## 2. Working Position
`CELL` and `INFO` should not be modeled as arbitrary callbacks into the evaluator.

They should be modeled as ordinary OxFunc functions over typed host-query facilities:
1. OxFunc receives normalized arguments and preserved reference identity where applicable.
2. OxFunc classifies the query kind.
3. OxFunc asks the FEC-side facility for the required fact through a typed interface.
4. OxFunc maps the returned fact into the worksheet-visible result or error.

This keeps Excel semantics in OxFunc while keeping live host state in FEC/F3E.

## 3. Split by Function
### 3.1 `CELL`
`CELL` is mixed:
1. some lanes are OxFunc-local over the reference identity:
   - `address`
   - `row`
   - `col`
2. some lanes are OxFunc-local plus ordinary dereference:
   - `contents`
   - `type`
3. some lanes require cell/workbook metadata:
   - `filename`
   - `format`
   - `color`
   - `parentheses`
   - `prefix`
   - `protect`
   - `width`
4. omitted-reference forms (`CELL(info_type)`) require host-side active-selection knowledge even for query kinds that are otherwise computable from an explicit reference.

So `CELL` is not “just a callback”, but it cannot be completed without a cell-info capability.

### 3.2 `INFO`
`INFO` is closer to a pure host-query function:
1. `directory`
2. `numfile`
3. `origin`
4. `osversion`
5. `recalc`
6. `release`
7. `system`
8. memory-related lanes

OxFunc still owns `type_text` normalization and observed error policy, but the answers themselves come from the host/application/workbook context.

## 4. Proposed Capability Shape
The standard seam should be typed, not stringly.

Candidate shape:

```text
CellInfoProvider
WorkbookInfoProvider
ApplicationInfoProvider
EnvironmentInfoProvider
```

or one combined facade if the implementation surface prefers that.

The important rule is:
1. OxFunc classifies request kinds into enums,
2. FEC answers enum-based queries,
3. string parsing and worksheet error policy stay in OxFunc.

## 5. Why This Split Is Better
1. It avoids smuggling workbook inspection logic into every function kernel.
2. It lets OxFunc remain the owner of Excel-specific semantics and error policy.
3. It fits the established provider pattern used already for time/random/locale seams.
4. It keeps the FEC contract auditable: capabilities are explicit and typed.

## 6. Current Evidence
1. `W12-CELL-PRE-20260309` already showed that the initial `CELL` slice depends on preserved reference identity and dereference.
2. `W9-XLL-GETINFO-20260314` proved the tester-XLL can fetch legacy `GET.CELL` / `GET.WORKSPACE` / `GET.WORKBOOK` facts for grounding.
3. `W15-INFO-PRE-20260315` showed that `INFO` returns current host/workbook facts directly for seeded lanes and that the memory lanes currently return `#N/A` on this host.
4. `W15-XLL-BRIDGE-20260315` showed that the typed provider seam is sufficient to drive generated `ox_INFO` / `ox_CELL` exports through the current baseline without special macro-type registration for those exports.

## 7. Consequences for OxFunc
1. `CELL` should be refactored away from a purely resolver-based story once broader lanes are admitted.
2. `INFO` should be implemented directly on a host-query provider seam rather than by attempting to infer environment facts from local state.
3. `CELL` and `INFO` should be advanced together because they share the same typed host-query substrate even though their query sets differ.
4. The current typed cell-query seam should allow optional preserved reference identity because omitted-reference `CELL(info_type)` routes through active-selection host context rather than through explicit reference text.
