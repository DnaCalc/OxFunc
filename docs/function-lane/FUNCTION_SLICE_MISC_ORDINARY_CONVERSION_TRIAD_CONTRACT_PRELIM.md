# Function Slice - Misc Ordinary Conversion Triad Contract (Prelim)

Status: `current-reference-aligned`
Workset: `W24`
Evidence IDs: `W24-B15-MISC-ORDINARY-CONVERSION-20260318`, `W109-G4-CONVERT-20260809`

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
   - admits the current supported same-category catalog:
     - length: `m`, `in`, `ft`, `yd`, `mi`, `Nmi`;
     - mass: `g`, `lbm`, `ozm`;
     - time: `sec`, `mn`, `hr`, `day`;
     - pressure: `Pa`, `atm`, `psi`;
     - volume: `l`, `tsp`, `tbs`, `oz`, `cup`, `pt`, `qt`, `gal`;
     - prefixes `Y`, `Z`, `E`, `P`, `T`, `G`, `M`, `k`, `h`, `da`, `d`, `c`, `m`, `u`, `n`, `p`, and `f` on the category base units;
   - treats `bar` as unsupported and returns `#N/A`, matching the current reference even though it appears in broader public unit catalogs;
   - uses integer angstrom factors for direct length units and independently rounded reciprocal factors for the pressure table;
   - keeps prefix resolution separate from the direct-unit table and applies one correctly rounded decimal `10^delta` only after the direct-factor core;
   - publishes the linear graph as three stored operations in order:
     1. `product = RN53(RN64(number * from_factor))`,
     2. `core = RN53(RN64(product / to_factor))`,
     3. `result = RN53(RN64(core * prefix_delta))`;
   - evaluates the six non-identity temperature pairs through direct binary64 affine formulas rather than composing every pair through Kelvin, and preserves identity inputs;
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
4. W109 G4-05 replaced the earlier bounded-example arithmetic assumption for `CONVERT` with a typed, bit-exact current-reference graph:
   - `7,026/7,026` discovery rows;
   - `5,586/5,586` explicitly retired v1 rows under the final graph;
   - `6,915/6,915` explicitly retired v2 rows under the final graph;
   - `4,226/4,226` refinement-only rows;
   - `18/18` independent Value2/readback controls;
   - `10,418/10,418` frozen, disjoint publication rows;
   - `34,189/34,189` direct compiled-production replay rows.
5. The publication capture is Excel 16.0 build 20228 x64, workbook Compatibility Version 2, NoCache, with typed `Range.Value2` argument cells and bulk `Range.Value2` result readback. The 192 preselected store-site discriminators reject the native-f64, length-only, and unstaged-quotient controls without any post-answer model change.

## 4. Completeness Axes
1. `scope_completeness`: `scope_complete`
2. `target_completeness`: `target_complete`
3. `integration_completeness`: `integrated`
4. `open_lanes`:
   - broader locale/version sweeps remain outside this packet,
   - alternate-architecture realization of the x87 current-reference graph remains an orthogonal platform phase,
   - the extracted `EUROCONVERT` / `RANDARRAY` work now belongs to `W025`, not to this slice.

## 5. W109 Reproducibility Binding

1. Identification record: `docs/function-lane/W109_CONVERT_IDENTIFICATION_20260809.md`.
2. Frozen generator/scorer/model sources:
   - `smart-fuzzer/tools/calc_graph_racer/src/bin/generate_convert_publication_heldout_v3.rs`;
   - `smart-fuzzer/tools/calc_graph_racer/src/bin/score_convert_unified_v3.rs`;
   - `smart-fuzzer/tools/calc_graph_racer/src/bin/convert_research/model_v3.rs`.
3. Mandatory compiled-production replay:
   - `smart-fuzzer/tools/calc_graph_racer/src/bin/replay_convert_production_v3.rs`.
4. Rust exact pins live beside `convert_kernel` in `crates/oxfunc_core/src/functions/misc_conversion_family.rs`.
5. Lean records the three stored linear sites, table/prefix/bar policy, and direct temperature-route binding in `formal/lean/OxFunc/Functions/MiscConversionFamily.lean` without duplicating the x87 engine.
