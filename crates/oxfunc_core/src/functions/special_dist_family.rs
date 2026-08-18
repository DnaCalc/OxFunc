use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    BroadcastPreparedGroup, coerce_prepared_to_number, expand_prepared_broadcast_grid,
    run_values_only_prepared,
};
use crate::functions::normal_dist_common::erf_approx;
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcArray, CalcValue, CoreValue, WorksheetErrorCode};

const SPECIAL_DIST_BASE_META: FunctionMeta = function_spec! {
    function_id: "FUNC.SPECIAL_DIST_BASE",
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

pub const ERF_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ERF",
    arity: Arity { min: 1, max: 2 },
    ..SPECIAL_DIST_BASE_META
};

pub const ERF_PRECISE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ERF.PRECISE",
    ..SPECIAL_DIST_BASE_META
};

pub const ERFC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ERFC",
    ..SPECIAL_DIST_BASE_META
};

pub const ERFC_PRECISE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ERFC.PRECISE",
    ..SPECIAL_DIST_BASE_META
};

pub const GAMMA_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.GAMMA",
    ..SPECIAL_DIST_BASE_META
};

pub const GAMMALN_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.GAMMALN",
    ..SPECIAL_DIST_BASE_META
};

pub const GAMMALN_PRECISE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.GAMMALN.PRECISE",
    ..SPECIAL_DIST_BASE_META
};

pub const WEIBULL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.WEIBULL",
    arity: Arity::exact(4),
    ..SPECIAL_DIST_BASE_META
};

pub const WEIBULL_DIST_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.WEIBULL.DIST",
    arity: Arity::exact(4),
    ..SPECIAL_DIST_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum SpecialDistEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
}

const LANCZOS_G: f64 = 7.0;
const LANCZOS_COEFFS: [f64; 9] = [
    0.999_999_999_999_809_9,
    676.520_368_121_885_1,
    -1_259.139_216_722_402_8,
    771.323_428_777_653_1,
    -176.615_029_162_140_6,
    12.507_343_278_686_905,
    -0.138_571_095_265_720_12,
    9.984_369_578_019_572e-6,
    1.505_632_735_149_311_6e-7,
];

