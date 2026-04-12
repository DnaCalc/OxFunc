# BUGREP-FUNC-018: XIRR shows bounded solver-precision drift versus Excel

## Summary
- **Report id**: `BUGREP-FUNC-018`
- **Filed**: `2026-04-10`
- **Status**: `triaged`
- **Canonical bug**: `BUG-FUNC-014`

## Intake
- **Source channel**: `user`
- **Reported against ref**: `2e818f03a71ba393690275a7fb437ddd9a6bf760`
- **Reported against kind**: `commit`
- **Report owner workset**: `W087`

## Prompt / Observation
1. User asked to confirm several small-variation differences between Excel and
   OxFunc and to separate real semantic/algorithm gaps from display policy.
2. Direct local replay on 2026-04-10 confirmed:
   - `XIRR({-10000,2750,4250,3250,2750},{44927,45108,45292,45473,45658})`
     -> `0.24449183218286558`
3. Live Excel `Value2` replay on 2026-04-10 confirmed:
   - the same formula -> `0.24449183344840997`
4. This is not a published-display rounding difference; it is a bounded local
   iterative-solver precision mismatch.

## Initial Classification
- **Ownership guess**: `OxFunc-owned bug`
- **Duplicate of existing report?**: `no`
- **Needs canonical stream?**: `yes`

## Notes
1. The current local implementation in `cashflow_rate_family.rs` uses bounded
   iterative solve controls (`ROOT_TOLERANCE`, `ROOT_MAX_ITERATIONS`) and a
   mixed solve path.
2. This intake is intentionally separate from the earlier `RATE` omitted-guess
   convergence repair under `W081`; the shared family direction is financial
   iteration, but the exact mismatch here is the `XIRR` solve target.
3. Existing `W037`/current evidence did not pin this exact witness at live
   Excel `Value2` precision, so the current mismatch was not blocked by the
   admitted floor.
