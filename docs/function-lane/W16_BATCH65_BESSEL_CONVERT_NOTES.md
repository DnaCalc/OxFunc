# W16 Batch 65 - Bessel Quartet and `CONVERT` Triage

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH65-BESSEL-CONVERT-20260316`

## Scope
1. `BESSELI`
2. `BESSELJ`
3. `BESSELK`
4. `BESSELY`
5. `CONVERT` deferred in this owned-file-only pass

## Native Excel Baseline
Pinned seed lanes:
1. `BESSELI(1.5,1) -> 0.981666428`
2. `BESSELJ(1.9,2) -> 0.329925829`
3. `BESSELK(1.5,1) -> 0.277387804`
4. `BESSELY(2.5,1) -> 0.145918138`

## Current Implementation Notes
1. The quartet is modeled as a self-contained numeric family with value-only argument preparation and custom kernels.
2. Order arguments are truncated toward zero before evaluation, matching the current Excel integer-order contract used by the quartet.
3. Negative orders are rejected with `#NUM!`.
4. `BESSELK` and `BESSELY` reject `x <= 0` with `#NUM!`; `BESSELI` and `BESSELJ` accept signed `x` and preserve odd-order sign parity.
5. The kernels use compact Numerical Recipes / Cephes-style rational approximations for orders `0` and `1`, plus recurrence for higher integer orders.
6. `CONVERT` is intentionally deferred here. Its unit-catalog and compatibility surface is materially larger than the Bessel quartet and does not fit cleanly inside this owned-file-only subtask without touching broader shared infrastructure.
