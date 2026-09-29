# BUGREP-FUNC-035: Implicit numeric text grammar and locale/date context

Status: `triaged`; canonical stream [BUG-FUNC-057](../streams/BUG-FUNC-057_implicit_numeric_text.md).

The exact-ingress typed sweep has1208 differing rows among2544 across53functions. Percent, parentheses, locale-specific grouping/currency, Unicode digits, date/time and ASCII-only whitespace distinguish Excel from Rust trim/parse. ADDRESS720controls show missing-year text depends on current host date. Locale/clock dependencies require cross-repo assessment; no hardcoded date or unsupported provider substitution is authorized by this evidence.

Reported against `11b23504fe8180d08397be5435bd215e1bbb9722`; reproduced through fresh live Excel20430/CV2 public interfaces on2026-09-29.
