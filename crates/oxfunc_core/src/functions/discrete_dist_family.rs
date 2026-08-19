use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, run_values_only_prepared};
use crate::functions::special_math_common::regularized_gamma_q;
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

const DISCRETE_DIST_BASE_META: FunctionMeta = function_spec! {
    function_id: "FUNC.DISCRETE_DIST_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
};

pub const BINOM_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BINOM.DIST",
    arity: Arity::exact(4),
    ..DISCRETE_DIST_BASE_META
};

pub const BINOM_DIST_RANGE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BINOM.DIST.RANGE",
    arity: Arity { min: 3, max: 4 },
    ..DISCRETE_DIST_BASE_META
};

pub const BINOM_INV_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BINOM.INV",
    arity: Arity::exact(3),
    ..DISCRETE_DIST_BASE_META
};

// The legacy compatibility surfaces (BINOMDIST/CRITBINOM/POISSON/HYPGEOMDIST/NEGBINOMDIST/
// EXPONDIST) are scalar-shaped by-index and broadcast their leading arguments over an array:
// BINOMDIST lifts its four arguments (`[0,1,2,3]`), the others their first three (`[0,1,2]`).
// The modern `.`-named surfaces lift natively and carry the default. Verified live Excel 16.0
// build 20026.
pub const BINOMDIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BINOMDIST",
    arity: Arity::exact(4),
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1, 2, 3]),
    ..DISCRETE_DIST_BASE_META
};

pub const CRITBINOM_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CRITBINOM",
    arity: Arity::exact(3),
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1, 2]),
    ..DISCRETE_DIST_BASE_META
};

pub const POISSON_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.POISSON",
    arity: Arity::exact(3),
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1, 2]),
    ..DISCRETE_DIST_BASE_META
};

pub const POISSON_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.POISSON.DIST",
    arity: Arity::exact(3),
    ..DISCRETE_DIST_BASE_META
};

pub const HYPGEOM_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.HYPGEOM.DIST",
    arity: Arity::exact(5),
    ..DISCRETE_DIST_BASE_META
};

pub const HYPGEOMDIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.HYPGEOMDIST",
    arity: Arity::exact(4),
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1, 2]),
    ..DISCRETE_DIST_BASE_META
};

pub const NEGBINOM_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NEGBINOM.DIST",
    arity: Arity::exact(4),
    ..DISCRETE_DIST_BASE_META
};

pub const NEGBINOMDIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.NEGBINOMDIST",
    arity: Arity::exact(3),
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1, 2]),
    ..DISCRETE_DIST_BASE_META
};

pub const EXPON_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.EXPON.DIST",
    arity: Arity::exact(3),
    ..DISCRETE_DIST_BASE_META
};

pub const EXPONDIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.EXPONDIST",
    arity: Arity::exact(3),
    lift_broadcast_profile: FunctionMeta::lift_at(&[0, 1, 2]),
    ..DISCRETE_DIST_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum DiscreteDistEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

fn prepared_len_error(meta: &FunctionMeta, actual: usize) -> DiscreteDistEvalError {
    DiscreteDistEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn number(prepared: &CalcValue) -> Result<f64, DiscreteDistEvalError> {
    coerce_prepared_to_number(prepared).map_err(DiscreteDistEvalError::Coercion)
}

fn cumulative_flag(value: f64) -> bool {
    value != 0.0
}

fn trunc_i64(value: f64) -> Result<i64, WorksheetErrorCode> {
    if !value.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(value.trunc() as i64)
}

fn validate_probability_closed_unit(value: f64) -> Result<(), WorksheetErrorCode> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(())
}

fn ln_choose(n: u64, k: u64) -> Result<f64, WorksheetErrorCode> {
    if k > n {
        return Err(WorksheetErrorCode::Num);
    }
    let k = k.min(n - k);
    let mut acc = 0.0;
    for i in 1..=k {
        acc += ((n - k + i) as f64).ln() - (i as f64).ln();
    }
    Ok(acc)
}

fn choose_direct(n: u64, k: u64) -> f64 {
    let k = k.min(n - k);
    let mut acc = 1.0;
    for i in 1..=k {
        acc *= (n - k + i) as f64;
        acc /= i as f64;
    }
    acc
}

fn pow_u64(base: f64, exponent: u64) -> f64 {
    base.powi(exponent as i32)
}

fn direct_combinatoric_lane(max_n: u64) -> bool {
    max_n <= 200
}

fn binom_pmf_direct(number_s: u64, trials: u64, probability_s: f64) -> f64 {
    choose_direct(trials, number_s)
        * pow_u64(probability_s, number_s)
        * pow_u64(1.0 - probability_s, trials - number_s)
}

