# Review prompt: W111 parity axis and W112 parity driver (2026-09-22)

You are reviewing two planning artifacts in the OxFunc repo before any code lands.
Read them cold, as someone who will have to live with them for years. This is a
review pass, not a rewrite: report findings, do not edit.

## Read, in this order

1. `docs/function-lane/BIT_EXACT_STOCK_TAKE_20260922.md` — what we believe today
   about which functions match Excel, and the correction note at the top.
2. `docs/decisions/ODR-FN-005-excel-parity-status-axis.md` — the `excel_parity`
   field on `FunctionMeta`, the ledger behind it, the held-out bar, the ordering
   scale.
3. `docs/worksets/W112_PARITY_DRIVER_CONSOLIDATION.md` — the `sf` CLI that turns
   the smart-fuzzer into the one engine for parity work.
4. `docs/WORKSET_REGISTER.md` sections W107 (absorbed), W111, W112.
5. Beads: epics `oxf-mwue` (W111, 12 children) and `oxf-irnl` (W112, 7 children).
   `br show <id>` for each.

Context you need: `docs/SMART_SEARCH_AND_ACTIVE_LEARNING_LOOP.md` is the existing
method for identifying a kernel; `smart-fuzzer/tools/calc_graph_racer/` is the
existing search tool; `docs/OXFUNC_EXCEL_DISCREPANCY_CATALOG.md` is the existing
narrative tracker. None of those are being replaced.

## What we are trying to do

Every worksheet function OxFunc implements should return the same bits as Excel,
except CUBE*, WEBSERVICE and STOCKHISTORY. We cannot check every bit and Excel
changes, so no function is ever "done"; a status is what the evidence supports on
the builds judged so far. Two things were wrong before this planning:

- parity knowledge lived in prose, catalog rows, a stale July status map and a
  stale Handbook ingest, with nothing the compiler could see;
- the classification mixed "how strong is our evidence" with "how far off is the
  known miss", so functions with a known differing row were graded "medium".

The intended fix: one field per function (`Unverified`, `Consistent`,
`Characterized`, `Divergent{severity}`, `Deferred`), one ledger of accumulated
evidence, and one engine that reads the ledger, says what is worth doing next,
runs it, and writes the answer back as a proposal. The split that matters
operationally is work that needs Excel (`sf ask`, scarce) versus work that does
not (everything else, run anywhere).

We deliberately kept it light: no banned-word lists, no append-only audit rows,
no mandated derivation script, no daemon, no plugin framework. Git is the history.
If something needs more machinery, a phase bead says so first.

## What to look for

- **Does the classification hold up?** Is there a real case the five statuses and
  four severities cannot express, or a case where two reviewers would file the
  same evidence under different values? Are the 4 and 1024 ULP thresholds sensible?
- **Is the field the right shape for the code?** `ExcelParity { status, as_of }` on
  `FunctionMeta`, default `Unverified`, exported through the registry. Anything
  that will fight the existing `function_spec!` macro or the golden fixture?
- **Is the held-out bar (ODR §4) honest without being theatre?** Fresh corpus,
  branch and edge coverage, a floor around 1,000 rows, build recorded, a stated
  reason the bits agree.
- **Does the ordering scale (ODR §5) actually pick the widest wins first**, or does
  it hide something cheap behind something ranked earlier?
- **Is W112 workable as written?** Six commands, seven phases, each phase ending
  with a real campaign. Is any phase secretly two, or premature, or missing a
  dependency on W111? Is folding the three oracle scripts into `sf ask` the right
  first cut, or should P2 come before P1?
- **What did absorbing W107 lose?** Its fourteen beads were closed with a mapping to
  W112 phases. Name anything from W107 that has no home now and should.
- **Where is this still too formal**, or where is it too loose to be followed? Both
  are findings.
- **What would you start on Monday**, given W111-1 (the field) is the stated entry
  point?

## Output

A short findings list, most important first, each with the file and section it
refers to and a one-line suggested change. Then one paragraph: would you proceed
as planned, proceed with changes, or stop and rethink, and why.
