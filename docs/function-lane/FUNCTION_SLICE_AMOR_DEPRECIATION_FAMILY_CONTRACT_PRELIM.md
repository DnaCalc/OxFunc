# Function Slice Contract (Preliminary) - AMOR Depreciation Family

Status: `provisional`
Workset: `W24`
Primary Functions: `AMORDEGRC`, `AMORLINC`

## 1. Scope
1. close the admitted current-baseline scalar depreciation slice for the AMOR family,
2. bind the integrated runtime and Lean substrate to a replayable native worksheet packet,
3. replace the older bounded-note standing with packet evidence for the seeded support-example, basis, and fractional-period lanes.

## 2. Admitted Current-Baseline Slice
1. scalar-only arguments with the 1900 date system.
2. basis admitted only for `0`, `1`, `3`, and `4`.
3. `AMORLINC`
   - prorated straight-line depreciation
   - `date_purchased = first_period` yields a full annual first-period depreciation
   - `0 < period < 1` normalizes to period `1`
4. `AMORDEGRC`
   - Excel-observed coefficient table
   - half-away-from-zero rounding
   - late-life `50%` then `100%` remaining-book rules
   - `0 < period < 1` normalizes to period `0`
5. `period >= 1` is floored in the admitted replayed slice.

## 3. Explicitly Out Of Slice
1. broader locale/version sweeps.
2. non-scalar coercion breadth beyond the admitted values-only slice.
3. any alternate workbook date system or broader serial-edge investigations.

## 4. Metadata Shape
1. determinism: `deterministic`
2. volatility: `nonvolatile`
3. host_interaction: `none`
4. thread_safety: `safe_pure`
5. arg_preparation_profile: `values_only_pre_adapter`
6. coercion_lift_profile: `custom`
7. fec_dependency_profile: `none`
8. surface_fec_dependency_profile: `none`

## 5. Evidence Basis
1. Rust runtime kernel and unit tests in `crates/oxfunc_core/src/functions/amor_depreciation_family.rs`
2. Lean metadata/binding in `formal/lean/OxFunc/Functions/AmorDepreciationFamily.lean`
3. Native worksheet packet in `docs/function-lane/W24_BATCH14_AMOR_DEPRECIATION_SCENARIO_MANIFEST_SEED.csv`
4. Runtime harness in `tools/w24-probe/run-w24-batch14-amor-depreciation-baseline.ps1`
5. Packet execution record in `docs/function-lane/W24_BATCH14_AMOR_DEPRECIATION_EXECUTION_RECORD.md`

## 6. Scope Boundary
1. The closure is bounded to the admitted current-baseline scalar depreciation slice above.
2. The packet now evidences the seeded support-example, basis, and fractional-period normalization lanes directly.
3. Broader date-system and locale/version validation remain follow-on concerns rather than unacknowledged gaps in this packet.
