# Floating-Point Edge XLL Harness Contract

## 1. Purpose
Define the minimum XLL function contract required to inject IEEE edge values into Excel for `FP-C` scenarios.

This contract intentionally separates value injection from OxFunc semantics.

## 2. Required Worksheet Functions
The harness should expose these worksheet-callable functions:
1. `OXFP_NEG_ZERO()`
2. `OXFP_POS_INF()`
3. `OXFP_NEG_INF()`
4. `OXFP_QNAN(payload_id)`
5. `OXFP_SNAN()`

## 3. Behavior Contract
1. `OXFP_NEG_ZERO()`:
   - returns a double with sign bit set and magnitude zero.
2. `OXFP_POS_INF()`:
   - returns positive infinity.
3. `OXFP_NEG_INF()`:
   - returns negative infinity.
4. `OXFP_QNAN(payload_id)`:
   - returns a quiet NaN.
   - `payload_id` selects among at least two distinct payload encodings.
5. `OXFP_SNAN()`:
   - returns a signaling NaN if platform/runtime allows;
   - otherwise return a documented fallback and emit diagnostic text in logs.

## 4. Registration Contract
1. Each function must be registered as non-volatile.
2. Function names and argument arity must be stable.
3. Registration metadata must include harness version.

## 5. Logging and Provenance
Each harness build/run must emit:
1. harness version token,
2. compiler/runtime identity,
3. mapping notes for NaN payload encodings,
4. fallback notes if signaling NaN cannot be emitted.

## 6. Safety Constraints
1. Harness functions must not mutate workbook state.
2. Harness functions must not perform network I/O.
3. Harness functions should avoid locale-dependent formatting in returned values.