// ===========================================================================
// W109 lane-8 BINOM.DIST(k, n, p, FALSE) pmf — the identified R `dbinom_raw`
// op-graph (agent-T rounds 1-6; landing agent-U). Excel evaluates BINOM.DIST's
// probability mass through R's `dbinom_raw`/`bd0`/`stirlerr` machinery, its
// transcendentals routed to the legacy x87 CRT chains. The chain-entry exp is
// fed the argument EXTENDED (see `excel_binom_pmf_exp`). End-to-end scores
// through this exact composition: b34 52.17%, b35 75.51%, b29 49.81% (the rest
// is a second, rarer lc-flip source + small-operand bd0-direct rounding, both
// sub-ULP residuals banked in agentT_results.md / agentU_results.md).
//
// Every constant below is the CR double emitted by `agentU_gen_consts.py`
// (bit-verified against agentT_model.py's SFE/S/M_LN_2PI). NEVER hand-edit —
// regenerate.

// ---- stirlerr CR-double table s(m), m = 1..=15 (index 0 unused) ----
// s(m) = ln(m!) - m*ln(m) + m - 0.5*ln(2*pi*m)  (mpmath prec 200)
const STIRLERR: [f64; 16] = [
    0.0,                                // m = 0 (unused; kept for 1-based indexing)
    f64::from_bits(0x3FB4C071BCDA0A5B), // s( 1)
    f64::from_bits(0x3FA52A9B923EA649), // s( 2)
    f64::from_bits(0x3F9C579A268D80B3), // s( 3)
    f64::from_bits(0x3F954A2662FD78A9), // s( 4)
    f64::from_bits(0x3F910B4E513FCBED), // s( 5)
    f64::from_bits(0x3F8C6B167BEBDF36), // s( 6)
    f64::from_bits(0x3F885D4D612E4A86), // s( 7)
    f64::from_bits(0x3F8552805E7B3076), // s( 8)
    f64::from_bits(0x3F82F4871B12AB64), // s( 9)
    f64::from_bits(0x3F810F9D4C0743A7), // s(10)
    f64::from_bits(0x3F7F0593088014F8), // s(11)
    f64::from_bits(0x3F7C7018733AA9C6), // s(12)
    f64::from_bits(0x3F7A40514700F36C), // s(13)
    f64::from_bits(0x3F786076C002D4A7), // s(14)
    f64::from_bits(0x3F76C08F6F194A10), // s(15)
];

// ---- R stirlerr large-n series coefficients (CR doubles) ----
const STIRL_S0: f64 = f64::from_bits(0x3FB5555555555555); // 1/12
const STIRL_S1: f64 = f64::from_bits(0x3F66C16C16C16C17); // 1/360
const STIRL_S2: f64 = f64::from_bits(0x3F4A01A01A01A01A); // 1/1260
const STIRL_S3: f64 = f64::from_bits(0x3F43813813813814); // 1/1680
const STIRL_S4: f64 = f64::from_bits(0x3F4B951E2B18FF23); // 1/1188

// ---- ln(2*pi) CR double ----
const M_LN_2PI: f64 = f64::from_bits(0x3FFD67F1C864BEB5); // ln(2*pi)

/// R `stirlerr(n)` — n an integer-valued f64. Table for n <= 15, else the R
/// asymptotic-series tiers (all arithmetic plain double, nn = n*n). Lane 7
/// proved this term is inert at sub-ULP scale, but faithfulness to R (= Excel)
/// requires the exact tier boundaries and coefficients above.
fn binom_stirlerr(n: f64) -> f64 {
    if n <= 15.0 {
        STIRLERR[n as usize]
    } else {
        let nn = n * n;
        if n > 500.0 {
            (STIRL_S0 - STIRL_S1 / nn) / n
        } else if n > 80.0 {
            (STIRL_S0 - (STIRL_S1 - STIRL_S2 / nn) / nn) / n
        } else if n > 35.0 {
            (STIRL_S0 - (STIRL_S1 - (STIRL_S2 - STIRL_S3 / nn) / nn) / nn) / n
        } else {
            (STIRL_S0 - (STIRL_S1 - (STIRL_S2 - (STIRL_S3 - STIRL_S4 / nn) / nn) / nn) / nn) / n
        }
    }
}

