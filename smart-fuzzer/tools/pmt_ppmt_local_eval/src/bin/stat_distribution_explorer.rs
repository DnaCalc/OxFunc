// stat_distribution_explorer
//
// Generates per-distribution argument bands across the BUG-FUNC-021
// statistical surface (BETADIST/BETAINV, CHIDIST/CHIINV, FDIST/FINV,
// GAMMADIST/GAMMAINV, HYPGEOMDIST, NEGBINOMDIST, NORMSDIST/NORMSINV,
// TDIST/TINV, CONFIDENCE.T, KURT, SKEW, SKEW.P, PERCENTRANK, Z.TEST),
// evaluates each candidate locally through eval_surface_value_call,
// and emits Excel candidate JSONL entries for the cell-ref comparator
// runner. Owning workset: W097.
//
// The candidate shape is intentionally similar to broad_scalar_explorer
// but with a tagged `args_typed` field so logical and matrix arguments
// can be expressed alongside numeric arguments. The legacy `args`
// field is retained for scalar-only candidates so existing Excel
// candidate consumers still work.

use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
use oxfunc_core::value::{
    ArrayCellValue, CallArgValue, EvalArray, EvalValue, ReferenceLike, WorksheetErrorCode,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum TypedArg {
    Number {
        value: f64,
    },
    Logical {
        value: bool,
    },
    Matrix {
        rows: usize,
        cols: usize,
        values: Vec<f64>,
    },
}

impl TypedArg {
    fn as_call_arg(&self) -> CallArgValue {
        match self {
            TypedArg::Number { value } => CallArgValue::Eval(EvalValue::Number(*value)),
            TypedArg::Logical { value } => CallArgValue::Eval(EvalValue::Logical(*value)),
            TypedArg::Matrix { rows, cols, values } => {
                let row_vecs: Vec<Vec<ArrayCellValue>> = (0..*rows)
                    .map(|r| {
                        (0..*cols)
                            .map(|c| ArrayCellValue::Number(values[r * cols + c]))
                            .collect()
                    })
                    .collect();
                let array = EvalArray::from_rows(row_vecs).expect("non-empty rectangular matrix");
                CallArgValue::Eval(EvalValue::Array(array))
            }
        }
    }
}

#[derive(Clone, Copy)]
struct StatFunctionEntry {
    func_id: &'static str,
    name: &'static str,
    family: StatFamily,
}

#[derive(Clone, Copy)]
enum StatFamily {
    BetaDist,
    BetaInv,
    ChiDist,
    ChiInv,
    FDist,
    FInv,
    GammaDist,
    GammaInv,
    HypGeomDist,
    NegBinomDist,
    NormSDist,
    NormSInv,
    TDist,
    TInv,
    ConfidenceT,
    Kurt,
    Skew,
    SkewP,
    PercentRank,
    ZTest,
}

const STAT_FUNCTIONS: &[StatFunctionEntry] = &[
    StatFunctionEntry {
        func_id: "FUNC.BETADIST",
        name: "BETADIST",
        family: StatFamily::BetaDist,
    },
    StatFunctionEntry {
        func_id: "FUNC.BETAINV",
        name: "BETAINV",
        family: StatFamily::BetaInv,
    },
    StatFunctionEntry {
        func_id: "FUNC.CHIDIST",
        name: "CHIDIST",
        family: StatFamily::ChiDist,
    },
    StatFunctionEntry {
        func_id: "FUNC.CHIINV",
        name: "CHIINV",
        family: StatFamily::ChiInv,
    },
    StatFunctionEntry {
        func_id: "FUNC.FDIST",
        name: "FDIST",
        family: StatFamily::FDist,
    },
    StatFunctionEntry {
        func_id: "FUNC.FINV",
        name: "FINV",
        family: StatFamily::FInv,
    },
    StatFunctionEntry {
        func_id: "FUNC.GAMMADIST",
        name: "GAMMADIST",
        family: StatFamily::GammaDist,
    },
    StatFunctionEntry {
        func_id: "FUNC.GAMMAINV",
        name: "GAMMAINV",
        family: StatFamily::GammaInv,
    },
    StatFunctionEntry {
        func_id: "FUNC.HYPGEOMDIST",
        name: "HYPGEOMDIST",
        family: StatFamily::HypGeomDist,
    },
    StatFunctionEntry {
        func_id: "FUNC.NEGBINOMDIST",
        name: "NEGBINOMDIST",
        family: StatFamily::NegBinomDist,
    },
    StatFunctionEntry {
        func_id: "FUNC.NORMSDIST",
        name: "NORMSDIST",
        family: StatFamily::NormSDist,
    },
    StatFunctionEntry {
        func_id: "FUNC.NORMSINV",
        name: "NORMSINV",
        family: StatFamily::NormSInv,
    },
    StatFunctionEntry {
        func_id: "FUNC.TDIST",
        name: "TDIST",
        family: StatFamily::TDist,
    },
    StatFunctionEntry {
        func_id: "FUNC.TINV",
        name: "TINV",
        family: StatFamily::TInv,
    },
    StatFunctionEntry {
        func_id: "FUNC.CONFIDENCE.T",
        name: "CONFIDENCE.T",
        family: StatFamily::ConfidenceT,
    },
    StatFunctionEntry {
        func_id: "FUNC.KURT",
        name: "KURT",
        family: StatFamily::Kurt,
    },
    StatFunctionEntry {
        func_id: "FUNC.SKEW",
        name: "SKEW",
        family: StatFamily::Skew,
    },
    StatFunctionEntry {
        func_id: "FUNC.SKEW.P",
        name: "SKEW.P",
        family: StatFamily::SkewP,
    },
    StatFunctionEntry {
        func_id: "FUNC.PERCENTRANK",
        name: "PERCENTRANK",
        family: StatFamily::PercentRank,
    },
    StatFunctionEntry {
        func_id: "FUNC.Z.TEST",
        name: "Z.TEST",
        family: StatFamily::ZTest,
    },
];

struct NoResolver;

impl ReferenceResolver for NoResolver {
    fn capabilities(&self) -> ResolverCapabilities {
        ResolverCapabilities::permissive_local()
    }

    fn resolve_reference(
        &self,
        reference: &ReferenceLike,
    ) -> Result<EvalValue, RefResolutionError> {
        Err(RefResolutionError::UnresolvedReference {
            target: reference.target.clone(),
        })
    }
}

#[derive(Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Outcome {
    Number {
        value: f64,
        bits_hex: String,
        digest_payload: String,
    },
    Error {
        code: String,
        digest_payload: String,
    },
    Other {
        summary: String,
        digest_payload: String,
    },
}

