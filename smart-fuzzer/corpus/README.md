# Candidate Corpus

Status: `candidate_only`

This directory is for minimized or semi-minimized smart-fuzzer cases that are
not yet promoted into canonical OxFunc bug, function-lane, or regression-test
surfaces.

A case should move out of this directory when it becomes durable evidence:

1. actionable mismatch: promote through `docs/bugs/`;
2. ordinary regression: add focused tests or scenario manifests under the owning
   workset;
3. seam issue: file the required handoff if another repo must act;
4. non-actionable harness limit: record it as a seam/harness limitation, not as
   a function-semantic mismatch.
