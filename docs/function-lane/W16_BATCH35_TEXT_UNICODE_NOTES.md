# W16 Batch 35 - Text Unicode Functions

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH35-TEXT-UNICODE-20260316`

## Scope
1. `UNICHAR`
2. `UNICODE`

## Baseline Inputs
Primary evidence reused from the W7 string baseline:
1. `docs/function-lane/STRING_BEHAVIOR_RESEARCH_NOTES.md`
2. `docs/function-lane/STRING_NORMALIZATION_AND_COMPARISON_POLICY_MAP.md`
3. `docs/function-lane/STRING_SCENARIO_MANIFEST_SEED.csv`
4. `.tmp/w16-batch35-36-text-probe.csv`
5. `.tmp/w16-batch36-text-slice-emoji-probe.csv`

Pinned lanes from that baseline:
1. `LEN(UNICHAR(128512)) -> 2`
2. `UNICODE(UNICHAR(128512)) -> 128512`
3. `UNICODE(token(precomposed_e_acute)) -> 233`
4. `UNICODE(LEFT(token(combining_e_acute),1)) -> 101`
5. `UNICODE(MID(token(combining_e_acute),2,1)) -> 769`
6. `UNICODE(RIGHT(over_cap_emoji_text,1)) -> #VALUE!` when interop truncation leaves a dangling high surrogate tail

## Current Implementation Notes
1. `UNICHAR` and `UNICODE` use the ordinary values-only preparation seam.
2. `UNICHAR` truncates numeric input toward zero before Unicode-domain validation, matching the existing scalar text family posture used by `CHAR` and `REPT`.
3. `UNICHAR` returns `#VALUE!` for non-finite, zero, or out-of-range numeric inputs and `#N/A` for reserved-surrogate code points in `0xD800..0xDFFF`.
4. `UNICODE` reads the first Unicode scalar from the UTF-16 payload and returns `#VALUE!` for empty text or any invalid leading surrogate structure, including dangling-high-surrogate and standalone-low-surrogate starts.
5. `UNICODE` therefore remains code-point aware for valid leading surrogate pairs rather than simply returning the first UTF-16 unit, which is required by the `UNICODE(UNICHAR(128512)) -> 128512` baseline.
