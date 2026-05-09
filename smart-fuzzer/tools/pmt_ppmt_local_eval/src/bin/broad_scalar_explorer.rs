use oxfunc_core::functions::surface_dispatch::eval_surface_value_call;
use oxfunc_core::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
use oxfunc_core::value::{CallArgValue, EvalValue, ReferenceLike, WorksheetErrorCode};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

// Family enumerates per-arity, per-domain numeric kernels useful for
// bit-exact differential probing against Excel.
#[derive(Clone, Copy)]
enum Family {
    Unary,
    UnaryPositive,    // domain ~ [0, +inf): SQRT, SQRTPI, LN, LOG10
    UnaryUnitOpen,    // domain (-1, +1): ATANH, FISHER
    UnaryUnitClosed,  // domain [-1, +1]: ASIN, ACOS
    UnaryAtLeastOne,  // domain [1, +inf): ACOSH
    UnaryNonZeroInt,  // small/medium signed integers: FACT(>=0), FACTDOUBLE
    UnarySmallNonNeg, // small non-negative integers ~0..170: FACT
    Trig,             // -1e16..+1e16 with argument-reduction interesting bands
    Exp,              // -745..+745 special bands
    Atan2,            // (y, x) with quadrant boundaries
    Power,            // (b, e) with base/exp boundary bands
    Mod,              // (n, d) with near-multiples and tiny d
    LogBase,          // (n, b) with near-1 bases
    RoundFamily,      // (n, num_digits) with negative digits, tied halves
    Mround,           // (n, mult) with negative, fractional mult
    Combin2,          // (n, k)
    Permut2,          // (n, k)
}

#[derive(Clone, Copy)]
struct FunctionEntry {
    func_id: &'static str,
    name: &'static str,
    family: Family,
}

