# Function Slice - Misc Ordinary Conversion Triad Contract (Prelim)

Status: `provisional`
Workset: `W24`
Evidence ID: `W24-B15-MISC-ORDINARY-CONVERSION-20260318`

## 1. Scope
This slice closes the ordinary current-baseline semantics for:
1. `BAHTTEXT`
2. `CONVERT`
3. `PERCENTOF`

This slice does not own:
1. `EUROCONVERT`
2. `RANDARRAY`

Those two functions are evidenced in the same native packet only to justify extraction to `W025`.

## 2. Current-Baseline Contract
1. `BAHTTEXT`
   - admits scalar numeric input,
   - rounds to satang using the current kernel rounding policy,
   - emits Thai-script baht/satang text,
   - rejects negative or excessively large magnitudes with `#NUM!`.
2. `CONVERT`
   - admits the bounded unit catalog already wired in `misc_conversion_family.rs`,
   - preserves the current-baseline linear and temperature conversions in that catalog,
   - returns `#N/A` for unsupported unit symbols or mismatched dimensions.
3. `PERCENTOF`
   - admits the current scalar-first ratio lane,
   - sums each operand under the local aggregate rule,
   - returns `subset_sum / total_sum`,
   - returns `#DIV/0!` when the total sum is zero.

## 3. Packet Findings
1. Native Excel replay on `2026-03-18` matched the seeded `BAHTTEXT`, `CONVERT`, and `PERCENTOF` rows on the current host baseline.
2. The same replay showed `EUROCONVERT(...) -> #NAME?` and `RANDARRAY() -> #NAME?` on this host baseline.
3. Those two outliers therefore do not belong in the ordinary `W24` closure slice and move to `W025`.

## 4. Completeness Axes
1. `scope_completeness`: `scope_complete`
2. `target_completeness`: `target_complete`
3. `integration_completeness`: `integrated`
4. `open_lanes`:
   - broader locale/version sweeps remain outside this packet,
   - the extracted `EUROCONVERT` / `RANDARRAY` work now belongs to `W025`, not to this slice.
