use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, run_values_only_prepared};
use crate::locale_format::{WorkbookDateSystem, excel_serial_from_ymd, ymd_from_excel_serial};
use crate::resolver::ReferenceSystemProvider;
use crate::value::{CalcValue, CoreValue, WorksheetErrorCode};

const BASE: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BOND_CORE_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: FunctionMeta::DEFAULT_ARG_PREPARATION_PROFILE,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    lift_broadcast_profile: FunctionMeta::DEFAULT_LIFT_BROADCAST_PROFILE,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
    real_result_policy: FunctionMeta::DEFAULT_REAL_RESULT_POLICY,
    error_collapse_profile: FunctionMeta::DEFAULT_ERROR_COLLAPSE_PROFILE,
    precision_rounding_profile: FunctionMeta::DEFAULT_PRECISION_ROUNDING_PROFILE,
};
pub const ACCRINT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ACCRINT",
    arity: Arity { min: 6, max: 8 },
    ..BASE
};
pub const ACCRINTM_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.ACCRINTM",
    arity: Arity { min: 3, max: 5 },
    ..BASE
};
pub const DURATION_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.DURATION",
    arity: Arity { min: 5, max: 6 },
    ..BASE
};
pub const MDURATION_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.MDURATION",
    arity: Arity { min: 5, max: 6 },
    ..BASE
};
pub const PRICE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PRICE",
    arity: Arity { min: 6, max: 7 },
    ..BASE
};
pub const PRICEMAT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.PRICEMAT",
    arity: Arity { min: 5, max: 6 },
    ..BASE
};
pub const YIELD_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.YIELD",
    arity: Arity { min: 6, max: 7 },
    ..BASE
};
pub const YIELDDISC_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.YIELDDISC",
    arity: Arity { min: 4, max: 5 },
    ..BASE
};
pub const YIELDMAT_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.YIELDMAT",
    arity: Arity { min: 5, max: 6 },
    ..BASE
};

