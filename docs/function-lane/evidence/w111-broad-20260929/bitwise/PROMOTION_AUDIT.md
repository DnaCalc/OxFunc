# Bitwise evidence and completion review

Recommendation: retain Divergent/Structural for all five functions under
ODR-FN-005. The original numeric kernel and typed packet agree, but later
current-baseline text observations contain ten admitted discrepancies per
function. Those rows cannot be treated as a future alternate-locale phase.

| Function | Qualified discovery | Qualified discriminator | Independent numeric | Typed discovery | Independent typed | Later text exact/total |
|---|---:|---:|---:|---:|---:|---:|
| BITAND | 1,235 | 256 | 4,500 | 34 | 144 | 38/48 |
| BITOR | 1,235 | 256 | 4,500 | 34 | 138 | 38/48 |
| BITXOR | 1,235 | 256 | 4,500 | 34 | 136 | 38/48 |
| BITLSHIFT | 1,270 | 400 | 6,468 | 34 | 140 | 38/48 |
| BITRSHIFT | 1,270 | 400 | 6,465 | 34 | 134 | 38/48 |

Every numeric and original typed row in this table agrees exactly. The numeric
total is 34,246 qualified observations. Another 183 raw discriminator rows agree
at output but had changed Value2 negative-zero ingress and are excluded from
that total. The original typed total is 862. Counts are observations per packet,
not a promise that discovery packets contain no repeated tuple.

`promotion-audit.json` reproduces these counts and verifies all seven recorded
v2 family/formal source hashes still agree. Shared parser dependencies have
changed since that freeze; it is not a whole-binary freeze. Numeric heldout was
generated after the v1 kernel change. Typed heldout was generated after the v2
missing-argument correction, with the numeric kernel unchanged. The latest
focused public-dispatch regression still passes all three tests.

The kernel explanation is exact integer admission through 2^48-1, integer
bitwise operations, and observed shift admission/publication branches. The
49..53 zero-result branch and zero-operand bypass are documented in README.md.
The Rat/Int Lean substrate binds admission, missing arguments and these branches;
it does not stand in for full text parsing or rectangle construction.

Fresh local replay of the unchanged captured shared-grammar packet gives
190/240 exact admitted bitwise outcomes. The 50 misses are direct and reference
forms of `"2 000"`, `"R2"`, `"2026/09/29"`, `"1:00"`, and `"24:00"`.
Local returns VALUE while live Excel returns a number or NUM. The exact cases,
formula/fixture provenance and both outcomes are retained in
`promotion-audit-current-text.json`; their original-file SHA-256 values are in
the audit JSON. This is the captured en-ZA current reference profile, not an
inferred en-US parser. The shared context dependency is already tracked by
BUG-FUNC-057 / HO-FN-022. Further incompatible-array error-order coverage is
also advisable: the binary helper presently represents an absent coordinate
as a whole-pair NA. That is a coverage concern here, not a newly observed
bitwise discrepancy or justification to edit the shared helper without evidence.

OPERATIONS section 12 review (no completion claim):

| Item | Result | Evidence or unresolved requirement |
|---|---|---|
| 1. Contract rows promoted | No | No promotion performed; full text behavior remains unresolved. |
| 2. Lean aligned | Yes, bounded kernel | Executable integer/rational model and five bindings; full parser remains a separate open substrate. |
| 3. Rust/tests pass | Yes, retained slice | Three focused dispatch regressions pass; known broader mismatches remain. |
| 4. Deterministic replay | Yes | Retained per-function numeric and typed outcomes. |
| 5. Reproducible evidence | Yes | Original/retained hashes, source freezes, current audit packet. |
| 6. Version axes explicit | Yes, qualified | Excel16.0 build20430 x64, CV2; channel explicitly unavailable. |
| 7. Public/empirical distinction | Yes | Observed shift exceptions documented and used by kernel. |
| 8. XLL seam documented where material | Yes for packet | Public Value2/direct dispatch evidence; workbook-engine reconstruction not claimed; ingress exclusions recorded. |
| 9. Cross-repo impact | No for full closure | Shared numeric-text context receiving dependency remains open. |
| 10. No known gap | No | Ten current-baseline text mismatches per function. |
| 11. Language audit | Yes | No function-level completion or promotion claimed. |
| 12. Canonical worklist updated for promotion | No | Canonical owner review remains pending; this audit makes no promotion. |
| 13. Bead state | Partial | Existing bitwise BUG-FUNC-051/oxf-mwue.28.3 and shared grammar dependency remain; root owns canonical updates. |

Section 14 self-audit: scope reread fails completion because the full function
surface has known text gaps; completion criteria therefore fail. The silent
scope-reduction check passes because the numeric and typed packet boundaries
are explicit. The misleading-completion check passes: exercised bounded kernel
evidence is distinguished from the unresolved parser and receiving dependency.
This document supplies the requested review result without canonical promotion.

Status: `scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: current-baseline numeric-text
context; remaining array precedence coverage; canonical contract/formal review
after semantic repair; receiving dependency; orthogonal version/channel/locale
and platform sweeps. The orthogonal phases alone are not the reason for refusal:
the fifty captured current-baseline discrepancies are.
