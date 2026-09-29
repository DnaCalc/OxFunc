# POWER consumer audit

Status: `scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes include untested POWER inputs in
consumers, existing FV/PV discrepancies, typed/reference preparation, and wider
financial domains. This is retained-observation dependency evidence, not a
whole-function parity claim.

The numeric public dispatcher was compared using immutable executable copies:

- Prior `.tmp/sf-snapshot-1735.exe`: SHA256
  `13e760982df3b51ec41c6761e858fcc1def00386172748237bcf97dca2f0ce3a`.
- Current `.tmp/sf-power-consumer-current-20260929.exe`: SHA256
  `ef46a5e55717395b63dc0802d5cde965aac0daca011217b08bc94ad1a4e62449`.

Every mismatch was requested from each engine. Per-input actual outputs were
compared, so unchanged totals cannot conceal gained and lost matches.

| Consumers | Retained observations | Old/current exact | Changed outputs |
|---|---:|---:|---:|
| FV/PV, seven banks | 2,681 | 2,364 | 0 |
| DB, eighth heldout | 35,783 | 35,783 | 0 |
| DDB, sixth heldout | 19,002 | 19,002 | 0 |
| VDB no-switch, fifth heldout | 16,975 | 16,975 | 0 |
| XNPV, five historical banks | 2,170 | 2,170 | 0 |

The three financial source files and shared `excel_numeric/mod.rs` and `x87.rs`
match the 1735 source hashes. POWER differs. `source-comparison.json` retains
those hashes. No consumer source was changed for this audit.

FV/PV summaries retain input IDs with negative-zero or subnormal Value2 ingress
qualifications; these are comparison observations and do not establish Excel
input-bit fidelity. The selected depreciation banks are already qualified by
their owning campaign. The 317 FV/PV discrepancies predate the POWER change.
Observation totals include repeated controls and are not globally unique inputs.

XNPV requires arrays, which the numeric `sf` CLI does not accept. The separate
probe extracts the unchanged consumer's 4,653 distinct POWER calls and compares
them directly against both frozen engines. All 4,653 agree in both engines.
It then composes the old factors through the unchanged staged XNPV additions
and divisions. The current composition is asserted equal to the current
production XNPV kernel on every row. All 2,170 retained Excel outcomes agree;
234 records fail unchanged validation before reaching POWER. This validates
the numerical dependency path, not new typed/reference semantics or a frozen
full public-dispatch XNPV executable.

`financial/` and `depreciation/` retain source-bank paths/hashes, both complete
miss reports, and change classifications. `xnpv/` retains the original combined
observations and their source hashes, derived POWER calls, both engine reports,
and the final comparison. The two reproducible audit tools are copied here.
No Excel session was launched and no financial production source was edited.

After POWER's subsequent integer-width and signed-decimal-scale refinement,
the audit was repeated with `.tmp/sf-power-width-candidate-20260929.exe`, SHA256
`670cdc9b2d642870533bac8be634bc25eb6bf097afbd74ffb5b1bb73bc5d5064`.
The `width-refined-financial/` and `width-refined-depreciation/` reports retain
the repeat comparison. All 74,441 numeric-dispatch observations still have
identical outputs, and all 4,653 derived XNPV POWER calls remain identical.
Thus the 2,170 XNPV compositions also remain unchanged, bringing the observed
consumer comparison to 76,611 outcomes, with 317 pre-existing FV/PV discrepancies.