#[derive(Debug, Clone, PartialEq)]
pub enum BondCoreEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayCountBasis {
    Us30_360,
    ActualActual,
    Actual360,
    Actual365,
    European30_360,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ctx {
    settlement: i64,
    maturity: i64,
    frequency: i64,
    basis: DayCountBasis,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Period {
    prev: i64,
    next: i64,
    n: i64,
}

fn max_serial() -> i64 {
    excel_serial_from_ymd(WorkbookDateSystem::System1900, 9999, 12, 31).unwrap() as i64
}
fn derr(c: WorksheetErrorCode) -> BondCoreEvalError {
    BondCoreEvalError::Domain(c)
}
fn arity(meta: &FunctionMeta, a: usize) -> BondCoreEvalError {
    BondCoreEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual: a,
    }
}
fn dyear(y: i64) -> f64 {
    if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
        366.0
    } else {
        365.0
    }
}
fn dim(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if dyear(y) == 366.0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}
fn act(s: i64, e: i64) -> f64 {
    (e - s) as f64
}
fn narg(a: &[CalcValue], i: usize) -> Result<f64, BondCoreEvalError> {
    a.get(i)
        .ok_or(derr(WorksheetErrorCode::Value))
        .and_then(|v| coerce_prepared_to_number(v).map_err(BondCoreEvalError::Coercion))
}
fn oarg(a: &[CalcValue], i: usize, d: f64) -> Result<f64, BondCoreEvalError> {
    match a.get(i) {
        None => Ok(d),
        Some(v) if matches!(v.core(), CoreValue::Missing) => Ok(d),
        Some(v) => coerce_prepared_to_number(v).map_err(BondCoreEvalError::Coercion),
    }
}
fn obool(a: &[CalcValue], i: usize, d: bool) -> Result<bool, BondCoreEvalError> {
    match a.get(i) {
        None => Ok(d),
        Some(v) if matches!(v.core(), CoreValue::Missing) => Ok(d),
        Some(v) => Ok(coerce_prepared_to_number(v)
            .map_err(BondCoreEvalError::Coercion)?
            .trunc()
            != 0.0),
    }
}
fn basis(v: f64) -> Result<DayCountBasis, BondCoreEvalError> {
    if !v.is_finite() {
        return Err(derr(WorksheetErrorCode::Num));
    }
    match v.trunc() as i64 {
        0 => Ok(DayCountBasis::Us30_360),
        1 => Ok(DayCountBasis::ActualActual),
        2 => Ok(DayCountBasis::Actual360),
        3 => Ok(DayCountBasis::Actual365),
        4 => Ok(DayCountBasis::European30_360),
        _ => Err(derr(WorksheetErrorCode::Num)),
    }
}
fn freq(v: f64) -> Result<i64, BondCoreEvalError> {
    if !v.is_finite() {
        return Err(derr(WorksheetErrorCode::Num));
    }
    match v.trunc() as i64 {
        1 | 2 | 4 => Ok(v.trunc() as i64),
        _ => Err(derr(WorksheetErrorCode::Num)),
    }
}
fn dser(v: f64) -> Result<i64, BondCoreEvalError> {
    if !v.is_finite() {
        return Err(derr(WorksheetErrorCode::Value));
    }
    let s = v.trunc() as i64;
    if s < 1 || s > max_serial() {
        return Err(derr(WorksheetErrorCode::Value));
    }
    ymd_from_excel_serial(WorkbookDateSystem::System1900, s as f64)
        .ok_or(derr(WorksheetErrorCode::Value))?;
    Ok(s)
}
fn pos(v: f64) -> Result<f64, BondCoreEvalError> {
    if !v.is_finite() {
        Err(derr(WorksheetErrorCode::Value))
    } else if v <= 0.0 {
        Err(derr(WorksheetErrorCode::Num))
    } else {
        Ok(v)
    }
}
fn rate(v: f64) -> Result<f64, BondCoreEvalError> {
    if !v.is_finite() {
        Err(derr(WorksheetErrorCode::Value))
    } else if v < 0.0 {
        Err(derr(WorksheetErrorCode::Num))
    } else {
        Ok(v)
    }
}
fn addm(s: i64, m: i64) -> Option<i64> {
    let (y, mo, d) = ymd_from_excel_serial(WorkbookDateSystem::System1900, s as f64)?;
    let idx = y.checked_mul(12)?.checked_add(mo - 1)?.checked_add(m)?;
    let ty = idx.div_euclid(12);
    let tm = idx.rem_euclid(12) + 1;
    let end = d == dim(y, mo);
    let td = if end { dim(ty, tm) } else { d.min(dim(ty, tm)) };
    excel_serial_from_ymd(WorkbookDateSystem::System1900, ty, tm, td).map(|v| v as i64)
}
fn d360us(s: i64, e: i64) -> Result<f64, BondCoreEvalError> {
    crate::functions::day_count_common::us_30_360(s, e).map_err(derr)
}
fn d360eu(s: i64, e: i64) -> Result<f64, BondCoreEvalError> {
    let (sy, sm, mut sd) = ymd_from_excel_serial(WorkbookDateSystem::System1900, s as f64)
        .ok_or(derr(WorksheetErrorCode::Value))?;
    let (ey, em, mut ed) = ymd_from_excel_serial(WorkbookDateSystem::System1900, e as f64)
        .ok_or(derr(WorksheetErrorCode::Value))?;
    if sd == 31 {
        sd = 30;
    }
    if ed == 31 {
        ed = 30;
    }
    Ok(((ey - sy) * 360 + (em - sm) * 30 + (ed - sd)) as f64)
}
fn actact(s: i64, e: i64) -> Result<f64, BondCoreEvalError> {
    if e <= s {
        return Ok(0.0);
    }
    let (sy, _, _) = ymd_from_excel_serial(WorkbookDateSystem::System1900, s as f64)
        .ok_or(derr(WorksheetErrorCode::Value))?;
    let (ey, _, _) = ymd_from_excel_serial(WorkbookDateSystem::System1900, e as f64)
        .ok_or(derr(WorksheetErrorCode::Value))?;
    if sy == ey {
        return Ok(act(s, e) / dyear(sy));
    }
    let sny = excel_serial_from_ymd(WorkbookDateSystem::System1900, sy + 1, 1, 1)
        .ok_or(derr(WorksheetErrorCode::Value))? as i64;
    let eys = excel_serial_from_ymd(WorkbookDateSystem::System1900, ey, 1, 1)
        .ok_or(derr(WorksheetErrorCode::Value))? as i64;
    let mut t = act(s, sny) / dyear(sy);
    for _ in (sy + 1)..ey {
        t += 1.0;
    }
    t += act(eys, e) / dyear(ey);
    Ok(t)
}
fn less_or_equal_to_a_year_apart(s: i64, e: i64) -> Result<bool, BondCoreEvalError> {
    let (sy, sm, sd) = ymd_from_excel_serial(WorkbookDateSystem::System1900, s as f64)
        .ok_or(derr(WorksheetErrorCode::Value))?;
    let (ey, em, ed) = ymd_from_excel_serial(WorkbookDateSystem::System1900, e as f64)
        .ok_or(derr(WorksheetErrorCode::Value))?;
    Ok((ey - sy) < 1 || ((ey - sy) == 1 && (em < sm || (em == sm && ed <= sd))))
}
fn consider_as_bisestile(s: i64, e: i64) -> Result<bool, BondCoreEvalError> {
    let (sy, _sm, _sd) = ymd_from_excel_serial(WorkbookDateSystem::System1900, s as f64)
        .ok_or(derr(WorksheetErrorCode::Value))?;
    let (ey, em, ed) = ymd_from_excel_serial(WorkbookDateSystem::System1900, e as f64)
        .ok_or(derr(WorksheetErrorCode::Value))?;
    if sy == ey {
        return Ok(dyear(sy) == 366.0);
    }
    if em == 2 && ed == 29 {
        return Ok(true);
    }
    for year in sy..=ey {
        if dyear(year) == 366.0 {
            let feb29 = excel_serial_from_ymd(WorkbookDateSystem::System1900, year, 2, 29)
                .ok_or(derr(WorksheetErrorCode::Value))? as i64;
            if s <= feb29 && feb29 <= e {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
fn days_in_year_for_mat(s: i64, e: i64, b: DayCountBasis) -> Result<f64, BondCoreEvalError> {
    match b {
        DayCountBasis::Us30_360 | DayCountBasis::Actual360 | DayCountBasis::European30_360 => {
            Ok(360.0)
        }
        DayCountBasis::Actual365 => Ok(365.0),
        DayCountBasis::ActualActual => {
            if !less_or_equal_to_a_year_apart(s, e)? {
                let (sy, _, _) = ymd_from_excel_serial(WorkbookDateSystem::System1900, s as f64)
                    .ok_or(derr(WorksheetErrorCode::Value))?;
                let (ey, _, _) = ymd_from_excel_serial(WorkbookDateSystem::System1900, e as f64)
                    .ok_or(derr(WorksheetErrorCode::Value))?;
                let tot_years = (ey - sy) + 1;
                let start_of_issue_year =
                    excel_serial_from_ymd(WorkbookDateSystem::System1900, sy, 1, 1)
                        .ok_or(derr(WorksheetErrorCode::Value))? as i64;
                let start_after_end_year =
                    excel_serial_from_ymd(WorkbookDateSystem::System1900, ey + 1, 1, 1)
                        .ok_or(derr(WorksheetErrorCode::Value))? as i64;
                Ok(act(start_of_issue_year, start_after_end_year) / tot_years as f64)
            } else if consider_as_bisestile(s, e)? {
                Ok(366.0)
            } else {
                Ok(365.0)
            }
        }
    }
}
fn yf(s: i64, e: i64, b: DayCountBasis) -> Result<f64, BondCoreEvalError> {
    match b {
        DayCountBasis::Us30_360 => Ok(d360us(s, e)? / 360.0),
        DayCountBasis::ActualActual => actact(s, e),
        DayCountBasis::Actual360 => Ok(act(s, e) / 360.0),
        DayCountBasis::Actual365 => Ok(act(s, e) / 365.0),
        DayCountBasis::European30_360 => Ok(d360eu(s, e)? / 360.0),
    }
}
fn dc(s: i64, e: i64, b: DayCountBasis, f: i64) -> Result<f64, BondCoreEvalError> {
    match b {
        DayCountBasis::Us30_360 => Ok(360.0 / f as f64),
        DayCountBasis::ActualActual => Ok(act(s, e)),
        DayCountBasis::Actual360 => Ok(360.0 / f as f64),
        DayCountBasis::Actual365 => Ok(365.0 / f as f64),
        DayCountBasis::European30_360 => Ok(360.0 / f as f64),
    }
}
fn dd(s: i64, e: i64, b: DayCountBasis) -> Result<f64, BondCoreEvalError> {
    match b {
        DayCountBasis::Us30_360 => d360us(s, e),
        DayCountBasis::ActualActual | DayCountBasis::Actual360 | DayCountBasis::Actual365 => {
            Ok(act(s, e))
        }
        DayCountBasis::European30_360 => d360eu(s, e),
    }
}
fn ctx(
    settlement: f64,
    maturity: f64,
    frequency: f64,
    b: Option<f64>,
) -> Result<Ctx, BondCoreEvalError> {
    let settlement = dser(settlement)?;
    let maturity = dser(maturity)?;
    let frequency = freq(frequency)?;
    let basis = basis(b.unwrap_or(0.0))?;
    if settlement >= maturity {
        return Err(derr(WorksheetErrorCode::Num));
    }
    Ok(Ctx {
        settlement,
        maturity,
        frequency,
        basis,
    })
}
fn period(c: Ctx) -> Result<Period, BondCoreEvalError> {
    let mpc = 12 / c.frequency;
    let mut next = c.maturity;
    let mut n = 1i64;
    loop {
        let prev = addm(next, -mpc).ok_or(derr(WorksheetErrorCode::Num))?;
        if prev <= c.settlement {
            return Ok(Period { prev, next, n });
        }
        next = prev;
        n += 1;
    }
}
fn pcomp(
    rate: f64,
    yld: f64,
    red: f64,
    c: Ctx,
    p: Period,
) -> Result<(f64, f64, f64, f64, f64), BondCoreEvalError> {
    let coup = 100.0 * rate / c.frequency as f64;
    let e = dc(p.prev, p.next, c.basis, c.frequency)?;
    let a = dd(p.prev, c.settlement, c.basis)?;
    let dsc = dd(c.settlement, p.next, c.basis)?;
    if yld <= -(c.frequency as f64) {
        return Err(derr(WorksheetErrorCode::Num));
    }
    let dirty = if p.n == 1 {
        let den = 1.0 + (yld / c.frequency as f64) * (dsc / e);
        if den <= 0.0 {
            return Err(derr(WorksheetErrorCode::Num));
        }
        (red + coup) / den
    } else {
        let base = 1.0 + yld / c.frequency as f64;
        if base <= 0.0 {
            return Err(derr(WorksheetErrorCode::Num));
        }
        let off = dsc / e;
        let mut pv = 0.0;
        for k in 0..p.n {
            pv += coup / base.powf(off + k as f64);
        }
        pv + red / base.powf(off + (p.n - 1) as f64)
    };
    let accr = coup * a / e;
    Ok((dirty - accr, dirty, coup, a, e))
}
fn solve(
    target: f64,
    f: i64,
    fun: impl Fn(f64) -> Result<f64, BondCoreEvalError>,
) -> Result<f64, BondCoreEvalError> {
    let low = -(f as f64) + 1e-10;
    // Price → +∞ as yield → -frequency, but the price kernel can overflow/reject
    // that near-degenerate endpoint. Treat an un-evaluable low-yield endpoint as
    // an effectively infinite price so the "is the target achievable" guard does
    // not spuriously fail a well-posed bond (BUG-FUNC-031). The bisection below
    // moves off `low` on its first step, so it never re-probes the endpoint.
    let lowp = fun(low).unwrap_or(f64::INFINITY);
    if target > lowp {
        return Err(derr(WorksheetErrorCode::Num));
    }
    let mut hi = 1.0;
    let mut hip = fun(hi)?;
    let mut g = 0;
    while hip > target && g < 80 {
        hi *= 2.0;
        hip = fun(hi)?;
        g += 1;
    }
    if hip > target {
        return Err(derr(WorksheetErrorCode::Num));
    }
    let mut lo = low;
    let mut hh = hi;
    for _ in 0..100 {
        let mid = (lo + hh) / 2.0;
        let mp = fun(mid)?;
        if (mp - target).abs() <= 1e-15 {
            return Ok(mid);
        }
        if mp > target { lo = mid } else { hh = mid }
    }
    Ok((lo + hh) / 2.0)
}

pub fn accrintm_kernel(
    issue: f64,
    settlement: f64,
    rate_: f64,
    par: Option<f64>,
    basis_: Option<f64>,
) -> Result<f64, BondCoreEvalError> {
    let issue = dser(issue)?;
    let settlement = dser(settlement)?;
    let rate_ = rate(rate_)?;
    let par = pos(par.unwrap_or(1000.0))?;
    let basis_ = basis(basis_.unwrap_or(0.0))?;
    if issue >= settlement {
        return Err(derr(WorksheetErrorCode::Num));
    }
    Ok(par * rate_ * yf(issue, settlement, basis_)?)
}
pub fn pricemat_kernel(
    settlement: f64,
    maturity: f64,
    issue: f64,
    rate_: f64,
    yld: f64,
    basis_: Option<f64>,
) -> Result<f64, BondCoreEvalError> {
    let settlement = dser(settlement)?;
    let maturity = dser(maturity)?;
    let issue = dser(issue)?;
    let rate_ = rate(rate_)?;
    let yld = rate(yld)?;
    let basis_ = basis(basis_.unwrap_or(0.0))?;
    if !(issue < settlement && settlement < maturity) {
        return Err(derr(WorksheetErrorCode::Num));
    }
    let b = days_in_year_for_mat(issue, settlement, basis_)?;
    let dim = dd(issue, maturity, basis_)?;
    let a = dd(issue, settlement, basis_)?;
    let dsm = dim - a;
    let future = 100.0 + (dim / b * rate_ * 100.0);
    let accrued = a / b * rate_ * 100.0;
    let den = 1.0 + (dsm / b * yld);
    if den <= 0.0 {
        return Err(derr(WorksheetErrorCode::Num));
    }
    Ok(future / den - accrued)
}
pub fn yieldmat_kernel(
    settlement: f64,
    maturity: f64,
    issue: f64,
    rate_: f64,
    price: f64,
    basis_: Option<f64>,
) -> Result<f64, BondCoreEvalError> {
    let settlement = dser(settlement)?;
    let maturity = dser(maturity)?;
    let issue = dser(issue)?;
    let rate_ = rate(rate_)?;
    let price = pos(price)?;
    let basis_ = basis(basis_.unwrap_or(0.0))?;
    if !(issue < settlement && settlement < maturity) {
        return Err(derr(WorksheetErrorCode::Num));
    }
    let b = days_in_year_for_mat(issue, settlement, basis_)?;
    let dim = dd(issue, maturity, basis_)?;
    let a = dd(issue, settlement, basis_)?;
    let dsm = dim - a;
    let term1 = dim / b * rate_ + 1.0 - price / 100.0 - a / b * rate_;
    let term2 = price / 100.0 + a / b * rate_;
    let term3 = b / dsm;
    Ok(term1 / term2 * term3)
}
pub fn yielddisc_kernel(
    settlement: f64,
    maturity: f64,
    price: f64,
    red: f64,
    basis_: Option<f64>,
) -> Result<f64, BondCoreEvalError> {
    let settlement = dser(settlement)?;
    let maturity = dser(maturity)?;
    let price = pos(price)?;
    let red = pos(red)?;
    let basis_ = basis(basis_.unwrap_or(0.0))?;
    if settlement >= maturity {
        return Err(derr(WorksheetErrorCode::Num));
    }
    // Faithful port of ExcelFinancialFunctions `yieldDisc` (bonds.fs):
    // `(redemption - pr) / pr * b / dim`, with `dim` the numerator day-count and
    // `b` DaysInYear. The prior `(red/price - 1) / yearfrac` form was algebraically
    // equal but ~5 ULP off (different operation order) and used the YEARFRAC
    // act/act algorithm rather than DaysBetween/DaysInYear.
    let dim = dd(settlement, maturity, basis_)?;
    let b = days_in_year_for_mat(settlement, maturity, basis_)?;
    Ok((red - price) / price * b / dim)
}
pub fn price_kernel(
    settlement: f64,
    maturity: f64,
    rate_: f64,
    yld: f64,
    red: f64,
    frequency: f64,
    basis_: Option<f64>,
) -> Result<f64, BondCoreEvalError> {
    let c = ctx(settlement, maturity, frequency, basis_)?;
    let p = period(c)?;
    Ok(pcomp(rate(rate_)?, rate(yld)?, pos(red)?, c, p)?.0)
}
pub fn yield_kernel(
    settlement: f64,
    maturity: f64,
    rate_: f64,
    price: f64,
    red: f64,
    frequency: f64,
    basis_: Option<f64>,
) -> Result<f64, BondCoreEvalError> {
    let c = ctx(settlement, maturity, frequency, basis_)?;
    let p = period(c)?;
    let rate_ = rate(rate_)?;
    let price = pos(price)?;
    let red = pos(red)?;
    if p.n == 1 {
        let coup = 100.0 * rate_ / c.frequency as f64;
        let e = dc(p.prev, p.next, c.basis, c.frequency)?;
        let a = dd(p.prev, c.settlement, c.basis)?;
        let dsc = dd(c.settlement, p.next, c.basis)?;
        return Ok((((red + coup) / (price + coup * a / e)) - 1.0) * c.frequency as f64 * e / dsc);
    }
    // Solve over candidate yields with `pcomp` directly. `price_kernel` rejects
    // `yld < 0` via `rate(yld)`, but the root-finder must probe negative candidate
    // yields (its bracket runs down to `-frequency`); `pcomp`'s own guards
    // (`yld <= -frequency`, `base <= 0`) keep the domain correct (BUG-FUNC-031).
    solve(price, c.frequency, |cand| {
        Ok(pcomp(rate_, cand, red, c, p)?.0)
    })
}
pub fn duration_kernel(
    settlement: f64,
    maturity: f64,
    coupon: f64,
    yld: f64,
    frequency: f64,
    basis_: Option<f64>,
) -> Result<f64, BondCoreEvalError> {
    let c = ctx(settlement, maturity, frequency, basis_)?;
    let p = period(c)?;
    let (_, dirty, coup, _, e) = pcomp(rate(coupon)?, rate(yld)?, 100.0, c, p)?;
    let dsc = dd(c.settlement, p.next, c.basis)?;
    let off = dsc / e;
    if p.n == 1 {
        return Ok((off / c.frequency as f64).max(0.0));
    }
    let base = 1.0 + yld / c.frequency as f64;
    let mut w = 0.0;
    for k in 0..p.n {
        let t = (off + k as f64) / c.frequency as f64;
        let disc = base.powf(off + k as f64);
        let cash = if k + 1 == p.n { coup + 100.0 } else { coup };
        w += t * cash / disc;
    }
    Ok(w / dirty)
}
pub fn mduration_kernel(
    settlement: f64,
    maturity: f64,
    coupon: f64,
    yld: f64,
    frequency: f64,
    basis_: Option<f64>,
) -> Result<f64, BondCoreEvalError> {
    let f = freq(frequency)? as f64;
    Ok(
        duration_kernel(settlement, maturity, coupon, yld, frequency, basis_)?
            / (1.0 + rate(yld)? / f),
    )
}
fn is_month_end(s: i64) -> bool {
    ymd_from_excel_serial(WorkbookDateSystem::System1900, s as f64)
        .map(|(y, mo, d)| d == dim(y, mo))
        .unwrap_or(false)
}
fn last_day_of_feb(s: i64) -> bool {
    ymd_from_excel_serial(WorkbookDateSystem::System1900, s as f64)
        .map(|(y, mo, d)| mo == 2 && d == dim(y, mo))
        .unwrap_or(false)
}
/// F# `changeMonth` with an explicit return-last-day flag (.NET `AddMonths`
/// semantics, or force the target month-end when `last_day` is set).
fn change_month_flag(s: i64, months: i64, last_day: bool) -> Option<i64> {
    let (y, mo, d) = ymd_from_excel_serial(WorkbookDateSystem::System1900, s as f64)?;
    let idx = y
        .checked_mul(12)?
        .checked_add(mo - 1)?
        .checked_add(months)?;
    let ty = idx.div_euclid(12);
    let tm = idx.rem_euclid(12) + 1;
    let td = if last_day {
        dim(ty, tm)
    } else {
        d.min(dim(ty, tm))
    };
    excel_serial_from_ymd(WorkbookDateSystem::System1900, ty, tm, td).map(|v| v as i64)
}
/// F# `dateDiff360Us` (US 30/360) with the `ModifyStartDate`/`ModifyBothDates` modes.
fn diff360_us(s: i64, e: i64, modify_both: bool) -> Result<f64, BondCoreEvalError> {
    let (sy, sm, mut sd) = ymd_from_excel_serial(WorkbookDateSystem::System1900, s as f64)
        .ok_or(derr(WorksheetErrorCode::Value))?;
    let (ey, em, mut ed) = ymd_from_excel_serial(WorkbookDateSystem::System1900, e as f64)
        .ok_or(derr(WorksheetErrorCode::Value))?;
    if last_day_of_feb(e) && (last_day_of_feb(s) || modify_both) {
        ed = 30;
    }
    if ed == 31 && (sd >= 30 || modify_both) {
        ed = 30;
    }
    if sd == 31 {
        sd = 30;
    }
    if last_day_of_feb(s) {
        sd = 30;
    }
    Ok(((ey - sy) * 360 + (em - sm) * 30 + (ed - sd)) as f64)
}
/// F# `findPcdNcd`: walk `start` toward `end` by `num_months` steps, returning the
/// bounding pair (the position at/past `end`, and the one before it).
fn find_pcd_ncd_accr(
    start: i64,
    end: i64,
    num_months: i64,
    last_day: bool,
) -> Result<(i64, i64), BondCoreEvalError> {
    let mut front = start;
    let mut trailing = end;
    loop {
        let stop = if num_months > 0 {
            front >= end
        } else {
            front <= end
        };
        if stop {
            return Ok((front, trailing));
        }
        trailing = front;
        front =
            change_month_flag(front, num_months, last_day).ok_or(derr(WorksheetErrorCode::Num))?;
    }
}
/// F# `IDayCount.DaysBetween` (numerator position) per basis.
fn days_between_num(s: i64, e: i64, b: DayCountBasis) -> Result<f64, BondCoreEvalError> {
    match b {
        DayCountBasis::Us30_360 => diff360_us(s, e, false),
        DayCountBasis::ActualActual | DayCountBasis::Actual360 | DayCountBasis::Actual365 => {
            Ok(act(s, e))
        }
        DayCountBasis::European30_360 => d360eu(s, e),
    }
}
/// F# `IDayCount.DaysBetween` (denominator position) per basis — only the
/// ActualActual / Actual360 / European cases are reached (the kernel handles
/// Us30_360 and Actual365 inline).
fn days_between_denum(s: i64, e: i64, b: DayCountBasis) -> Result<f64, BondCoreEvalError> {
    match b {
        DayCountBasis::ActualActual => Ok(act(s, e)),
        DayCountBasis::Actual360 => diff360_us(s, e, false),
        DayCountBasis::European30_360 => d360eu(s, e),
        DayCountBasis::Us30_360 => diff360_us(s, e, false),
        DayCountBasis::Actual365 => Ok(act(s, e)),
    }
}
/// F# `actualCoupDays pcd firstInterest`: the actual length of the coupon period
/// (on `first`'s schedule) that bounds `pcd`. Works in both regimes — `pcd` a
/// period before `first` (odd first coupon) or after it (settlement past the first
/// interest date), where the bounding period differs.
fn actual_coup_days_accr(
    pcd: i64,
    first: i64,
    num_months: i64,
    last_day: bool,
) -> Result<f64, BondCoreEvalError> {
    let (prev, next) = find_pcd_ncd_accr(first, pcd, -num_months, last_day)?;
    Ok(act(prev, next))
}
/// F# `IDayCount.CoupDays pcd firstInterest`: the normal length of the coupon
/// period anchored at `pcd`. ActualActual uses the actual bounding-period days; the
/// 30/360 bases use `360/freq`, Actual/365 uses `365/freq`.
fn coup_days_accr(
    pcd: i64,
    first: i64,
    num_months: i64,
    last_day: bool,
    fc: f64,
    b: DayCountBasis,
) -> Result<f64, BondCoreEvalError> {
    match b {
        DayCountBasis::ActualActual => actual_coup_days_accr(pcd, first, num_months, last_day),
        DayCountBasis::Actual365 => Ok(365.0 / fc),
        _ => Ok(360.0 / fc),
    }
}

pub fn accrint_kernel(
    issue: f64,
    first_interest: f64,
    settlement: f64,
    rate_: f64,
    par: Option<f64>,
    frequency: f64,
    basis_: Option<f64>,
    calc_method: Option<bool>,
) -> Result<f64, BondCoreEvalError> {
    let issue = dser(issue)?;
    let first = dser(first_interest)?;
    let settlement = dser(settlement)?;
    let rate_ = rate(rate_)?;
    let par = pos(par.unwrap_or(1000.0))?;
    let f = freq(frequency)?;
    let basis_ = basis(basis_.unwrap_or(0.0))?;
    // calc_method: TRUE = FromIssueToSettlement (Excel default), FALSE = FromFirstToSettlement.
    let from_issue = calc_method.unwrap_or(true);
    if !(issue < first && issue < settlement) {
        return Err(derr(WorksheetErrorCode::Num));
    }
    // Faithful port of ExcelFinancialFunctions `accrInt` (bonds.fs) for the first-coupon
    // regime, extended to match live Excel for settlement past the first interest date
    // (which F# rejects). Excel normalises the settlement-side fraction by the *canonical*
    // last coupon period length `CoupDays(first - 1 period, first)` — so a leap-crossing
    // period is never measured by its own actual length (the ~0.07% act/act error of the
    // prior kernel — BUG-FUNC-030). The two regimes measure differently: settlement within
    // the first coupon span is measured *backward* from `pcd`, settlement past `first` is
    // accrued *forward* from the accrual start with whole periods counting as 1.
    let fc = f as f64;
    let num_months = 12 / f;
    let end_flag = is_month_end(first);
    let pcd =
        change_month_flag(first, -num_months, end_flag).ok_or(derr(WorksheetErrorCode::Num))?;
    let canonical = coup_days_accr(pcd, first, num_months, end_flag, fc, basis_)?;

    if settlement <= first {
        let first_date = issue.max(pcd);
        let days = days_between_num(first_date, settlement, basis_)?;
        let mut a = days / canonical;
        // datesAggregate1: walk pcd back toward issue, one quasi-coupon period per step.
        let mut front = pcd;
        while front > issue {
            let ncd_i = front;
            let pcd_i = change_month_flag(front, -num_months, end_flag)
                .ok_or(derr(WorksheetErrorCode::Num))?;
            front = pcd_i;
            if issue <= pcd_i {
                a += if from_issue { 1.0 } else { 0.0 };
            } else {
                // The period that contains `issue`: a partial forward fraction.
                let fd = issue.max(pcd_i);
                let days_i = match basis_ {
                    DayCountBasis::Us30_360 => diff360_us(fd, ncd_i, false)?,
                    _ => days_between_num(fd, ncd_i, basis_)?,
                };
                let coup_days_i = match basis_ {
                    DayCountBasis::Us30_360 => diff360_us(pcd_i, ncd_i, true)?,
                    DayCountBasis::Actual365 => 365.0 / fc,
                    _ => days_between_denum(pcd_i, ncd_i, basis_)?,
                };
                a += days_i / coup_days_i;
            }
        }
        return Ok(par * rate_ / fc * a);
    }

    // Settlement past the first interest date: accrue forward, whole periods counting as 1
    // and the final partial by the canonical length. calc_method TRUE accrues from issue,
    // FALSE from one quasi-coupon period before `first`.
    let accr_start = if from_issue {
        issue
    } else {
        change_month_flag(first, -num_months, end_flag).ok_or(derr(WorksheetErrorCode::Num))?
    };
    let mut p_start = first;
    while p_start > accr_start {
        p_start = change_month_flag(p_start, -num_months, end_flag)
            .ok_or(derr(WorksheetErrorCode::Num))?;
    }
    let mut a = 0.0;
    loop {
        if p_start >= settlement {
            break;
        }
        let p_end = change_month_flag(p_start, num_months, end_flag)
            .ok_or(derr(WorksheetErrorCode::Num))?;
        let ov_start = accr_start.max(p_start);
        let ov_end = settlement.min(p_end);
        if ov_end > ov_start {
            if ov_start == p_start && ov_end == p_end {
                a += 1.0;
            } else {
                a += days_between_num(ov_start, ov_end, basis_)? / canonical;
            }
        }
        p_start = p_end;
    }
    Ok(par * rate_ / fc * a)
}
fn evaln(
    args: &[CalcValue],
    r: &(impl ReferenceSystemProvider + ?Sized),
    m: &FunctionMeta,
    k: impl FnOnce(&[CalcValue]) -> Result<f64, BondCoreEvalError>,
) -> Result<CalcValue, BondCoreEvalError> {
    run_values_only_prepared(
        args,
        r,
        |p| {
            if !m.arity.accepts(p.len()) {
                return Err(arity(m, p.len()));
            }
            Ok(CalcValue::number(k(p)?))
        },
        BondCoreEvalError::Coercion,
    )
}
pub fn eval_accrint_surface(
    args: &[CalcValue],
    r: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BondCoreEvalError> {
    evaln(args, r, &ACCRINT_META, |p| {
        accrint_kernel(
            narg(p, 0)?,
            narg(p, 1)?,
            narg(p, 2)?,
            narg(p, 3)?,
            Some(oarg(p, 4, 1000.0)?),
            narg(p, 5)?,
            Some(oarg(p, 6, 0.0)?),
            Some(obool(p, 7, true)?),
        )
    })
}
pub fn eval_accrintm_surface(
    args: &[CalcValue],
    r: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BondCoreEvalError> {
    evaln(args, r, &ACCRINTM_META, |p| {
        accrintm_kernel(
            narg(p, 0)?,
            narg(p, 1)?,
            narg(p, 2)?,
            Some(oarg(p, 3, 1000.0)?),
            Some(oarg(p, 4, 0.0)?),
        )
    })
}
pub fn eval_duration_surface(
    args: &[CalcValue],
    r: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BondCoreEvalError> {
    evaln(args, r, &DURATION_META, |p| {
        duration_kernel(
            narg(p, 0)?,
            narg(p, 1)?,
            narg(p, 2)?,
            narg(p, 3)?,
            narg(p, 4)?,
            Some(oarg(p, 5, 0.0)?),
        )
    })
}
pub fn eval_mduration_surface(
    args: &[CalcValue],
    r: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BondCoreEvalError> {
    evaln(args, r, &MDURATION_META, |p| {
        mduration_kernel(
            narg(p, 0)?,
            narg(p, 1)?,
            narg(p, 2)?,
            narg(p, 3)?,
            narg(p, 4)?,
            Some(oarg(p, 5, 0.0)?),
        )
    })
}
pub fn eval_price_surface(
    args: &[CalcValue],
    r: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BondCoreEvalError> {
    evaln(args, r, &PRICE_META, |p| {
        price_kernel(
            narg(p, 0)?,
            narg(p, 1)?,
            narg(p, 2)?,
            narg(p, 3)?,
            narg(p, 4)?,
            narg(p, 5)?,
            Some(oarg(p, 6, 0.0)?),
        )
    })
}
pub fn eval_pricemat_surface(
    args: &[CalcValue],
    r: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BondCoreEvalError> {
    evaln(args, r, &PRICEMAT_META, |p| {
        pricemat_kernel(
            narg(p, 0)?,
            narg(p, 1)?,
            narg(p, 2)?,
            narg(p, 3)?,
            narg(p, 4)?,
            Some(oarg(p, 5, 0.0)?),
        )
    })
}
pub fn eval_yield_surface(
    args: &[CalcValue],
    r: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BondCoreEvalError> {
    evaln(args, r, &YIELD_META, |p| {
        yield_kernel(
            narg(p, 0)?,
            narg(p, 1)?,
            narg(p, 2)?,
            narg(p, 3)?,
            narg(p, 4)?,
            narg(p, 5)?,
            Some(oarg(p, 6, 0.0)?),
        )
    })
}
pub fn eval_yielddisc_surface(
    args: &[CalcValue],
    r: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BondCoreEvalError> {
    evaln(args, r, &YIELDDISC_META, |p| {
        yielddisc_kernel(
            narg(p, 0)?,
            narg(p, 1)?,
            narg(p, 2)?,
            narg(p, 3)?,
            Some(oarg(p, 4, 0.0)?),
        )
    })
}
pub fn eval_yieldmat_surface(
    args: &[CalcValue],
    r: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BondCoreEvalError> {
    evaln(args, r, &YIELDMAT_META, |p| {
        yieldmat_kernel(
            narg(p, 0)?,
            narg(p, 1)?,
            narg(p, 2)?,
            narg(p, 3)?,
            narg(p, 4)?,
            Some(oarg(p, 5, 0.0)?),
        )
    })
}
pub fn map_bond_core_error_to_ws(e: &BondCoreEvalError) -> WorksheetErrorCode {
    match e {
        BondCoreEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        BondCoreEvalError::Coercion(CoercionError::WorksheetError(c)) => *c,
        BondCoreEvalError::Coercion(_) => WorksheetErrorCode::Value,
        BondCoreEvalError::Domain(c) => *c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::ReferenceSystemCapabilities;

    fn serial(y: i64, m: i64, d: i64) -> f64 {
        excel_serial_from_ymd(WorkbookDateSystem::System1900, y, m, d).unwrap()
    }
    fn num(n: f64) -> CalcValue {
        CalcValue::number(n)
    }
    struct Dummy;
    impl ReferenceSystemProvider for Dummy {
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
    fn close(a: f64, b: f64, t: f64) {
        assert!((a - b).abs() <= t, "a={a}, b={b}");
    }
    #[test]
    fn yielddisc_bit_exact_vs_excel() {
        // G6 three-way: the prior `(red/price - 1) / yearfrac` form was ~5 ULP off
        // Excel. The faithful `yieldDisc` port `(red - pr)/pr * b / dim` is now
        // bit-exact with live Excel 16.0 b20026 and the F# reference. Value pinned
        // by exact bits.
        let y = yielddisc_kernel(44013.0, 44562.0, 95.0, 100.0, Some(0.0))
            .expect("yielddisc should succeed");
        assert_eq!(y.to_bits(), 0.035_087_719_298_245_61_f64.to_bits());
    }
    #[test]
    fn d360us_rolls_end_day_31_into_next_month() {
        assert_eq!(
            d360us(serial(2023, 11, 15) as i64, serial(2024, 1, 31) as i64),
            Ok(76.0)
        );
    }

    #[test]
    fn d360us_start_at_least_30_collapses_end_in_place() {
        assert_eq!(
            d360us(serial(2024, 1, 30) as i64, serial(2024, 3, 31) as i64),
            Ok(60.0)
        );
    }

    #[test]
    fn accrint_leap_february_and_settlement_after_first_bit_exact() {
        // BUG-FUNC-030 residual closed by the faithful F# accrInt port: act/act and
        // act/365 partial periods crossing leap February, and settlement past the first
        // interest date (which F# itself rejects), all match live Excel 16.0 b20026
        // bit-for-bit. Bits pinned from the G6 three-way ledger.
        let p1 = accrint_kernel(
            serial(2019, 3, 1),
            serial(2020, 9, 1),
            serial(2020, 1, 15),
            0.05,
            Some(1000.0),
            2.0,
            Some(1.0),
            None,
        )
        .unwrap();
        assert_eq!(p1.to_bits(), 0x4045_e000_0000_0000); // 43.75, act/act
        let p3 = accrint_kernel(
            serial(2019, 3, 1),
            serial(2020, 9, 1),
            serial(2020, 1, 15),
            0.05,
            Some(1000.0),
            2.0,
            Some(3.0),
            None,
        )
        .unwrap();
        assert_eq!(p3.to_bits(), 0x4045_d96c_b65b_2d97); // act/365
        let after = accrint_kernel(
            serial(2019, 1, 1),
            serial(2020, 1, 1),
            serial(2020, 6, 1),
            0.05,
            Some(1000.0),
            2.0,
            Some(1.0),
            None,
        )
        .unwrap();
        assert_eq!(after.to_bits(), 0x4051_a9bd_37a6_f4df); // settlement after first_interest
    }

    #[test]
    fn meta_shape() {
        assert_eq!(ACCRINT_META.arity.max, 8);
        assert_eq!(PRICE_META.arity.min, 6);
        assert_eq!(YIELDMAT_META.function_id, "FUNC.YIELDMAT");
    }
    #[test]
    fn price_yield_round_trip() {
        let s = serial(2029, 6, 15);
        let m = serial(2029, 11, 15);
        let p = price_kernel(s, m, 0.0575, 0.065, 100.0, 2.0, Some(0.0)).unwrap();
        close(
            yield_kernel(s, m, 0.0575, p, 100.0, 2.0, Some(0.0)).unwrap(),
            0.065,
            1e-10,
        );
    }
    #[test]
    fn yield_converges_for_well_posed_multi_period_discount_bond() {
        // BUG-FUNC-031: the root-finder probes negative candidate yields (its
        // bracket runs to -frequency), which price_kernel rejected via rate(yld),
        // so a well-posed discount bond returned #NUM!. Solving over pcomp directly
        // (its own yld<=-freq / base<=0 guards handle the domain) fixes it. Excel
        // 16.0 b20026 returns ~0.0862487 for this 3-coupon bond priced at 95.
        let y = yield_kernel(
            serial(2020, 7, 1),
            serial(2022, 1, 1),
            0.05,
            95.0,
            100.0,
            2.0,
            Some(0.0),
        )
        .expect("yield converges for a well-posed discount bond");
        close(y, 0.08624873995, 1e-7);
    }
    #[test]
    fn duration_relation() {
        let s = serial(2024, 3, 15);
        let m = serial(2031, 11, 15);
        let d = duration_kernel(s, m, 0.06, 0.0675, 2.0, Some(0.0)).unwrap();
        let md = mduration_kernel(s, m, 0.06, 0.0675, 2.0, Some(0.0)).unwrap();
        close(md, d / (1.0 + 0.0675 / 2.0), 1e-12);
    }
    #[test]
    fn mat_round_trip() {
        let s = serial(2024, 6, 15);
        let m = serial(2025, 12, 31);
        let i = serial(2024, 1, 1);
        let p = pricemat_kernel(s, m, i, 0.0525, 0.061, Some(1.0)).unwrap();
        close(p, 98.598_113_405_460_48, 1e-12);
        close(
            yieldmat_kernel(s, m, i, 0.0525, p, Some(1.0)).unwrap(),
            0.061,
            1e-12,
        );
    }
    #[test]
    fn accrint_slices() {
        // BUG-FUNC-030 witness: an odd first coupon spanning two quasi-coupon
        // periods must sum over the periods, not interpolate issue->first once.
        // Excel 16.0 build 20026 returns 25 exactly; the old kernel returned 12.5.
        close(
            accrint_kernel(
                serial(2020, 1, 1),
                serial(2021, 1, 1),
                serial(2020, 7, 1),
                0.05,
                Some(1000.0),
                2.0,
                Some(0.0),
                Some(true),
            )
            .unwrap(),
            25.0,
            1e-9,
        );

        // Regular one-period first coupon: calc_method TRUE and FALSE coincide
        // (Excel-verified) because FALSE accrues from one period before
        // first_interest, which equals issue here.
        let i = serial(2024, 1, 1);
        let f = serial(2024, 7, 1);
        let s = serial(2024, 10, 1);
        close(
            accrint_kernel(i, f, s, 0.12, Some(1000.0), 2.0, Some(0.0), Some(true)).unwrap(),
            accrint_kernel(i, f, s, 0.12, Some(1000.0), 2.0, Some(0.0), Some(false)).unwrap(),
            1e-12,
        );

        // Long (multi-period) first coupon: TRUE accrues from issue, FALSE from
        // one period before first_interest, so TRUE strictly exceeds FALSE.
        let (li, lf, ls) = (serial(2018, 1, 1), serial(2020, 1, 1), serial(2020, 7, 15));
        assert!(
            accrint_kernel(li, lf, ls, 0.06, Some(1000.0), 2.0, Some(0.0), Some(true)).unwrap()
                > accrint_kernel(li, lf, ls, 0.06, Some(1000.0), 2.0, Some(0.0), Some(false))
                    .unwrap()
        );

        close(
            accrintm_kernel(i, serial(2024, 10, 1), 0.08, Some(1000.0), Some(3.0)).unwrap(),
            1000.0 * 0.08 * (act(i as i64, serial(2024, 10, 1) as i64) / 365.0),
            1e-12,
        );
    }
    #[test]
    fn surface_and_domain() {
        let r = Dummy;
        assert!(matches!(
            eval_price_surface(
                &[
                    num(serial(2024, 3, 15)),
                    num(serial(2029, 11, 15)),
                    num(0.0575),
                    num(0.065),
                    num(100.0),
                    num(2.0),
                    num(0.0)
                ],
                &r
            )
            .unwrap(),
            value if matches!(value.core(), CoreValue::Number(_))
        ));
        assert_eq!(basis(9.0), Err(derr(WorksheetErrorCode::Num)));
        assert_eq!(
            map_bond_core_error_to_ws(&BondCoreEvalError::ArityMismatch {
                expected_min: 3,
                expected_max: 5,
                actual: 1
            }),
            WorksheetErrorCode::Value
        );
    }
}
