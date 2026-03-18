# W16 Batch 78 - Statistical Tests Family

Status: `packet-evidenced`
Workset: `W16`

## Scope
1. `CHISQ.TEST`
2. `CHITEST`
3. `F.TEST`
4. `FTEST`
5. `T.TEST`
6. `TTEST`

## Current Bounded Slice
1. This family now participates in shared dispatch, export, and root Lean import surfaces; the earlier self-contained wording is obsolete.
2. `CHISQ.TEST` / `CHITEST` are executable for same-shape numeric scalar-or-array inputs, with `#N/A` on the degenerate `1x1` lane and chi-square right-tail probability delegated to the existing chi-square distribution substrate.
3. `F.TEST` / `FTEST` are executable for numeric samples with current-baseline array/reference behavior that ignores text, logical, and empty cells but propagates worksheet errors.
4. `T.TEST` / `TTEST` are executable for `tails ∈ {1,2}` and `type ∈ {1,2,3}`:
   - paired test (`type=1`) with equal expanded cardinality and pairwise numeric filtering
   - equal-variance two-sample test (`type=2`)
   - unequal-variance Welch test (`type=3`) using continuous Welch-Satterthwaite degrees of freedom inside the local probability path
5. Compatibility aliases delegate directly to their modern entrypoints in the same family.

## Seeded Evidence Lanes
1. `CHISQ.TEST(A2:B4,A6:B8) -> 0.0003082` using the Microsoft help example arrays.
2. `F.TEST(A2:A6,B2:B6) -> 0.64831785` using the Microsoft help example arrays.
3. `T.TEST(A2:A10,B2:B10,2,1) -> 0.196016` using the Microsoft help example arrays.
4. `T.TEST(...,1,2)` and `T.TEST(...,2,2)` satisfy the expected one-tail/two-tail doubling relation; same for `type=3`.

## Open Lanes
1. `W24` Batch 07 closed the previously open `CHISQ.TEST` geometry issue: Excel accepts equal-cardinality arguments and reshapes the second argument row-major to the first argument's layout.
2. Mixed text/logical/blank survivor ignoring and worksheet-error propagation are now packet-evidenced for the admitted `F.TEST` and `T.TEST` lanes.
3. The family is now packet-evidenced on the integrated shared surfaces rather than remaining local-only scaffolding.