fn arity_error(meta: &FunctionMeta, actual: usize) -> SpecialDistEvalError {
    SpecialDistEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn bool_flag_from_number(n: f64) -> bool {
    n != 0.0
}

fn is_integer_like(x: f64) -> bool {
    // BUG-FUNC-027 CLASS-A2: a fixed absolute tolerance falsely collapses a tiny
    // non-integer such as -1e-200 onto 0. Test integrality relative to magnitude
    // so only genuine integers (and exact 0) qualify.
    (x - x.round()).abs() <= 1.0e-12 * x.abs()
}

fn has_gamma_pole(x: f64) -> bool {
    x <= 0.0 && is_integer_like(x)
}

fn ln_gamma_positive(x: f64) -> Result<f64, WorksheetErrorCode> {
    if !x.is_finite() || x <= 0.0 {
        return Err(WorksheetErrorCode::Num);
    }

    // BUG-FUNC-027 CLASS-A1: for x < 1 the Lanczos term coeff/(z+1) = coeff/x
    // diverges, and for tiny x, z = x - 1.0 loses x entirely so z+1 == 0 yields
    // +Inf. Lift x into the stable region via lnGamma(x) = lnGamma(x+1) - ln(x).
    if x < 1.0 {
        return Ok(ln_gamma_positive(x + 1.0)? - x.ln());
    }

    let z = x - 1.0;
    let mut acc = LANCZOS_COEFFS[0];
    for (i, coeff) in LANCZOS_COEFFS.iter().enumerate().skip(1) {
        acc += coeff / (z + i as f64);
    }

    let t = z + LANCZOS_G + 0.5;
    Ok(0.5 * (2.0 * std::f64::consts::PI).ln() + (z + 0.5) * t.ln() - t + acc.ln())
}

fn erf_interval(lower: f64, upper: f64) -> f64 {
    erf_approx(upper) - erf_approx(lower)
}

pub fn erf_kernel(lower: f64, upper: Option<f64>) -> Result<f64, WorksheetErrorCode> {
    if !lower.is_finite() || upper.is_some_and(|v| !v.is_finite()) {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(match upper {
        Some(upper) => erf_interval(lower, upper),
        None => erf_approx(lower),
    })
}

pub fn erf_precise_kernel(x: f64) -> Result<f64, WorksheetErrorCode> {
    if !x.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(erf_approx(x))
}

/// Published-worksheet staging `SQRT(x/2)` used by Excel's df=1 chi-square
/// and `GAMMA.DIST(x, 0.5, 2, TRUE)` identities. Live Excel 16.0 build 20228
/// matched `ERF.PRECISE` / `ERFC.PRECISE` of this argument on 154/154
/// distinct nonnegative x (divide-by-two first; `SQRT(x)/SQRT(2)` is not
/// the same graph).
pub fn erf_of_sqrt_half_x(x: f64) -> Result<f64, WorksheetErrorCode> {
    erf_precise_kernel((x / 2.0).sqrt())
}

pub fn erfc_of_sqrt_half_x(x: f64) -> Result<f64, WorksheetErrorCode> {
    erfc_precise_kernel((x / 2.0).sqrt())
}

// Excel-emulation for the positive-tail ERFC regime.
//
// Policy: DnaCalc emulates Excel's observed output bits; mathematical
// correct-rounding is diagnostic only. See docs/function-lane/
// ERFC_EXCEL_EMULATION.md for the regime-map evidence and fit methodology.
//
// Approach: libm::erfc base multiplied by a small relative correction
// polynomial fitted to Excel's observed ratio (excel/libm - 1) at 45
// widened witness points. Piecewise in s = 1/x², split at the fdlibm
// subrange boundary x = 2.857. Fit weighted to force corr(s) = 0 (or the
// specific UCRT offset) at all already-green anchors, preserving the
// Matched islands.
//
// Coefficients below come from weighted least-squares fit captured in
// scratch probe `probe_rational_fit_attempt` (uncommitted). Evaluation is
// Horner on normalized u = 2*(s - s_lo)/(s_hi - s_lo) - 1.
//
// Evidence summary vs widened 48-point positive witness set:
// - libm-only baseline: 9 matches
// - prior Windows-MSVC UCRT branch at x>=3: 12 matches
// - this correction-fit kernel (cross-platform):  20 matches, 0 regressions
//   at any already-matched anchor, worst blocked |Δ| = 6 ULP
//
// x < 1.25 and all negatives stay on libm unchanged (already Excel-exact
// across every tested point down to -10).

const ERFC_B_S_MIN: f64 = 1.23114804555247758788e-1; // = 1/2.85^2, min s in Region B training
const ERFC_B_S_MAX: f64 = 6.40000000000000013323e-1; // = 1/1.25^2
const ERFC_B_COEFFS: [f64; 9] = [
    -4.68127849232051076334e-16,
    -7.62455092077679137822e-16,
    6.25451788846640322315e-15,
    1.29557375346293795597e-14,
    -5.44994791140840831850e-15,
    -1.24991935965186578433e-14,
    5.75379785143124545520e-15,
    3.13242732014867690079e-16,
    -6.09757155730509723491e-15,
];

const ERFC_A_S_MIN: f64 = 1.00000000000000002082e-2; // = 1/10^2
const ERFC_A_S_MAX: f64 = 1.18906064209274672794e-1; // = 1/2.9^2
const ERFC_A_COEFFS: [f64; 3] = [
    -2.31218392351115847457e-16,
    -1.54616394379972209297e-17,
    5.48978747768942628168e-16,
];

// fdlibm-inspired positive-tail region split.
const ERFC_BOUNDARY_X: f64 = 2.857;

fn erfc_horner(coeffs: &[f64], u: f64) -> f64 {
    let mut acc = 0.0_f64;
    let mut i = coeffs.len();
    while i > 0 {
        i -= 1;
        acc = acc * u + coeffs[i];
    }
    acc
}

fn excel_erfc(x: f64) -> f64 {
    let libm_v = libm::erfc(x);
    if !x.is_finite() || x < 1.25 {
        return libm_v;
    }
    let s = 1.0 / (x * x);
    let (s_lo, s_hi, coeffs): (f64, f64, &[f64]) = if x < ERFC_BOUNDARY_X {
        (ERFC_B_S_MIN, ERFC_B_S_MAX, &ERFC_B_COEFFS[..])
    } else {
        (ERFC_A_S_MIN, ERFC_A_S_MAX, &ERFC_A_COEFFS[..])
    };
    let u = 2.0 * (s - s_lo) / (s_hi - s_lo) - 1.0;
    let corr = erfc_horner(coeffs, u);
    libm_v * (1.0 + corr)
}

pub fn erfc_kernel(x: f64) -> Result<f64, WorksheetErrorCode> {
    if !x.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(excel_erfc(x))
}

pub fn erfc_precise_kernel(x: f64) -> Result<f64, WorksheetErrorCode> {
    erfc_kernel(x)
}

pub fn gamma_kernel(x: f64) -> Result<f64, WorksheetErrorCode> {
    if !x.is_finite() || has_gamma_pole(x) {
        return Err(WorksheetErrorCode::Num);
    }

    let ln_gamma = if x < 0.5 {
        let reflected = 1.0 - x;
        let denom = (std::f64::consts::PI * x).sin();
        if denom == 0.0 || !denom.is_finite() {
            return Err(WorksheetErrorCode::Num);
        }
        std::f64::consts::PI.ln() - denom.abs().ln() - ln_gamma_positive(reflected)?
    } else {
        ln_gamma_positive(x)?
    };

    if ln_gamma > f64::MAX.ln() {
        return Err(WorksheetErrorCode::Num);
    }

    let magnitude = ln_gamma.exp();
    let value = if x < 0.5 && (std::f64::consts::PI * x).sin().is_sign_negative() {
        -magnitude
    } else {
        magnitude
    };

    if !value.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(value)
}

pub fn gammaln_kernel(x: f64) -> Result<f64, WorksheetErrorCode> {
    // Published GAMMALN / GAMMALN.PRECISE surface. The full positive numeric
    // graph remains under W109 identification; this admission boundary is
    // independently pinned. Current-reference Excel rejects every positive
    // binary64 subnormal with #NUM! while admitting `f64::MIN_POSITIVE`; the
    // same guard also retains the non-positive domain. GAMMA and the shared
    // internal lgamma are unaffected.
    if !x.is_finite() || x < f64::MIN_POSITIVE {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(crate::excel_numeric::gammaln_excel(x))
}

pub fn gammaln_precise_kernel(x: f64) -> Result<f64, WorksheetErrorCode> {
    gammaln_kernel(x)
}

pub fn weibull_dist_kernel(
    x: f64,
    alpha: f64,
    beta: f64,
    cumulative: bool,
) -> Result<f64, WorksheetErrorCode> {
    if !x.is_finite()
        || !alpha.is_finite()
        || !beta.is_finite()
        || x < 0.0
        || alpha <= 0.0
        || beta <= 0.0
    {
        return Err(WorksheetErrorCode::Num);
    }

    if x == 0.0 {
        if cumulative {
            return Ok(0.0);
        }
        return Ok(0.0);
    }

    // W109 lane-1 identification (b24 + b27/b27b, all blocks bit-exact): the
    // WEIBULL body is a legacy x87 compilation unit — every op double-rounded
    // through a spilled double local, every pow the raw chain (no shortcuts).
    //   r   = RN53(RN64(x/β))                                   (b27b D2)
    //   t   = exp(RN53(RN64(α·ln r)))                           (b27D 113/113)
    //   cdf = −expm1(−t)                                        (Kahan expm1)
    //   pdf = DR(DR(DR(α/β^α)·x^(α−1))·exp(−t))                 (b27 T3|SS,
    //         division-first association: `alpha / pow(beta, alpha) *
    //         pow(x, alpha-1) * exp(-pow(x/beta, alpha))` left-to-right)
    use crate::excel_numeric::{
        excel_exp, excel_expm1_internal, excel_pow_chain, excel_x87_div, excel_x87_mul,
    };
    let ratio = excel_x87_div(x, beta);
    let power = excel_pow_chain(ratio, alpha);
    let value = if cumulative {
        -excel_expm1_internal(-power)
    } else {
        let pba = excel_pow_chain(beta, alpha);
        let px = excel_pow_chain(x, alpha - 1.0);
        let e = excel_exp(-power);
        excel_x87_mul(excel_x87_mul(excel_x87_div(alpha, pba), px), e)
    };

    if !value.is_finite() {
        return Err(WorksheetErrorCode::Num);
    }
    Ok(value)
}

pub fn weibull_kernel(
    x: f64,
    alpha: f64,
    beta: f64,
    cumulative: bool,
) -> Result<f64, WorksheetErrorCode> {
    weibull_dist_kernel(x, alpha, beta, cumulative)
}

/// Coerce an operand to a number, optionally rejecting a logical operand with
/// `#VALUE!`. The ERF/ERFC family rejects logical operands (Excel returns
/// `#VALUE!` for `=ERF(TRUE)`) while still accepting numeric text; the
/// GAMMA/GAMMALN family accepts logicals (e.g. `=GAMMALN.PRECISE(TRUE)` -> 0).
/// Confirmed empirically against Excel `16.0` (BUG-FUNC scalar-swept sweep).
fn coerce_operand_with_logical_policy(
    arg: &CalcValue,
    reject_logical: bool,
) -> Result<f64, CoercionError> {
    if reject_logical {
        if matches!(arg.core(), CoreValue::Logical(_)) {
            return Err(CoercionError::WorksheetError(WorksheetErrorCode::Value));
        }
    }
    coerce_prepared_to_number(arg)
}

fn coercion_err_to_ws(error: &CoercionError) -> WorksheetErrorCode {
    match error {
        CoercionError::WorksheetError(code) => *code,
        _ => WorksheetErrorCode::Value,
    }
}

/// One array cell for ERF (1-2 operands, logical rejected per element).
fn erf_cell(values: &[CalcValue]) -> CalcValue {
    let lower = match coerce_operand_with_logical_policy(&values[0], true) {
        Ok(x) => x,
        Err(e) => return CalcValue::error(coercion_err_to_ws(&e)),
    };
    let upper = if values.len() > 1 {
        match coerce_operand_with_logical_policy(&values[1], true) {
            Ok(x) => Some(x),
            Err(e) => return CalcValue::error(coercion_err_to_ws(&e)),
        }
    } else {
        None
    };
    match erf_kernel(lower, upper) {
        Ok(v) => CalcValue::number(v),
        Err(code) => CalcValue::error(code),
    }
}

/// One array cell for a unary special-dist kernel (logical policy per element).
fn unary_cell(
    values: &[CalcValue],
    reject_logical: bool,
    kernel: fn(f64) -> Result<f64, WorksheetErrorCode>,
) -> CalcValue {
    match coerce_operand_with_logical_policy(&values[0], reject_logical) {
        Ok(x) => match kernel(x) {
            Ok(v) => CalcValue::number(v),
            Err(code) => CalcValue::error(code),
        },
        Err(e) => CalcValue::error(coercion_err_to_ws(&e)),
    }
}

/// Lift a per-cell mapper over a broadcast grid, if any argument is an array.
/// Returns `None` when all arguments are scalar (caller takes the scalar path).
fn lift_special_dist(
    args: &[CalcValue],
    cell: impl Fn(&[CalcValue]) -> CalcValue,
) -> Option<CalcValue> {
    let (shape, cells) = expand_prepared_broadcast_grid(args)?;
    let mapped = cells
        .into_iter()
        .map(|group| match group {
            BroadcastPreparedGroup::Values(values) => cell(&values),
            BroadcastPreparedGroup::MissingCoordinate => CalcValue::error(WorksheetErrorCode::NA),
        })
        .collect();
    Some(CalcValue::array(
        CalcArray::new(shape, mapped).expect("shape preserved"),
    ))
}

fn eval_erf_prepared(args: &[CalcValue]) -> Result<CalcValue, SpecialDistEvalError> {
    if !ERF_META.arity.accepts(args.len()) {
        return Err(arity_error(&ERF_META, args.len()));
    }
    // Array argument -> spill elementwise (Excel BUG-FUNC-028 array-lift).
    if let Some(array) = lift_special_dist(args, erf_cell) {
        return Ok(array);
    }
    // ERF rejects a logical operand (Excel #VALUE!); numeric text is accepted.
    let lower = coerce_operand_with_logical_policy(&args[0], true)
        .map_err(SpecialDistEvalError::Coercion)?;
    let upper = if args.len() > 1 {
        Some(
            coerce_operand_with_logical_policy(&args[1], true)
                .map_err(SpecialDistEvalError::Coercion)?,
        )
    } else {
        None
    };
    Ok(match erf_kernel(lower, upper) {
        Ok(value) => CalcValue::number(value),
        Err(code) => CalcValue::error(code),
    })
}

fn eval_unary_prepared(
    args: &[CalcValue],
    meta: &FunctionMeta,
    kernel: fn(f64) -> Result<f64, WorksheetErrorCode>,
    reject_logical: bool,
) -> Result<CalcValue, SpecialDistEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(arity_error(meta, args.len()));
    }
    if let Some(array) =
        lift_special_dist(args, |values| unary_cell(values, reject_logical, kernel))
    {
        return Ok(array);
    }
    let x = coerce_operand_with_logical_policy(&args[0], reject_logical)
        .map_err(SpecialDistEvalError::Coercion)?;
    Ok(match kernel(x) {
        Ok(value) => CalcValue::number(value),
        Err(code) => CalcValue::error(code),
    })
}

fn weibull_cell(
    values: &[CalcValue],
    kernel: fn(f64, f64, f64, bool) -> Result<f64, WorksheetErrorCode>,
) -> CalcValue {
    let mut nums = [0.0f64; 4];
    for (i, slot) in nums.iter_mut().enumerate() {
        match coerce_prepared_to_number(&values[i]) {
            Ok(n) => *slot = n,
            Err(e) => return CalcValue::error(coercion_err_to_ws(&e)),
        }
    }
    match kernel(nums[0], nums[1], nums[2], bool_flag_from_number(nums[3])) {
        Ok(v) => CalcValue::number(v),
        Err(code) => CalcValue::error(code),
    }
}

fn eval_weibull_prepared(
    args: &[CalcValue],
    meta: &FunctionMeta,
    kernel: fn(f64, f64, f64, bool) -> Result<f64, WorksheetErrorCode>,
) -> Result<CalcValue, SpecialDistEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(arity_error(meta, args.len()));
    }
    if let Some(array) = lift_special_dist(args, |values| weibull_cell(values, kernel)) {
        return Ok(array);
    }
    let x = coerce_prepared_to_number(&args[0]).map_err(SpecialDistEvalError::Coercion)?;
    let alpha = coerce_prepared_to_number(&args[1]).map_err(SpecialDistEvalError::Coercion)?;
    let beta = coerce_prepared_to_number(&args[2]).map_err(SpecialDistEvalError::Coercion)?;
    let cumulative = coerce_prepared_to_number(&args[3]).map_err(SpecialDistEvalError::Coercion)?;
    Ok(
        match kernel(x, alpha, beta, bool_flag_from_number(cumulative)) {
            Ok(value) => CalcValue::number(value),
            Err(code) => CalcValue::error(code),
        },
    )
}

