# Strict NEGBINOM seed discrepancy rechecked

The existing unit assertion for NEGBINOM.DIST(5,3,0.4,TRUE) expects
0x3fe5e849aaeed68d. Fresh Excel 20430/CV2 Value2 observations, including three
same-process repeats, confirm that value. Both the frozen starting-revision
engine and the current engine produce 0x3fe5e849aaeed68e. The strict expected
value is preserved; this is an unresolved numeric discrepancy, not a stale pin.

Eight adjacent probability bit patterns on either side and the repeated center
produce 20 observations each for NEGBINOM.DIST and its paired BETA.DIST CDF.
Each engine agrees on 7/20 per function, with identical baseline/current output.
This supports locating the retained error in the existing beta CDF dependency;
it does not identify a general repair. Exact rows and both engine judgements
are retained. Only public interfaces were used.

scope_completeness=scope_partial; target_completeness=target_partial;
integration_completeness=partial. Open lanes: beta CDF arithmetic and exact
primitive alignment. The combined Rust test remains legitimately failing.
