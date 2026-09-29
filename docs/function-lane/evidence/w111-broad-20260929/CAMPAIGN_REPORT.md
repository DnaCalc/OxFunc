# W111 campaign progress and status, 2026-09-29

The user requested this round be wrapped up and committed. Code, models, tests,
tooling and retained evidence are in commit `ffcb9a7728195f359bdb54a1e97af0a07c8f816e` on
`codex/excel-parity-20260929`. The campaign automation is paused. W111 remains
`in_progress`; no whole-function promotion is made.

`scope_completeness=scope_partial`; `target_completeness=target_partial`;
`integration_completeness=partial`.

The broad discovery waves exercised 288 distinct functions, including 39
previously unverified functions in the typed sweep. A separate provider-qualified
locale packet covers six additional surfaces. Functions requiring unsupported
host/reference context remain explicitly unverified.

## Replayed progress

The [final frozen snapshot](snapshot-wrap-1952/report.json) compares the original
executable and the final candidate on exactly the same qualified numerical inputs.

| Bank | Admitted observations | Original exact | Final exact | Newly exact | New misses |
|---|---:|---:|---:|---:|---:|
| First numerical wave | 138,345 | 126,462 | 132,866 | 6,404 | 0 |
| Second numerical wave | 83,245 | 74,939 | 81,899 | 6,960 | 0 |
| Total | 221,590 | 201,401 | 214,765 | 13,364 | 0 |

The broad typed and numeric-text packets retain 3,425 admitted comparisons,
2,899 exact and 526 differing. All 3,542 raw typed outcomes are unchanged from
snapshot-1735; qualified new misses are zero. That comparison uses an earlier
exact-ingress snapshot, not the initial captures affected by the JSON parser
defect. [Regression accounting](snapshot-wrap-1952/regression-summary.json)
retains every transition and withheld-row reason.

The ledger remains conservative: 260 divergent, 219 consistent and 48
unverified functions. Passing a sampled family bank does not erase older
counterexamples or establish whole-domain identity. There are 6,825 numerical
and 526 admitted typed differences in the broad snapshot; additional targeted
residuals remain in the family records.

## Repair evidence retained

These are observations, including deliberate repeat controls. Each family
record separates discovery, failed candidates and fresh independent validation.

| Family | Current retained evidence and qualification |
|---|---|
| ATAN | 181,495 numerical and 841 typed matches; reduced-angle primitive and platform limits retained. |
| INT | 123,229 numerical matches, including 31,210 fresh inputs after the last boundary refinement. |
| POWER | 61,742 numerical matches; the first 38 independent failures led to width and signed-scale refinements. |
| LOG / POWER / FISHER / MEDIAN / HARMEAN / DEVSQ preparation | 4,944 typed matches, including 2,190 fresh origin-order cases; LOG omission adds 4,827 numeric controls. |
| HARMEAN / DEVSQ arithmetic | 39,890 numerical matches; prepared contributions preserve numeric order while checking direct scalar coercion errors first. |
| MROUND | 47,964 numerical and 1,514 typed matches, including 384 fresh reference cases; empirical halfway cutoff and broader reference limits remain explicit. |
| MOD | 151,438 numerical and 1,067 typed matches, including 10,784 fresh production numeric and 288 final fresh typed cases. |
| QUOTIENT | 3,141 numerical and 1,720 typed matches, including 624 fresh asymmetric-shape/reference cases. |
| Switched VDB | 113,329 numerical and 522 fresh typed matches; 71,760 unchanged DB/DDB/no-switch consumer controls. Huge-start progress failures remain open. |

Further retained repairs and counterexamples cover FACT, radix conversions,
dates, SLN/DB/DDB/SYD, inverse trigonometry, numerical publication, rounding,
text slicing/rendering, counting, conditional preparation and distributions.
The [campaign evidence index](README.md) links the individual packets.

## Combined checks

- Rust workspace: **2,003 pass, 3 fail, 8 ignored**, across 61 test targets.
- The three failures are the retained NEGBINOM one-ULP test and two normal-CDF
  replay tests. Fresh Excel rechecks support retaining those expectations;
  assertions were not weakened to obtain a pass.
- Full Lean build: **526 jobs pass**. An earlier Windows module-name case
  collision in generated outputs was corrected and both logs are retained.
- Smart-fuzzer engine: two tests pass; typed helper package: five tests pass.
- All 1,851 captured source hashes remained unchanged through final replay and
  validation. Raw hashes and LF-normalized source hashes are both retained.
- Staged source hashes match the frozen candidate. The byte audit verified
  2,436 retained evidence files against Git's index.
  The evidence directory disables text conversion so checkout line endings
  cannot change its artifact hashes.

