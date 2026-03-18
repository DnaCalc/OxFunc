# W16 Batch 64 - Workday / Networkdays

Scope: `WORKDAY`, `WORKDAY.INTL`, `NETWORKDAYS`, `NETWORKDAYS.INTL` as a self-contained date-family packet.

The Rust family carries its own Excel-1900 serial checks, weekend-number and weekend-mask parsing, holiday-range collection, and inclusive business-day counting. `WORKDAY.INTL` rejects the all-weekend mask `1111111`, while `NETWORKDAYS.INTL` accepts it and returns `0`, matching the documented split between the two Microsoft help pages.

Current local unit coverage pins:
- default Saturday/Sunday behavior
- weekend-number and seven-bit weekend-mask handling
- `WORKDAY` / `WORKDAY.INTL` zero-day behavior
- holiday ranges with reference-text ignored and direct invalid text rejected
- domain and error-mapping lanes
