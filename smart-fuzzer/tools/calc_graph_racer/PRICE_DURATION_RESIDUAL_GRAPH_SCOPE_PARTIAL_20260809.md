# PRICE / DURATION residual calculation graph — scope-partial handback

Date: 2026-08-09
Lane: W109 G6-03d PRICE and G6-03c DURATION forward calculation only
Excluded: YIELD, ODDFYIELD, every heldout answer, production changes, shared docs/state/beads

## Status axes

- `scope_completeness: scope_partial`
- `target_completeness: target_partial`
- `integration_completeness: partial`
- `open_lanes`:
  - exact PRICE coupon-accumulator/publication graph at low-redemption boundaries;
  - exact DURATION numerator/denominator accumulator graph;
  - heldout validation, still sealed pending an exact discovery survivor;
  - production integration, intentionally not attempted.

## Corpus and provenance audit

The historical identification bank was audited before generating new inputs:

- `work/w109/G6-b2b3/answers-b37-price.json`: 7,472 PRICE rows, of which 7,328 are basis 2/3.
- `work/w109/G6-b2b3/answers-b44-duration.json`: 6,360 DURATION rows, of which 2,544 are basis 2/3.
- `work/w109/G6-b2b3/agentV_residual14.json`: 14 unique b37 residual rows; exact IDs, argument bits, and expected bits agree with b37.
- `work/w109/G6-b2b3/agentX_b44_residual143.json`: 143 unique b44 residual rows; exact IDs, argument bits, and expected bits agree with b44.

Those historical answer JSON files do not embed `capture_provenance`. The adjacent reports attest Excel build 20131, but not Value2/NoCache/CV/bitness inside the answer artifacts. They are therefore identification controls, not the current reference capture.

Fresh discovery captures were serialized, discovery-only, and asserted per file:

| Function | Rows | Batch SHA-256 | Answer SHA-256 | Captured UTC |
|---|---:|---|---|---|
| PRICE | 528 | `43FB36C45796DE0CBAF27BB181EF8D8360BB0BA5E09C055222C5A209034D51CE` | `6B05A1254D6E7B550F210C36B3F6BDAE3D765C50931E769EE67BF627EE0A47DF` | `2026-08-09T10:18:18.7298582Z` |
| DURATION | 264 | `A7AE8F148804587C8F822F1C4F957EC391F669CAFFB0D8F3B5044A8466A18FC0` | `B3C054FA2F338CB19F427A32EC6D8F992C6AEB84163E0889ACC0313D5A918E31` | `2026-08-09T10:18:40.0952824Z` |
| PRICE companion | 72 | `A882F53125B11B861FA0826C672BD1849317D504D3B9803509F5D039003D832B` | `6C1895B31203FC2F0F1C98E8E43BF34AB20695E18C1E9BBAE916EAD1FFCA6550` | `2026-08-09T10:46:10.4529316Z` |

Every fresh answer asserts Excel 16.0 build 20228, 64-bit, workbook compatibility 2, `cell_value2_bulk`, and `no_cache` with zero cache hits/misses. Function, count, unique nonempty IDs, ordered ID/argument-bit equality, and numeric result kinds were checked against the unchanged frozen batch. Each serialized use started at Excel process count 0 and released at process count 0.

The initially frozen heldout inputs remain uncaptured and their answers were not opened:

- PRICE heldout input SHA-256 `DF7BF0E631A698C9DC5D801F34D29F19AEE19B15C04375B3C8F011D570FE8B30`
- DURATION heldout input SHA-256 `7AAF03BAD90C0819FEE920FD62D545237A39C2946EF29857472E80BE032297A0`
- heldout metadata SHA-256 `9ADA172F24D8A7FA0982F103834938A2333C70FC9215EB107B52630B41B8A6F3`

## Answer-blind battery design

The 528-row PRICE and 264-row DURATION discovery split uses four disjoint Actual/360 or Actual/365 contexts with frequencies 1, 2, and 4. Each context publishes n=2..12 consecutive truncated maturities and the center yield plus its two adjacent binary64 values. PRICE uses a coupon/zero-coupon by redemption-100/redemption-1 grid. DURATION uses matching coupon and zero-coupon rows.