const FUNCTIONS: &[FunctionEntry] = &[
    // unary on full real domain (excluding possible singularities)
    FunctionEntry { func_id: "FUNC.ABS", name: "ABS", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.SIGN", name: "SIGN", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.INT", name: "INT", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.TRUNC", name: "TRUNC", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.EVEN", name: "EVEN", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.ODD", name: "ODD", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.ATAN", name: "ATAN", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.ASINH", name: "ASINH", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.SINH", name: "SINH", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.COSH", name: "COSH", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.TANH", name: "TANH", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.DEGREES", name: "DEGREES", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.RADIANS", name: "RADIANS", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.GAUSS", name: "GAUSS", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.PHI", name: "PHI", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.ERF", name: "ERF", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.ERFC", name: "ERFC", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.ERF.PRECISE", name: "ERF.PRECISE", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.ERFC.PRECISE", name: "ERFC.PRECISE", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.ACOT", name: "ACOT", family: Family::Unary },

    // unary positive / non-negative
    FunctionEntry { func_id: "FUNC.SQRT", name: "SQRT", family: Family::UnaryPositive },
    FunctionEntry { func_id: "FUNC.SQRTPI", name: "SQRTPI", family: Family::UnaryPositive },
    FunctionEntry { func_id: "FUNC.LN", name: "LN", family: Family::UnaryPositive },
    FunctionEntry { func_id: "FUNC.LOG10", name: "LOG10", family: Family::UnaryPositive },
    FunctionEntry { func_id: "FUNC.GAMMA", name: "GAMMA", family: Family::Unary }, // negative non-int allowed
    FunctionEntry { func_id: "FUNC.GAMMALN", name: "GAMMALN", family: Family::UnaryPositive },
    FunctionEntry { func_id: "FUNC.GAMMALN.PRECISE", name: "GAMMALN.PRECISE", family: Family::UnaryPositive },

    // unary unit-open (-1, +1)
    FunctionEntry { func_id: "FUNC.ATANH", name: "ATANH", family: Family::UnaryUnitOpen },
    FunctionEntry { func_id: "FUNC.FISHER", name: "FISHER", family: Family::UnaryUnitOpen },
    FunctionEntry { func_id: "FUNC.FISHERINV", name: "FISHERINV", family: Family::Unary },

    // unary unit-closed [-1, +1]
    FunctionEntry { func_id: "FUNC.ASIN", name: "ASIN", family: Family::UnaryUnitClosed },
    FunctionEntry { func_id: "FUNC.ACOS", name: "ACOS", family: Family::UnaryUnitClosed },

    // unary at-least-one
    FunctionEntry { func_id: "FUNC.ACOSH", name: "ACOSH", family: Family::UnaryAtLeastOne },
    FunctionEntry { func_id: "FUNC.ACOTH", name: "ACOTH", family: Family::UnaryAtLeastOne }, // |x|>1

    // small non-negative integers
    FunctionEntry { func_id: "FUNC.FACT", name: "FACT", family: Family::UnarySmallNonNeg },
    FunctionEntry { func_id: "FUNC.FACTDOUBLE", name: "FACTDOUBLE", family: Family::UnarySmallNonNeg },

    // trig functions wide-arg
    FunctionEntry { func_id: "FUNC.SIN", name: "SIN", family: Family::Trig },
    FunctionEntry { func_id: "FUNC.COS", name: "COS", family: Family::Trig },
    FunctionEntry { func_id: "FUNC.TAN", name: "TAN", family: Family::Trig },
    FunctionEntry { func_id: "FUNC.COT", name: "COT", family: Family::Trig },
    FunctionEntry { func_id: "FUNC.SEC", name: "SEC", family: Family::Trig },
    FunctionEntry { func_id: "FUNC.CSC", name: "CSC", family: Family::Trig },
    FunctionEntry { func_id: "FUNC.SECH", name: "SECH", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.CSCH", name: "CSCH", family: Family::Unary },
    FunctionEntry { func_id: "FUNC.COTH", name: "COTH", family: Family::Unary },

    // exp wide range
    FunctionEntry { func_id: "FUNC.EXP", name: "EXP", family: Family::Exp },

    // 2-arg
    FunctionEntry { func_id: "FUNC.ATAN2", name: "ATAN2", family: Family::Atan2 },
    FunctionEntry { func_id: "FUNC.POWER", name: "POWER", family: Family::Power },
    FunctionEntry { func_id: "FUNC.MOD", name: "MOD", family: Family::Mod },
    FunctionEntry { func_id: "FUNC.LOG", name: "LOG", family: Family::LogBase },
    FunctionEntry { func_id: "FUNC.ROUND", name: "ROUND", family: Family::RoundFamily },
    FunctionEntry { func_id: "FUNC.ROUNDUP", name: "ROUNDUP", family: Family::RoundFamily },
    FunctionEntry { func_id: "FUNC.ROUNDDOWN", name: "ROUNDDOWN", family: Family::RoundFamily },
    FunctionEntry { func_id: "FUNC.MROUND", name: "MROUND", family: Family::Mround },
    FunctionEntry { func_id: "FUNC.QUOTIENT", name: "QUOTIENT", family: Family::Mod },
    FunctionEntry { func_id: "FUNC.COMBIN", name: "COMBIN", family: Family::Combin2 },
    FunctionEntry { func_id: "FUNC.COMBINA", name: "COMBINA", family: Family::Combin2 },
    FunctionEntry { func_id: "FUNC.PERMUT", name: "PERMUT", family: Family::Permut2 },
    FunctionEntry { func_id: "FUNC.PERMUTATIONA", name: "PERMUTATIONA", family: Family::Permut2 },
    FunctionEntry { func_id: "FUNC.NonUsed", name: "X", family: Family::UnaryNonZeroInt }, // tombstone
];

// Filter out tombstone at runtime.
fn live_functions() -> Vec<FunctionEntry> {
    FUNCTIONS
        .iter()
        .copied()
        .filter(|e| !matches!(e.family, Family::UnaryNonZeroInt))
        .collect()
}

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
    fn signed(&mut self) -> f64 {
        if self.next_u64() & 1 == 0 { 1.0 } else { -1.0 }
    }
}

// ---- numeric band samplers ----

fn band_unary(rng: &mut Lcg) -> (f64, &'static str) {
    // Excel formula literal parser does not accept subnormal magnitudes
    // (~5e-324) or non-finite values; constrain bands to formula-representable
    // finite normals so the Excel side does not throw on parse.
    let band = rng.choose_usize(12);
    match band {
        0 => (0.0, "u:zero"),
        1 => (rng.signed() * 1e-300, "u:tiny_pos_neg_1e-300"),
        2 => (rng.signed() * 1e-200, "u:tiny_pos_neg_1e-200"),
        3 => (rng.signed() * 1e-100, "u:tiny_pos_neg_1e-100"),
        4 => (rng.signed() * 1e-12, "u:tiny_1e-12"),
        5 => (rng.signed() * (rng.next_f64() * 1e-3), "u:near_zero"),
        6 => (rng.signed() * (1.0 + rng.next_f64() * 0.001), "u:near_one"),
        7 => (rng.signed() * (rng.next_f64() * 10.0), "u:0_to_10"),
        8 => (rng.signed() * (1.0 + rng.next_f64() * 99.0), "u:1_to_100"),
        9 => (rng.signed() * (1.0 + rng.next_f64() * 999.0), "u:1_to_1k"),
        10 => (rng.signed() * 1e6 * rng.next_f64(), "u:up_to_1e6"),
        11 => {
            // dyadic fraction
            let n = (rng.next_u64() % 1024) as i64 - 512;
            let d_pow = (rng.next_u64() % 30) as i32;
            let v = (n as f64) / (1u64 << d_pow) as f64;
            (v, "u:dyadic_fraction")
        }
        _ => (rng.signed() * 1e15 * rng.next_f64(), "u:up_to_1e15"),
    }
}

fn band_unary_positive(rng: &mut Lcg) -> (f64, &'static str) {
    let (v, b) = band_unary(rng);
    let mapped: &'static str = match b {
        "u:zero" => "p:zero",
        "u:tiny_pos_neg_1e-300" => "p:tiny_1e-300",
        "u:tiny_pos_neg_1e-200" => "p:tiny_1e-200",
        "u:tiny_pos_neg_1e-100" => "p:tiny_1e-100",
        "u:tiny_1e-12" => "p:tiny_1e-12",
        "u:near_zero" => "p:near_zero",
        "u:near_one" => "p:near_one",
        "u:0_to_10" => "p:0_to_10",
        "u:1_to_100" => "p:1_to_100",
        "u:1_to_1k" => "p:1_to_1k",
        "u:up_to_1e6" => "p:up_to_1e6",
        "u:up_to_1e15" => "p:up_to_1e15",
        _ => "p:dyadic",
    };
    (v.abs(), mapped)
}

