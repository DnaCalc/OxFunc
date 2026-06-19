# OxFunc Decision Register

Lightweight decision tracking using `ODR-FN-NNN` pattern (OxFunc Decision Record).

## Format

Each decision is a separate file: `ODR-FN-NNN-<SLUG>.md`

## Template

```markdown
# ODR-FN-NNN: <Title>

- **Status**: proposed | accepted | superseded | rejected
- **Date**: YYYY-MM-DD
- **Context**: <why this decision is needed>
- **Decision**: <what was decided>
- **Consequences**: <what follows from this decision>
- **Cross-repo impact**: <impact on OxFml/OxCalc/Foundation if any>
```

## Index

| ID | Title | Status | Date | File |
|----|-------|--------|------|------|
| ODR-FN-001 | Full Empirical Function Identity | accepted | 2026-03-09 | `../function-lane/DOCTRINE_DECISION_FULL_EMPIRICAL_FUNCTION_IDENTITY_20260309.md` |
| ODR-FN-002 | Invocation Test Category Split — Context-Sensitive vs Locally-Evaluable | accepted | 2026-06-18 | `ODR-FN-002-invocation-test-category-split.md` |
| ODR-FN-003 | Single OxFunc↔Excel Discrepancy Catalog | accepted | 2026-06-19 | `ODR-FN-003-single-discrepancy-catalog.md` |
