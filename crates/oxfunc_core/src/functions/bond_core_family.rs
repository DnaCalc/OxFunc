use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::locale_format::{WorkbookDateSystem, excel_serial_from_ymd, ymd_from_excel_serial};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

const BASE: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BOND_CORE_BASE",
    arity: Arity::exact(1),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::RefOnly,
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
fn narg(a: &[PreparedArgValue], i: usize) -> Result<f64, BondCoreEvalError> {
    a.get(i)
        .ok_or(derr(WorksheetErrorCode::Value))
        .and_then(|v| coerce_prepared_to_number(v).map_err(BondCoreEvalError::Coercion))
}
fn oarg(a: &[PreparedArgValue], i: usize, d: f64) -> Result<f64, BondCoreEvalError> {
    match a.get(i) {
        None | Some(PreparedArgValue::MissingArg) => Ok(d),
        Some(v) => coerce_prepared_to_number(v).map_err(BondCoreEvalError::Coercion),
    }
}
fn obool(a: &[PreparedArgValue], i: usize, d: bool) -> Result<bool, BondCoreEvalError> {
    match a.get(i) {
        None | Some(PreparedArgValue::MissingArg) => Ok(d),
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
    let (sy, sm, mut sd) = ymd_from_excel_serial(WorkbookDateSystem::System1900, s as f64)
        .ok_or(derr(WorksheetErrorCode::Value))?;
    let (ey, em, mut ed) = ymd_from_excel_serial(WorkbookDateSystem::System1900, e as f64)
        .ok_or(derr(WorksheetErrorCode::Value))?;
    let sfl = sm == 2 && sd == dim(sy, sm);
    let efl = em == 2 && ed == dim(ey, em);
    if sd == 31 || sfl {
        sd = 30;
    }
    if ed == 31 {
        ed = if sd < 30 { 1 } else { 30 };
    }
    if efl && sfl {
        ed = 30;
    }
    Ok(((ey - sy) * 360 + (em - sm) * 30 + (ed - sd)) as f64)
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
    let lowp = fun(low)?;
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
        if (mp - target).abs() <= 1e-12 {
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
    let future = 100.0 + 100.0 * rate_ * yf(issue, maturity, basis_)?;
    let accrued = 100.0 * rate_ * yf(issue, settlement, basis_)?;
    let den = 1.0 + yld * yf(settlement, maturity, basis_)?;
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
    let future = 100.0 + 100.0 * rate_ * yf(issue, maturity, basis_)?;
    let accrued = 100.0 * rate_ * yf(issue, settlement, basis_)?;
    let frac = yf(settlement, maturity, basis_)?;
    Ok((future / (price + accrued) - 1.0) / frac)
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
    Ok((red / price - 1.0) / yf(settlement, maturity, basis_)?)
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
    solve(price, c.frequency, |cand| {
        price_kernel(settlement, maturity, rate_, cand, red, frequency, basis_)
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
    let calc = calc_method.unwrap_or(true);
    if !(issue < first && issue < settlement) {
        return Err(derr(WorksheetErrorCode::Num));
    }
    let coup = par * rate_ / f as f64;
    if settlement <= first {
        let den = dd(issue, first, basis_)?;
        return Ok(coup * dd(issue, settlement, basis_)? / den);
    }
    let mut total = if calc { coup } else { 0.0 };
    let m = 12 / f;
    let mut prev = first;
    let mut next = addm(prev, m).ok_or(derr(WorksheetErrorCode::Num))?;
    while settlement > next {
        total += coup;
        prev = next;
        next = addm(prev, m).ok_or(derr(WorksheetErrorCode::Num))?;
    }
    Ok(total + coup * dd(prev, settlement, basis_)? / dc(prev, next, basis_, f)?)
}
fn evaln(
    args: &[CallArgValue],
    r: &impl ReferenceResolver,
    m: &FunctionMeta,
    k: impl FnOnce(&[PreparedArgValue]) -> Result<f64, BondCoreEvalError>,
) -> Result<EvalValue, BondCoreEvalError> {
    run_values_only_prepared(
        args,
        r,
        |p| {
            if !m.arity.accepts(p.len()) {
                return Err(arity(m, p.len()));
            }
            Ok(EvalValue::Number(k(p)?))
        },
        BondCoreEvalError::Coercion,
    )
}
pub fn eval_accrint_surface(
    args: &[CallArgValue],
    r: &impl ReferenceResolver,
) -> Result<EvalValue, BondCoreEvalError> {
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
    args: &[CallArgValue],
    r: &impl ReferenceResolver,
) -> Result<EvalValue, BondCoreEvalError> {
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
    args: &[CallArgValue],
    r: &impl ReferenceResolver,
) -> Result<EvalValue, BondCoreEvalError> {
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
    args: &[CallArgValue],
    r: &impl ReferenceResolver,
) -> Result<EvalValue, BondCoreEvalError> {
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
    args: &[CallArgValue],
    r: &impl ReferenceResolver,
) -> Result<EvalValue, BondCoreEvalError> {
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
    args: &[CallArgValue],
    r: &impl ReferenceResolver,
) -> Result<EvalValue, BondCoreEvalError> {
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
    args: &[CallArgValue],
    r: &impl ReferenceResolver,
) -> Result<EvalValue, BondCoreEvalError> {
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
    args: &[CallArgValue],
    r: &impl ReferenceResolver,
) -> Result<EvalValue, BondCoreEvalError> {
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
    args: &[CallArgValue],
    r: &impl ReferenceResolver,
) -> Result<EvalValue, BondCoreEvalError> {
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
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{CallArgValue, EvalValue, ReferenceLike};
    fn serial(y: i64, m: i64, d: i64) -> f64 {
        excel_serial_from_ymd(WorkbookDateSystem::System1900, y, m, d).unwrap()
    }
    fn num(n: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(n))
    }
    struct Dummy;
    impl ReferenceResolver for Dummy {
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
    fn close(a: f64, b: f64, t: f64) {
        assert!((a - b).abs() <= t, "a={a}, b={b}");
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
        close(
            yieldmat_kernel(s, m, i, 0.0525, p, Some(1.0)).unwrap(),
            0.061,
            1e-12,
        );
    }
    #[test]
    fn accrint_slices() {
        let i = serial(2024, 1, 1);
        let f = serial(2024, 7, 1);
        let s = serial(2024, 10, 1);
        assert!(
            accrint_kernel(i, f, s, 0.12, Some(1000.0), 2.0, Some(0.0), Some(true)).unwrap()
                > accrint_kernel(i, f, s, 0.12, Some(1000.0), 2.0, Some(0.0), Some(false)).unwrap()
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
            EvalValue::Number(_)
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
