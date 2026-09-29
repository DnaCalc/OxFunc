# SYD arithmetic and typed evidence

BUG-FUNC-067 / bead oxf-mwue.28.17. Black-box Excel 16.0 build 20430,
64-bit Windows, workbook Compatibility Version 2; numeric Value2 inputs retain
their exact recorded normal binary64 values. Channel is not available in these
bulk capture records. Other Excel, locale and platform axes remain unvalidated.

The primary 1,207 observations matched the earlier kernel but contained no
negative cost and no tiny valid lifetime. Expanded 2,192 discovery rows matched
only 663. Excel accepts negative cost while rejecting negative salvage. A
positive period excess smaller than the minimum normal is published as zero
for admission. The ordered arithmetic is:

1. Store `life + 1` through x87 addition.
2. Form `(life * stored_sum) / 2`. Publish nonfinite or subnormal denominator
   as zero, then return DIV/0 for that zero.
3. Store `cost - salvage` through x87 subtraction. A subnormal basis becomes
   positive zero; basis overflow returns NUM after the denominator-zero check.
4. Store `stored_sum - period`, multiply by basis, then divide by denominator.
   Multiplication and division use the existing x87 store substrate. Final
   nonfinite values publish NUM; final subnormals publish positive zero.

The first 300 selected store probes match ordinary arithmetic 0/300, x87
multiplication 181/300, x87 division 119/300, and both 300/300. Frozen v1 matched
3,699 discovery observations. Its independent 11,556 cohort matched 11,452,
with 104 small-bit differences around `life = 2^-53`; x87 addition explains
all 104. A 396-row halfway-neighbor discriminator then distinguishes x87
subtraction: ordinary subtraction 241/396, x87 subtraction 396/396. Frozen v2
matches all 15,651 discovery observations. The next 13,544 independently
generated numeric observations match 13,544/13,544. Subsequent exact-maximum
discovery reveals 30 structural misses in 144 rows: overflowing negative cost
minus positive salvage returns NUM. The decimal-exponent random cohorts stopped
at 1e307 and had not tested that stage. Candidate v3 corrects this branch and
matches 144/144, retaining the denominator's earlier DIV/0 priority. Its fresh
21,526-row cohort samples every normal IEEE exponent and matches 21,526/21,526.
The v3 discovery count is 29,339 observations; earlier independent cohorts are
discovery for this revision. The frozen selected numeric bodies remain unchanged.
Old independent failures remain
discovery evidence, with the exact original source snapshots and judgements.

Typed discovery 97 initially matched 84. Required explicit missing arguments
coerce to zero. Arrays and reference areas lift through the existing function
adapter; row/column broadcasting preserves scalar results and errors. After
those corrections, strict typed replay matches 97/97, with every row admitted
on both sides. A new 104-row typed cohort matches 104/104: 92 independent controls
and 12 discovery probes for incompatible shape/error precedence. Dedicated
financial padding probes subsequently reveal the shared earlier-error rule;
see [the impact assessment](POSITIONAL_PADDING.md) for its separate observations.
Numeric text
is limited to simple ASCII decimals; the wider shared grammar is a separate
lane.

The changes use existing prepared-value and reference interfaces. No provider
contract, reference identity or evaluator clause changes. The function now
uses the same positional-error lifting helper as the other depreciation functions. Its admitted
behavior and arithmetic-store ordering have a Lean substrate binding and
reduced Rust tests. The Lean binding names IEEE store/publication dependencies;
it does not assert exact rational arithmetic for every binary64 operation.

Status: in progress. `scope_completeness=scope_partial`,
`target_completeness=target_partial`, `integration_completeness=partial`.
Open lanes: receiving-dependency acknowledgement/integration for positional padding;
shared numeric-text coercion; other baseline and platform axes.