The 72-row adaptive companion was frozen before its answer capture. It keeps the exact d1 Actual/365 frequency-4 n=7/n=8 base/exponent inputs that produced the six PRICE misses, then varies only:

- coupon by ±1 raw ULP;
- redemption around 1 by ±1 raw ULP, plus tiny and 2.0 scales;
- zero-coupon terminal controls;
- n=6/n=9 neighboring truncated ladders;
- exact 0.5 and 1.0 coupon-cash controls.

The companion generator reads input batches only, explicitly excludes every path containing `heldout`, and never reads oracle answers. Regeneration was byte-identical; all 72 tuples were unique and absent from the 8,945-row non-heldout PRICE input bank.

## Schedule/model controls

The local basis-2/3 schedule and cashflow model is not the residual source:

| Control | Equality |
|---|---:|
| production PRICE vs scorer reference graph, historical | 7,328 / 7,328 |
| production DURATION vs scorer reference graph, historical | 2,544 / 2,544 |
| production PRICE vs scorer reference graph, fresh discovery | 528 / 528 |
| production DURATION vs scorer reference graph, fresh discovery | 264 / 264 |
| production PRICE vs scorer reference graph, discovery + companion | 600 / 600 |

## Fixed candidate results

Scores are exact rows / total, followed by maximum ULP distance and summed ULP distance.

| Corpus / family | Candidates | Exact survivors | Best score | Best coherent graph |
|---|---:|---:|---|---|
| historical PRICE, original fixed | 1,152 | 0 | 7,320/7,328, max 1, sum 8 | Chain pow; forward; divide; separate redemption; `coup*(a/e)`; stored-body variant |
| historical DURATION, original fixed | 2,592 | 0 | 2,482/2,544, max 2, sum 72 | Chain pow; forward; `(diff*cash)/disc`; separate redemption |
| fresh PRICE, production | 1 | 0 | 519/528, max 1, sum 9 | current graph |
| fresh PRICE, original fixed | 1,152 | 0 | 522/528, max 1, sum 6 | Chain pow; forward; divide; separate redemption; `coup*(a/e)` |
| fresh DURATION, production/original fixed | 2,592 | 0 | 237/264, max 3, sum 45 | current graph; 24 observationally tied store/final-ratio variants |
| fresh PRICE, retained-PC64 | 288 | 0 | 522/528, max 1, sum 6 | 27 tied; no improvement over stored Chain |
| fresh DURATION, retained-PC64 | 288 | 0 | 237/264, max 3, sum 45 | 24 tied; no improvement |
| fresh PRICE, factorized coupon | 80 | 0 | 474/528, max 3, sum 66 | reverse discount-factor sum, then coupon factor |
| fresh DURATION, factorized coupon | 72 | 0 | 237/264, max 2, sum 42 | exact count unchanged; summed distance reduced |
| fresh PRICE, fixed associations | 48 | 0 | 522/528, max 1, sum 6 | left fold; every balanced/block/lane family worse |
| PRICE discovery + companion, original fixed | 1,152 | 0 | 571/600, max 1, sum 29 | same Chain/forward/separate-redemption graph |
| PRICE discovery + companion, retained-PC64 | 288 | 0 | 571/600, max 1, sum 29 | 27 tied; no retained-PC64 benefit |
| PRICE discovery + companion, factorized coupon | 80 | 0 | 511/600, max 2, sum 105 | worse |
| PRICE discovery + companion, fixed associations | 48 | 0 | 571/600, max 1, sum 29 | left fold; alternatives worse |

The combined best has signed residuals `{ -1: 29, 0: 571 }`. On the 72-row companion alone it is 49/72, with all 23 misses at −1 ULP. This is a coherent negative result, not a point-fitted coefficient family.

## Six-row localization

All six original PRICE misses share context `d1-a365-f4`, coupon mode `c1`, redemption 1, and one of two consecutive maturity lengths. Each mismatch repeats at yield raw bits −1/center/+1 because those three inputs publish the same stored binary64 base.

