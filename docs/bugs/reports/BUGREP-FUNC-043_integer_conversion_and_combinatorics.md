# BUGREP-FUNC-043: INT parity TRUNC GCD LCM and PERMUTATIONA conversion gaps

Status: `triaged`; canonical stream `BUG-FUNC-064`; owner `oxf-mwue.28.14`.

Fresh boundary discovery exposes decimal preparation in INT, a small absolute parity tolerance, wide integer saturation in GCD/LCM, and truncation/power-dispatch errors in TRUNC and PERMUTATIONA. Retain all 45888 numeric and302 typed discovery observations across the broader boundary group; integer conversion and typed aggregate rules remain under characterization. No integer repair is yet promoted.

All three completion axes remain partial; open lanes are listed in the canonical stream.
