# W24 Batch 01 - SWITCH Runtime Requirements

## 1. Purpose
Define the runtime expectations for the native Excel `SWITCH` baseline used by `W24` Batch 01.

## 2. Inputs
1. scenario manifest:
   - `docs/function-lane/W24_BATCH01_SWITCH_SCENARIO_MANIFEST_SEED.csv`
2. runner:
   - `tools/w24-probe/run-w24-batch01-switch-baseline.ps1`
3. output artifact:
   - `.tmp/w24-batch01-switch-results.csv`

## 3. Environment
1. local Excel COM host
2. default workbook baseline
3. locale `en-US`

## 4. Expected Output Shape
CSV columns:
1. `scenario_id`
2. `lane`
3. `formula`
4. `text`
5. `value2`
6. `expected_text`
7. `matches_expected`
8. `notes`

## 5. Acceptance Rule
1. every seeded row must emit `matches_expected=True`
2. worksheet error rows must be compared through the worksheet text surface
3. any mismatch is a semantic drift candidate unless shown to be a host-entry or runner defect

## 6. Explain Notes
1. this packet is intended to prove ordinary worksheet semantics, not host metadata seams
2. no special XLL replay is required for closure because no `SWITCH`-specific XLL limitation is currently known