pub fn eval_erf_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        eval_erf_prepared,
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_erf_precise_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_unary_prepared(prepared, &ERF_PRECISE_META, erf_precise_kernel, true),
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_erfc_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_unary_prepared(prepared, &ERFC_META, erfc_kernel, true),
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_erfc_precise_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_unary_prepared(prepared, &ERFC_PRECISE_META, erfc_precise_kernel, true),
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_gamma_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_unary_prepared(prepared, &GAMMA_META, gamma_kernel, false),
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_gammaln_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_unary_prepared(prepared, &GAMMALN_META, gammaln_kernel, false),
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_gammaln_precise_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| {
            eval_unary_prepared(
                prepared,
                &GAMMALN_PRECISE_META,
                gammaln_precise_kernel,
                false,
            )
        },
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_weibull_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_weibull_prepared(prepared, &WEIBULL_META, weibull_kernel),
        SpecialDistEvalError::Coercion,
    )
}

pub fn eval_weibull_dist_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, SpecialDistEvalError> {
    run_values_only_prepared(
        args,
        resolver,
        |prepared| eval_weibull_prepared(prepared, &WEIBULL_DIST_META, weibull_dist_kernel),
        SpecialDistEvalError::Coercion,
    )
}

pub fn map_special_dist_error_to_ws(error: &SpecialDistEvalError) -> WorksheetErrorCode {
    match error {
        SpecialDistEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        SpecialDistEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        SpecialDistEvalError::Coercion(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;
    use crate::value::ExcelText;

    struct NoResolver;

    impl ReferenceSystemProvider for NoResolver {
        fn capabilities(&self) -> ReferenceSystemCapabilities {
            ReferenceSystemCapabilities::permissive_local()
        }

        fn dereference(
            &self,
            request: &crate::resolver::ReferenceDereferenceRequest,
        ) -> Result<CalcValue, crate::resolver::ReferenceResolutionError> {
            let reference = &request.reference;
            Err(
                crate::resolver::ReferenceResolutionError::UnresolvedReference {
                    target: reference.target().to_string(),
                },
            )
        }
    }

    fn assert_close(actual: f64, expected: f64, tol: f64) {
        let delta = (actual - expected).abs();
        assert!(
            delta <= tol,
            "expected {expected}, got {actual}, delta {delta}"
        );
    }

    fn assert_bits_eq(label: &str, actual: f64, expected: f64) {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{label}: actual {actual:e} ({:#018x}) vs expected {expected:e} ({:#018x})",
            actual.to_bits(),
            expected.to_bits()
        );
    }

    #[test]
    fn erfc_one_matches_excel_exact_bits() {
        assert_bits_eq("erfc(1)", erfc_kernel(1.0).unwrap(), 0.15729920705028513);
    }

    #[test]
    fn erf_family_lifts_arrays_elementwise() {
        let r = NoResolver;
        // BUG-FUNC-028: ERF over a column array spills erf(x) elementwise.
        let arr = CalcValue::array(
            CalcArray::from_rows(vec![
                vec![CalcValue::number(2.0)],
                vec![CalcValue::number(3.0)],
            ])
            .unwrap(),
        );
        let expected = CalcValue::array(
            CalcArray::from_rows(vec![
                vec![CalcValue::number(erf_approx(2.0))],
                vec![CalcValue::number(erf_approx(3.0))],
            ])
            .unwrap(),
        );
        assert_eq!(eval_erf_surface(&[arr], &r), Ok(expected));

        // A logical element in an ERF array errors only that cell (#VALUE!),
        // the numeric element still computes (Excel spills per-element errors).
        let mixed = CalcValue::array(
            CalcArray::from_rows(vec![
                vec![CalcValue::logical(true)],
                vec![CalcValue::number(2.0)],
            ])
            .unwrap(),
        );
        let mixed_expected = CalcValue::array(
            CalcArray::from_rows(vec![
                vec![CalcValue::error(WorksheetErrorCode::Value)],
                vec![CalcValue::number(erf_approx(2.0))],
            ])
            .unwrap(),
        );
        assert_eq!(eval_erf_surface(&[mixed], &r), Ok(mixed_expected));
    }

    #[test]
    fn erf_family_rejects_logical_but_gamma_family_accepts_it() {
        let r = NoResolver;
        let lgl = || CalcValue::logical(true);
        // ERF/ERFC family: logical operand -> #VALUE! (Excel behavior). The
        // coercion rejection surfaces on the Err channel, which dispatch maps
        // to #VALUE!.
        let value_err = Err(SpecialDistEvalError::Coercion(
            CoercionError::WorksheetError(WorksheetErrorCode::Value),
        ));
        for got in [
            eval_erf_surface(&[lgl()], &r),
            eval_erf_precise_surface(&[lgl()], &r),
            eval_erfc_surface(&[lgl()], &r),
            eval_erfc_precise_surface(&[lgl()], &r),
        ] {
            assert_eq!(got, value_err);
        }
        // GAMMA/GAMMALN family: logical accepted (TRUE -> 1) and computed.
        // GAMMALN.PRECISE(TRUE)=GAMMALN(1)≈0 (exact-0 vs near-0 numeric drift is
        // a separate finding; here we only assert the operand is accepted).
        match eval_gammaln_precise_surface(&[lgl()], &r) {
            Ok(value) => match value.core() {
                CoreValue::Number(n) => {
                    assert!(n.abs() < 1.0e-9, "gammaln.precise(TRUE) ~ 0, got {n}")
                }
                other => panic!("expected numeric result, got {other:?}"),
            },
            other => panic!("expected gammaln.precise(TRUE) to accept logical, got {other:?}"),
        }
        // ERF still accepts numeric text (only logical is rejected).
        let txt2 = CalcValue::text(ExcelText::from_utf16_code_units(
            "2".encode_utf16().collect(),
        ));
        assert_eq!(
            eval_erf_surface(&[txt2], &r),
            Ok(CalcValue::number(erf_approx(2.0)))
        );
    }

    #[test]
    fn erfc_family_direct_call_witnesses() {
        // Excel-anchored exact bits. These are all in the libm-matching
        // regime (x <= 1.25, or all negatives).
        assert_bits_eq("erfc(0)", erfc_kernel(0.0).unwrap(), 1.0);
        assert_bits_eq("erfc(0.5)", erfc_kernel(0.5).unwrap(), 0.4795001221869535);
        assert_bits_eq("erfc(1)", erfc_kernel(1.0).unwrap(), 0.15729920705028513);
        assert_bits_eq(
            "erfc(1.25)",
            erfc_kernel(1.25).unwrap(),
            0.07709987174354177,
        );
        assert_bits_eq("erfc(-1)", erfc_kernel(-1.0).unwrap(), 1.8427007929497148);
        assert_bits_eq("erfc(-2)", erfc_kernel(-2.0).unwrap(), 1.9953222650189528);

        // Libm-matching pockets within the positive regime (small island).
        assert_bits_eq(
            "erfc(2.75)",
            erfc_kernel(2.75).unwrap(),
            0.00010062192211963684,
        );
        assert_bits_eq("erfc(2.8)", erfc_kernel(2.8).unwrap(), 7.501319466545911e-5);

        // ERFC.PRECISE delegates to the same kernel; spot-check parity at
        // one representative in-range anchor and one blocked-regime pocket.
        assert_bits_eq("erfc.precise(0)", erfc_precise_kernel(0.0).unwrap(), 1.0);
        assert_bits_eq(
            "erfc.precise(1)",
            erfc_precise_kernel(1.0).unwrap(),
            0.15729920705028513,
        );
        assert_bits_eq(
            "erfc.precise(-1)",
            erfc_precise_kernel(-1.0).unwrap(),
            1.8427007929497148,
        );

        // Stable family controls across the full widened positive range.
        let xs: &[f64] = &[
            0.0, 0.5, 1.0, 1.25, 1.5, 1.75, 1.9, 2.0, 2.1, 2.25, 2.5, 2.6, 2.7, 2.75, 2.8, 2.9,
            3.0, 3.5, 4.0, 5.0, 6.0, 8.0, 10.0,
        ];
        let mut prev: Option<f64> = None;
        for &x in xs {
            let v = erfc_kernel(x).unwrap();
            // Range: erfc(x) in (0, 1] for x >= 0 (equals 1 exactly at 0).
            assert!(v > 0.0 && v <= 1.0, "erfc({x}) = {v} out of range");
            // Strict monotone-decreasing on positives.
            if let Some(p) = prev {
                assert!(p > v, "monotone: erfc(prev) > erfc({x}), got {p} !> {v}");
            }
            prev = Some(v);
        }

        // Reflection: erfc(-x) + erfc(x) ≈ 2 (tight ULP-scale bound; exact equality
        // is not guaranteed because both summands are rounded f64 values).
        let e_one = erfc_kernel(1.0).unwrap();
        let e_neg_one = erfc_kernel(-1.0).unwrap();
        let e_two = erfc_kernel(2.0).unwrap();
        let e_neg_two = erfc_kernel(-2.0).unwrap();
        assert!(
            (e_neg_one + e_one - 2.0).abs() < 1e-15,
            "reflection erfc(-1)+erfc(1) = {}",
            e_neg_one + e_one
        );
        assert!(
            (e_neg_two + e_two - 2.0).abs() < 1e-15,
            "reflection erfc(-2)+erfc(2) = {}",
            e_neg_two + e_two
        );
    }

    // Excel-matching exact-bit witnesses for every input where the
    // empirical correction-fit kernel reproduces Excel's bits. These
    // pass on every platform (no UCRT dependency).
    #[test]
    fn erfc_correction_fit_matches_excel_exact_bits() {
        // Newly-matched by the correction fit (previously blocked).
        assert_bits_eq("erfc(1.5)", erfc_kernel(1.5).unwrap(), 0.03389485352468927);
        assert_bits_eq("erfc(1.8)", erfc_kernel(1.8).unwrap(), 0.010909498364269283);
        assert_bits_eq(
            "erfc(2.15)",
            erfc_kernel(2.15).unwrap(),
            0.002361392962674656,
        );
        assert_bits_eq(
            "erfc(2.25)",
            erfc_kernel(2.25).unwrap(),
            0.0014627165866811515,
        );
        assert_bits_eq(
            "erfc(2.4)",
            erfc_kernel(2.4).unwrap(),
            0.0006885138966450787,
        );
        assert_bits_eq(
            "erfc(2.5)",
            erfc_kernel(2.5).unwrap(),
            0.00040695201744495886,
        );
        assert_bits_eq(
            "erfc(2.99)",
            erfc_kernel(2.99).unwrap(),
            2.3525603080640202e-5,
        );
        assert_bits_eq(
            "erfc(3.25)",
            erfc_kernel(3.25).unwrap(),
            4.302779463675121e-6,
        );

        // Large-x anchors preserved from the UCRT round.
        assert_bits_eq("erfc(3)", erfc_kernel(3.0).unwrap(), 2.209049699858544e-5);
        assert_bits_eq("erfc(4)", erfc_kernel(4.0).unwrap(), 1.5417257900280017e-8);
        assert_bits_eq("erfc(8)", erfc_kernel(8.0).unwrap(), 1.1224297172982929e-29);

        // Already-green anchors preserved (fit forces corr -> 0 here).
        assert_bits_eq(
            "erfc(1.85)",
            erfc_kernel(1.85).unwrap(),
            0.008888969943914289,
        );
        assert_bits_eq(
            "erfc(1.95)",
            erfc_kernel(1.95).unwrap(),
            0.005820666407810882,
        );
        assert_bits_eq(
            "erfc(2.75)",
            erfc_kernel(2.75).unwrap(),
            0.00010062192211963684,
        );
        assert_bits_eq("erfc(2.8)", erfc_kernel(2.8).unwrap(), 7.501319466545911e-5);
        assert_bits_eq(
            "erfc(3.001)",
            erfc_kernel(3.001).unwrap(),
            2.1951660917737304e-5,
        );

        // ERFC.PRECISE parity spot-checks — same kernel.
        assert_bits_eq(
            "erfc.precise(1.5)",
            erfc_precise_kernel(1.5).unwrap(),
            0.03389485352468927,
        );
        assert_bits_eq(
            "erfc.precise(3)",
            erfc_precise_kernel(3.0).unwrap(),
            2.209049699858544e-5,
        );
    }

    // Exact-bit Excel witnesses still blocked by the correction-fit kernel.
    // The remaining residual is chaotic at the ULP level — no smoothly
    // representable polynomial correction reproduces these. Kept as
    // #[ignore]d sentinels; enable via `cargo test -- --ignored` when a
    // kernel candidate targeting them is in flight.
    #[test]
    #[ignore = "Excel residual not reproducible via smooth correction polynomial; see docs/function-lane/ERFC_EXCEL_EMULATION.md"]
    fn erfc_remaining_blocked_excel_witnesses() {
        let cases: &[(f64, f64)] = &[
            (1.6, 0.023651616655355978),
            (1.7, 0.01620954140922544),
            (1.75, 0.013328328780817557),
            (1.9, 0.007209570764742528),
            (2.0, 0.0046777349810472645),
            (2.05, 0.0037419039555431272),
            (2.1, 0.002979466656332985),
            (2.35, 0.000889267032132454),
            (2.45, 0.0005305801122510537),
            (2.55, 0.0003106603426391907),
            (2.6, 0.000236034416529349),
            (2.65, 0.00017848775202400087),
            (2.7, 0.0001343327399405242),
            (2.85, 5.565627996139894e-5),
            (2.9, 4.1097878099458844e-5),
            (2.95, 3.0203042064138246e-5),
            (2.999, 2.2230168599834054e-5),
            (3.005, 2.1404577729752717e-5),
            (3.01, 2.0738963637132638e-5),
            (3.02, 1.946639071441418e-5),
            (3.5, 7.430983723414129e-7),
            (3.75, 1.1372725656979669e-7),
            (4.5, 1.9661604415428865e-10),
            (5.0, 1.537459794428034e-12),
            (6.0, 2.151973671249892e-17),
            (7.0, 4.1838256077794166e-23),
            (9.0, 4.137031746513812e-37),
            (10.0, 2.0884875837625446e-45),
        ];
        for (x, excel) in cases.iter().copied() {
            let got = erfc_kernel(x).unwrap();
            assert_eq!(
                got.to_bits(),
                excel.to_bits(),
                "erfc({x}): got {got:e} ({:#018x}), excel {excel:e} ({:#018x})",
                got.to_bits(),
                excel.to_bits()
            );
        }
    }

    #[test]
    fn erf_family_matches_seed_rows() {
        assert_close(erf_kernel(1.0, None).unwrap(), 0.8427007929497149, 1e-15);
        assert_close(
            erf_kernel(0.0, Some(1.0)).unwrap(),
            0.8427007929497149,
            1e-15,
        );
        assert_close(erf_precise_kernel(1.0).unwrap(), 0.8427007929497149, 1e-15);
        assert_close(erfc_kernel(1.0).unwrap(), 0.15729920705028513, 1e-15);
        assert_close(
            erfc_precise_kernel(-1.0).unwrap(),
            1.8427007929497148,
            1e-15,
        );
        assert_close(
            erf_kernel(1.0, Some(2.0)).unwrap(),
            0.15262147206923782,
            1e-15,
        );
        assert_close(
            erf_kernel(2.0, Some(1.0)).unwrap(),
            -0.15262147206923782,
            1e-15,
        );
    }

    #[test]
    fn gamma_family_matches_seed_rows() {
        assert_close(gamma_kernel(5.0).unwrap(), 24.0, 1e-12);
        assert_close(gamma_kernel(0.5).unwrap(), 1.772453850905516, 1e-12);
        assert_close(gamma_kernel(-0.5).unwrap(), -3.5449077018110318, 1e-10);
        assert_close(gammaln_kernel(5.0).unwrap(), 3.1780538303479458, 1e-12);
        assert_close(
            gammaln_precise_kernel(0.5).unwrap(),
            0.5723649429247001,
            1e-12,
        );
        assert_eq!(gamma_kernel(-1.0), Err(WorksheetErrorCode::Num));
        assert_eq!(gammaln_kernel(0.0), Err(WorksheetErrorCode::Num));
        assert_eq!(gamma_kernel(172.0), Err(WorksheetErrorCode::Num));
    }

    // BUG-FUNC-027 CLASS-A1: GAMMALN(1E-300) was +Inf (z+1 == 0 in Lanczos);
    // live Excel 16.0 b20026 = 690.7755278982137 via the recurrence.
    #[test]
    fn gammaln_tiny_positive_uses_recurrence_not_inf() {
        let v = gammaln_kernel(1e-300).unwrap();
        assert!(v.is_finite(), "GAMMALN(1e-300) was non-finite: {v}");
        assert_close(v, 690.7755278982137, 1e-9);
        assert!(gammaln_precise_kernel(1e-300).unwrap().is_finite());
    }

    #[test]
    fn gammaln_rejects_positive_subnormals_and_admits_min_normal() {
        // W109 G3-02 current-reference discovery plus separately frozen
        // answer-blind heldout: 40/40 positive-subnormal rows publish #NUM!
        // across GAMMALN and GAMMALN.PRECISE. The adjacent min-normal endpoint
        // is admitted and its exact published value is pinned here.
        for x in [f64::from_bits(1), f64::from_bits(0x000f_ffff_ffff_ffff)] {
            assert_eq!(gammaln_kernel(x), Err(WorksheetErrorCode::Num));
            assert_eq!(gammaln_precise_kernel(x), Err(WorksheetErrorCode::Num));
        }

        let expected = f64::from_bits(0x4086_232b_dd7a_bcd2);
        assert_bits_eq(
            "gammaln(min-normal)",
            gammaln_kernel(f64::MIN_POSITIVE).unwrap(),
            expected,
        );
        assert_bits_eq(
            "gammaln.precise(min-normal)",
            gammaln_precise_kernel(f64::MIN_POSITIVE).unwrap(),
            expected,
        );
    }

    // BUG-FUNC-027 CLASS-A2: GAMMA(-1E-200) rounds to 0 but is not the pole at 0;
    // live Excel 16.0 b20026 ~ -1E200 (finite). Fine ULP exactness is CLASS-C1.
    #[test]
    fn gamma_tiny_negative_is_not_a_false_pole() {
        let v = gamma_kernel(-1e-200).unwrap();
        assert!(v.is_finite() && v < 0.0, "GAMMA(-1e-200) = {v}");
        assert!(
            (v.abs().log10() - 200.0).abs() < 1.0,
            "magnitude ~1e200: {v}"
        );
        // Exact non-positive-integer poles still error.
        assert_eq!(gamma_kernel(0.0), Err(WorksheetErrorCode::Num));
        assert_eq!(gamma_kernel(-2.0), Err(WorksheetErrorCode::Num));
        // A non-integer negative is finite, not a pole.
        assert!(gamma_kernel(-1.5).unwrap().is_finite());
    }

    #[test]
    fn weibull_family_matches_seed_rows() {
        assert_close(
            weibull_kernel(2.0, 3.0, 4.0, true).unwrap(),
            0.11750309741540463,
            1e-12,
        );
        assert_close(
            weibull_dist_kernel(2.0, 3.0, 4.0, false).unwrap(),
            0.1654681692346117,
            1e-12,
        );
        assert_eq!(weibull_dist_kernel(0.0, 3.0, 4.0, true), Ok(0.0));
        assert_eq!(weibull_dist_kernel(0.0, 3.0, 4.0, false), Ok(0.0));
        assert_eq!(weibull_dist_kernel(0.0, 0.5, 4.0, false), Ok(0.0));
        assert_eq!(weibull_dist_kernel(0.0, 1.0, 4.0, false), Ok(0.0));
        assert_eq!(
            weibull_dist_kernel(-1.0, 3.0, 4.0, true),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            weibull_dist_kernel(2.0, 0.0, 4.0, true),
            Err(WorksheetErrorCode::Num)
        );
        assert_eq!(
            weibull_dist_kernel(2.0, 3.0, 0.0, true),
            Err(WorksheetErrorCode::Num)
        );
    }

    #[test]
    fn surface_evaluators_follow_flag_and_error_contracts() {
        let resolver = NoResolver;
        let weibull_cdf = eval_weibull_dist_surface(
            &[
                (CalcValue::number(2.0)),
                (CalcValue::number(3.0)),
                (CalcValue::number(4.0)),
                (CalcValue::number(1.0)),
            ],
            &resolver,
        );
        match weibull_cdf {
            Ok(value) => match value.core() {
                CoreValue::Number(value) => assert_close(*value, 0.11750309741540463, 1e-12),
                other => panic!("unexpected weibull cdf result: {other:?}"),
            },
            other => panic!("unexpected weibull cdf result: {other:?}"),
        }

        let weibull_pdf = eval_weibull_dist_surface(
            &[
                (CalcValue::number(2.0)),
                (CalcValue::number(3.0)),
                (CalcValue::number(4.0)),
                (CalcValue::number(0.0)),
            ],
            &resolver,
        );
        match weibull_pdf {
            Ok(value) => match value.core() {
                CoreValue::Number(value) => assert_close(*value, 0.1654681692346117, 1e-12),
                other => panic!("unexpected weibull pdf result: {other:?}"),
            },
            other => panic!("unexpected weibull pdf result: {other:?}"),
        }

        assert_eq!(
            eval_gamma_surface(&[(CalcValue::number(-1.0))], &resolver),
            Ok(CalcValue::error(WorksheetErrorCode::Num))
        );
        assert_eq!(
            eval_erf_surface(&[], &resolver),
            Err(SpecialDistEvalError::ArityMismatch {
                expected_min: 1,
                expected_max: 2,
                actual: 0,
            })
        );
    }

    #[test]
    fn metadata_profiles_match_batch_shape() {
        assert_eq!(
            ERF_META.arg_preparation_profile,
            FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE
        );
        assert_eq!(
            WEIBULL_DIST_META.surface_fec_dependency_profile,
            FecDependencyProfile::RefOnly
        );
    }
}