fn band_unit_open(rng: &mut Lcg) -> (f64, &'static str) {
    match rng.choose_usize(7) {
        0 => (0.0, "uo:zero"),
        1 => (rng.signed() * 1e-100, "uo:near_zero_tiny"),
        2 => (rng.signed() * 1e-9, "uo:tiny"),
        3 => (rng.signed() * 0.5 * rng.next_f64(), "uo:moderate"),
        4 => (rng.signed() * (1.0 - 1e-9), "uo:near_one"),
        5 => (rng.signed() * (1.0 - 1e-15), "uo:edge_one"),
        _ => (rng.signed() * rng.next_f64() * 0.999_9, "uo:0_to_1"),
    }
}

fn band_unit_closed(rng: &mut Lcg) -> (f64, &'static str) {
    match rng.choose_usize(8) {
        0 => (0.0, "uc:zero"),
        1 => (1.0, "uc:plus_one"),
        2 => (-1.0, "uc:minus_one"),
        3 => (rng.signed() * (1.0 - 1e-15), "uc:near_pm_one"),
        4 => (rng.signed() * 1e-15, "uc:near_zero"),
        5 => (rng.signed() * (rng.next_f64() * 0.999_999), "uc:moderate"),
        6 => (rng.signed() * 0.5, "uc:plus_minus_half"),
        _ => (rng.signed() * rng.next_f64(), "uc:random"),
    }
}

fn band_at_least_one(rng: &mut Lcg) -> (f64, &'static str) {
    match rng.choose_usize(7) {
        0 => (1.0, "ao:one"),
        1 => (1.0 + 1e-15, "ao:near_one_above"),
        2 => (1.0 + 1e-3, "ao:slightly_above"),
        3 => (1.0 + rng.next_f64() * 9.0, "ao:1_to_10"),
        4 => (10.0 + rng.next_f64() * 990.0, "ao:10_to_1000"),
        5 => (1e6 + rng.next_f64() * 1e9, "ao:huge"),
        _ => (1e15 * rng.next_f64() + 1.0, "ao:up_to_1e15"),
    }
}

fn band_small_nonneg(rng: &mut Lcg) -> (f64, &'static str) {
    match rng.choose_usize(7) {
        0 => (0.0, "sn:zero"),
        1 => (1.0, "sn:one"),
        2 => ((rng.next_u64() % 20) as f64, "sn:0_to_20"),
        3 => ((rng.next_u64() % 50) as f64, "sn:0_to_50"),
        4 => ((rng.next_u64() % 170) as f64, "sn:0_to_170"),
        5 => ((rng.next_u64() % 200) as f64, "sn:0_to_200"),
        _ => {
            // include non-integers near small ints to force INT-coerce branch
            let n = rng.next_u64() % 30;
            (n as f64 + rng.next_f64() * 0.95, "sn:non_integer")
        }
    }
}