/// R `bd0(x, np)` — the deviance term. Near-equal operands take R's series loop
/// (all plain double); otherwise the DIRECT form `x*ln(x/np) + (np - x)` with a
/// SINGLE hardware ln of the plain-double quotient (agent-T round 6: bd0 keeps
/// the quotient-ln — the log-split that fixed `lf` does NOT extend to bd0).
fn binom_bd0(x: f64, np: f64) -> f64 {
    if (x - np).abs() < 0.1 * (x + np) {
        // R's convergent series in v = (x-np)/(x+np), plain double throughout.
        let v = (x - np) / (x + np);
        let mut s = (x - np) * v;
        let mut ej = 2.0 * x * v;
        let vv = v * v;
        let mut j = 1.0_f64;
        loop {
            ej *= vv;
            let s1 = s + ej / (2.0 * j + 1.0);
            if s1 == s {
                return s1;
            }
            s = s1;
            j += 1.0;
        }
    } else {
        x * crate::excel_numeric::excel_log(x / np) + (np - x)
    }
}

/// The non-degenerate BINOM.DIST pmf (0 < p < 1). Dispatches the k=0 / k=n
/// closed forms and the general `dbinom_raw` body.
fn binom_pmf(number_s: u64, trials: u64, probability_s: f64) -> f64 {
    let n = trials as f64;
    let q = 1.0 - probability_s;
    if number_s == 0 {
        if trials == 0 {
            return 1.0; // dbinom_raw n==0 empty product
        }
        // k == 0 (b29b: 383+/400). p < 0.1: bd0 form fed to the regular RN53
        // chain exp with a plain-double argument; else the raw pow chain q^n.
        return if probability_s < 0.1 {
            crate::excel_numeric::excel_exp(-binom_bd0(n, n * q) - n * probability_s)
        } else {
            crate::excel_numeric::excel_pow_chain(q, n)
        };
    }
    if number_s == trials {
        // k == n mirror.
        return if q < 0.1 {
            crate::excel_numeric::excel_exp(-binom_bd0(n, n * probability_s) - n * q)
        } else {
            crate::excel_numeric::excel_pow_chain(probability_s, n)
        };
    }
    // General 1 <= k <= n-1: dbinom_raw = exp(lc - 0.5*lf), argument extended.
    let k = number_s as f64;
    let nk = n - k;
    let np = n * probability_s;
    let nq = n * q;
    // lc: O3 grouping (agent-T round 6, 403/475 flips predicted, 0 false pos):
    //   ((s(n) - s(k)) - (s(n-k) + bd0(k,np))) - bd0(n-k,nq)
    let lc = ((binom_stirlerr(n) - binom_stirlerr(k)) - (binom_stirlerr(nk) + binom_bd0(k, np)))
        - binom_bd0(nk, nq);
    // lf: 2lnA (agent-T round 5) — log1p(-k/n) realized as a DIFFERENCE OF TWO
    //   SEPARATE hardware lns, left-to-right: (M_LN_2PI + ln k) + (ln(n-k) - ln n).
    let ln = crate::excel_numeric::excel_log;
    let lf = (M_LN_2PI + ln(k)) + (ln(nk) - ln(n));
    crate::excel_numeric::excel_binom_pmf_exp(lc, lf)
}

pub fn binom_dist_kernel(
    number_s: f64,
    trials: f64,
    probability_s: f64,
    cumulative: bool,
) -> Result<f64, WorksheetErrorCode> {
    validate_probability_closed_unit(probability_s)?;
    let number_s = trunc_i64(number_s)?;
    let trials = trunc_i64(trials)?;
    if trials < 0 || number_s < 0 || number_s > trials {
        return Err(WorksheetErrorCode::Num);
    }
    let number_s = number_s as u64;
    let trials = trials as u64;
    if cumulative {
        let mut sum = 0.0;
        for k in 0..=number_s {
            sum += binom_dist_kernel(k as f64, trials as f64, probability_s, false)?;
        }
        Ok(sum)
    } else if probability_s == 0.0 {
        Ok(if number_s == 0 { 1.0 } else { 0.0 })
    } else if probability_s == 1.0 {
        Ok(if number_s == trials { 1.0 } else { 0.0 })
    } else {
        // W109 lane 8: the identified R dbinom_raw op-graph (replaces the old
        // ln_choose log-composed pmf). cdf keeps the summation loop above.
        Ok(binom_pmf(number_s, trials, probability_s))
    }
}

