# W16 Batch 75 - NUMBERVALUE / REGEX* / TRANSLATE

Scope: historical bounded mixed-family note for `NUMBERVALUE`, `REGEXEXTRACT`, `REGEXREPLACE`, `REGEXTEST`, and `TRANSLATE` before the live `W24` split.

Admitted slice:
- `NUMBERVALUE`
  - scalar text-to-number parsing with optional single-character decimal and group separators
  - whitespace ignored, trailing percent signs accumulated, default separators `.` and `,`
- `REGEXEXTRACT`, `REGEXREPLACE`, `REGEXTEST`
  - deterministic local regex subset only: literals, `.`, character classes, ASCII-aware ranges, `\\d`, `\\w`, `\\s`, and postfix quantifiers `*`, `+`, `?`
  - bounded to first-match extraction for `REGEXEXTRACT` (`return_mode = 0` only)
  - bounded to literal replacement text for `REGEXREPLACE`; occurrence `0` means replace all, positive `n` means replace the `n`th match
  - `case_sensitivity` follows the current support-doc surface (`0`/omitted = case-sensitive, `1` = case-insensitive), with ASCII-only case folding in this bounded slice
- `TRANSLATE`
  - metadata preserves the real external-provider seam
  - executable evaluator is intentionally bounded to a tiny deterministic phrasebook for a few `en`/`es`/`fr`/`de` lanes plus same-language no-op
  - omitted target language is not admitted in this slice

Local unit coverage pins:
- `NUMBERVALUE` default/custom separator and percent lanes
- regex literal/class/quantifier parsing, extract/test/replace behavior, unsupported-pattern rejection, and surface wrappers
- `TRANSLATE` same-language no-op, phrasebook translation, source inference for one lane, and omitted-target rejection

Current standing after live `W24` replay:
- `REGEXEXTRACT`, `REGEXREPLACE`, and `REGEXTEST` are now closed by `W24 Batch 10` under `docs/function-lane/FUNCTION_SLICE_REGEX_TRIAD_CONTRACT_PRELIM.md`.
- `NUMBERVALUE` omitted-default semantics are host-locale/profile-sensitive on the current baseline and are blocked under `BLK-FN-006`.
- `TRANSLATE` is empirically external-provider-bound (`#BUSY!` for a real translation request) and is also blocked under `BLK-FN-006`.
- The old combined note should no longer be read as a closure claim for the whole mixed family.