fn band_trig(rng: &mut Lcg) -> (f64, &'static str) {
    use std::f64::consts::PI;
    match rng.choose_usize(12) {
        0 => (0.0, "t:zero"),
        1 => (rng.signed() * PI, "t:pi_exact"),
        2 => (rng.signed() * PI / 2.0, "t:half_pi"),
        3 => (rng.signed() * PI / 4.0, "t:quarter_pi"),
        4 => (rng.signed() * (PI - 1e-12), "t:near_pi"),
        5 => (rng.signed() * 1e-12, "t:tiny"),
        6 => (rng.signed() * 100.0 * PI * rng.next_f64(), "t:moderate_pi_multiples"),
        7 => (rng.signed() * 1e6 * rng.next_f64(), "t:up_to_1e6"),
        8 => (rng.signed() * 1e10 * rng.next_f64(), "t:up_to_1e10"),
        9 => (rng.signed() * 1e15 * rng.next_f64(), "t:up_to_1e15"),
        10 => (rng.signed() * 2.0_f64.powi(53), "t:two_to_53"),
        _ => (rng.signed() * rng.next_f64() * 4.0 * PI, "t:0_to_4pi"),
    }
}

fn band_exp(rng: &mut Lcg) -> (f64, &'static str) {
    match rng.choose_usize(11) {
        0 => (0.0, "e:zero"),
        1 => (1.0, "e:one"),
        2 => (-1.0, "e:minus_one"),
        3 => (rng.signed() * 1e-12, "e:tiny"),
        4 => (rng.signed() * (rng.next_f64() * 5.0), "e:moderate"),
        5 => (rng.signed() * (10.0 + rng.next_f64() * 90.0), "e:10_to_100"),
        6 => (-700.0 - rng.next_f64() * 45.0, "e:near_underflow"),
        7 => (700.0 + rng.next_f64() * 9.0, "e:near_overflow"),
        8 => (709.0, "e:overflow_threshold"),
        9 => (rng.signed() * 100.0 * rng.next_f64(), "e:0_to_100_signed"),
        _ => (rng.signed() * 500.0 * rng.next_f64(), "e:0_to_500_signed"),
    }
}

fn band_atan2(rng: &mut Lcg) -> (Vec<f64>, String) {
    let pick_axis = |rng: &mut Lcg| -> (f64, &'static str) {
        match rng.choose_usize(9) {
            0 => (0.0, "ax:zero"),
            1 => (rng.signed() * 1e-200, "ax:tiny"),
            2 => (1.0, "ax:plus_one"),
            3 => (-1.0, "ax:minus_one"),
            4 => (rng.signed() * (rng.next_f64() * 10.0), "ax:0_to_10"),
            5 => (rng.signed() * (rng.next_f64() * 1e6), "ax:0_to_1e6"),
            6 => (rng.signed() * 1e15 * rng.next_f64(), "ax:large"),
            7 => (rng.signed() * 1e200 * rng.next_f64(), "ax:huge"),
            _ => (rng.signed() * rng.next_f64(), "ax:0_to_1"),
        }
    };
    let (x, xb) = pick_axis(rng);
    let (y, yb) = pick_axis(rng);
    // Excel ATAN2 takes (x, y) per spec
    (vec![x, y], format!("a2:x={xb}|y={yb}"))
}

