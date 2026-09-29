# QUOTIENT: W111 numerical publication and typed preparation

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Open lanes: shared locale/date/year numeric-text preparation (HO-FN-022),
reference/evaluator integration, unexercised multi-area or unresolved
reference forms and final whole-function review.

The original 1,362-row numeric capture has 1,344 admitted rows. Eighteen raw
negative-zero or nonzero subnormal inputs are withheld because Excel Value2
changes those inputs. Explicit positive zero remains admitted. Of the 151
admitted baseline differences, 149 published negative zero where Excel returned
positive zero; two published infinity where Excel returned #NUM!.

The numerical candidate divides in binary64, rejects a zero denominator with
#DIV/0!, rejects nonfinite division with #NUM!, truncates the finite quotient
toward zero, and publishes either signed integer zero as positive zero. It
matches all 1,344 admitted discovery rows and all 1,797 fresh independent rows.
The independent packet includes 1,200 random normal bit pairs, 480 neighboring
integer-ratio probes, and 117 explicit zero/normal-boundary/overflow controls.
None of its inputs needed ingress exclusion.

The first 30 typed probes found additional argument rules: logicals return
#VALUE!, and explicitly omitted arguments return #N/A. The 186-row refinement
establishes these rules for direct, referenced and lifted array values, and
left-to-right coercion/error precedence. Referenced blank cells still become
zero; admitted text uses the shared numeric parser. QUOTIENT now prepares its
resolved values with those two local kind substitutions, then reuses the
existing binary broadcast and coercion machinery. All 216 discovery/refinement
rows and 288 newly seeded independent typed/broadcast rows replay exactly.

`w111_quotient_live_replay.rs` exercises production dispatch for 3,141 admitted
numeric rows; `w111_numeric_text_live_replay.rs` exercises the 504 typed rows.
Both test targets pass. The executable Lean QUOTIENT model now includes local
kind handling, left-to-right precedence, signed-zero publication and nonfinite
publication. `lake build OxFunc.Functions.QuotientFn` passes.

At the first typed freeze, `QUOTIENT_META.coercion_lift_profile` remained
`UnaryNumericScalarOnly`, despite exercised binary lifting and custom kind
policy. That historical declaration mismatch motivated the later root-owned
cross-repository assessment. The reference refinement below now changes only
QUOTIENT's declarations to `Custom` and `RefsVisibleInAdapter`. The
numeric-only Q dispatch already receives prepared numbers and continues to call
the same kernel; the ordinary by-index dispatch calls `eval_quotient_surface`.

Artifacts retain full input bits or typed cases and exact Excel outcomes.
`candidate-freeze.json` predates the 1,797 numeric and first 30 typed captures;
`typed-candidate-freeze.json` predates the 288-row independent typed capture.
The additional typed observations refine the original numeric candidate, so
they are discovery evidence rather than a retroactive independent validation.

A later cross-family audit distinguishes array values from true multi-cell
references. None of the retained 30 + 186 + 288 typed cases supplies a
multi-cell-reference argument. They therefore establish no such reference
admission rule. MROUND's separate fresh controls exposed a reference-origin
lane, motivating this explicit qualification without asserting that QUOTIENT
shares its behavior. No QUOTIENT production change follows from that analogy.

## Reference and positional refinement

A later independent 80-case reference packet has 54 baseline structural
failures. It distinguishes true multi-cell references, which return VALUE,
from explicitly materialized arrays, which retain ordinary array lifting.
Caller alignment does not select a cell. The local adapter now preserves the
original reference kind through resolution and inserts VALUE at that argument
position when the resolved extent has more than one cell. Unit references
retain existing scalar policy. `REFERENCE_IMPACT_ASSESSMENT.md` records the
resulting `RefsVisibleInAdapter` and `Custom` declaration changes for HO-FN-027.

The first frozen 512-case follow-up matches every reference-origin control but
fails five materialized asymmetric-array cases. Those failures retain an earlier
left coercion error over a later missing right coordinate. The source now uses
the existing ordered binary helper after QUOTIENT's own kind/reference mapping.
No generic adapter or numerical kernel changes. The 512-case failed candidate
and original outcomes remain under `reference-heldout/`.

Current replay matches 1,720 typed observations through both direct and public
dispatch, with all 3,141 numeric observations unchanged. Five prepared replay
tests, two numeric tests, the metadata golden and seven targeted Lean jobs pass.
The successor seven-file freeze precedes 624 new asymmetric shape and reference
controls, all exact. All seven source hashes and the QUOTIENT golden-row hash
remain unchanged at the independent replay. The `reference-origin-audit.json`
record describes only the earlier 504-case corpus and remains historical.
