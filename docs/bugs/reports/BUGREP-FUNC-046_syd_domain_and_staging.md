# BUGREP-FUNC-046: SYD domain, publication and arithmetic staging

Status: `triaged`; canonical stream `BUG-FUNC-067`; owner `oxf-mwue.28.17`.

Live Excel discovery admits negative cost and distinguishes period comparison, tiny intermediate handling, and staged multiplication/division. The first frozen candidate matches 3,699 discovery observations but fails 104 of 11,556 fresh independent rows near a life-plus-one rounding boundary. Those failures are retained and drive another arithmetic probe; no repair is promoted.

All three completion axes remain partial; open lanes are listed in the canonical stream.