| n | Stored base | Stored exponent | Stored discount | Model bits | Excel bits |
|---:|---|---|---|---|---|
| 7 | `0x3ff055e6f7c91a2b` | `0x401a1d7520f6e1d7` | `0x3ff25266188b9d84` | `0x401ba7767d74f195` | `0x401ba7767d74f196` |
| 8 | `0x3ff055e6f7c91a2b` | `0x401e1d7520f6e1d7` | `0x3ff2b4c45327f545` | `0x401f00fe5c054677` | `0x401f00fe5c054678` |

The scorer explicitly raced:

- stored, quotient-stored, and raw-PC64 base formation;
- stored, offset-stored, and raw-PC64 exponent formation;
- stored Chain, Chain with retained final PC64 result, all-PC64 Chain, and direct FYL2X/F2XM1;
- publication at discount, term, per-step accumulator, or final boundary;
- stored or raw-PC64 accrued interest;
- left/right, balanced, adjacent-round, block-2..6, and lane-2..4 associations;
- direct coupon cash terms versus factorized discount sums.

None clears discovery. The shared evidence remains coarse: PRICE and DURATION strongly prefer the published Chain pow substrate, forward accumulation, and separate terminal redemption. The corpus does not identify a coherent shared retained-Ext80 fractional-pow graph, and the remaining PRICE boundary is more consistent with an unmodeled accumulator/publication detail than a different pow primitive.

## Durable paths and hashes

Source:

- `smart-fuzzer/tools/calc_graph_racer/src/bin/generate_price_duration_residual_graph.rs`
  SHA-256 `6442D2A16C8D2C5E73A63709BB98F5A8F984F88E0C69150BE3FD1692329E2171`
- `smart-fuzzer/tools/calc_graph_racer/src/bin/generate_price_residual_companion.rs`
  SHA-256 `E7D4ED32578ED65228F012CAD82F21E963AB3EBE8857FB1E8ED071270B5FF9A5`
- `smart-fuzzer/tools/calc_graph_racer/src/bin/race_price_duration_residual_graph.rs`
  capture-time/final SHA-256 `ADDBDBE7FF0265D4DC40429261B6473C5D28582F4FC1D015BEAB3FCB309178E8`

Work artifacts:

- `smart-fuzzer/work/w109/G6-price-duration-exact/batch-price-residual-graph-discovery-20260809.json`
- `smart-fuzzer/work/w109/G6-price-duration-exact/batch-duration-residual-graph-discovery-20260809.json`
- `smart-fuzzer/work/w109/G6-price-duration-exact/meta-price-duration-residual-graph-discovery-20260809.csv`
- `smart-fuzzer/work/w109/G6-price-duration-exact/answers-price-residual-graph-discovery-20260809.json`
- `smart-fuzzer/work/w109/G6-price-duration-exact/answers-duration-residual-graph-discovery-20260809.json`
- `smart-fuzzer/work/w109/G6-price-duration-exact/batch-price-residual-companion-discovery-20260809.json`
- `smart-fuzzer/work/w109/G6-price-duration-exact/meta-price-residual-companion-discovery-20260809.csv`
- `smart-fuzzer/work/w109/G6-price-duration-exact/answers-price-residual-companion-discovery-20260809.json`
- `smart-fuzzer/work/w109/G6-price-duration-exact/batch-price-residual-graph-heldout-20260809.json` — input only, sealed
- `smart-fuzzer/work/w109/G6-price-duration-exact/batch-duration-residual-graph-heldout-20260809.json` — input only, sealed
- `smart-fuzzer/work/w109/G6-price-duration-exact/meta-price-duration-residual-graph-heldout-20260809.csv` — input metadata only
- isolated build target: `smart-fuzzer/work/w109/G6-price-duration-exact/cargo-target/`

## Verification performed

- exact-file `rustfmt` on the three new Rust sources;
- isolated-target `cargo test --no-run` for both generators and the scorer;
- byte-identical regeneration and SHA checks for the discovery and companion inputs;
- local finite production-kernel validation for every generated tuple;
- fresh live-capture provenance and per-file input/output bit assertions;
- post-capture Excel process count 0 after every serialized use;
- no edits by this lane to production, shared docs, state, or beads.

No exact discovery survivor exists, so heldout capture and production promotion are not authorized by this evidence.
