# W16 Batch 62 - Confidence/Test Helpers

Status: `packet-evidenced`
Workset: `W16`
Evidence ID: `W16-BATCH62-CONFIDENCE-TEST-20260316`

## Scope
1. `CONFIDENCE.T`
2. `Z.TEST`

## Current Batch Shape
1. `CONFIDENCE.T` is implemented as a direct scalar numeric wrapper over the existing `T.INV.2T` substrate.
2. `Z.TEST` scans numeric survivors from the first operand, then computes the one-tailed normal probability from the sample mean.

## Pinned Lanes
1. `CONFIDENCE.T(0.05,2.5,50) -> 0.7104971266...`
2. `Z.TEST({3;6;7;8;6},4,1.5) -> 0.090574...`
3. Invalid alpha/size/sigma domain lanes map to `#NUM!`.

## Open Issues
1. `W24` Batch 05 now pins mixed-survivor `Z.TEST` behavior for the admitted slice: text, logical, and blank survivors are ignored, while an error survivor propagates.
