# W16 Batch 79 - Bond Core

Scope: bounded scalar current-baseline packet for `ACCRINT`, `ACCRINTM`, `DURATION`, `MDURATION`, `PRICE`, `PRICEMAT`, `YIELD`, `YIELDDISC`, and `YIELDMAT` in a self-contained family file.

Admitted slice:
- 1900 date system only.
- Frequency admitted only for `1`, `2`, `4`.
- Basis admitted only for `0..4` with the same bounded day-count substrate already used elsewhere in W16.
- `PRICE`, `YIELD`, `DURATION`, and `MDURATION` use a regular-coupon schedule inferred by stepping backward from maturity.
- `PRICE` / `YIELD` include a dedicated `N = 1` lane using the standard short final-period linear denominator rather than the multi-period power form.
- `PRICEMAT`, `YIELDMAT`, and `YIELDDISC` are implemented as direct algebraic transforms over the bounded year-fraction slice.
- `ACCRINTM` is `par * rate * year-fraction(issue, settlement)`.
- `ACCRINT` is bounded to a regular schedule anchored on `first_interest`; `calc_method = TRUE` carries the initial stub forward, `FALSE` drops the pre-first-coupon carry once settlement is beyond `first_interest`.

Local unit coverage pins:
- `PRICE` / `YIELD` round-trip on a regular coupon example.
- `DURATION` / `MDURATION` relation.
- `PRICEMAT` / `YIELDMAT` round-trip.
- `YIELDDISC` direct-form consistency.
- `ACCRINT` calc-method distinction and `ACCRINTM` year-fraction consistency.
- surface wrapper and domain-error sanity rows.

Open beyond this bounded slice:
- irregular long/short coupon schedules beyond the simple `ACCRINT` first-interest anchor.
- broader Excel parity for edge conventions around serial `60`, ex-coupon behavior, and any one-coupon / odd-period lanes not covered by the current regular-schedule model.
- shared dispatch/export/import integration is intentionally untouched in this task.
