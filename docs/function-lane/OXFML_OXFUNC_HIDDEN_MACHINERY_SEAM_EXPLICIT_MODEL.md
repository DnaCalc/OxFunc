# OxFml / OxFunc Hidden Machinery Seam - Explicit Model

Status: `provisional`
Owner lane: `OxFunc`
Charter anchor:
1. `CHARTER.md` Section 1: OxFunc exists to define explicit, auditable semantics rather than rely on folklore.
2. `CHARTER.md` Section 4: OxFunc owns value/function/operator semantics; OxFml owns formula grammar/parse/bind.
3. `CHARTER.md` Section 5: seam ambiguity must be logged explicitly, never silently absorbed.

## 1. Purpose
Use the OxFml/OxFunc boundary as an opportunity to define Excel's hidden machinery explicitly.

This document is not a frozen API design.
It is a central seam-model note for:
1. what semantic distinctions must survive,
2. which lane should own which distinctions,
3. which tensions are real and should be documented rather than hidden,
4. which open questions are worth iterative cross-repo tightening.

The working objective is not a super-clean split at any cost.
The objective is an explicit, formal, and empirically grounded model of how Excel appears to work.

## 2. Design Posture
Working rules:
1. preserve semantic distinctions first,
2. keep transport shape provisional until the semantics are clear,
3. prefer explicit capability- and profile-bearing carriers over implicit evaluator folklore,
4. keep OxFunc runtime-state-free even when the broader library context is dynamic,
5. do not force symmetry where Excel itself is asymmetric or historically layered.

## 3. Core Ownership Split
Current best split:
1. OxFunc owns:
   - the worksheet value universe,
   - function semantics,
   - operator semantics,
   - function/operator profiles and capability declarations,
   - empirical/evidence meaning,
   - boundary invariants discovered through function work.
2. OxFml owns:
   - lexical grammar,
   - parse structure,
   - binding,
   - helper-form structure (`LET`, `LAMBDA`, names),
   - localized token surface such as separators and literal spelling rules,
   - semantic-plan formation and prepared-call orchestration.
3. FEC/F3E / host-facing layers own:
   - caller/workbook/application/environment capabilities,
   - publication/rendering,
   - host policy and scheduler-facing lifecycle.

This split is intentionally not perfectly pure.
Excel itself mixes syntax, workbook context, function metadata, and publication rules in ways that have to be represented honestly.

## 4. Canonical Seam Themes
The seam is best thought of as five coupled surfaces:
1. catalog/library-context seam,
2. prepared-argument/result seam,
3. callable/helper-value seam,
4. typed host-capability seam,
5. publication/replay seam.

### 4.1 Catalog / Library Context Seam
OxFunc should own the canonical function/operator catalog.

That catalog should include:
1. canonical function/operator ids,
2. aliases and compatibility names,
3. localized names,
4. arity/admission and function-profile metadata,
5. volatility/host/capability declarations,
6. feature/version/profile gates,
7. operator registry facts where operator meaning is catalog-owned.

OxFml should not hardcode a stale closed world of names.
Instead it should bind against a `LibraryContextSnapshot` or equivalent external context.

Working seam model:
1. library context is externally allocated and versioned,
2. OxFml uses it for parse/bind and early rejection,
3. OxFunc uses the same context for evaluation,
4. context mutation or registration updates happen outside OxFunc,
5. OxFunc remains runtime-state-free because it consumes passed-in context rather than owning global mutable registry state.

This is especially important for:
1. user-defined functions,
2. add-in functions,
3. VBA-registered functions,
4. feature-gated functions,
5. localized function names.

### 4.2 Prepared Argument / Result Seam
The minimum shared vocabulary should preserve distinctions that real Excel semantics depend on.

Current proven minimums include:
1. direct scalar argument vs array-like argument,
2. omitted argument vs blank cell vs empty string vs error,
3. values-only function path vs reference-visible function path,
4. value result vs may-return-reference result,
5. caller-context-sensitive evaluation,
6. capability-sensitive or host-sensitive outcomes.