fn band_power(rng: &mut Lcg) -> (Vec<f64>, String) {
    let (b, bb) = match rng.choose_usize(10) {
        0 => (0.0, "b:zero"),
        1 => (1.0, "b:one"),
        2 => (-1.0, "b:minus_one"),
        3 => (rng.signed() * (rng.next_f64() * 10.0), "b:0_to_10"),
        4 => (rng.signed() * (rng.next_f64() * 100.0), "b:0_to_100"),
        5 => (2.0, "b:two"),
        6 => (10.0, "b:ten"),
        7 => (rng.signed() * 1e6 * rng.next_f64(), "b:large"),
        8 => (rng.signed() * 1e-3, "b:tiny"),
        _ => (rng.signed() * rng.next_f64(), "b:fractional"),
    };
    let (e, eb) = match rng.choose_usize(11) {
        0 => (0.0, "e:zero"),
        1 => (1.0, "e:one"),
        2 => (-1.0, "e:minus_one"),
        3 => (0.5, "e:half"),
        4 => (-0.5, "e:minus_half"),
        5 => ((rng.next_u64() % 10) as f64, "e:int_0_10"),
        6 => (-((rng.next_u64() % 10) as f64), "e:int_neg"),
        7 => (rng.signed() * (rng.next_f64() * 10.0), "e:fractional"),
        8 => (rng.signed() * (rng.next_f64() * 100.0), "e:0_to_100"),
        9 => (rng.signed() * 700.0, "e:near_limit"),
        _ => (rng.signed() * (rng.next_f64() * 30.0), "e:0_to_30"),
    };
    (vec![b, e], format!("pw:{bb}|{eb}"))
}

fn band_mod(rng: &mut Lcg) -> (Vec<f64>, String) {
    let n_choices = ["0_to_100", "near_zero", "negative", "large", "huge"];
    let d_choices = ["one", "small", "fractional", "near_zero", "negative"];
    let n_pick = rng.choose_usize(n_choices.len());
    let d_pick = rng.choose_usize(d_choices.len());
    let n = match n_pick {
        0 => rng.signed() * (rng.next_f64() * 100.0),
        1 => rng.signed() * 1e-12,
        2 => -(rng.next_f64() * 1e6),
        3 => rng.signed() * 1e10 * rng.next_f64(),
        _ => rng.signed() * 1e15 * rng.next_f64(),
    };
    let d = match d_pick {
        0 => 1.0,
        1 => 0.1 + rng.next_f64() * 9.9,
        2 => rng.next_f64(),
        3 => 1e-12 * (1.0 + rng.next_f64()),
        _ => -(0.1 + rng.next_f64() * 9.9),
    };
    (vec![n, d], format!("md:n={}|d={}", n_choices[n_pick], d_choices[d_pick]))
}

fn band_log_base(rng: &mut Lcg) -> (Vec<f64>, String) {
    let n = match rng.choose_usize(7) {
        0 => 1.0,
        1 => 10.0,
        2 => rng.next_f64() * 10.0,
        3 => 1e-12,
        4 => 1e12 * rng.next_f64(),
        5 => 1e-100,
        _ => rng.next_f64() * 1000.0,
    };
    let b = match rng.choose_usize(7) {
        0 => 10.0,
        1 => 2.0,
        2 => std::f64::consts::E,
        3 => 1.0 + 1e-9,
        4 => rng.next_f64() * 9.0 + 1.0,
        5 => 100.0,
        _ => 0.5 + rng.next_f64() * 0.4,
    };
    (vec![n, b], format!("lb:n={}|b={}", fmt_short(n), fmt_short(b)))
}

fn band_round(rng: &mut Lcg) -> (Vec<f64>, String) {
    // produce tied halves and other rounding-edge inputs
    let n = match rng.choose_usize(8) {
        0 => 0.5 + (rng.next_u64() % 100) as f64,
        1 => -0.5 - (rng.next_u64() % 100) as f64,
        2 => rng.signed() * (rng.next_f64() * 1e6),
        3 => rng.signed() * 1e-9,
        4 => rng.signed() * (rng.next_f64() * 1000.0).floor() + 0.5, // many ties
        5 => rng.signed() * 9.999_999_999_999_999_e15,
        6 => rng.signed() * (1.0 + rng.next_f64()),
        _ => rng.signed() * rng.next_f64() * 1e10,
    };
    let d = match rng.choose_usize(7) {
        0 => 0.0,
        1 => 1.0,
        2 => 2.0,
        3 => -1.0,
        4 => -2.0,
        5 => (rng.next_u64() % 8) as f64,
        _ => -((rng.next_u64() % 8) as f64),
    };
    (vec![n, d], format!("rd:n={}|d={}", fmt_short(n), fmt_short(d)))
}

