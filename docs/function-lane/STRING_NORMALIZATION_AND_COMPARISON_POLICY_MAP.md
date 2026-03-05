# String Normalization and Comparison Policy Map

Status: `draft`
Workset: `W7`

## 1. Purpose
Hold the synthesized, version-scoped policy map for Excel string semantics after W7 evidence closure.

## 2. Policy Sections (To Be Filled)
1. Equality semantics (`=`, `<>`, `EXACT`, lookup matching).
2. Ordering/collation semantics (where observable in worksheet functions).
3. Case, accent, punctuation treatment.
4. Whitespace and non-printable normalization.
5. Length limits and overflow/error behavior.
6. Unicode handling behavior (`LEN`, `UNICODE`, `UNICHAR`, slicing functions).
7. Boundary differences:
   - formula evaluation,
   - materialized cell value,
   - reference reuse,
   - persistence/text round-trip,
   - interop ingress/egress.

## 3. Integration Targets
1. W3 value-universe string subtype and boundary tags.
2. W4 coercion primitives touching string conversion/comparison.
3. W6 (`XMATCH`) text-comparison and matching-mode obligations.
