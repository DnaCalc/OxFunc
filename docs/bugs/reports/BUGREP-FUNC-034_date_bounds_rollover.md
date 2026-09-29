# BUGREP-FUNC-034: Date serial bounds, conversion and rollover

Status: `triaged`; canonical stream [BUG-FUNC-056](../streams/BUG-FUNC-056_date_bounds_rollover.md).

The numeric discovery exposes missing upper and raw-negative bounds, half-second serial conversion, 1900 rollover differences and fractional selector conversion. Repeated unsupported WEEKNUM selectors produce contradictory Excel answers for identical inputs; those observations remain retained and cannot establish a deterministic repair.

Reported against `11b23504fe8180d08397be5435bd215e1bbb9722`; reproduced through fresh live Excel20430/CV2 public interfaces on2026-09-29.