#[derive(Serialize)]
struct CandidateRecord {
    schema_version: &'static str,
    case_id: String,
    function_id: String,
    function_name: String,
    generator_id: &'static str,
    formula_text: String,
    args: Vec<f64>,
    args_typed: Vec<TypedArg>,
    coverage_buckets: Vec<String>,
    local_outcome: Outcome,
    selection_reason: String,
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed | 1 }
    }
    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }
    fn next_f64(&mut self) -> f64 {
        let bits = self.next_u64() >> 11;
        (bits as f64) / ((1_u64 << 53) as f64)
    }
    fn choose_usize(&mut self, len: usize) -> usize {
        (self.next_u64() as usize) % len
    }
    fn pick_bool(&mut self) -> bool {
        self.next_u64() & 1 == 0
    }
}

// ---- per-distribution argument samplers ----
//
// All samplers stay strictly inside Excel's accepted formula-literal
// domain (no subnormals, no non-finites, no out-of-range probability /
// degree-of-freedom values). When a function admits a "cumulative"
// boolean it is sampled randomly.

fn shape_param(rng: &mut Lcg, label: &str) -> (f64, String) {
    // Positive shape parameter: tiny, small fractional, moderate, large.
    match rng.choose_usize(7) {
        0 => (0.5, format!("{label}:half")),
        1 => (1.0, format!("{label}:one")),
        2 => (1.0 + rng.next_f64() * 9.0, format!("{label}:1_to_10")),
        3 => (10.0 + rng.next_f64() * 90.0, format!("{label}:10_to_100")),
        4 => (rng.next_f64() * 0.5 + 0.001, format!("{label}:tiny_pos")),
        5 => (
            100.0 + rng.next_f64() * 900.0,
            format!("{label}:100_to_1000"),
        ),
        _ => (rng.next_f64() * 50.0 + 0.5, format!("{label}:0.5_to_50.5")),
    }
}

