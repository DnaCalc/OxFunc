# W111 read-only evidence audit

Status: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.

All Markdown evidence links in HO-FN-022 through HO-FN-030 and the broad README
resolve. Hash/size inspection covered 783 entries across 19 family manifests.
One stable inconsistency was found: distribution-surface/negbinom-seed-recheck/
artifact-manifest.json has a stale hash for validation.json. MROUND's impact
assessment hash and size were also stale while its owner was actively refining
reference semantics; refresh that manifest only after its final artifact edits.
No missing retained file was found in those manifest entries.

The broad README has outdated statements requiring the root's final summary
pass: switched VDB is described as independently validated production near the
opening and as research later; the serial Excel statement omits the explicitly
retained overlapping provisional runs and serialized repetitions; combined
validation counts and the handoff list predate current results and HO-FN-030.
The narrow parser description likewise predates its later decimal refinement.
Historical snapshot counts should remain marked historical, not overwritten.

A source hash scan covered 82 repository source paths named in family freezes.
Most unmatched hashes belong to historical shared-family snapshots, later
prepared refinements, or active MOD/MROUND/routing/metadata changes. No new
unexplained numerical drift was established by that coarse scan. The scan is
not a substitute for the root's final combined source freeze and replay.

The QUOTIENT 504-case prior typed corpus contains no multi-cell references.
The later independent 80-case reference packet now establishes 54 differences;
that separate repair is assigned after this audit and must preserve its failed
baseline. Handoff wording should qualify materialized-array claims by origin.
No canonical registers, handoffs, source files or old freezes were edited during
this audit. The root owns the listed summary/hash corrections.
