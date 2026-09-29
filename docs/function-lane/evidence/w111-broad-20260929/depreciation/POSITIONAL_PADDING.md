# Financial argument padding impact

Receiving-dependency record: [HO-FN-028](../../../../handoffs/HO-FN-028_W111_POSITIONAL_ARGUMENT_PADDING.md),
bead oxf-mwue.28.14.3. This packet's scope is SLN, SYD, DB, DDB,
and VDB; this is a function-boundary observation, not a new resolver contract.

The 75-row public Excel packet contains a wider earlier argument and a narrower
later argument. At a coordinate absent only from the later argument, an earlier
explicit REF/DIV0/NUM error or invalid text retains its scalar coercion result.
The former generic adapter replaced that result with NA. The baseline matched
15/75 controls and failed 60 cases, evenly across the five functions. The 15
matching controls distinguish an earlier invalid numeric domain from an earlier
coercion error: all arguments are coerced before the scalar numeric kernel runs.

The financial wrappers now reuse the already exercised positional-NA helper:
an absent coordinate becomes NA in its original argument position, then normal
left-to-right scalar coercion chooses the error. The generic adapter and
dispatcher are unchanged. All five function declarations use SurfaceNative;
there is no LiftAt declaration correction. Fresh controls also compare scalar
arguments with one-cell arrays alongside incompatible shapes.

Strict replay of discovery is 75/75. The independent 584-row packet matches
584/584 with every execution admitted: SLN 83, SYD 124, DB 125, DDB 125 and
VDB 127. It varies horizontal and vertical shapes, both error orders, optional
positions, invalid numeric domains, and one-cell-array versus scalar controls.
The Lean family binding reuses DistributionArguments' ordered numeric coercion
substrate; its theorem pins both REF/NA orders and missing-to-zero preparation.
Rectangular construction is exercised by Rust and these live shape observations.

The existing reference preparation, reference identity and value origins are
preserved. Explicit required missing arguments become zero on these financial
surfaces; SYD's correction is evidenced separately. For DB/DDB/VDB optional
positions, absence selects the documented existing default while explicit
missing or blank numeric positions become zero. Padding NA is an error value,
not a blank cell or missing argument, and must remain distinct upstream.

OxFml should retain the function's native lifting result rather than applying
an additional padding-priority rule. This packet requests review of that
expectation, including explicit missing/default distinctions; it does not claim
that upstream has acknowledged or integrated the change. No OxFml files were
modified here. The shared helper is currently named distribution_common even
though its prepared-value algorithm is generic; renaming is a separate local
organization decision.

Status: `scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: receiving-repository acknowledgement/integration;
shared numeric-text grammar and orthogonal baseline axes.
