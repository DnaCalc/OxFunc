# Local Runs

Generated smart-fuzzer run outputs go here.

This directory is ignored by default except for this README. Run outputs may be
large, machine-specific, or tied to a local Excel installation. Promote only
reduced, evidence-bearing artifacts into canonical repo surfaces.

Expected shape:

1. compact telemetry rows for ordinary passes,
2. aggregate coverage and throughput summaries,
3. full packets only for mismatches, unstable outcomes, blocked harness rows,
   and reduced reproducers.

Minimum run metadata before promotion consideration:

1. manifest hash,
2. generator id and seed,
3. runner version,
4. git revision and tree state,
5. Excel version/build/channel,
6. workbook compatibility descriptor,
7. locale/profile,
8. comparison policy id,
9. mismatch classification.