fn dof_pos(rng: &mut Lcg, label: &str) -> (f64, String) {
    // Excel's degrees-of-freedom parameters truncate to integer.
    let bands = [
        (1u64, format!("{label}:1")),
        (2, format!("{label}:2")),
        (5, format!("{label}:5")),
        (10, format!("{label}:10")),
        (30, format!("{label}:30")),
        (100, format!("{label}:100")),
        (rng.next_u64() % 1000 + 1, format!("{label}:rand_1_1000")),
    ];
    let pick = rng.choose_usize(bands.len());
    (bands[pick].0 as f64, bands[pick].1.clone())
}

fn probability_open(rng: &mut Lcg) -> (f64, String) {
    // (0, 1) probability with extra weight on tails / extreme.
    match rng.choose_usize(8) {
        0 => (0.001, "p:0.001".into()),
        1 => (0.005, "p:0.005".into()),
        2 => (0.01, "p:0.01".into()),
        3 => (0.05, "p:0.05".into()),
        4 => (0.5, "p:0.5".into()),
        5 => (0.95, "p:0.95".into()),
        6 => (0.999, "p:0.999".into()),
        _ => {
            let v = (rng.next_f64() * 0.998) + 0.001;
            (v, "p:0.001_0.999".into())
        }
    }
}

fn x_positive_band(rng: &mut Lcg) -> (f64, String) {
    match rng.choose_usize(7) {
        0 => (0.001, "x:0.001".into()),
        1 => (0.5, "x:0.5".into()),
        2 => (1.0, "x:1".into()),
        3 => (5.0, "x:5".into()),
        4 => (rng.next_f64() * 100.0, "x:0_to_100".into()),
        5 => (rng.next_f64() * 1000.0, "x:0_to_1k".into()),
        _ => (rng.next_f64() * 50.0 + 0.5, "x:0.5_to_50.5".into()),
    }
}

fn x_unit(rng: &mut Lcg) -> (f64, String) {
    // (0, 1) value, used for BETADIST x.
    let v = rng.next_f64() * 0.998 + 0.001;
    (v, "x:0.001_0.999".into())
}

fn z_signed(rng: &mut Lcg) -> (f64, String) {
    let sign = if rng.pick_bool() { 1.0 } else { -1.0 };
    match rng.choose_usize(6) {
        0 => (0.0, "z:0".into()),
        1 => (sign * 0.5, "z:0.5_signed".into()),
        2 => (sign * 1.0, "z:1_signed".into()),
        3 => (sign * 2.0, "z:2_signed".into()),
        4 => (sign * 3.0, "z:3_signed".into()),
        _ => (sign * rng.next_f64() * 5.0, "z:0_to_5_signed".into()),
    }
}

fn t_signed(rng: &mut Lcg) -> (f64, String) {
    let sign = if rng.pick_bool() { 1.0 } else { -1.0 };
    match rng.choose_usize(6) {
        0 => (0.0, "t:0".into()),
        1 => (sign * 0.5, "t:0.5_signed".into()),
        2 => (sign * 1.0, "t:1_signed".into()),
        3 => (sign * 2.0, "t:2_signed".into()),
        4 => (sign * 5.0, "t:5_signed".into()),
        _ => (
            sign * (rng.next_f64() * 9.0 + 0.1),
            "t:abs_0.1_9.1_signed".into(),
        ),
    }
}

fn small_array(rng: &mut Lcg, min_len: usize, max_len: usize) -> Vec<f64> {
    let n = rng.choose_usize(max_len - min_len + 1) + min_len;
    (0..n).map(|_| (rng.next_f64() * 100.0) - 50.0).collect()
}

