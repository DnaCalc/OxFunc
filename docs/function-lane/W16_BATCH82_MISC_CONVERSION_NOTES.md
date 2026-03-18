# W16 Batch 82 - Misc Conversion and Formatting Family

Status: `split-by-w24`
Workset: `W16`
Evidence ID: `W16-BATCH82-MISC-CONVERSION-20260316`

## Scope
Historical mixed-family note for:
1. `BAHTTEXT`
2. `CONVERT`
3. `EUROCONVERT`
4. `PERCENTOF`
5. `RANDARRA`

## Current Semantics Baseline
1. This owned-file-only family is self-contained and intentionally not wired into shared Rust dispatch, XLL export, or root Lean import surfaces in this pass.
2. `BAHTTEXT` is admitted only as a scalar numeric-to-text kernel over non-negative values. The current baseline rounds to satang with ordinary half-away-from-zero rounding, emits Thai-script baht/satang text, and rejects negative or excessively large magnitudes with `#NUM!`.
3. `CONVERT` is admitted only for a bounded unit catalog:
   - length: `m`, `in`, `ft`, `yd`, `mi`, `Nmi`
   - mass: `g`, `lbm`, `ozm`
   - time: `sec`, `mn`, `hr`, `day`
   - pressure: `Pa`, `bar`, `atm`, `psi`
   - volume: `l`, `tsp`, `tbs`, `oz`, `cup`, `pt`, `qt`, `gal`
   - temperature: `C`, `F`, `K`
   - metric-prefix support on `m`, `g`, `l`, `Pa`, and `sec` for the prefixes implemented in the Rust file
4. `CONVERT` currently returns `#N/A` for unsupported unit symbols or mismatched dimensions instead of pretending broader catalog coverage.
5. `EUROCONVERT` uses a bounded legacy euro-currency table (`EUR`, `ATS`, `BEF`, `DEM`, `ESP`, `FIM`, `FRF`, `GRD`, `IEP`, `ITL`, `LUF`, `NLG`, `PTE`) with euro triangulation through fixed rates. `full_precision = FALSE` or omitted rounds to the target currency's current local decimal table; `full_precision = TRUE` leaves the final result unrounded. `triangulation_precision` is admitted only for `3..15`.
6. `PERCENTOF` is admitted as a scalar-first ratio kernel: it sums each operand, then returns `subset_sum / total_sum`. Direct scalars are coerced numerically; array operands are flattened locally and sum numeric/logical cells while ignoring text and empty cells.
7. `RANDARRA` is treated as the inventory-stated owned-file target name, not as a shared integration claim about `RANDARRAY`. The current kernel returns a scalar or rectangular in-memory array, defaults to `1 x 1`, defaults the numeric range to `[0, 1)`, and supports a bounded whole-number mode.

## Pinned Local Test Rows
1. `BAHTTEXT(1234) -> "หนึ่งพันสองร้อยสามสิบสี่บาทถ้วน"`
2. `BAHTTEXT(1234.56) -> "หนึ่งพันสองร้อยสามสิบสี่บาทห้าสิบหกสตางค์"`
3. `CONVERT(1,"lbm","kg") -> 0.45359237`
4. `CONVERT(68,"F","C") -> 20`
5. `CONVERT(3.5,"km","m") -> 3500`
6. `EUROCONVERT(10,"DEM","EUR") -> 5.11` in the rounded default slice
7. `PERCENTOF(15,60) -> 0.25`
8. `RANDARRA()` with a deterministic test provider yields a `1 x 1` array
9. `RANDARRA(2,2,10,12,TRUE)` over deterministic provider samples yields a bounded `2 x 2` whole-number matrix

## Current Standing After W24 Batch 15
1. `BAHTTEXT`, `CONVERT`, and `PERCENTOF` are now closed for the admitted ordinary current-baseline slice by `W24 Batch 15`.
2. `EUROCONVERT` and `RANDARRA` were replayed natively on `2026-03-18` and returned `#NAME!` on the current host baseline.
3. Those two outliers were extracted to `W025` and no longer belong to the ordinary `W24` closure slice.

## Open Lanes
1. Wider `CONVERT` catalog breadth remains orthogonal future widening work, not a blocker for the admitted ordinary slice.
2. `EUROCONVERT` add-in parity now belongs to `W025`.
3. `RANDARRA` dynamic-array/version reconciliation now belongs to `W025`.
