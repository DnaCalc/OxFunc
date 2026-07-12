# W109: CHISQ.TEST & F.TEST decomposition (2026-07-12)

Both test functions were "blocked on the G3-01 distribution substrate."
Identity-decomposition probing unblocks their STATISTIC layers today and
proves their drift is inherited, not independent.

## CHISQ.TEST / CHITEST (G3-05)

Proven on live bits, 4 contingency tables (2x2, 2x3, 2x5):

    CHISQ.TEST(obs, exp) == CHIDIST(S, df) exactly,  df = (r-1)(c-1)

where S is Excel's internal statistic. Probing CHIDIST at candidate S
bit-patterns (the tail cancels in the equality) pins S: on the two tables
whose CHIDIST tail is steep enough to be injective (df=4 large-stat; df=2
large-stat), the unique matching S is **offset 0 from the plain-double
ROW-MAJOR accumulation** `S = Σ_rows Σ_cols (o-e)²/e`. The flat-tail tables
(df=1, df=2 small-stat) are consistent (±1 ULP of S maps to the same
p-value bits, non-discriminating). => the statistic IS plain-double
row-major `Σ(o-e)²/e`; all CHISQ.TEST drift is inherited from CHIDIST.

## F.TEST / FTEST (G3-06)

Proven on live bits, 3 two-sample sets, df up to (5,6):

    F.TEST(a, b) == 2 · FDIST(F, df_hi, df_lo) exactly

where F = max(var_a, var_b) / min(var_a, var_b) with unbiased variances
(divisor n-1), df ordered (df of the larger-variance group, df of the
smaller). The factor 2 is exact for binary64. The larger/smaller ("ba"
in the numerator for all three) matched every table; one set matched at
statistic offset -1 ULP, isolating a minor variance-accumulation-order
detail in var() (one more probe round pins it). All F.TEST drift is
inherited from FDIST.

## Consequence

G3-05 and G3-06 are NOT independent numeric bugs — they are downstream
consumers of the CHIDIST/FDIST tails, i.e. the incomplete-gamma /
incomplete-beta substrate tracked as G3-01. Fixing G3-01 fixes both for
free (modulo the F.TEST variance-order ULP). Their statistic layers are
identified and can be pinned in OxFunc now.

Corpora: G3-05-answers-chi-multi.json / -chidist-multi.json;
G3-06-answers-ftest.json / -fdist.json.
