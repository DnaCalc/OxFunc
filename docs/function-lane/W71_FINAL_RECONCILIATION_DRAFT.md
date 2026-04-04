# W71 Final Reconciliation Draft

This draft records the W71 coverage shape after the remaining batches are
finished and the final reconciliation has been emitted.

## 1. Current Counts
1. supported non-deferred rows: `517`
2. witness-covered rows: `517`
3. remaining supported rows: `0`
4. seam-heavy retained rows: `17`

## 2. Publication Shape
The eventual final reconciliation should show:
1. all ordinary supported rows covered by actual witness payloads,
2. seam-heavy rows covered by explicit dependency-gated witness records,
3. no leftover supported-row witness gap in the parked baseline,
4. downstream references updated away from the frozen W69 ledgers.

## 3. Draft Status
This draft is a control artifact for W071. It remains in-progress until the
remaining witness batches are finished and the final reconciliation is emitted.