pub fn binom_dist_range_kernel(
    trials: f64,
    probability_s: f64,
    number_s: f64,
    number_s2: Option<f64>,
) -> Result<f64, WorksheetErrorCode> {
    validate_probability_closed_unit(probability_s)?;
    let trials = trunc_i64(trials)?;
    let number_s = trunc_i64(number_s)?;
    let number_s2 = trunc_i64(number_s2.unwrap_or(number_s as f64))?;
    if trials < 0 || number_s < 0 || number_s2 < 0 || number_s > number_s2 || number_s2 > trials {
        return Err(WorksheetErrorCode::Num);
    }
    let trials_u = trials as u64;
    let mut sum = 0.0;
    if probability_s != 0.0 && probability_s != 1.0 && direct_combinatoric_lane(trials_u) {
        for k in number_s..=number_s2 {
            sum += binom_pmf_direct(k as u64, trials_u, probability_s);
        }
    } else {
        for k in number_s..=number_s2 {
            sum += binom_dist_kernel(k as f64, trials as f64, probability_s, false)?;
        }
    }
    Ok(sum)
}

pub fn binom_inv_kernel(
    trials: f64,
    probability_s: f64,
    alpha: f64,
) -> Result<f64, WorksheetErrorCode> {
    validate_probability_closed_unit(probability_s)?;
    validate_probability_closed_unit(alpha)?;
    let trials = trunc_i64(trials)?;
    if trials < 0 {
        return Err(WorksheetErrorCode::Num);
    }
    for x in 0..=trials {
        let cdf = binom_dist_kernel(x as f64, trials as f64, probability_s, true)?;
        if cdf >= alpha {
            return Ok(x as f64);
        }
    }
    Ok(trials as f64)
}

pub fn poisson_dist_kernel(x: f64, mean: f64, cumulative: bool) -> Result<f64, WorksheetErrorCode> {
    let x = trunc_i64(x)?;
    if x < 0 || !mean.is_finite() || mean < 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    let x = x as u64;
    if cumulative {
        // Inverse-problem identity, live Excel 16.0 b20228 (70 pairs, k in
        // {0,1,2,3,5}): POISSON.DIST(k,μ,TRUE) == CHIDIST(2μ, 2(k+1)) ==
        // CHISQ.DIST.RT(2μ, 2(k+1)) bit-exactly. Worksheet 2*μ / μ*2 / μ+μ
        // all agree (multiply-by-two is exact). A fold of published PMFs is
        // not the graph (45/70). 1-GAMMA.DIST(μ,k+1,1,TRUE) is not the
        // graph (45/70). k=0 is the identified CHIDIST(df=2) elementary
        // EXP(-μ); k≥1 is GRATIO Q(k+1, μ), i.e. CHIDIST internals after
        // the exact 2μ / 2 recovery.
        if x == 0 {
            Ok(crate::excel_numeric::excel_exp(-mean))
        } else {
            Ok(regularized_gamma_q((x + 1) as f64, mean))
        }
    } else if mean == 0.0 {
        Ok(if x == 0 { 1.0 } else { 0.0 })
    } else {
        let mut ln_fact = 0.0;
        for i in 2..=x {
            ln_fact += (i as f64).ln();
        }
        // W109: the internal exp is the x87 fFEXP chain (identified via the
        // POISSON k=0 window, 30,000/30,000). Route staging (direct product
        // vs this log composition) is a separate open lane.
        Ok(crate::excel_numeric::excel_exp(
            -(mean) + (x as f64) * mean.ln() - ln_fact,
        ))
    }
}

pub fn hypergeom_dist_kernel(
    sample_s: f64,
    number_sample: f64,
    population_s: f64,
    number_pop: f64,
    cumulative: bool,
) -> Result<f64, WorksheetErrorCode> {
    let sample_s = trunc_i64(sample_s)?;
    let number_sample = trunc_i64(number_sample)?;
    let population_s = trunc_i64(population_s)?;
    let number_pop = trunc_i64(number_pop)?;
    if sample_s < 0
        || number_sample < 0
        || population_s < 0
        || number_pop < 0
        || population_s > number_pop
        || number_sample > number_pop
        || sample_s > number_sample
        || sample_s > population_s
        || number_sample - sample_s > number_pop - population_s
    {
        return Err(WorksheetErrorCode::Num);
    }
    let sample_s = sample_s as u64;
    let number_sample = number_sample as u64;
    let population_s = population_s as u64;
    let number_pop = number_pop as u64;
    let lower = number_sample.saturating_sub(number_pop - population_s);
    if cumulative {
        let mut sum = 0.0;
        for x in lower..=sample_s {
            sum += hypergeom_dist_kernel(
                x as f64,
                number_sample as f64,
                population_s as f64,
                number_pop as f64,
                false,
            )?;
        }
        Ok(sum)
    } else if direct_combinatoric_lane(number_pop) {
        Ok(choose_direct(population_s, sample_s)
            * choose_direct(number_pop - population_s, number_sample - sample_s)
            / choose_direct(number_pop, number_sample))
    } else {
        let log_pmf = ln_choose(population_s, sample_s)?
            + ln_choose(number_pop - population_s, number_sample - sample_s)?
            - ln_choose(number_pop, number_sample)?;
        Ok(log_pmf.exp())
    }
}

