# MOD publication research

State: in_progress; scope_completeness=scope_partial;
target_completeness=target_partial; integration_completeness=partial.
Root owns BUG-FUNC-071, report050, catalogG8-34 and the associated bead.

All observations are black-box worksheet calls through public Excel interfaces.
The retained answers include their capture provenance: Excel16.0 build20430,
64-bit Windows, workbook CompatibilityVersion2, Value2 fixture ingress and no
oracle cache. Channel is not recorded by this bulk runner; no channel is inferred.
Inputs are hexadecimal binary64 values, with only signed nonzero normal numbers
or positive zero admitted to avoid known Value2 subnormal/signed-zero changes.
No workbook display values or decimal-formula inputs are used for numeric claims.

## Failed freezes and subsequent discovery

- Endpoint normalization (negative zero and an adjusted value equal to divisor)
  matched22,722 discovery observations. Its first independent21,000 bank had one
  Excel NUM discrepancy for a subnormal remainder. That failure remains retained.
- Tiny boundary6,144 exposed3,076 further differences. A separate reordered,
  fresh-process336 observation bank confirmed stable behavior.
- Discriminator32,400 separated normal/power-of-two divisors and the strict
  MIN_NORMAL/16 sign-adjustment boundary. Research mode8 matched32,400+6,144.
- The first frozen tiny independent34,600 bank matched34,538 and failed62. Every
  miss predicted a normal finite number while Excel returned NUM. The original
  mode8 source, source hash, first judgement and all62 misses remain unchanged.
- After that failure, research mode14 additionally rejects a nonzero subnormal
  native remainder when the absolute quotient is at least2^26. It matches the
  failed34,600 as discovery, and all32,400+6,144 prior observations. Competing
  quotient thresholds2^24,2^25,2^27,2^28 lose4,2,8,10 observations respectively.
- A new1,108-row discriminator brackets quotient2^26 using integer and adjacent
  binary64 inputs, multiple divisor scales and tiny/normal remainder boundaries.
  Mode14 matches1,108/1,108; thresholds2^25 and2^27 lose64 and88 respectively.
- Research mode14 is frozen independently for a new22,344-row bank, with zero
  exact input-tuple overlap against prior captured MOD banks. First judgement
  matches22,344/22,344; frozen source hash is unchanged.
- Production translates the retained generic branches and matches all140,654
  historical observations through public dispatch. This aggregate total includes
  discovery, failed earlier freezes and the22,344 independent research bank;
  it is not140,654 independent validation cases.
- Production/model/test sources are frozen for10,784 new numeric observations
  with no earlier numeric input-tuple overlap, plus353 typed discovery cases.
  First numeric judgement matches10,784/10,784, with all frozen hashes unchanged.
  The cumulative numeric replay is151,438/151,438. Typed discovery matches335/353:
  all18 differences involve explicit Missing, which Excel treats as zero.
- A local MOD prepared wrapper maps only top-level explicit Missing to zero;
  the generator now binds that surface, with the numerical body unchanged.
  This revised discovery matches353/353. The old failed local outcomes remain.
- Review corrected MOD's declaration from UnaryNumericScalarOnly to Custom;
  ValuesOnlyPreAdapter, SurfaceNative and RefOnly remain unchanged. Rust, Lean
  and only the MOD golden row are aligned. candidate-freeze-metadata-v3.json
  supersedes the earlier pre-capture freeze without changing the426 fresh inputs.
- The next frozen426 prepared cases first match414/426. All12failures are
  positional padding: a missing right coordinate must not mask an earlier
  present left VALUE/NUM error. Four failures repeat through reference grids.
  All8 frozen source/helper hashes were unchanged; failedv3 remains retained.
- The v4 local surface now binds the existing ordered_binary prepared helper,
  with its same BinaryNumericExecSpec numeric kernel. No shared executor/helper
  changes are made. The two previous typed banks now match779/779 as discovery.
  Custom metadata is retained; an executable ordered-pair formal binding pins
  both error/padding directions. Three focused MOD tests pass; the metadata
  golden test and8generator guards pass (2regeneration helpers ignored).
  The revised Lean binding compiles in11jobs.
- Fresh288 prepared cases are frozen in mod-padding-heldout-request, with a
  distinct capture run id, w111-mod-padding-independent-run-20260929. The first
  judgement matches288/288, with all10source/model/helper hashes unchanged and
  no harness exclusions. Total final typed replay is1,067/1,067 observations
  (353discovery +426failed earlier freeze used as discovery +288independent).
  Final numerical replay on the same prepared route remains151,438/151,438.
  No additional research or source changes are planned during wrap-up.

The research graph computes native fmod, applies the observed quotient/admission
rules, normalizes the ordinary zero/divisor endpoint, selects the strict
MIN_NORMAL/16 opposite-sign branch, and rejects a nonzero subnormal published
result. These are observed generic branches, not a lookup table. The supplied
fmod primitive remains empirical; the Lean model binds that primitive explicitly
rather than claiming a proof about Microsoft internals.

Open lanes: broader baseline semantic characterization, HO-FN-030 receiving
integration, shared locale
coercion, full current-baseline semantic characterization, and orthogonal
locale/version/platform sweeps. No whole-function promotion is supported.

The retainer stores lossless compact JSON and source snapshots. The manifest
records hashes of both original bytes and retained representation. Failed banks
are never relabeled as independent success after a candidate changes.
