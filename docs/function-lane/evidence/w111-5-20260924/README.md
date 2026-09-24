# W111-5 live judging, 2026-09-24

First live judging of 24 functions the parity ledger had as Unverified (bead `oxf-mwue.5`).

- Oracle: Excel 16.0 build 20430, 64-bit, CV2, via `smart-fuzzer/tools/Run-W109BulkBatch.ps1`
  (Range.Value2 argument plumbing), cache root `smart-fuzzer/cache/oracle-20430` (local, gitignored).
- Corpus: fresh, deterministic, `smart-fuzzer/tools/w111/gen_w111_5_batches.py` seed 20260924.
  Regenerate the probe batches with `python smart-fuzzer/tools/w111/gen_w111_5_batches.py <dir>`.
- Excel answers: `smart-fuzzer/runs/w111-5-20260924/answers/` (local only, gitignored) (WitnessSet shape: args and Excel result bits).
- Judgement: `sf judge-witnesses answers/*.json` (`smart-fuzzer/engine`) against production
  OxFunc at the commit that adds this directory. `summary.txt` is the one-line-per-function view,
  `judge.json` has every miss.

Outcome: ACOSH, NETWORKDAYS, NETWORKDAYS.INTL, PRICEMAT, WORKDAY.INTL agree on every row; the
other 19 diverge and are catalog rows G8-01..G8-12. COUPDAYSNC's basis-0 settlement-on-the-31st
rule (COUPDAYS - COUPDAYBS) landed in the same commit and holds 65/65 on this corpus; its 9
remaining misses are the February-schedule and serial-0 rows in G8-08.

Promoted here (tracked): `summary.txt`, `misses.json` (every disagreeing row: args bits, Excel
outcome, OxFunc outcome, severity, ULP), `capture_provenance.json` (Excel build, bitness, CV,
CPU, plumbing). Comparison policy: exact typed-bit match, no tolerance. Locale: none passed to
OxFunc (numeric functions; BAHTTEXT text is locale-independent Thai).