pub fn negbinom_dist_kernel(
    number_f: f64,
    number_s: f64,
    probability_s: f64,
    cumulative: bool,
) -> Result<f64, WorksheetErrorCode> {
    validate_probability_closed_unit(probability_s)?;
    let number_f = trunc_i64(number_f)?;
    let number_s = trunc_i64(number_s)?;
    if number_f < 0 || number_s <= 0 {
        return Err(WorksheetErrorCode::Num);
    }
    let number_f = number_f as u64;
    let number_s = number_s as u64;
    if cumulative {
        let mut sum = 0.0;
        for failures in 0..=number_f {
            sum += negbinom_dist_kernel(failures as f64, number_s as f64, probability_s, false)?;
        }
        Ok(sum)
    } else if probability_s == 0.0 {
        Ok(0.0)
    } else if probability_s == 1.0 {
        Ok(if number_f == 0 { 1.0 } else { 0.0 })
    } else if direct_combinatoric_lane(number_f + number_s - 1) {
        Ok(choose_direct(number_f + number_s - 1, number_f)
            * pow_u64(probability_s, number_s)
            * pow_u64(1.0 - probability_s, number_f))
    } else {
        let log_pmf = ln_choose(number_f + number_s - 1, number_f)?
            + (number_s as f64) * probability_s.ln()
            + (number_f as f64) * (1.0 - probability_s).ln();
        Ok(log_pmf.exp())
    }
}

pub fn expon_dist_kernel(x: f64, lambda: f64, cumulative: bool) -> Result<f64, WorksheetErrorCode> {
    if !x.is_finite() || !lambda.is_finite() || x < 0.0 || lambda <= 0.0 {
        return Err(WorksheetErrorCode::Num);
    }
    // W109 lane-1 (b28b): the EXPON body is legacy x87 per-op double-rounded —
    // both the inner `λ·x` product (14/14) and the pdf's outer `λ·e` product
    // (24/24) are RN53(RN64(·)) spills, like WEIBULL's body.
    let lx = crate::excel_numeric::excel_x87_mul(lambda, x);
    if cumulative {
        // W109: Excel's cdf is -expm1(-lambda*x) via its Kahan-correction
        // internal expm1 (identified 17,992/18,000), NOT 1 - exp(...).
        Ok(-crate::excel_numeric::excel_expm1_internal(-lx))
    } else {
        // pdf site = the chain exp, nearest-published (bit-identical to the
        // POISSON k=0 window).
        Ok(crate::excel_numeric::excel_x87_mul(
            lambda,
            crate::excel_numeric::excel_exp(-lx),
        ))
    }
}

fn eval_binom_dist_prepared(args: &[CalcValue]) -> Result<CalcValue, DiscreteDistEvalError> {
    if !BINOM_DIST_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&BINOM_DIST_META, args.len()));
    }
    let number_s = number(&args[0])?;
    let trials = number(&args[1])?;
    let probability_s = number(&args[2])?;
    let cumulative = number(&args[3])?;
    Ok(
        match binom_dist_kernel(number_s, trials, probability_s, cumulative_flag(cumulative)) {
            Ok(value) => CalcValue::number(value),
            Err(code) => CalcValue::error(code),
        },
    )
}

fn eval_binom_dist_range_prepared(args: &[CalcValue]) -> Result<CalcValue, DiscreteDistEvalError> {
    if !BINOM_DIST_RANGE_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&BINOM_DIST_RANGE_META, args.len()));
    }
    let trials = number(&args[0])?;
    let probability_s = number(&args[1])?;
    let number_s = number(&args[2])?;
    let number_s2 = if args.len() == 4 {
        Some(number(&args[3])?)
    } else {
        None
    };
    Ok(
        match binom_dist_range_kernel(trials, probability_s, number_s, number_s2) {
            Ok(value) => CalcValue::number(value),
            Err(code) => CalcValue::error(code),
        },
    )
}

fn eval_binom_inv_prepared(args: &[CalcValue]) -> Result<CalcValue, DiscreteDistEvalError> {
    if !BINOM_INV_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&BINOM_INV_META, args.len()));
    }
    let trials = number(&args[0])?;
    let probability_s = number(&args[1])?;
    let alpha = number(&args[2])?;
    Ok(match binom_inv_kernel(trials, probability_s, alpha) {
        Ok(value) => CalcValue::number(value),
        Err(code) => CalcValue::error(code),
    })
}