fn band_mround(rng: &mut Lcg) -> (Vec<f64>, String) {
    let n = match rng.choose_usize(6) {
        0 => 0.0,
        1 => 1.0,
        2 => rng.signed() * (rng.next_f64() * 1000.0),
        3 => rng.signed() * 1e6 * rng.next_f64(),
        4 => rng.signed() * 0.5,
        _ => rng.signed() * rng.next_f64() * 1e3,
    };
    let m = match rng.choose_usize(6) {
        0 => 1.0,
        1 => 0.5,
        2 => 0.1,
        3 => 0.25,
        4 => rng.next_f64() * 9.999 + 0.001,
        _ => -(rng.next_f64() * 9.999 + 0.001),
    };
    (vec![n, m], format!("mr:n={}|m={}", fmt_short(n), fmt_short(m)))
}

fn band_combin2(rng: &mut Lcg) -> (Vec<f64>, String) {
    let n = (rng.next_u64() % 200) as f64;
    let k = (rng.next_u64() % (n.max(1.0) as u64 + 1)) as f64;
    (vec![n, k], format!("cb:n={}|k={}", n as u64, k as u64))
}

fn band_permut2(rng: &mut Lcg) -> (Vec<f64>, String) {
    let n = (rng.next_u64() % 170) as f64;
    let k = (rng.next_u64() % (n.max(1.0) as u64 + 1)) as f64;
    (vec![n, k], format!("pm:n={}|k={}", n as u64, k as u64))
}

// short-format helper: keep coverage keys compact
fn fmt_short(v: f64) -> String {
    if v == 0.0 { "0".into() }
    else if v.is_nan() { "nan".into() }
    else if v.is_infinite() { if v > 0.0 { "inf".into() } else { "-inf".into() } }
    else {
        let log10 = v.abs().log10();
        if log10.is_finite() && (-3.0..=6.0).contains(&log10) { format!("{v:.4}") }
        else { format!("{v:e}") }
    }
}

// Round-trip canonical literal for Excel formula. Excel's formula parser
// rejects subnormal magnitudes (~5e-324) and non-finite values, so the
// generator constrains values to formula-representable normals.
fn fmt_num(v: f64) -> String {
    if v == 0.0 { return "0".into(); }
    if !v.is_finite() {
        // Should not occur after band constraints; fall back to a clamped value.
        return if v > 0.0 { "1E+307".into() } else { "-1E+307".into() };
    }
    if v.is_subnormal() {
        // Should not occur after band constraints; clamp to smallest normal.
        return if v > 0.0 { "2.2250738585072014E-308".into() }
               else { "-2.2250738585072014E-308".into() };
    }
    let mag = v.abs();
    if mag < 1e-4 || mag >= 1e16 {
        format!("{v:.17E}")
    } else {
        format!("{v:.17}")
    }
}

fn build_formula(name: &str, args: &[f64]) -> String {
    let parts = args.iter().map(|v| fmt_num(*v)).collect::<Vec<_>>().join(",");
    format!("={name}({parts})")
}

