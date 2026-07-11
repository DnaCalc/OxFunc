# W109 PERMUT Identification + COMBIN Findings — 2026-07-11

## PERMUT — identified and signed off

`PERMUT(n,k)` is the **ascending legacy x87 spill-loop product**:
`acc = RN53(RN64(acc · f))` for `f = n-k+1 ..= n`. Unique surviving candidate
out of 6 stagings (strict/spill/extended × forward/reverse) over 402 live
witnesses; production kernel (`permut_fn::permut_kernel`) verified
**702/702 bit-exact** across discovery + fresh held-out sweeps (build 20131).
The former factorial-ratio `n!/(n-k)!` staging was 1 ULP off on the catalog
witness `PERMUT(61,20)` and overflowed spuriously for `n > 170`.
In-crate pin: `permut_matches_live_excel_pinned_witnesses`.

## COMBIN — substantial findings, kernel still open

Positive identification: **Excel reduces `k -> min(k, n-k)`** —
`COMBIN(23,13)` publishes bit-identical results to `COMBIN(23,10)`.

Ruled out (505-row live corpus; see the ruled-out ledger):
- every product-loop family: `c *= num/den` and `c = (c·num)/den`, ascending
  and descending, strict / per-step double-rounded / fully extended
  (best candidate 82/505);
- factorial ratios `n!/(n-k)!/k!` and permutations thereof (strict doubles;
  matches only 29/81 of the `n <= 170` rows even with k-reduction);
- reciprocal-multiply loops `c·num·RN(1/den)` (66/505);
- `EXP(GAMMALN(n+1)-GAMMALN(k+1)-GAMMALN(n-k+1))` composed from Excel's
  PUBLISHED GAMMALN values through the identified x87 EXP (30-2000 ULP off).

Error signature: ±1 ULP around exactly-representable results at small `n`
(`(23,10)` +1, `(200,3)` −1) growing to ~7 ULP at `2^400` magnitudes — smaller
than any published-GAMMALN composition, larger than extended product loops.
Leading hypothesis: a dedicated internal extended-precision
lgamma/exp substrate (the Phase-5 statistical-kernel lane). COMBIN therefore
moves out of the quick-win batch; re-attack alongside GAMMALN identification.

## ACOTH — blocked on the ATANH kernel (recorded 2026-07-11)

All 13 natural graphs ruled out (platform/portable log1p, CRT-branchy log1p,
worksheet-ln ratios, fully extended ratios, internal atanh-of-reciprocal
stagings): Excel sits consistently 1 ULP above the log-identity forms at the
witnesses. ACOTH almost certainly publishes through Excel's internal ATANH
kernel — the unidentified piecewise G4-02 row. Re-race ACOTH as
`atanh(1/x)`-staging variants once G4-02 lands.

## CONVERT — deferred (harness gap)

CONVERT takes text unit arguments; `CellRefBatch.psm1` currently plumbs
numeric cells only. Needs a small harness extension (string-literal unit args
inside the formula text are bit-exact-safe) before the factor-bits probes can
run. Not attempted this pass.
