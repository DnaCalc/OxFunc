# WORKSET - Bug Intake Root-Cause And Regression Stream Protocol (W072)

## 1. Purpose
Establish a canonical OxFunc bug-processing mechanism so defects are recorded
against exact source refs, triaged by ownership, grouped into canonical bug
streams, and carried through reproduction, root-cause analysis, similar-risk
scanning, validation, and closure with a durable local record.

## 2. Why This Packet Exists
Current OxFunc already has:
1. worksets for bounded implementation ownership,
2. `.beads/` for live execution state and blockers,
3. handoffs for cross-repo seam changes,
4. function-lane execution records and evidence artifacts for specific semantic
   slices.

What it did not have was one canonical local mechanism for:
1. intake of every incoming bug report,
2. duplicate-preserving linkage into a canonical known-bug stream,
3. explicit ownership and root-cause classification,
4. similar-risk scanning across adjacent semantic families,
5. stream-level closure criteria stronger than "fixed locally".

This packet adds that mechanism in an OxFunc-native shape after `W070`, keeping
bug intake separate from blocker and workset execution truth.

## 3. Scope
In scope:
1. define the canonical bug intake and regression-stream protocol in
   `OPERATIONS.md`,
2. add machine-readable scaffolding for:
   - individual bug reports,
   - canonical bug streams,
   - duplicate linkage between reports and streams,
3. require exact source-ref capture for every bug report and canonical stream,
4. require ownership classification, root-cause classification, and a
   "Why did we get this wrong?" section for every canonical stream,
5. require similar-risk scanning and follow-on-opening recording for every
   canonical stream,
6. add templates and empty directory scaffolding for immediate future use,
7. update the ordered workset surfaces so this packet is visible in the active
   sequence.

Out of scope:
1. solving any specific bug stream beyond the protocol/setup work itself,
2. replacing `.beads/` as the live blocker and execution-state surface,
3. reintroducing `CURRENT_BLOCKERS.md` or another prose blocker ledger,
4. automation for duplicate detection, ref inference, or stream generation,
5. changing OxFunc/OxFml seam ownership boundaries.

## 4. Deliverables
This packet should produce:
1. an `OPERATIONS.md` bug-protocol section with explicit lifecycle rules,
2. `docs/bugs/README.md`,
3. `docs/bugs/BUG_REPORT_REGISTER.csv`,
4. `docs/bugs/BUG_STREAM_REGISTER.csv`,
5. `docs/bugs/BUG_REPORT_TEMPLATE.md`,
6. `docs/bugs/BUG_STREAM_TEMPLATE.md`,
7. empty `docs/bugs/reports/` and `docs/bugs/streams/` scaffolding with
   `.gitkeep`,
8. workset-register and workset-readme updates that surface `W072`.

## 5. Local Integration Rules
1. `.beads/` remains the live execution-state and blocker truth for bug work.
2. worksets remain the bounded owner for code/spec/test/handoff delivery tied to
   a bug.
3. canonical bug streams own defect intake, duplicate linkage, root cause,
   similar-risk scanning, validation recording, and closure evidence.
4. exact source-ref capture is mandatory:
   - preferred: release/tag,
   - fallback: exact git commit SHA,
   - otherwise `unknown` with a reason.
5. duplicate reports are kept and linked; they are never discarded.
6. non-OxFunc ownership still requires an honest local stream plus any required
   handoff.

## 6. Closure Condition
W072 is complete for declared scope when:
1. `OPERATIONS.md` defines the required bug intake and lifecycle sequence,
2. the `docs/bugs/` scaffolding exists with registers, templates, and
   directories ready for use,
3. duplicate preservation, exact-ref capture, ownership classification,
   root-cause classification, and similar-risk scanning are explicit repo rules,
4. the ordered workset surfaces expose `W072`,
5. the local validation floor for workset/register shape passes.

## 7. Notes
This packet lands process and scaffolding only.
Future individual bug streams still need to populate the new mechanism with
actual report rows, stream notes, validation evidence, and follow-on workset or
handoff links as defects are discovered.