fn pick_args(family: StatFamily, rng: &mut Lcg) -> (Vec<TypedArg>, Vec<String>) {
    let mut buckets: Vec<String> = Vec::new();
    let typed: Vec<TypedArg> = match family {
        StatFamily::BetaDist => {
            let (x, xb) = x_unit(rng);
            let (a, ab) = shape_param(rng, "alpha");
            let (b, bb) = shape_param(rng, "beta");
            buckets.extend([xb, ab, bb]);
            vec![
                TypedArg::Number { value: x },
                TypedArg::Number { value: a },
                TypedArg::Number { value: b },
            ]
        }
        StatFamily::BetaInv => {
            let (p, pb) = probability_open(rng);
            let (a, ab) = shape_param(rng, "alpha");
            let (b, bb) = shape_param(rng, "beta");
            buckets.extend([pb, ab, bb]);
            vec![
                TypedArg::Number { value: p },
                TypedArg::Number { value: a },
                TypedArg::Number { value: b },
            ]
        }
        StatFamily::ChiDist => {
            let (x, xb) = x_positive_band(rng);
            let (d, db) = dof_pos(rng, "dof");
            buckets.extend([xb, db]);
            vec![TypedArg::Number { value: x }, TypedArg::Number { value: d }]
        }
        StatFamily::ChiInv => {
            let (p, pb) = probability_open(rng);
            let (d, db) = dof_pos(rng, "dof");
            buckets.extend([pb, db]);
            vec![TypedArg::Number { value: p }, TypedArg::Number { value: d }]
        }
        StatFamily::FDist => {
            let (x, xb) = x_positive_band(rng);
            let (d1, d1b) = dof_pos(rng, "dof1");
            let (d2, d2b) = dof_pos(rng, "dof2");
            buckets.extend([xb, d1b, d2b]);
            vec![
                TypedArg::Number { value: x },
                TypedArg::Number { value: d1 },
                TypedArg::Number { value: d2 },
            ]
        }
        StatFamily::FInv => {
            let (p, pb) = probability_open(rng);
            let (d1, d1b) = dof_pos(rng, "dof1");
            let (d2, d2b) = dof_pos(rng, "dof2");
            buckets.extend([pb, d1b, d2b]);
            vec![
                TypedArg::Number { value: p },
                TypedArg::Number { value: d1 },
                TypedArg::Number { value: d2 },
            ]
        }
        StatFamily::GammaDist => {
            let (x, xb) = x_positive_band(rng);
            let (a, ab) = shape_param(rng, "alpha");
            let (b, bb) = shape_param(rng, "beta");
            let cumulative = rng.pick_bool();
            buckets.extend([xb, ab, bb]);
            buckets.push(format!("cum:{cumulative}"));
            vec![
                TypedArg::Number { value: x },
                TypedArg::Number { value: a },
                TypedArg::Number { value: b },
                TypedArg::Logical { value: cumulative },
            ]
        }
        StatFamily::GammaInv => {
            let (p, pb) = probability_open(rng);
            let (a, ab) = shape_param(rng, "alpha");
            let (b, bb) = shape_param(rng, "beta");
            buckets.extend([pb, ab, bb]);
            vec![
                TypedArg::Number { value: p },
                TypedArg::Number { value: a },
                TypedArg::Number { value: b },
            ]
        }
        StatFamily::HypGeomDist => {
            // (sample_s, number_sample, population_s, number_pop)
            let pop_size = (rng.next_u64() % 1000 + 10) as f64;
            let pop_s = ((rng.next_u64() % (pop_size as u64)) + 1) as f64;
            let n_sample = ((rng.next_u64() % (pop_size as u64)) + 1) as f64;
            let max_sample_s = n_sample.min(pop_s) as u64;
            let sample_s = if max_sample_s > 0 {
                (rng.next_u64() % max_sample_s) as f64
            } else {
                0.0
            };
            buckets.push(format!("pop:{}", pop_size as u64));
            buckets.push(format!("pop_s:{}", pop_s as u64));
            buckets.push(format!("n:{}", n_sample as u64));
            buckets.push(format!("sample_s:{}", sample_s as u64));
            vec![
                TypedArg::Number { value: sample_s },
                TypedArg::Number { value: n_sample },
                TypedArg::Number { value: pop_s },
                TypedArg::Number { value: pop_size },
            ]
        }
        StatFamily::NegBinomDist => {
            // (number_f, number_s, probability_s)
            let s = (rng.next_u64() % 50 + 1) as f64;
            let f = (rng.next_u64() % 100) as f64;
            let p = rng.next_f64() * 0.98 + 0.01;
            buckets.push(format!("s:{}", s as u64));
            buckets.push(format!("f:{}", f as u64));
            buckets.push("p:0.01_0.99".into());
            vec![
                TypedArg::Number { value: f },
                TypedArg::Number { value: s },
                TypedArg::Number { value: p },
            ]
        }
        StatFamily::NormSDist => {
            let (z, zb) = z_signed(rng);
            buckets.push(zb);
            vec![TypedArg::Number { value: z }]
        }
        StatFamily::NormSInv => {
            let (p, pb) = probability_open(rng);
            buckets.push(pb);
            vec![TypedArg::Number { value: p }]
        }
        StatFamily::TDist => {
            let (x, xb) = t_signed(rng);
            let (d, db) = dof_pos(rng, "dof");
            let tails = if rng.pick_bool() { 1.0 } else { 2.0 };
            buckets.extend([xb, db]);
            buckets.push(format!("tails:{}", tails as u64));
            // TDIST requires non-negative x in Excel; take abs to keep
            // domain valid.
            vec![
                TypedArg::Number { value: x.abs() },
                TypedArg::Number { value: d },
                TypedArg::Number { value: tails },
            ]
        }
        StatFamily::TInv => {
            let (p, pb) = probability_open(rng);
            let (d, db) = dof_pos(rng, "dof");
            buckets.extend([pb, db]);
            vec![TypedArg::Number { value: p }, TypedArg::Number { value: d }]
        }
        StatFamily::ConfidenceT => {
            let (alpha, ab) = probability_open(rng);
            let std = rng.next_f64() * 5.0 + 0.001;
            let size = (rng.next_u64() % 500 + 2) as f64;
            buckets.push(ab);
            buckets.push("std:0_5".into());
            buckets.push(format!("size:{}", size as u64));
            vec![
                TypedArg::Number { value: alpha },
                TypedArg::Number { value: std },
                TypedArg::Number { value: size },
            ]
        }
        StatFamily::Kurt | StatFamily::Skew | StatFamily::SkewP => {
            let arr = small_array(rng, 4, 12);
            buckets.push(format!("len:{}", arr.len()));
            vec![TypedArg::Matrix {
                rows: 1,
                cols: arr.len(),
                values: arr,
            }]
        }
        StatFamily::PercentRank => {
            let arr = small_array(rng, 4, 12);
            let x = arr[rng.choose_usize(arr.len())];
            buckets.push(format!("len:{}", arr.len()));
            buckets.push("x:from_arr".into());
            vec![
                TypedArg::Matrix {
                    rows: 1,
                    cols: arr.len(),
                    values: arr,
                },
                TypedArg::Number { value: x },
            ]
        }
        StatFamily::ZTest => {
            let arr = small_array(rng, 4, 12);
            let x = (rng.next_f64() * 100.0) - 50.0;
            buckets.push(format!("len:{}", arr.len()));
            buckets.push("x:moderate".into());
            vec![
                TypedArg::Matrix {
                    rows: 1,
                    cols: arr.len(),
                    values: arr,
                },
                TypedArg::Number { value: x },
            ]
        }
    };
    (typed, buckets)
}

