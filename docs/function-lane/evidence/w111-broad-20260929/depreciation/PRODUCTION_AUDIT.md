# Switched VDB production and progress audit

Status: `scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: huge-period progress and index
semantics, shared locale-sensitive coercion, receiving-dependency integration,
alternate baseline/platform validation.

The final source is frozen in `switched-production-v1-formatted-freeze.json`.
The earlier unformatted production freeze remains preserved. Its 8,000 numeric
and 522 typed independent cases all matched before formatting. Final replay of
113,329 switched observations and 71,760 financial compatibility observations
matched 185,089/185,089. Twenty-eight focused tests pass, including 44 new exact
public-dispatch witnesses. The Lean family compiles; recovery validation proves
21 prior model bodies definitionally equal and the recursive DB book model
extensionally equal. `formal-recovery-validation.json` states the recovery basis
and does not count the discarded empty-source compilation.

The source audit checked the initial fractional adjustment, rate and amount
publication, clip-before-switch order, inclusive 2^-1026 comparison, fixed
straight-line tail, stored absolute cursor, final-step stop, and different book
subtraction stores. The production source also checks the advance total for
finiteness; the research path discards that total. No observable difference was
found in retained observations or 1,600 additional bounded MAX-cost advance
controls, but those additional controls compare two local models and are not
Excel evidence. Unit-period multiplication by one is replaced by its identical
stored amount. No arbitrary iteration cap or invented worksheet error was added.

The large-index bead `oxf-mwue.28.9.1` is in progress; its oracle blocker child
`oxf-mwue.28.9.1.1` remains open. Only 16 rows of the hazardous oracle packet were
observed. The next formula at start 2^32-1, even with factor 0, stalled during
Formula2 entry. Manual cancellation of the verified owned Excel process after
more than 120 CPU seconds is distinct from a timer outcome. The other 26 rows
remain unobserved. Neither those rows nor the admitted 2^53 local example were
replayed during this audit.

The existing no-switch `year += 1` loses progress at 2^53. The new switched
advance/requested cursors can also cease progressing or require infeasible
numbers of iterations at huge indices. Replacing plus-one with nextafter would
skip calendar years and lacks support. A limited exact acceleration is possible
when an advance step has zero amount and leaves every state field unchanged:
all subsequent identical advance steps preserve that state. This does not settle
nonzero repeated-rounding or annual-index conversion, and no acceleration was
added in this audit.

The minimal safe next packet has 10 DDB annual controls near 2^32 and 2^53, using
its previously exercised closed-form annual path, plus 20 VDB controls with the
same huge lifetimes but start/end restricted to 0, 1, 2. Files are
`progress-safe-ddb-controls.json` and `progress-safe-vdb-controls.json` in the
campaign cache. These can distinguish huge-lifetime admission and annual power
publication from the interval-progress failure. They cannot establish the
unobserved huge-start VDB outputs or justify a replacement error. The hazardous
42-row packet should not be retried as part of this proposal.


The safe follow-up was captured live and matches DDB 10/10 and VDB 20/20.
`progress-safe-controls-judgement.json` retains the result and executable hash;
raw oracle answers remain beside the exact input packets. These support the
annual/lifetime distinction only. The huge-start blocker remains open.
