# Function Slice - Dynamic Array Shaping And Reshaping Family Contract (Prelim)

Status: `provisional`
Workset: `W39`

## 1. Purpose
1. define the admitted current-baseline semantic slice for the dynamic-array shaping and reshaping family,
2. keep the packet honest about what is covered locally in Rust, Lean, and native Excel evidence,
3. distinguish spill-shape/value semantics from later publication and seam questions that belong elsewhere.

## 2. In-Scope Members
1. `CHOOSECOLS`
2. `CHOOSEROWS`
3. `DROP`
4. `EXPAND`
5. `FILTER`
6. `SORT`
7. `SORTBY`
8. `TAKE`
9. `TOCOL`
10. `TOROW`
11. `TRANSPOSE`
12. `UNIQUE`
13. `VSTACK`
14. `WRAPCOLS`
15. `WRAPROWS`

## 3. Admitted Slice
1. values-only array constants and ordinary scalar arguments are in scope.
2. row/column orientation, row-major vs column-major flattening, and `#N/A` padding behavior are in scope.
3. seeded selector rules for positive and negative row/column selectors are in scope.
4. seeded boolean/numeric include-mask filtering is in scope.
5. seeded sort, sort-index, and sort-direction behavior is in scope.
6. row-wise distinctness and exactly-once distinctness for `UNIQUE` are in scope.

## 4. Main Rules Pinned In This Packet
1. `CHOOSECOLS` and `CHOOSEROWS` preserve selector order and allow duplicates.
2. selector `0` is invalid.
3. `TAKE` keeps from the start for positive counts and from the end for negative counts.
4. `DROP` removes from the start for positive counts and from the end for negative counts.
5. `TAKE(...,0,...)` and fully-empty `DROP(...)` slices surface `#CALC!` on the admitted slice.
6. `EXPAND` rejects target dimensions smaller than the source dimensions.
7. `EXPAND` pads with `#N/A` by default and with the provided scalar fill when supplied.
8. `TOCOL` and `TOROW` flatten row-major by default and column-major when the seeded `scan_by_column` flag is true.
9. `TOCOL` and `TOROW` honor the admitted ignore modes `0..3` for blank/error suppression.
10. `FILTER` admits row-mask and column-mask shapes that align with the target array axis.
11. `FILTER` without a match surfaces `#CALC!` unless `if_empty` is provided.
12. `SORT` admits row-wise or column-wise sorting on the seeded scalar sort key slice.
13. `SORTBY` is admitted for one aligned sort-key array on the seeded slice.
14. `TRANSPOSE` swaps axes without changing cell payload values.
15. `UNIQUE` is admitted for row-wise distinctness and exactly-once filtering on the seeded slice.
16. `VSTACK` pads narrower arrays with `#N/A`.
17. `WRAPROWS` and `WRAPCOLS` flatten row-major and then repack with `#N/A` pad by default.

## 5. Out Of Scope
1. helper/callable-driven array construction; that belongs with `W38`.
2. implicit intersection and legacy CSE interaction; that remains with `W014` and the operator seam work.
3. locale/profile/provider-sensitive behavior.
4. workbook-version sweep or historical-version sweep.
5. advanced sort collation beyond the seeded scalar/text lanes.

## 6. Boundary Notes
1. this packet is about function/operator semantics over prepared values, not parser ownership.
2. spill publication mechanics above the ordinary worksheet array result are not reopened here.
3. no fake event-stream or replay-specific shape is introduced for this family.