fn eval_poisson_dist_prepared(args: &[CalcValue]) -> Result<CalcValue, DiscreteDistEvalError> {
    if !POISSON_DIST_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&POISSON_DIST_META, args.len()));
    }
    let x = number(&args[0])?;
    let mean = number(&args[1])?;
    let cumulative = number(&args[2])?;
    Ok(
        match poisson_dist_kernel(x, mean, cumulative_flag(cumulative)) {
            Ok(value) => CalcValue::number(value),
            Err(code) => CalcValue::error(code),
        },
    )
}

fn eval_hypgeom_dist_prepared(args: &[CalcValue]) -> Result<CalcValue, DiscreteDistEvalError> {
    if !HYPGEOM_DIST_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&HYPGEOM_DIST_META, args.len()));
    }
    let sample_s = number(&args[0])?;
    let number_sample = number(&args[1])?;
    let population_s = number(&args[2])?;
    let number_pop = number(&args[3])?;
    let cumulative = number(&args[4])?;
    Ok(
        match hypergeom_dist_kernel(
            sample_s,
            number_sample,
            population_s,
            number_pop,
            cumulative_flag(cumulative),
        ) {
            Ok(value) => CalcValue::number(value),
            Err(code) => CalcValue::error(code),
        },
    )
}

fn eval_negbinom_dist_prepared(args: &[CalcValue]) -> Result<CalcValue, DiscreteDistEvalError> {
    if !NEGBINOM_DIST_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&NEGBINOM_DIST_META, args.len()));
    }
    let number_f = number(&args[0])?;
    let number_s = number(&args[1])?;
    let probability_s = number(&args[2])?;
    let cumulative = number(&args[3])?;
    Ok(
        match negbinom_dist_kernel(
            number_f,
            number_s,
            probability_s,
            cumulative_flag(cumulative),
        ) {
            Ok(value) => CalcValue::number(value),
            Err(code) => CalcValue::error(code),
        },
    )
}

fn eval_expon_dist_prepared(args: &[CalcValue]) -> Result<CalcValue, DiscreteDistEvalError> {
    if !EXPON_DIST_META.arity.accepts(args.len()) {
        return Err(prepared_len_error(&EXPON_DIST_META, args.len()));
    }
    let x = number(&args[0])?;
    let lambda = number(&args[1])?;
    let cumulative = number(&args[2])?;
    Ok(
        match expon_dist_kernel(x, lambda, cumulative_flag(cumulative)) {
            Ok(value) => CalcValue::number(value),
            Err(code) => CalcValue::error(code),
        },
    )
}

pub fn eval_binom_dist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_binom_dist_prepared,
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_binom_dist_range_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_binom_dist_range_prepared,
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_binom_inv_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_binom_inv_prepared,
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_binomdist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DiscreteDistEvalError> {
    eval_binom_dist_surface(args, resolver)
}

pub fn eval_critbinom_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DiscreteDistEvalError> {
    eval_binom_inv_surface(args, resolver)
}

pub fn eval_poisson_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_poisson_dist_prepared,
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_poisson_dist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DiscreteDistEvalError> {
    eval_poisson_surface(args, resolver)
}

pub fn eval_hypgeom_dist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_hypgeom_dist_prepared,
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_hypgeomdist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !HYPGEOMDIST_META.arity.accepts(prepared.len()) {
                return Err(prepared_len_error(&HYPGEOMDIST_META, prepared.len()));
            }
            let sample_s = number(&prepared[0])?;
            let number_sample = number(&prepared[1])?;
            let population_s = number(&prepared[2])?;
            let number_pop = number(&prepared[3])?;
            Ok(
                match hypergeom_dist_kernel(
                    sample_s,
                    number_sample,
                    population_s,
                    number_pop,
                    false,
                ) {
                    Ok(value) => CalcValue::number(value),
                    Err(code) => CalcValue::error(code),
                },
            )
        },
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_negbinom_dist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_negbinom_dist_prepared,
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_negbinomdist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            if !NEGBINOMDIST_META.arity.accepts(prepared.len()) {
                return Err(prepared_len_error(&NEGBINOMDIST_META, prepared.len()));
            }
            let number_f = number(&prepared[0])?;
            let number_s = number(&prepared[1])?;
            let probability_s = number(&prepared[2])?;
            Ok(
                match negbinom_dist_kernel(number_f, number_s, probability_s, false) {
                    Ok(value) => CalcValue::number(value),
                    Err(code) => CalcValue::error(code),
                },
            )
        },
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_expon_dist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DiscreteDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_expon_dist_prepared,
        DiscreteDistEvalError::Coercion,
    )
}

