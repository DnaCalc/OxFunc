# W16 Batch 79 - Bond Core

Superseded in part by `W27`.
This historical `W16` note records the original bounded family packet, but the current admitted current-baseline bond-core position is now owned by:
- `docs/worksets/W027_DEFERRED_ADVANCED_BOND_AND_ODD_BOND_HARDENING.md`
- `docs/function-lane/FUNCTION_SLICE_BOND_CORE_FAMILY_CONTRACT_PRELIM.md`
- `docs/function-lane/W27_EXECUTION_RECORD.md`

Scope: bounded scalar current-baseline packet for `ACCRINT`, `ACCRINTM`, `DURATION`, `MDURATION`, `PRICE`, `PRICEMAT`, `YIELD`, `YIELDDISC`, and `YIELDMAT` in a self-contained family file.

Admitted slice:
- 1900 date system only.
- Frequency admitted only for `1`, `2`, `4`.
- Basis admitted only for `0..4` with the same bounded day-count substrate already used elsewhere in W16.
- `PRICE`, `YIELD`, `DURATION`, and `MDURATION` use a regular-coupon schedule inferred by stepping backward from maturity.
- `PRICE` / `YIELD` include a dedicated `N = 1` lane using the standard short final-period linear denominator rather than the multi-period power form.
- `YIELDDISC` remains a direct scalar algebraic transform on the admitted bounded slice.
- `PRICEMAT` and `YIELDMAT` are no longer accurately described as pure year-fraction transforms; `W27` replaces the older bounded model with direct Excel-valued parity on the admitted maturity-security slice, including the Excel-style `DaysInYear(issue,settlement)` denominator.
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
- direct current-baseline bond-core closure is no longer tracked here; use `W27`.
