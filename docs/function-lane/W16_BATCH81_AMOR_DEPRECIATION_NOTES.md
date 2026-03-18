# W16 Batch 81 - AMOR Depreciation Family

Status: `packet-evidenced-by-w24`

Status: `in_progress-provisional`
Workset: `W16`

## Scope
1. `AMORDEGRC`
2. `AMORLINC`

## Current Implementation Notes
1. This pass is self-contained in `amor_depreciation_family.rs` plus a matching Lean metadata file; it does not modify shared dispatch, export, or root Lean import surfaces.
2. The executable slice covers the current baseline for integer periods, admitted bases `0`, `1`, `3`, and `4`, and the Excel-observed rejection of basis `2` with `#NUM!`.
3. `AMORLINC` is modeled as prorated straight-line depreciation with a special current-baseline lane where `date_purchased = first_period` yields a full annual first-period depreciation.
4. `AMORDEGRC` is modeled with the Excel-observed coefficient table, integer rounding, and the late-life `50%` then `100%` remaining-book rules that reproduce the seeded Microsoft-style examples.
5. A small current-baseline fractional-period normalization lane is also pinned: `AMORLINC` maps `0 < period < 1` to period `1`, while `AMORDEGRC` maps `0 < period < 1` to period `0`; both floor `period >= 1` in the replayed slice.

## Seeded Unit Lanes
1. `AMORLINC(2400,39679,39813,300,0,0.15,1)` -> `131.8032786885...`
2. `AMORLINC(2400,39679,39813,300,6,0.15,1)` -> `168.1967213114...`
3. `AMORDEGRC(2400,39679,39813,300,0..6,0.15,1)` -> `330, 776, 485, 303, 190, 158, 0`
4. `AMORDEGRC(2400,39679,39813,0,0..7,0.15,1)` -> `330, 776, 485, 303, 190, 158, 158, 0`

Current standing:
1. `W24` Batch 14 now adds native Excel replay evidence for the admitted slice.
2. This note remains the original bounded batch snapshot, not the current closure record.
5. Basis `0/1/3/4` first-period rows are pinned for both functions; basis `2` is pinned as `#NUM!`.
6. `date_purchased > first_period` is pinned as `#NUM!` for both functions.

## Open Lanes
1. Broader cross-version and workbook-compatibility sweeps remain open.
2. Fractional-period behavior is only spot-replayed in the bounded current-baseline slice; it is not yet claimed as a wider closure across all argument regions.
3. Other financial edge-policy lanes such as `salvage > cost` and broader invalid-date admission are kept conservative in this slice and may need widening after further native replay.
4. No shared integration wiring was changed in this task.
