# Date Serial System And Workbook Mode Notes

Status: `active`
Owner lane: `OxFunc`

## 1. Purpose
Capture the shared worksheet date-serial substrate that sits beneath `DATE`, `TODAY`, `NOW`, and later date/time functions.

This note exists so Excel's date-system quirks are handled once as a semantic environment seam, not rediscovered piecemeal in per-function contracts.

## 2. Core Rule
Excel date semantics depend on both:
1. the worksheet date serial system in force for the workbook or evaluation context, and
2. the historical quirks of the selected serial system.

These are not locale concerns and should not be treated as later cosmetic validation passes.

## 3. Current Baseline
The current OxFunc reference baseline is the ordinary Excel `1900` date system observed in:
1. Excel `16.0 (build 19725)`, channel `http://officecdn.microsoft.com/pr/492350f6-3a01-4f97-b9c0-c7c6ddf67d60`, locale `en-US`
2. workbook lanes `default` and `compat_template`

Current baseline facts already pinned empirically:
1. `DATE(1900,1,0) = 0`
2. `DATE(1900,0,1) = #NUM!`
3. `DATE(1900,2,29) = 60`
4. years in `[0,1899]` are normalized by adding `1900` in the current slice
5. `TODAY` and `NOW` produce numeric serials in the same serial system

## 4. The Two Important Quirks
### 4.1 1900-System Historical Root
The `1900` system is not just "days since a clean epoch".

Observed and required current-baseline behavior includes:
1. serial `0` is admitted as the day before `1900-01-01`
2. serial `1` corresponds to `1900-01-01`
3. serial `60` is reserved for the historical Excel `1900-02-29` bug
4. dates on or after `1900-03-01` are shifted by that preserved bug relative to a mathematically clean Gregorian count

This is a shared serial-model rule, not a `DATE()`-specific oddity.

### 4.2 1904 Workbook Mode
Excel also supports the `1904` date system as a workbook-level environment choice.

This is not a per-function option. It is a workbook/evaluation-context mode that affects how date/time serials are interpreted and produced.

Implication:
1. `DATE`, `TODAY`, and `NOW` cannot be regarded as fully environment-agnostic forever
2. the serial system must eventually be an explicit evaluation-context input
3. per-function closure over the current `1900` baseline remains valid while the workbook-mode seam is still being raised explicitly

## 5. Modeling Rule
Date-system selection belongs to the evaluation seam, not the function kernel.

Recommended separation:
1. function kernels work over an explicit serial-system model
2. the evaluation context chooses `1900` versus `1904`
3. function contracts say which serial-system assumptions are currently admitted

This keeps:
1. `DATE` focused on calendar-to-serial conversion
2. `TODAY` and `NOW` focused on provider serial acquisition plus any format hinting
3. workbook mode as a boundary concern rather than duplicated per function

## 6. Current OxFunc Position
For the current phase:
1. `DATE` is `function-phase-complete` for the current `1900`-system reference baseline
2. `TODAY` and `NOW` are likewise closed for the current baseline as provider-backed serial producers
3. the broader date-system seam is now recognized as an explicit shared substrate that must be handled before broader date/time-family expansion

This means:
1. existing closure claims for `DATE`, `TODAY`, and `NOW` remain valid for the declared current baseline
2. future date/time expansion should not proceed as if the serial system were an invisible constant

## 7. OxFml / FEC Requirement
The workbook date system should be represented as evaluation-context data, not hard-coded as an ambient constant.

Minimum requirement:
1. OxFml/FEC should expose the active workbook date system at the function-evaluation boundary
2. OxFunc should be able to consume that setting without reinterpreting function contracts ad hoc

Candidate vocabulary:
1. `DateSystem1900`
2. `DateSystem1904`

## 8. Execution Guidance
Recommended next substrate pass:
1. define the shared date-serial-system contract and terminology in OxFunc docs
2. keep current `DATE`/`TODAY`/`NOW` closures scoped to the pinned `1900` baseline
3. introduce explicit workbook date-system handling before broader date/time function expansion
4. add `1904` empirical replay once that seam is wired into the evaluation context and probe path

## 9. Why This Matters
If the date-system seam is left implicit:
1. function contracts will overstate portability of current closure claims
2. later date/time functions will duplicate serial rules inconsistently
3. workbook-environment semantics will be mistaken for per-function quirks

Treating the serial system as a shared substrate keeps the model honest and reduces future rework.