fn evaluate(func_id: &str, typed_args: &[TypedArg]) -> Outcome {
    let arg_values: Vec<CallArgValue> = typed_args.iter().map(|a| a.as_call_arg()).collect();
    let resolver = NoResolver;
    match eval_surface_value_call(func_id, &arg_values, &resolver, None, None, None, None) {
        Ok(EvalValue::Number(value)) => {
            let bits_hex = format!("0x{:016x}", value.to_bits());
            Outcome::Number {
                value,
                bits_hex: bits_hex.clone(),
                digest_payload: format!("number:{bits_hex}"),
            }
        }
        Ok(EvalValue::Error(code)) => err_outcome(code),
        Ok(other) => Outcome::Other {
            summary: format!("{other:?}"),
            digest_payload: format!("other:{other:?}"),
        },
        Err(code) => err_outcome(code),
    }
}

fn err_outcome(code: WorksheetErrorCode) -> Outcome {
    let code = format!("{code:?}");
    Outcome::Error {
        digest_payload: format!("error:{code}"),
        code,
    }
}

fn outcome_bucket(o: &Outcome) -> String {
    match o {
        Outcome::Number { .. } => "out:number".into(),
        Outcome::Error { code, .. } => format!("out:err:{code}"),
        Outcome::Other { summary, .. } => format!("out:other:{summary}"),
    }
}

