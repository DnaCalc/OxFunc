# Cross-Boundary Invariant Checklist Template

Purpose:
1. make empirical and formal worksets declare invariants across all relevant value-flow boundaries,
2. prevent one-lane assumptions from silently drifting.

Usage:
1. copy this template into the active workset folder or function-lane note,
2. bind each invariant to scenario IDs and evidence IDs,
3. mark unresolved items explicitly.

## Metadata
1. Workset / slice:
2. Version scope (Excel build/channel + compatibility):
3. Locale scope:
4. Evidence IDs:

## Boundary Lanes
1. Formula evaluation lane (`formula -> value`)
2. Interop ingress lane (`host/API -> cell`)
3. Reference reuse lane (`cell/reference -> formula`)
4. Persistence lane (`save/reopen`)
5. Interchange lane (`CSV/text roundtrip`)
6. Optional UDF/XLL lane (`ABI boundary`)

## Invariants Checklist
1. Invariant ID:
   - Statement:
   - Boundaries covered:
   - Scenario IDs:
   - Expected observation:
   - Status (`open`|`provisional`|`validated`):
   - Notes:
2. Invariant ID:
   - Statement:
   - Boundaries covered:
   - Scenario IDs:
   - Expected observation:
   - Status (`open`|`provisional`|`validated`):
   - Notes:
3. Invariant ID:
   - Statement:
   - Boundaries covered:
   - Scenario IDs:
   - Expected observation:
   - Status (`open`|`provisional`|`validated`):
   - Notes:

## Required Minimum Set
1. Boundary divergence invariant:
   - explicitly state whether formula path and interop path are equivalent or intentionally divergent.
2. Error-surface invariant:
   - map numeric/COM/error sentinels to worksheet-visible errors.
3. Stability invariant:
   - confirm whether value survives reference reuse and persistence unchanged.
4. Admission invariant:
   - define parse/set admission boundaries versus runtime evaluation boundaries.

## Closure Rule
1. A workset cannot claim characterization closure without at least one validated invariant per active boundary lane.