fn evaluate(func_id: &str, args: &[f64]) -> Outcome {
    let arg_values = args
        .iter()
        .copied()
        .map(|v| CallArgValue::Eval(EvalValue::Number(v)))
        .collect::<Vec<_>>();
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

fn pick_args(family: Family, rng: &mut Lcg) -> (Vec<f64>, Vec<String>) {
    let mut buckets: Vec<String> = Vec::with_capacity(2);
    let args = match family {
        Family::Unary => {
            let (v, b) = band_unary(rng);
            buckets.push(b.into());
            vec![v]
        }
        Family::UnaryPositive => {
            let (v, b) = band_unary_positive(rng);
            buckets.push(b.into());
            vec![v]
        }
        Family::UnaryUnitOpen => {
            let (v, b) = band_unit_open(rng);
            buckets.push(b.into());
            vec![v]
        }
        Family::UnaryUnitClosed => {
            let (v, b) = band_unit_closed(rng);
            buckets.push(b.into());
            vec![v]
        }
        Family::UnaryAtLeastOne => {
            let (v, b) = band_at_least_one(rng);
            buckets.push(b.into());
            vec![v]
        }
        Family::UnarySmallNonNeg | Family::UnaryNonZeroInt => {
            let (v, b) = band_small_nonneg(rng);
            buckets.push(b.into());
            vec![v]
        }
        Family::Trig => {
            let (v, b) = band_trig(rng);
            buckets.push(b.into());
            vec![v]
        }
        Family::Exp => {
            let (v, b) = band_exp(rng);
            buckets.push(b.into());
            vec![v]
        }
        Family::Atan2 => {
            let (a, b) = band_atan2(rng);
            buckets.push(b);
            a
        }
        Family::Power => {
            let (a, b) = band_power(rng);
            buckets.push(b);
            a
        }
        Family::Mod => {
            let (a, b) = band_mod(rng);
            buckets.push(b);
            a
        }
        Family::LogBase => {
            let (a, b) = band_log_base(rng);
            buckets.push(b);
            a
        }
        Family::RoundFamily => {
            let (a, b) = band_round(rng);
            buckets.push(b);
            a
        }
        Family::Mround => {
            let (a, b) = band_mround(rng);
            buckets.push(b);
            a
        }
        Family::Combin2 => {
            let (a, b) = band_combin2(rng);
            buckets.push(b);
            a
        }
        Family::Permut2 => {
            let (a, b) = band_permut2(rng);
            buckets.push(b);
            a
        }
    };
    (args, buckets)
}

fn parse_args() -> Result<(PathBuf, usize, u64, usize), String> {
    let args: Vec<String> = env::args().collect();
    let mut run_dir = None;
    let mut cases = 5_000_000_usize;
    let mut seed = 17_u64;
    let mut candidate_limit = 800_usize;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--run-dir" => { i += 1; run_dir = args.get(i).map(PathBuf::from); }
            "--cases" => {
                i += 1;
                cases = args.get(i).ok_or("missing --cases")?.parse().map_err(|_| "bad --cases")?;
            }
            "--seed" => {
                i += 1;
                seed = args.get(i).ok_or("missing --seed")?.parse().map_err(|_| "bad --seed")?;
            }
            "--candidate-limit" => {
                i += 1;
                candidate_limit = args
                    .get(i).ok_or("missing --candidate-limit")?
                    .parse().map_err(|_| "bad --candidate-limit")?;
            }
            _ => return Err(format!("unknown arg: {}", args[i])),
        }
        i += 1;
    }
    Ok((run_dir.ok_or("missing --run-dir")?, cases, seed, candidate_limit))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (run_dir, case_count, seed, candidate_limit) =
        parse_args().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    fs::create_dir_all(run_dir.join("candidates"))?;

    let started = Instant::now();
    let mut rng = Lcg::new(seed);
    let funcs = live_functions();
    let mut by_function = BTreeMap::<String, u64>::new();
    let mut by_outcome = BTreeMap::<String, u64>::new();
    let mut by_bucket = BTreeMap::<String, u64>::new();
    let mut by_area = BTreeMap::<String, u64>::new();
    let mut selected_keys = BTreeSet::<String>::new();
    let mut candidates: Vec<CandidateRecord> = Vec::new();

    for index in 0..case_count {
        let f_idx = (rng.next_u64() as usize) % funcs.len();
        let f = funcs[f_idx];
        let (args, buckets) = pick_args(f.family, &mut rng);
        let outcome = evaluate(f.func_id, &args);

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
        if candidates.len() < candidate_limit && (must_select || selected_keys.insert(key.clone())) {
            selected_keys.insert(key.clone());
            let formula = build_formula(f.name, &args);
            candidates.push(CandidateRecord {
                schema_version: "oxfunc.smart_fuzzer.broad_scalar_candidate.v0",
                case_id: format!("BSE-{index:09}"),
                function_id: f.func_id.into(),
                function_name: f.name.into(),
                generator_id: "broad_scalar_explorer.v0",
                formula_text: formula,
                args: args.clone(),
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
        "schema_version": "oxfunc.smart_fuzzer.broad_scalar_local_rollup.v0",
        "run_kind": "broad_scalar_explorer",
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
    fs::write(run_dir.join("local_rollup.json"), serde_json::to_string_pretty(&local_rollup)?)?;

    let mut md = String::new();
    md.push_str("# Broad Scalar Explorer Trace\n\n");
    md.push_str(&format!("- Generated/evaluated locally: `{case_count}`\n"));
    md.push_str(&format!("- Excel candidate samples selected: `{}`\n", candidates.len()));
    md.push_str(&format!("- Distinct coverage areas: `{}`\n", by_area.len()));
    md.push_str(&format!("- Local throughput: `{:.2}` cases/sec\n", (case_count as f64) / elapsed.max(f64::MIN_POSITIVE)));
    fs::write(run_dir.join("roadmap_trace.md"), md)?;

    println!(
        "broad scalar explorer: generated {case_count}, selected {}, areas {}, cps {:.2}",
        candidates.len(),
        by_area.len(),
        (case_count as f64) / elapsed.max(f64::MIN_POSITIVE)
    );

    Ok(())
}
