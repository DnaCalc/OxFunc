# ODR-FN-002: Invocation Test Category Split — Context-Sensitive vs Locally-Evaluable

- **Status**: accepted
- **Date**: 2026-06-18
- **Context**: <see below>
- **Decision**: <see below>
- **Consequences**: <see below>
- **Cross-repo impact**: OxFml and OxCalc own the downstream evaluation path for the
  context-sensitive category; OxFunc publishes the catalog but does not evaluate it.

## Context

OxFunc test evidence has two structurally different sources of truth, and conflating
them costs us twice:

1. Some function invocations are **context-sensitive**: their result depends on
   references, implicit intersection, spill neighborhood, reference transforms,
   caller location, host/provider state, or formula-binding scope. Evaluating these
   honestly from OxFunc requires a faithful OxFml/OxCalc context. The only ways to
   produce that locally are (a) stand up scaffolding that *fakes* OxFml/OxCalc
   context, or (b) compare against a partial mock. Both effectively re-implement
   downstream behavior inside OxFunc test rigs — the scaffolding starts to mirror the
   real implementation work that belongs in OxFml/OxCalc, and a green local result
   then proves the mock, not the product.

2. Some function invocations are **context-free**: they can be expressed as simple
   formulas whose inputs are literals, typed cell fixtures, or array literals, and
   compared against Excel through a single `Formula2` evaluation. These need no
   OxFml/OxCalc binding to be meaningful, and OxFunc can — and should — drive them
   directly and at volume.

The smart-fuzzer already encodes an embryonic form of this split as its
"Current Scope Boundary" / "OxFunc-accessible region" versus the deferred seam lanes
(`smart-fuzzer/README.md`, `smart-fuzzer/planning/BLOCKED_DEFERRED_SEAM_CLASSIFICATION_MAP.md`).
This decision promotes that boundary from a fuzzer-local scoping note to a named,
forward repo testing strategy and fixes the publishing/evaluation policy for each side.

## Decision

Every in-scope invocation under test is classified into exactly one category.

### Category 1 — Context-Sensitive Invocations (publish, evaluate downstream)

Invocations whose result depends on any of: cell/range references, implicit
intersection (explicit `@`), spill neighborhood, reference transforms
(`OFFSET`/`INDEX` reference form, `ADDRESS`/`AREAS`), cross-sheet or structured
references, caller location, host/workbook/provider state (`CELL`, `INFO`,
`FORMULATEXT`, `SHEET(S)`, `INDIRECT`, `NOW`/`TODAY`, RTD, cube/web providers), or
formula-binding scope (`LET`, `LAMBDA`, `BYROW`, `BYCOL`, `MAP`, `REDUCE`, `SCAN`,
`MAKEARRAY`, `ISOMITTED`).

Policy:

1. OxFunc **publishes** a versioned catalog of these invocations with the seam each
   exercises and the expected behavior described in prose — see the catalog under
   `smart-fuzzer/corpus/` named by W104. The catalog is a smart-fuzzer corpus, not an
   out-of-scope register (see "Both categories are smart-fuzzer scope" below).
2. OxFunc **does not** build scaffolding that fakes OxFml/OxCalc context to make these
   runnable inside a local OxFunc rig. The current local-Rust + Excel-COM runner does
   not evaluate them.
3. They are evaluated through the comprehensive **OxCalc → OxFml → OxFunc** path, where
   the real binder, resolver, and provider context exist. A smart-fuzzer runner that
   drives that downstream stack as its evaluation engine is the intended executor for
   this category and **will be added later**; until then the catalog is published and
   grown but not executed.
4. A row only graduates to the Category-2 local lane if it can be reduced to a genuinely
   context-free form without faking context.

### Category 2 — Context-Free Invocations (evaluate locally via the smart-fuzzer)

Invocations expressible as simple formulas over literals, typed cell fixtures, or
array literals, comparable against Excel through one `Formula2` evaluation.

Policy:

1. OxFunc evaluates these **locally** via the smart-fuzzer's existing runnable Excel
   comparison harness (`smart-fuzzer/tools/Run-ArraySupportTranche.ps1` +
   `smart-fuzzer/tools/CellRefBatch.psm1` + the `pmt_ppmt_local_eval` Rust crate).
2. Two interests are first-class and both are bugs when they diverge:
   - **2a. Evaluation-class behavior** — kind/shape outcomes: logical→number
     coercion, text→number coercion, blank handling, error code and error shape,
     array lift, returned-value kind. These are *structural* under CHARTER §4.1.
   - **2b. Bit-exactness** — IEEE-754 bit equality of numeric results, measured
     through the binding `Range.Value2` cell-ref plumbing rule
     (`smart-fuzzer/planning/EXCEL_RUNNER_PLUMBING_NOTE.md`). `> 1` ULP is worse than
     `1` ULP, but both are numeric-drift bugs.
3. Excel remains the sole oracle. External fuzzing tooling (AFL/AFL++) informs
   *methodology* only — feedback-guided queue, typed mutators, corpus culling,
   comparison-guided exploration — per `smart-fuzzer/planning/SMART_FUZZER_DESIGN.md` §2.1.

### Both categories are smart-fuzzer scope

The split is about **which runner**, not about what is in scope. Both categories are
inside the smart-fuzzer's testing scope:

1. **Category 2** runs today on the existing runner: local Rust (`oxfunc_core` value
   surface) plus the Excel-COM oracle, comparing typed outcomes for evaluation-class
   behavior and bit-exactness.
2. **Category 1** runs on a **different runner that does not exist yet**: one that drives
   the downstream **OxCalc → OxFml → OxFunc** stack as the evaluation engine (with its
   own oracle/comparison), so references, implicit intersection, spill, host/provider,
   locale, and formula-binding context are real instead of faked. That runner is future
   work; the published catalog is its seed corpus.

As the smart-fuzzer expands to AFL-style feedback-guided exploration and beyond
(typed mutators, semantic feedback queue, corpus culling, comparison-guided generation),
those techniques apply to **both** categories — the Category-1 catalog is the corpus the
future downstream-driven runner mutates and explores, exactly as the Category-2 case sets
are for the local runner today.

## Consequences

1. We stop building local test scaffolding that mirrors downstream OxFml/OxCalc
   implementation work. Context-sensitive coverage moves to a published catalog and
   downstream evaluation.
2. The smart-fuzzer's Category-2 work gains an explicit second axis — evaluation-class
   behavior — alongside the bit-exactness sweeps it already runs.
3. Blocked/deferred seam lanes already tracked by the fuzzer are re-read as the
   Category-1 catalog source, not as OxFunc-accessible coverage gaps.
4. W104 owns the rollout: the published catalog, the Category-2 evaluation-class probe
   lane, and the registers/beads that carry them.
5. A Category-1 smart-fuzzer runner over the downstream OxCalc→OxFml→OxFunc stack is a
   planned future lane; the catalog is its seed corpus, so context-sensitive coverage is
   deferred by runner, not dropped from the smart-fuzzer's scope.

## Cross-repo impact

OxFml and OxCalc own the OxCalc→OxFml→OxFunc evaluation of the Category-1 catalog. The
W100 explicit-`@` follow-up (`HO-FN-018`) is the first concrete catalog consumer: those
rows are Category-1 by construction and must not be forced through a faked-context local
rig. No OxFunc API change follows from this decision.