pub fn eval_expondist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, DiscreteDistEvalError> {
    eval_expon_dist_surface(args, resolver)
}

pub fn map_discrete_dist_error_to_ws(err: &DiscreteDistEvalError) -> WorksheetErrorCode {
    match err {
        DiscreteDistEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        DiscreteDistEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        DiscreteDistEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::{CoreValue, ExcelText};
    use std::collections::HashMap;

    struct MockResolver {
        resolved_values: HashMap<String, CalcValue>,
    }

    impl MockResolver {
        fn empty() -> Self {
            Self {
                resolved_values: HashMap::new(),
            }
        }
    }

    impl ReferenceSystemProvider for MockResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            self.resolved_values.get(reference.target()).cloned().ok_or(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    fn num(n: f64) -> CalcValue {
        CalcValue::number(n)
    }

    fn text(s: &str) -> CalcValue {
        CalcValue::text(ExcelText::from_interop_assignment(s))
    }

    fn bool_arg(b: bool) -> CalcValue {
        CalcValue::logical(b)
    }

    fn assert_ok_number_close(got: Result<CalcValue, DiscreteDistEvalError>, expected: f64) {
        match got {
            Ok(value) => match value.core() {
                CoreValue::Number(value) => assert!((*value - expected).abs() < 1e-12),
                other => panic!("expected numeric result, got {other:?}"),
            },
            other => panic!("expected numeric result, got {other:?}"),
        }
    }

    fn assert_bits(actual: f64, expected_bits: u64) {
        assert_eq!(
            actual.to_bits(),
            expected_bits,
            "{actual} vs {}",
            f64::from_bits(expected_bits)
        );
    }

    #[test]
    fn binom_family_matches_seed_lanes() {
        assert!((binom_dist_kernel(2.0, 4.0, 0.25, false).unwrap() - 0.2109375).abs() < 1e-12);
        assert!((binom_dist_kernel(2.0, 4.0, 0.25, true).unwrap() - 0.94921875).abs() < 1e-12);
        assert!(
            (binom_dist_range_kernel(4.0, 0.25, 2.0, Some(3.0)).unwrap() - 0.2578125).abs() < 1e-12
        );
        assert_eq!(binom_inv_kernel(6.0, 0.5, 0.7).unwrap(), 4.0);
        assert_eq!(binom_inv_kernel(6.0, 0.5, 0.0).unwrap(), 0.0);
    }

    #[test]
    fn finite_combinatoric_witnesses_match_excel_bits() {
        assert_bits(
            binom_dist_range_kernel(4.0, 0.25, 2.0, Some(3.0)).unwrap(),
            0x3fd0_8000_0000_0000,
        );
        assert_bits(
            negbinom_dist_kernel(5.0, 3.0, 0.4, true).unwrap(),
            0x3fe5_e849_aaee_d68d,
        );
    }

    #[test]
    fn poisson_family_matches_seed_lanes() {
        assert!((poisson_dist_kernel(3.0, 2.0, false).unwrap() - 0.1804470443154836).abs() < 1e-12);
        assert!((poisson_dist_kernel(3.0, 2.0, true).unwrap() - 0.857123460498547).abs() < 1e-12);
        assert_eq!(poisson_dist_kernel(0.0, 0.0, false).unwrap(), 1.0);
    }

    #[test]
    fn poisson_cdf_matches_chidist_even_df_identity() {
        for &(k, mu) in &[
            (0.0, 0.0),
            (0.0, 0.5),
            (0.0, 2.0),
            (1.0, 0.25),
            (1.0, 2.0),
            (2.0, 3.0),
            (3.0, 2.0),
            (5.0, 8.0),
        ] {
            let cdf = poisson_dist_kernel(k, mu, true).unwrap();
            let chi = crate::functions::chi_f_t_family::chisq_dist_rt_kernel(
                2.0 * mu,
                2.0 * (k + 1.0),
            )
            .unwrap();
            assert_eq!(cdf.to_bits(), chi.to_bits(), "k={k} mu={mu}");
        }
        // k=0 is the identified EXP(-μ) elementary, not a GRATIO a=1 wrapper.
        assert_eq!(
            poisson_dist_kernel(0.0, 1.5, true).unwrap().to_bits(),
            crate::excel_numeric::excel_exp(-1.5).to_bits()
        );
    }

    #[test]
    fn hypergeom_family_matches_seed_lanes() {
        assert!(
            (hypergeom_dist_kernel(2.0, 5.0, 4.0, 10.0, false).unwrap() - 0.47619047619047616)
                .abs()
                < 1e-12
        );
        assert!(
            (hypergeom_dist_kernel(2.0, 5.0, 4.0, 10.0, true).unwrap() - 0.7380952380952381).abs()
                < 1e-12
        );
    }

    #[test]
    fn negbinom_family_matches_seed_lanes() {
        assert!((negbinom_dist_kernel(3.0, 2.0, 0.5, false).unwrap() - 0.125).abs() < 1e-12);
        assert!((negbinom_dist_kernel(3.0, 2.0, 0.5, true).unwrap() - 0.8125).abs() < 1e-12);
        assert_eq!(negbinom_dist_kernel(0.0, 2.0, 1.0, false).unwrap(), 1.0);
    }

    #[test]
    fn expon_family_matches_seed_lanes() {
        assert!((expon_dist_kernel(2.0, 1.5, false).unwrap() - 0.07468060255179591).abs() < 1e-12);
        assert!((expon_dist_kernel(2.0, 1.5, true).unwrap() - 0.950212931632136).abs() < 1e-12);
    }

    #[test]
    fn domain_errors_match_seed_expectations() {
        assert_eq!(
            binom_dist_kernel(5.0, 4.0, 0.25, false),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            binom_dist_range_kernel(4.0, 0.25, 3.0, Some(2.0)),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            poisson_dist_kernel(-1.0, 2.0, false),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            hypergeom_dist_kernel(4.0, 5.0, 2.0, 10.0, false),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            negbinom_dist_kernel(1.0, 0.0, 0.5, false),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            expon_dist_kernel(-1.0, 1.5, false),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn compatibility_alias_surfaces_follow_modern_kernels() {
        let resolver = MockResolver::empty();
        assert_ok_number_close(
            eval_binomdist_surface(&[num(2.0), num(4.0), num(0.25), bool_arg(false)], &resolver),
            0.2109375,
        );
        assert_eq!(
            eval_critbinom_surface(&[num(6.0), num(0.5), num(0.7)], &resolver),
            Ok(CalcValue::number(4.0))
        );
        assert_ok_number_close(
            eval_poisson_surface(&[num(3.0), num(2.0), bool_arg(false)], &resolver),
            0.1804470443154836,
        );
        assert_ok_number_close(
            eval_hypgeomdist_surface(&[num(2.0), num(5.0), num(4.0), num(10.0)], &resolver),
            0.47619047619047616,
        );
        assert_ok_number_close(
            eval_negbinomdist_surface(&[num(3.0), num(2.0), num(0.5)], &resolver),
            0.125,
        );
        assert_ok_number_close(
            eval_expondist_surface(&[num(2.0), num(1.5), bool_arg(true)], &resolver),
            0.950212931632136,
        );
        assert_ok_number_close(
            eval_hypgeom_dist_surface(
                &[num(2.0), num(5.0), num(4.0), num(10.0), bool_arg(true)],
                &resolver,
            ),
            0.7380952380952381,
        );
        assert_ok_number_close(
            eval_negbinom_dist_surface(&[num(3.0), num(2.0), num(0.5), bool_arg(true)], &resolver),
            0.8125,
        );
    }

    #[test]
    fn values_only_surface_coercion_admits_numeric_text_and_logical_flags() {
        let resolver = MockResolver::empty();
        assert_eq!(
            eval_binom_inv_surface(&[text("6"), num(0.5), text("0.7")], &resolver),
            Ok(CalcValue::number(4.0))
        );
        assert_ok_number_close(
            eval_poisson_dist_surface(&[num(3.0), text("2"), bool_arg(true)], &resolver),
            0.857123460498547,
        );
        assert_ok_number_close(
            eval_expon_dist_surface(&[num(2.0), text("1.5"), bool_arg(false)], &resolver),
            0.07468060255179591,
        );
    }

    #[test]
    fn metadata_shapes_and_error_mapping_are_exercised() {
        assert!(BINOMDIST_META.arity.accepts(4));
        assert!(CRITBINOM_META.arity.accepts(3));
        assert!(POISSON_META.arity.accepts(3));
        assert!(EXPONDIST_META.arity.accepts(3));
        assert_eq!(
            map_discrete_dist_error_to_ws(&DiscreteDistEvalError::ArityMismatch {
                expected_min: 3,
                expected_max: 4,
                actual: 2,
            }),
            WorksheetErrorCode::Value,
        );
    }

    #[test]
    fn missing_optional_upper_bound_defaults_to_exact_binom_mass() {
        let resolver = MockResolver::empty();
        assert_ok_number_close(
            eval_binom_dist_range_surface(&[num(4.0), num(0.25), num(2.0)], &resolver),
            0.2109375,
        );
    }
}
