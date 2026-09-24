# W112 Parity Driver: Smart-Fuzzer As The One Engine

Status: `planned`
Date: 2026-09-22
Absorbs: W107 (smart-fuzzer platform roadmap, never started)
Builds on: W109 (racer, oracle, cache), W111 (ExcelParity field and ledger)

## 1. The problem in one paragraph

Getting a function to bit-exact today means hand-assembling a campaign from a Rust
racer with 847 one-off binaries, a PowerShell oracle and cache, forty-odd probe
scripts, and then writing the result into prose that nobody regenerates. That works
for one hard kernel at a time. It does not work for "every function, checked again
when Excel changes, for years". We want one engine that knows what is unverified,
what is divergent and by how much, what to run next, and where to write the answer.
Everything else is a tool it calls.

## 2. What it looks like when it works

```
sf plan                  # reads the ledger, prints the next things worth doing
sf judge COUPDAYS ...    # generate a corpus, ask Excel, compare bits, propose ledger rows
sf sweep TEXT --locale en-US
sf race oddfyield.json  # race candidate graphs offline against banked witnesses
sf ask <batch>           # the only command that touches Excel; everything else is offline
sf report               # ledger proposal, draft catalog rows, status map, Handbook export
```

That is the whole surface. One Rust CLI in `smart-fuzzer/engine/`, PowerShell kept
for the COM part, files in git, no daemon, no database.

## 3. Two kinds of work

The split that matters most in practice is **needs Excel** versus **does not**.

- **Needs Excel**: asking for bits. This is the scarce resource (one instance, one
  host profile, ~8.8k probes/s). `sf ask` is the only path, it goes through
  `Range.Value2`, and every answer lands in the cache with build, bitness, CPU and
  Compatibility Version. Nothing else in the engine talks to Excel.
- **Does not need Excel**: everything else. Generating corpora, comparing bits,
  classifying severity, racing candidate graphs against banked witnesses, ranking
  which probes would be most informative to ask next, writing reports. This can run
  anywhere, in parallel, overnight.

The engine's job is to make the offline side do as much as possible so the Excel
side is only asked the questions that matter.

## 4. Four things the engine does, matching the ledger statuses

| You have | You run | You get |
|---|---|---|
| `Unverified` function | `sf judge` | `Consistent` or `Divergent{severity}` with the rows and build recorded |
| `Consistent` function | `sf sweep` (fresh held-out corpus, edge rows) | `Characterized`, or a miss that demotes it |
| `Divergent` function | `sf race` + `sf ask` in a loop, then `sf judge` on a held-out set | a landing, or a wall note with the residual's structure |
| a new Excel build | `sf judge --rebuild` over judged functions, oldest first | updated `excel_builds_judged` |

`sf plan` just orders the queue by the ODR-FN-005 scale (unverified with a corpus,
then structural, then harness-blocked, gross, numeric, last-bit, promotion sweeps,
walls last) and by how many functions a shared substrate unblocks. It prints a list.
A human picks from it. The engine writes ledger *proposals*; a human accepts them,
usually by reading the diff and committing.

## 5. Where the existing pieces go

- **calc_graph_racer** core (DSL, enumerate, eval, score, scheduler): linked in as
  `sf race` / `sf schedule`. Candidates are JSON, witnesses are banked oracle answers.
  The 847 bins retire in batches once their evidence refs are in the ledger; new
  work is a JSON candidate set, not a new bin.
- **Run-W109BulkBatch, CellRefBatch, OracleCache**: become `sf ask`. Same plumbing,
  one entry point, cache in front.
- **Build-* / Run-* scripts**: each one is either a generator (`sf judge --gen X`),
  a lane (`--lane workbook|lambda|locale|random`), or retired with a line in the
  inventory. None is run by hand in a campaign.
- **Build-FunctionStatusMap**: becomes `sf report`; the status map is a view over
  the ledger.
- **runs/*/rollup.json**: same artifacts, one schema; `sf report` turns them into
  evidence refs and draft catalog rows.
- **Discrepancy catalog, wall-clues ledger, method doc**: unchanged. The engine
  writes draft rows into the catalog; the method doc is how you drive `sf race`.

## 6. Rules that stay

1. The engine never changes production kernels. A landing is an ordinary bead in the
   owning lane; the engine gives it the evidence and the held-out re-judge.
2. Only typed-bit comparison against live Excel through Value2 is evidence. Local
   agreement, sampling and metamorphic checks are hints.
3. Black-box only.

## 7. Phases

Each is one bead and ends with a real campaign, not a document.

- **P0 Inventory**: one page listing every tool, script, bin and run dir with keep /
  fold / retire. Bank the racer bins' evidence as ledger refs while doing it.
- **P1 `sf ask`**: fold the three oracle scripts into one entry point with the cache
  in front; one artifact schema for what comes back. Run 10k rows through it.
- **P2 `sf plan` + `sf judge`**: the CLI reads the ledger, prints the queue, runs
  the Judge loop on the COUP* family, writes a proposal. Needs W111-3.
- **P3 Generators and `sf sweep`**: domain-covering corpora with edge rows;
  severity classification equal to `DivergenceSeverity`; first Characterize run on
  one exact-by-construction family and one numeric family.
- **P4 `sf race`**: racer linked in, richer probe ranking, held-out re-judge; first
  Identify run on ODDFYIELD; start retiring bins.
- **P5 Lanes**: workbook-context, lambda, locale, random. Unblocks the harness-
  blocked and Cat-1 sets. Needs P2's judge, not P3, so it runs alongside P3.
- **P6 `sf report` and rebuild**: evidence refs, catalog rows, status map, Handbook
  export, minimal repro for every new Divergent, and the re-judge pass on a new
  reference build.

## 8. Non-goals

No UI, no service, no scheduler daemon, no plugin framework. If a later phase needs
one of those it says so in its bead first.

## 9. Where W107 went

W107's fourteen beads (`oxf-7b0z.*`) were closed when W112 absorbed it.

| W107 bead | Home in W112 |
|---|---|
| .1 audit, status-map regeneration | P0 inventory; status map in W111-4, then P6 |
| .2 non-interference guardrail | Rule 6.1; no separate check |
| .3 schema and artifact dialect v1 | P1 answer schema |
| .4 typed invocation model | P1 answer schema (arg bits, types, shape); P3 generators |
| .5 batched Excel oracle executor | P1 `sf ask` |
| .6 local runner, prepared-call harness | P2 local side (OxFunc registry dispatch); OxFml prepared-call lane only if a P5 lane needs it |
| .7 coverage taxonomy, interestingness | P3 branch-region coverage |
| .8 typed mutator, coverage scheduler | P3 edge-set generators; mutator engine not built |
| .9 semantic feedback queue | Not built; revisit if a P3 sweep finds a miss the branch regions did not predict |
| .10 minimizer and promotion | P6 minimal repro per new Divergent |
| .11 experiment index, scheduler | P2 `sf plan` over the ledger; no SQLite index |
| .12 Category-1 downstream runner | P5 lanes, with W104 `oxf-oyrz.5` |
| .13 CI smoke, nightly, runbook | P6 offline CI smoke and operator section; no nightly daemon |
| .14 terminal audit | P0 inventory and P6 |
