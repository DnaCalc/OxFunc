# XLL Add-in Bridge Registration Notes

Status: `provisional`
Workset: `W9`

## 1. Purpose
Record seed registration posture for U-style vs Q-style surfaces in the OxFunc XLL bridge.

## 2. Registration Matrix
| worksheet_name | export_name | type_text | posture | rationale |
| --- | --- | --- | --- | --- |
| `ox_ABS` | `OX_ABS` | `QU` | `U-style` | Reference-admitting surface for dereference-policy experiments and future selective probing. |
| `ox_ABS_Q` | `OX_ABS_Q` | `BB` | `Q-style` | Numeric control lane with minimal transport overhead and no reference path. |
| `ox_PI` | `OX_PI` | `B` | `Q-style` | Nullary constant control lane. |

## 3. Registration Call Path
1. Self-registration in `xlAutoOpen`.
2. `xlfRegister` invocation through direct Rust callback binding to Excel `MdCallBack12`.
3. Category label: `OxFunc Bridge`.
4. Source-of-truth export rows are declared in `crates/oxfunc_core/src/xll_export_specs.rs`.
5. Export wrappers and registration rows are generated during build from that core source.
6. CSV snapshot (`tools/xll-addin/oxfunc_xll/export_specs.csv`) is generated from core for audit/review.

## 4. Current Policy Decisions
1. Keep both U-style and Q-style ABS exports to compare behavior and transport costs.
2. Do not collapse to a single signature policy yet.
3. Keep core function semantics in `oxfunc_core`; XLL layer remains transport + registration + type conversion only.
4. Keep export rows declarative in core; wrappers/rows and CSV snapshots are generated mechanically.

## 5. Follow-on Decisions
1. Decide whether `ox_ABS_Q` remains public or becomes internal benchmark/control.
2. Evaluate if future reference-subset dereference requests should be exposed in bridge policy.
3. Expand registration rows for additional seed functions once W9 baseline is stable.
