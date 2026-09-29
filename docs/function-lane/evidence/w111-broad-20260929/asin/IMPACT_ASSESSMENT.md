# ASIN and ACOS arithmetic impact

Status: `scope_completeness=scope_partial`, `target_completeness=target_partial`,
`integration_completeness=partial`. Open lanes: broader argument/context coverage, and alternate arithmetic platforms.

The change concerns numeric kernels. ASIN reuses the published `1-x` in its
second factor, stages its arithmetic and square root through RN64 before
binary64 publication, and uses the characterized reduced arctangent. ACOS
already depended on `asin_kernel`; its final subtraction now uses the existing
RN64-to-binary64 primitive. Function metadata, argument preparation, arity,
coercion policy, domain, array shape and evaluator scheduling declarations are
unchanged. No new external runtime dependency is introduced.

Public COM observations demonstrate the old oddness assumption is wrong:
the exact numeric input `0x3fe52a3a8df2eadb` gives ASIN result
`0x3fe720476e0c1ac2`; its negative gives `0xbfe720476e0c1ac3`.
`asin-exact-readback.json` verifies source cells and `=ASIN(A1)` formulas.
The signed capture contains 687 such non-odd pairs. The reused rounded factor
explains them as one general expression, without a sign-specific correction.

The ACOS caller capture prevents treating ASIN as isolated. With the new ASIN,
plain subtraction differs on 24 signed neighbors just above `2^-53`; the
staged subtraction resolves all 24. Earlier failures remain separately retained.
The new kernels pass 14,140 ASIN and 6,180 ACOS public-dispatch observations;
the Lean graphs and their primitive bindings build. The source freeze precedes
the fresh 16,104-row combined independent capture, which matches every row
with all four frozen source hashes unchanged. Total retained observations are
22,192 ASIN and 14,232 ACOS, all exact in 5 public-dispatch tests.

These observations establish bounded numeric behavior on Excel 16.0 build
20430, workbook Compatibility Version 2, on the recorded x86-64 host. They
do not establish every coercion/reference/array context or another CPU's
transcendental last bits. Shared numeric text grammar and evaluator ingress
remain separately qualified by the campaign's existing seam records.