This seam should avoid premature collapse of:
1. references into unconditional eager values,
2. scalar and array-like inputs into a generic payload bucket,
3. blank-like states into one empty category.

### 4.3 Callable / Helper Value Seam
`LET` and `LAMBDA` are now top-tier seam topics.

Current intended split:
1. OxFml owns:
   - helper syntax,
   - sequential helper binding,
   - shadowing,
   - lambda formation,
   - lexical capture,
   - invocation planning.
2. OxFunc should not require raw helper AST ownership.
3. OxFunc should be able to consume either:
   - an opaque callable value,
   - or a callable summary plus a typed invocation facility,
   without losing lexical meaning.

The key invariant is:
1. a created helper lambda keeps lexical meaning and does not dynamically re-read helper names after creation.

### 4.4 Typed Host Capability Seam
Host-sensitive functions should not be modeled as arbitrary callbacks or raw workbook objects crossing the seam.

Instead, the seam should expose typed capability views for:
1. caller cell and active selection,
2. referenced cell metadata,
3. workbook facts,
4. application/environment facts,
5. row-visibility and host-view state where required,
6. provider-bound services where required.

This follows the direction already pressured by:
1. `CELL`,
2. `INFO`,
3. later `ISFORMULA`,
4. `SUBTOTAL` / `AGGREGATE`,
5. provider-bound functions such as `TRANSLATE`.

### 4.5 Publication / Replay Seam
The seam must preserve the difference between:
1. pure function semantics,
2. worksheet-surface publication,
3. proving-host evidence and replay artifacts.

This includes:
1. format hints,
2. spill publication,
3. CSE/legacy publication context where needed,
4. direct-cell-binding proving-host artifacts,
5. seam limitation vs semantic failure classification,
6. packet-first replay witnesses rather than invented event streams.

## 5. Value Universe Position
OxFunc owns the worksheet value universe.

Current direction:
1. scalar, error, array, reference-like, and extended values are OxFunc-owned semantic categories,
2. OxFml should parse and bind literals/operators against that semantic universe without redefining it independently.

### 5.1 Callable Lambda Values As First-Class Values
The value universe should now explicitly leave room for a lambda/callable value class.

Working position:
1. lambda values are first-class semantic values,
2. they are still not ordinary cell-display values on the current Excel baseline,
3. they may nevertheless exist as:
   - helper-produced values,
   - function arguments,
   - function return values,
   - defined-name values,
   - UDF interop payloads.

This is consistent with the stated prototype direction:
1. native and user-defined lambdas can be passed as function arguments,
2. native and user-defined lambdas can be returned,
3. lambda-curry functions become representable,
4. interop may use an `IExcelLambda` COM carrier or equivalent host-facing bridge.

So the seam should assume:
1. callable values are first-class in the semantic universe,
2. callable values are not automatically admissible as worksheet cell display payloads,
3. publication rules for callable values remain separate from their semantic existence.

This distinction is important because "not a valid cell value" must not be mis-modeled as "not a value at all".

## 6. Operators, Literals, and Grammar Tension
There is a real ownership tension here, and it should be documented explicitly.

Current best working split:
1. OxFunc owns:
   - canonical operator ids,
   - operator semantics,
   - operator profiles,
   - value-type implications of operator results.
2. OxFml owns:
   - operator tokens in the grammar,
   - precedence/associativity in the parser,
   - localized separators,
   - numeric/text literal tokenization,
   - locale-sensitive lexical forms.

This means:
1. OxFml can own how something is spelled,
2. OxFunc owns what it means.

A clean split is not always possible.
For example:
1. localized list separators are syntactic,
2. decimal/group/currency literal forms are lexical and locale-bound,
3. operator meaning and result class are semantic,
4. some operator availability may still depend on catalog/profile decisions.

The tension should be acknowledged rather than hidden.

