# BUG-FUNC-062: Text slice defaults Unicode positions and arrays

Status: `investigating`; owner W111; bead `oxf-mwue.28.13`.

Typed discovery has 599 differences among 1945 executed cases, with seven formula-entry rejections. Raw UTF16 capture retains 2508 exact-ingress observations and withholds 264 NUL-input rows altered by Value2. CV2 MID uses paired-surrogate character positions while MIDB uses raw UTF16 units. Numeric count conversion, defaults, arrays and context declarations remain under characterization.

Reference: public-interface Excel 16.0 build 20430, 64-bit, Compatibility Version
2; update channel is unverified. Evidence is retained under
`docs/function-lane/evidence/w111-broad-20260929/text-slice/`.
Only black-box observations and public arithmetic operations inform this work.

`scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: the stated residuals and refined
independent validation, context/platform limits, evaluator-visible assessment,
formal alignment and combined verification.
