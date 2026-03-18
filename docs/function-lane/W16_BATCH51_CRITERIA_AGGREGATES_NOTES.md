# W16 Batch 51 - Criteria Aggregates

Status: `function-phase-complete` (current reference baseline)
Workset: `W16`

## Scope
1. `COUNTIF`
2. `COUNTIFS`
3. `SUMIFS`
4. `AVERAGEIF`
5. `AVERAGEIFS`
6. `MAXIFS`
7. `MINIFS`

## Current Implementation Slice
1. Criteria parsing covers direct numeric/logical criteria and text criteria with `=`, `<>`, `<`, `<=`, `>`, `>=`, blank criteria, and wildcard text matching.
2. Matching is case-insensitive for text and uses the existing wildcard matcher for `*`, `?`, and `~` escaping.
3. `SUMIFS`, `AVERAGEIF(S)`, `MAXIFS`, and `MINIFS` aggregate only numeric target cells after criteria filtering.
4. `AVERAGEIF(S)` returns `#DIV/0!` when no numeric matches survive.
5. `MAXIFS` and `MINIFS` currently return `0` when no numeric match survives.
6. Narrowed shape rule after `W22`:
   - `AVERAGEIF` top-left anchors an explicit mismatched `average_range`,
   - `COUNTIFS`, `SUMIFS`, `AVERAGEIFS`, `MAXIFS`, and `MINIFS` remain exact-shape on the current baseline.

## Focused Evidence Lanes
1. numeric comparison criteria such as `">=2"`
2. wildcard text criteria such as `"alp*"`
3. blank criteria `""`
4. multi-range intersection for `COUNTIFS` and `SUMIFS`
5. omitted `average_range` in `AVERAGEIF`
6. no-match behavior for `AVERAGEIF(S)`, `MAXIFS`, and `MINIFS`

## Narrowed Resolution
1. The earlier generic shape-open issue was resolved by `W22`.
2. `AVERAGEIF` is the only member of this packet currently evidenced to top-left-anchor a mismatched target range.
3. The `*IFS` members in this packet remain exact-shape and return `#VALUE!` on the mismatched lanes captured in `W22`.
4. No known semantic gap remains in this packet for the current reference baseline after `W22`.
