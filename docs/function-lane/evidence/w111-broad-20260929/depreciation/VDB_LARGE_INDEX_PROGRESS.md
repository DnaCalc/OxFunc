# VDB large-index progress defect

Status: `scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: large-period index semantics,
guaranteed progress, switched VDB arithmetic, and independent validation after repair.
Tracking: `oxf-mwue.28.9.1`.

The first oracle attempt returned 16 exact observations through a start index of
`2^32-2`. Formula entry then stopped making progress on the next queued row,
whose start is `2^32-1`. The runner's initial watchdog covered explicit
calculation but not Formula2 entry. Root stopped only the verified newly created
Excel process (PID 22604) after more than 120 CPU seconds. The retained RPC
failure therefore records manual owned-session cancellation, **not** a
30-second timer outcome. The other 26 rows remain unobserved; neither an Excel
error nor an Excel number can be inferred for them. Root retains the original
runner and provenance correction separately; the raw answer is unchanged.

The current no-switch VDB loop advances an annual index stored as binary64 by
adding one. At `year = 9007199254740992` (`2^53`), round-to-nearest-even gives
`year + 1 == year`. With a larger end period and a nonzero annual amount, the
loop cannot progress. This is a source-level proof; the dangerous input has
deliberately not been executed locally. It is separate from the already observed
finite-interval arithmetic differences and must not be hidden by their bounded
matching cohorts.

An illustrative admitted tuple is `VDB(1000,0,18014398509481988,
9007199254740992,9007199254740994,2,TRUE)`. The start and end are distinct finite
binary64 values, and the interval width is two. Both `year + 1` and the loop's
`year += 1` can round back to the same year. The existing switched path also has
unbounded work from zero for huge periods; its numerical schedule is separately
known to differ from Excel.

`oracle-only-vdb-large-index.json` in the campaign cache contains 42 black-box
Excel requests around `2^31`, `2^32`, `2^52`, `2^53`, `2^54`, and `1e20`, with
short representable intervals and factors zero and two. Its adjacent manifest
records exact inputs, widths, and generator hashes. Local replay is explicitly
forbidden until the progress defect has a supported repair. The source generator
is `smart-fuzzer/tools/w111/gen_vdb_large_index_oracle_20260929.py`.

A repair must establish Excel's annual-index conversion and interval behavior
from those observations. Candidate approaches are an exact integer index with
an explicitly bounded iteration count, or a mathematically derived grouped or
closed-form calculation where its floating-point store graph has been verified.
Simply moving to the next representable float skips calendar years and is not
yet justified. An arbitrary iteration cap or an invented `#NUM!` result is not a
semantic repair. No such cap or replacement error has been added.

The existing no-switch evidence (8,299 discovery rows and 6,473 independently
generated heldout rows for its second candidate) exercises finite, progressing
intervals only. It does not discharge this defect or characterize all VDB input
periods. Root owns the canonical issue and register updates.
