# BUGREP-FUNC-036: DB DDB VDB period admission and arithmetic

Status: `triaged`; canonical stream [BUG-FUNC-058](../streams/BUG-FUNC-058_depreciation_period_arithmetic.md).

The broad sweep exposes fractional period and operation-graph differences. DB truncates period/month but preserves fractional life and iterates book value. DDB uses a characterized power path and stored quotient; VDB interval and switch rules remain under investigation. Candidate and independent oracle corpora are retained separately.

Reported against `11b23504fe8180d08397be5435bd215e1bbb9722`; reproduced through fresh live Excel20430/CV2 public interfaces on2026-09-29.
