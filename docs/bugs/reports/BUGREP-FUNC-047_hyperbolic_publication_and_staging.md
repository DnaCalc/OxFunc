# BUGREP-FUNC-047: Hyperbolic publication and staged arithmetic

Status: `triaged`; canonical stream `BUG-FUNC-068`; owner `oxf-mwue.28.18`.

A fresh 44,950-observation sweep across SINH, COSH, TANH, COTH and CSCH exposes missing TANH saturation metadata, CSCH tiny-result publication, and last-bit differences in compound arithmetic. Candidate staging improves the observed rows; SINH branch structure and the TANH numerator remain under characterization. Independent candidate validation and primitive alignment remain open.

All three completion axes remain partial; open lanes are listed in the canonical stream.