After the tested code/evidence commit, three redundant source/test/tool EOF
blank lines were removed. The [before/after hash record](verification/eof-formatting.json)
verifies that no other bytes changed. Captured evidence logs retain their original
EOF bytes. This formatting-only follow-up does not alter numerical behavior.

Commands, exact failures, source binding and logs are in
[combined-checks.json](verification/combined-checks.json). The retained compressed
ADDRESS artifact contains oracle witnesses, not session history. No session
transcripts are part of these commits.

## Open lanes and next work

1. Distribution backends: normal CDF/ERFC, NEGBINOM/BETA and discrete interior
   arithmetic. The numerical wrappers have separate evidence; backend errors
   remain visible in strict tests.
2. Small-input SINH and dependent TANH/COTH, complex arithmetic/formatting, and
   exceptional decimal parser/rendering/rounding boundaries. Rejected graphs
   and unsuccessful independent candidates remain retained.
3. Reference and evaluator semantics beyond the admitted resolved-reference
   slices, including multi-area/unresolved references, resolver failure order,
   context providers, scheduling and reference-returning expressions.
4. HO-FN-022 through HO-FN-030 require receiving acknowledgment and exercised
   OxFml integration. QUOTIENT now preserves reference origin until its native
   adapter; MROUND and MOD have separate, exercised preparation policies.
5. The unsupported WEEKNUM oracle contradiction (`oxf-mwue.28.7.1`) and large-start
   VDB oracle nonprogress (`oxf-mwue.28.9.1.1`) require better bounded public-
   interface probes. Stable ordinary cases do not discharge those blockers.

The current reference is Excel 16.0 build 20430, 64-bit, Compatibility Version 2,
1900 dates, precision-as-displayed disabled; channel is unverified. Alternate
locale/version/platform sweeps are orthogonal future phases. The current
baseline still has semantic gaps independently of those future phases.

Only public interfaces and reproducible behavior were used. Value2 substitutions,
unsupported selector instability, malformed formulas and missing host context
are qualified explicitly. Normal-input subnormal or nonfinite-encoding outputs
remain in scope. One accidental overlap of two typed captures was repeated
serially with identical results; only the serial repeats count.

Reviewed inbound observations: `../OxFml/docs/upstream/NOTES_FOR_OXFUNC.md` was
reread during wrap-up. No receiving acknowledgment for the new packets was
established. Filing a handoff remains an open dependency.

## OPERATIONS Section 12 checklist

This audit covers the declared broad W111 scope, not just successful banks.

| # | Result | Evidence or remaining obligation |
|---|---|---|
| 1 Contract rows promoted across scope | No | No whole-function promotion; unverified and divergent functions remain. |
| 2 Formal obligations across scope | No | Exercised family models/build pass; empirical primitive and wider semantic obligations remain. |
| 3 Required Rust tests across scope | No | Three strict tests fail. |
| 4 Replay per in-scope behavior | No | Replays cover the admitted banks, not all non-deferred behavior. |
| 5 Evidence links reproducible across scope | No | Final banks/source freezes are retained; the earlier SINH lifetime source was unavailable and is labelled historical/nonreplayable. |
| 6 Both version axes explicit | Yes | Build 20430/CV2, channel unverified, host profile retained. |
| 7 Documentation/empirical discrepancies resolved | No | Remaining semantic and oracle conflicts are explicit. |
| 8 Material verification-seam limits recorded | Yes | Per-packet ingress, provider, reference and output-publication qualifications; no full XLL-host claim. |
| 9 Cross-repo impact and handoffs | Yes | HO022-HO030 and receiving dependencies filed; integration not inferred. |
| 10 No known semantic gap | No | Open lanes listed above. |
| 11 Completion-language audit | Yes | Successful observations are not labelled whole-function completion. |
| 12 Feature worklist updated | Yes | W111 round status and later family refinements recorded. |
| 13 Bead state updated | Yes | Campaign/refinement tasks remain in progress; named receiving/oracle blockers remain open. |

## OPERATIONS Section 14 self-audit

| Step | Result |
|---|---|
| 1 Re-read full scope | Fail for completion: all non-deferred functions remain the objective; this round exercises subsets. |
| 2 Re-read pass criteria | Fail for completion: strict failures, wider semantic evidence and receiving integration remain. |
| 3 Silent scope reduction | Pass: no scope was silently narrowed; sampled banks and exclusions are explicit. |
| 4 Misleading completion patterns | Pass: failed candidates, executable model limits, ignored tests and unacknowledged handoffs are visible. |
| 5 Include audit result | Pass: this report retains both audits and all three partial axes. |

W111 remains in progress. The next round can resume from the retained snapshot,
family counterexamples and ordinary bead graph without repeating the discovery
sweeps.
