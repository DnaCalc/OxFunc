# W48 OxFml Consumer Reconciliation

Status: `active`
Packet: `W048`

## 1. Basis
This reconciliation note records the final OxFml note reading against the frozen `W048` return-surface artifacts:
1. `FUNCTION_SLICE_RETURN_SURFACE_AND_PUBLICATION_HINT_CONTRACT_PRELIM.md`
2. `W48_RETURN_SURFACE_CLASS_MAP.csv`

## 2. Result
Current OxFunc reading of the final OxFml note is:
1. current three-way return split accepted for the first bounded consumer round
2. `ValueWithPresentation` accepted as the shared publication-aware carrier
3. typed host/provider outcome projection accepted as distinct from ordinary value
4. no concrete consumer mismatch currently identified

## 3. Current Reconciliation Outcome
1. ordinary value: accepted as default class
2. `ValueWithPresentation`: accepted for `NOW`, `TODAY`, `HYPERLINK`
3. typed host/provider outcome projection: accepted for `TRANSLATE`, `RTD`
4. `IMAGE`: acknowledged as sibling rich-value/publication pressure, not a reason to widen the current freeze

## 4. Current Honest Limit
This note does not claim cross-repo closure. It records that the current OxFml note produced no concrete mismatch against the current `W048` return split artifacts.