## 7. High-Priority Coupled Topics
The highest-value coupled topics are now:
1. `@` / `SINGLE`,
2. `LET`,
3. `LAMBDA`.

### 7.1 `@` / `SINGLE`
Current intended split:
1. OxFml owns:
   - syntax,
   - attachment point,
   - compatibility alias recognition,
   - caller-cell association,
   - preservation of explicit-`@` provenance.
2. OxFunc owns:
   - scalarization semantics,
   - function-profile-aware admission/result behavior,
   - the semantic rule distinguishing scalar, reference, array, and error cases.

Working seam minimum:
1. operand/result class must survive as at least:
   - `scalar`
   - `reference`
   - `array`
   - `error`
2. caller context must be visible,
3. final placement in the pipeline remains open.

### 7.2 `LET`
OxFml should own:
1. sequential helper binding,
2. name shadowing,
3. helper-environment construction.

OxFunc should see:
1. prepared values or callable values with preserved meaning,
2. not a lossy flattening that erases helper-environment semantics.

### 7.3 `LAMBDA`
OxFml should own:
1. parameter binding,
2. closure formation,
3. lexical capture,
4. invocation planning.

OxFunc should be able to:
1. recognize callable values as part of the value universe,
2. accept them where function semantics allow,
3. return them where function semantics allow,
4. interact with a typed callable carrier without owning helper syntax.

## 8. Availability, Feature Gates, and Host/Profile Sensitivity
Processed functions already show that "is this function available here?" is a real seam issue.

The seam should distinguish at least:
1. known in catalog,
2. feature-gated,
3. compatibility-gated,
4. host-profile unavailable,
5. add-in absent,
6. provider unavailable.

This matters for:
1. parse/bind,
2. early rejection,
3. runtime outcome classification,
4. replay explanation,
5. avoiding false `#NAME?` equivalence claims.

## 9. Locale / Profile / Version Context
Locale and profile are not only rendering concerns.

The seam should have room for:
1. workbook compatibility version,
2. application/version/channel context,
3. host/profile identity,
4. locale/format profile,
5. feature/provider availability.

Why:
1. `NUMBERVALUE` omitted defaults can be locale/profile-sensitive,
2. width-conversion functions can be host/profile-sensitive,
3. compatibility affects whole-axis and older-surface behavior,
4. function availability can depend on host/add-in/profile state.

## 10. Seam Themes Revealed By Function Work So Far
The processed functions suggest these seam themes are not optional:
1. reference identity vs eager dereference,
2. direct-cell-binding proving-host truth,
3. whole-axis geometry and compatibility-sensitive sheet assumptions,
4. locale/profile-sensitive semantics, not just formatting,
5. typed host-query capability families,
6. provider-bound functions,
7. feature availability and `#NAME?` classification,
8. volatility, random/time providers, and recalculation profile,
9. semantic failure vs capability denial vs seam limitation,
10. format hints and publication behavior above the pure kernel,
11. family-specific admission/coercion policy as catalog metadata.

## 11. Current Open Questions Worth Iteration
These are the questions most worth tightening over the next cross-repo rounds:
1. what is the minimum shared provenance vocabulary,
2. what is the smallest honest library-context snapshot shape,
3. what callable facts must cross the seam beyond an opaque callable token,
4. where should explicit `@` ultimately live,
5. which operator facts belong in the catalog vs only in grammar,
6. what typed host-capability families should be first-class next,
7. how should callable values be represented in defined names and UDF interop,
8. what should be publication-only restrictions vs true value-universe restrictions.

## 12. Working Rule For The Next Round
For the next OxFml/OxFunc seam round:
1. use this document as the central semantic theme list,
2. let `docs/upstream/NOTES_FOR_OXFML.md` remain the live handoff ledger,
3. refine transport sketches only when doing so helps preserve a needed semantic distinction,
4. prefer explicit modeling of Excel's hidden machinery over premature API neatness.
