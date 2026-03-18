# W16 Batch 48 - Date Value Family

Status: `in_progress-provisional`
Workset: `W16`

## Scope
1. `DATEVALUE`
2. `TIMEVALUE`
3. `DAYS360`
4. `DATEDIF` seed lanes in the same family file

## Native Excel Baseline
Pinned spot rows on the current reference Excel baseline:
1. `DATEVALUE("2024-02-03") -> 45325`
2. `DATEVALUE("2024-02-03 6:35 AM") -> 45325`
3. `DATEVALUE("6:35 AM") -> 0`
4. `DATEVALUE("22-Aug-2008") -> 39682`
5. `DATEVALUE("1/2/2024") -> #VALUE!`
6. `TIMEVALUE("2:24 AM") -> 0.1`
7. `TIMEVALUE("22-Aug-2008 6:35 AM") -> 0.2743055555547471`
8. `TIMEVALUE("2024-02-03") -> 0`
9. `TIMEVALUE("22-Aug-2008") -> 0`
10. `TIMEVALUE("1/2/2024 6:35 AM") -> #VALUE!`
11. `DAYS360(DATE(2024,2,29),DATE(2024,3,31),FALSE) -> 30`
12. `DAYS360(DATE(2024,2,29),DATE(2024,3,31),TRUE) -> 31`
13. `DAYS360(DATE(2011,2,28),DATE(2011,3,31),FALSE) -> 30`
14. `DAYS360(DATE(2011,2,28),DATE(2011,3,31),TRUE) -> 32`
15. `DATEDIF(DATE(2001,6,1),DATE(2002,8,15),"YD") -> 75`
16. `DATEDIF(DATE(2001,1,31),DATE(2001,2,28),"MD") -> 28`
17. `DATEDIF(DATE(2001,1,31),DATE(2001,3,1),"MD") -> -2`
18. `DATEDIF(...,"Q") -> #NUM!`

## Current Implementation Notes
1. The Rust family file is self-contained on purpose because this request forbids shared dispatch/catalog edits.
2. The family preserves the current `1900` serial baseline, including serial `0 -> 1900-01-00` and the fake serial `60 -> 1900-02-29`.
3. `DATEVALUE` and `TIMEVALUE` currently admit the host-like text subset pinned above: ISO dates, `d-MMM-yyyy`, optional trailing `h:mm[:ss] [AM|PM]`, and pure time text.
4. Slash-date text remains rejected in the current-host slice, matching the same host profile already pinned for `VALUE`.
5. `DAYS360` includes both NASD/US and European methods, including the February-end divergence.
6. `DATEDIF` is included in the same file with the seeded units `Y`, `M`, `D`, `YM`, `YD`, and the empirically pinned quirky `MD` lane; broader closure still needs full packet wiring and wider replay.
