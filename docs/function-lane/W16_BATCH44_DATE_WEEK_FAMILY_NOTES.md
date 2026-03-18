# W16 Batch 44 - Date Week Family

Status: `in_progress-provisional`
Workset: `W16`
Evidence ID: `W16-BATCH44-DATE-WEEK-20260316`

## Scope
1. `EDATE`
2. `EOMONTH`
3. `WEEKDAY`
4. `WEEKNUM`
5. `ISOWEEKNUM`

## Native Excel Baseline
Probe artifacts:
1. `.tmp/w16-batch44-date-week-probe.csv`
2. `.tmp/w16-batch44-date-week-edge-probe.csv`

Pinned lanes:
1. `EDATE(DATE(2024,1,31),1) -> 45351`
2. `EDATE(60,0) -> 59`
3. `EOMONTH(DATE(2024,1,31),1) -> 45351`
4. `EOMONTH(60,0) -> 59`
5. `WEEKDAY(DATE(2024,1,1)) -> 2`
6. `WEEKDAY(DATE(2024,1,1),3) -> 0`
7. `WEEKDAY(60) -> 4`
8. `WEEKNUM(DATE(2024,1,1),21) -> 1`
9. `WEEKNUM(60) -> 9`
10. `ISOWEEKNUM(DATE(2020,12,31)) -> 53`
11. `ISOWEEKNUM(0) -> 52`
12. `EDATE(-1,1) -> #NUM!`
13. `WEEKDAY(...,0) -> #NUM!`
14. `WEEKNUM(...,3) -> #NUM!`

## Current Implementation Notes
1. The batch uses the existing `1900` serial baseline and preserves Excel's fake serial-`60` lane.
2. `EDATE` and `EOMONTH` clamp target days against the real Gregorian month length, so `serial 60` normalizes back to `1900-02-28`.
3. `WEEKDAY` follows Excel's serial-timeline weekday numbering rather than the real Gregorian weekday for `1900-01-01`.
4. `WEEKNUM(...,21)` and `ISOWEEKNUM` use the ISO-week Thursday rule over the same serial timeline.
