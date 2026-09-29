# BUGREP-FUNC-042: SQRT staged rounding and numerical publication boundaries

Status: `triaged`; canonical stream `BUG-FUNC-063`; owner `oxf-mwue.28.14`.

Discovery covers SQRT, PHI, normal densities, RADIANS, STANDARDIZE, SECH and exponential distributions. SQRT uses an extended root followed by a binary64 store; selected functions flush tiny output and reject an overflowing intermediate. Fresh independent data supports SQRT, PHI, STANDARDIZE and exponential numeric candidates but exposes five RADIANS and two SECH last-bit residuals, repaired with staged arithmetic and awaiting new confirmation. Normal CDF and generalized density arithmetic remain partial.

All three completion axes remain partial; open lanes are listed in the canonical stream.
