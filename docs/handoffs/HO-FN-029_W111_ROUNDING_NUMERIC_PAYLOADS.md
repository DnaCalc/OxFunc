# HO-FN-029: rounding endpoint numeric payload publication

Direction: OxFunc → OxFml. Status: `filed`, acknowledgment pending.
Owner: W111, `oxf-mwue.28.14`; receiving dependency `oxf-mwue.28.14.4`
(`BLK-W111-ROUNDING-PUBLICATION`).

The [impact assessment](../function-lane/evidence/w111-broad-20260929/rounding-boundaries/ENDPOINT_IMPACT_ASSESSMENT.md)
records a result-carrier question exposed by black-box rounding observations.
For maximum finite binary64 input and count zero, ROUND, ROUNDDOWN and TRUNC
publish Value2 bits `0x7ff000000000000a`. ROUND at count -308 publishes
`0x7ff1ccf385ebc8a0`. Negative inputs preserve the sign bit. These observations
come from normal, finite, exactly read-back inputs, so the existing signed-zero
and subnormal input exclusions do not apply.

Worksheet ISNUMBER returns TRUE, ISERROR returns FALSE and TYPE returns 1.
Text displays numerical strings, including `2E+308`; IFERROR preserves the
result and self-equality returns TRUE. Adding zero produces NUM. A separate
224-row worksheet packet and 960 repeated controls across two Excel processes
reproduce the newer endpoint capture. The latter varies cell number format,
Calculate/CalculateFull, Formula2/Formula2R1C1 and dependent recalculation;
input readback remains exact and output bits remain stable throughout.

The earlier broad TRUNC capture reports a lower finite result at MAX instead.
That conflicting observation remains unresolved and retained. The differences
in the retained bulk runner versions do not explain it. The newer repeats
establish reproducibility under their recorded conditions, not a universal
resolution of the earlier contradiction.

A blanket finite-to-NUM publication rule loses the observed numeric kind and
bits. Simply passing these bits as an ordinary IEEE NaN does not explain the
observed display and comparison behavior either. CalcValue::Number(f64) can
carry the bits, but function dispatch, arithmetic, comparisons and display must
agree on what they mean. This packet asks OxFml to review whether an extended
numeric publication carrier is required and which downstream operations convert
such values to NUM. No shared carrier or evaluator policy is changed here.

The lower endpoint is a separate finite lane: directed rounding of minimum
normal can publish `0x000ffffffffffffa`, a subnormal numeric output. It survives
Value2 and addition of zero, and displays zero. OxFunc's finite directed-rounding
repair is evaluated separately; it must not be blocked by this upper-endpoint
question or silently flushed by the receiving evaluator.

Reference: Excel 16.0 build 20430, 64-bit, workbook Compatibility Version 2;
channel unverified. Retained public-interface scripts and raw typed/bit records
are under `rounding-boundaries/`. No binary inspection or internal inference is
used as evidence.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: prior-capture discrepancy,
payload publication semantics, receiving acknowledgment and exercised consumer
integration; finite endpoint candidate validation is tracked independently.
