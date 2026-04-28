# Expanded Smart-Fuzzer Run Roadmap

Status: `planning_sandbox`

This note records the intended shape for larger smart-fuzzer runs without
making every generated case a durable documentation artifact.

## Current Expanded Lane

The first expanded run targets the financial time-value neighborhood around the
known PMT/PPMT exactness deviation:

1. `PMT`
2. `PPMT`
3. `IPMT`

The run is intentionally split into:

1. a high-volume local exploration lane, about `10,000,000` generated cases;
2. compact coverage and outcome rollups;
3. selected Excel candidate samples;
4. failure packets only for unexpected mismatches.

## Explored Space Areas

The run records coverage by these area axes:

1. function identity,
2. arity shape,
3. rate band,
4. payment horizon band,
5. present-value magnitude and sign,
6. future-value presence and magnitude,
7. payment timing,
8. PPMT/IPMT period position,
9. local outcome class,
10. Excel sample comparison class.

## Expected Deviation Handling

PMT/PPMT/IPMT non-zero-rate exactness drift is currently expected and should be
used as a reference test class for the smart-fuzzer. It should not trigger an
implementation repair unless the blocked `BUG-FUNC-015` lane is explicitly
reopened.

Unexpected mismatches outside that known class remain promotion candidates and
should be recorded under the ordinary failure-packet path.

## First 10M Run Highlights

Run id: `expanded-finance-10m-20260428`

1. Local generated/evaluated cases: `10,000,000`
2. Excel candidate samples: `640`
3. Exact sample matches: `536`
4. Expected known financial exactness or formula-literal encoding deviations:
   `102`
5. Unexpected mismatch samples: `2`
6. Blocked sample rows: `0`
7. Local generation/evaluation throughput: about `126,040` cases/sec
8. Excel sample wall time: about `2.5` seconds for 640 formulas

The two unexpected samples are both `PPMT` high-rate/long-horizon cases where
the local outcome is `#NUM!` and Excel returns a tiny numeric value or zero.
They are recorded as expanded-run failure packets and should be treated as
adjacent evidence for the blocked `BUG-FUNC-015` financial-payment exactness
lane unless later reduction shows a separate root cause.