fn build_formula(name: &str, typed: &[TypedArg]) -> String {
    let parts: Vec<String> = typed
        .iter()
        .map(|a| match a {
            TypedArg::Number { value } => fmt_num(*value),
            TypedArg::Logical { value } => {
                if *value {
                    "TRUE".into()
                } else {
                    "FALSE".into()
                }
            }
            TypedArg::Matrix { rows, cols, values } => {
                let mut row_parts: Vec<String> = Vec::with_capacity(*rows);
                for r in 0..*rows {
                    let mut col_parts: Vec<String> = Vec::with_capacity(*cols);
                    for c in 0..*cols {
                        col_parts.push(fmt_num(values[r * cols + c]));
                    }
                    row_parts.push(col_parts.join(","));
                }
                format!("{{{}}}", row_parts.join(";"))
            }
        })
        .collect();
    format!("={name}({})", parts.join(","))
}

fn fmt_num(v: f64) -> String {
    if v == 0.0 {
        return "0".into();
    }
    let mag = v.abs();
    if mag < 1e-4 || mag >= 1e16 {
        format!("{v:.17E}")
    } else {
        format!("{v:.17}")
    }
}

fn parse_args() -> Result<(PathBuf, usize, u64, usize), String> {
    let args: Vec<String> = env::args().collect();
    let mut run_dir = None;
    let mut cases = 1_000_000_usize;
    let mut seed = 17_u64;
    let mut candidate_limit = 800_usize;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--run-dir" => {
                i += 1;
                run_dir = args.get(i).map(PathBuf::from);
            }
            "--cases" => {
                i += 1;
                cases = args
                    .get(i)
                    .ok_or("missing --cases")?
                    .parse()
                    .map_err(|_| "bad --cases")?;
            }
            "--seed" => {
                i += 1;
                seed = args
                    .get(i)
                    .ok_or("missing --seed")?
                    .parse()
                    .map_err(|_| "bad --seed")?;
            }
            "--candidate-limit" => {
                i += 1;
                candidate_limit = args
                    .get(i)
                    .ok_or("missing --candidate-limit")?
                    .parse()
                    .map_err(|_| "bad --candidate-limit")?;
            }
            _ => return Err(format!("unknown arg: {}", args[i])),
        }
        i += 1;
    }
    Ok((
        run_dir.ok_or("missing --run-dir")?,
        cases,
        seed,
        candidate_limit,
    ))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (run_dir, case_count, seed, candidate_limit) =
        parse_args().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    fs::create_dir_all(run_dir.join("candidates"))?;

    let started = Instant::now();
    let mut rng = Lcg::new(seed);
    let funcs: Vec<StatFunctionEntry> = STAT_FUNCTIONS.to_vec();
    let mut by_function = BTreeMap::<String, u64>::new();
    let mut by_outcome = BTreeMap::<String, u64>::new();
    let mut by_bucket = BTreeMap::<String, u64>::new();
    let mut by_area = BTreeMap::<String, u64>::new();
    let mut selected_keys = BTreeSet::<String>::new();
    let mut candidates: Vec<CandidateRecord> = Vec::new();

    for index in 0..case_count {
        let f_idx = (rng.next_u64() as usize) % funcs.len();
        let f = funcs[f_idx];
        let (typed, buckets) = pick_args(f.family, &mut rng);
        let outcome = evaluate(f.func_id, &typed);

        *by_function.entry(f.func_id.into()).or_default() += 1;
        let ob = outcome_bucket(&outcome);
        *by_outcome.entry(ob.clone()).or_default() += 1;
        for b in &buckets {
            *by_bucket.entry(b.clone()).or_default() += 1;
        }

        let bucket_join = buckets.join("|");
        let key = format!("{}|{}|{}", f.func_id, bucket_join, ob);
        *by_area.entry(key.clone()).or_default() += 1;

        let must_select = ob.starts_with("out:err") || ob.starts_with("out:other");
        if candidates.len() < candidate_limit && (must_select || selected_keys.insert(key.clone()))
        {
            selected_keys.insert(key.clone());
            // Flat scalar arg list (number-only); empty if any arg is matrix or logical.
            let scalar_args: Vec<f64> = typed
                .iter()
                .filter_map(|a| match a {
                    TypedArg::Number { value } => Some(*value),
                    _ => None,
                })
                .collect();
            let formula = build_formula(f.name, &typed);
            candidates.push(CandidateRecord {
                schema_version: "oxfunc.smart_fuzzer.stat_distribution_candidate.v0",
                case_id: format!("STAT-{index:09}"),
                function_id: f.func_id.into(),
                function_name: f.name.into(),
                generator_id: "stat_distribution_explorer.v0",
                formula_text: formula,
                args: scalar_args,
                args_typed: typed,
                coverage_buckets: buckets,
                local_outcome: outcome.clone(),
                selection_reason: if must_select {
                    "local_error_or_other_witness".into()
                } else {
                    "new_coverage_area".into()
                },
            });
        }
    }

    let candidates_path = run_dir.join("candidates").join("excel_candidates.jsonl");
    let mut writer = BufWriter::new(File::create(&candidates_path)?);
    for c in &candidates {
        serde_json::to_writer(&mut writer, c)?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;

    let elapsed = started.elapsed().as_secs_f64();

    let mut top_areas = by_area.iter().collect::<Vec<_>>();
    top_areas.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    top_areas.truncate(60);

    let local_rollup = serde_json::json!({
        "schema_version": "oxfunc.smart_fuzzer.stat_distribution_local_rollup.v0",
        "run_kind": "stat_distribution_explorer",
        "seed": seed,
        "generated": case_count,
        "local_evaluated": case_count,
        "candidate_count": candidates.len(),
        "local_wall_seconds": elapsed,
        "local_cases_per_second": (case_count as f64) / elapsed.max(f64::MIN_POSITIVE),
        "by_function": by_function,
        "by_outcome": by_outcome,
        "by_coverage_bucket": by_bucket,
        "area_key_count": by_area.len(),
        "top_area_keys": top_areas,
    });
    fs::write(
        run_dir.join("local_rollup.json"),
        serde_json::to_string_pretty(&local_rollup)?,
    )?;

    let mut md = String::new();
    md.push_str("# Stat Distribution Explorer Trace\n\n");
    md.push_str(&format!("- Generated/evaluated locally: `{case_count}`\n"));
    md.push_str(&format!(
        "- Excel candidate samples selected: `{}`\n",
        candidates.len()
    ));
    md.push_str(&format!("- Distinct coverage areas: `{}`\n", by_area.len()));
    md.push_str(&format!(
        "- Local throughput: `{:.2}` cases/sec\n",
        (case_count as f64) / elapsed.max(f64::MIN_POSITIVE)
    ));
    fs::write(run_dir.join("roadmap_trace.md"), md)?;

    println!(
        "stat distribution explorer: generated {case_count}, selected {}, areas {}, cps {:.2}",
        candidates.len(),
        by_area.len(),
        (case_count as f64) / elapsed.max(f64::MIN_POSITIVE)
    );

    Ok(())
}
