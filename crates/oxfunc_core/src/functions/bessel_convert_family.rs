use crate::coercion::CoercionError;
use crate::function::{
    Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile, FunctionMeta,
    HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{coerce_prepared_to_number, run_values_only_prepared};
use crate::resolver::ReferenceSystemProvider;
use crate::value::CalcValue;
use crate::value::WorksheetErrorCode;

const BESSEL_BASE_META: FunctionMeta = function_spec! {
    function_id: "FUNC.BESSEL_BASE",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::None,
    surface_fec_dependency_profile: FecDependencyProfile::None,
};

pub const BESSELI_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BESSELI",
    ..BESSEL_BASE_META
};

pub const BESSELJ_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BESSELJ",
    ..BESSEL_BASE_META
};

pub const BESSELK_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BESSELK",
    ..BESSEL_BASE_META
};

pub const BESSELY_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BESSELY",
    ..BESSEL_BASE_META
};

#[derive(Debug, Clone, PartialEq)]
pub enum BesselConvertEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Coercion(CoercionError),
    Domain(WorksheetErrorCode),
}

const ACC: f64 = 40.0;
const BIGNO: f64 = 1.0e10;
const BIGNI: f64 = 1.0e-10;
/// Excel's Bessel functions use the Numerical Recipes algorithm with its *truncated*
/// 9-digit `2/π` constant `0.636619772`, not the full-precision `2/π`. Using the exact
/// constant makes OxFunc more accurate than Excel but diverges by up to ~6e11 ULP on
/// BESSELY (BUG-FUNC-024). Match Excel.
///
/// The J/Y coefficient tables and expression groupings below are pinned to Excel's
/// published bits (BUG-FUNC-024, W097 93-case live-Excel oracle grid), not to any
/// printed edition of NR — where they disagree, Excel wins.
const NR_2_OVER_PI: f64 = 0.636_619_772;

fn horner(y: f64, coeffs: &[f64]) -> f64 {
    coeffs.iter().rev().fold(0.0, |acc, coeff| acc * y + coeff)
}

fn arity_error(meta: &FunctionMeta, actual: usize) -> BesselConvertEvalError {
    BesselConvertEvalError::ArityMismatch {
        expected_min: meta.arity.min,
        expected_max: meta.arity.max,
        actual,
    }
}

fn number_arg(args: &[CalcValue], idx: usize) -> Result<f64, BesselConvertEvalError> {
    args.get(idx)
        .ok_or(BesselConvertEvalError::Domain(WorksheetErrorCode::Value))
        .and_then(|value| {
            coerce_prepared_to_number(value).map_err(BesselConvertEvalError::Coercion)
        })
}

fn trunc_order(value: f64) -> Result<i32, BesselConvertEvalError> {
    if !value.is_finite() {
        return Err(BesselConvertEvalError::Domain(WorksheetErrorCode::Value));
    }
    let truncated = value.trunc();
    if truncated < 0.0 || truncated > i32::MAX as f64 {
        return Err(BesselConvertEvalError::Domain(WorksheetErrorCode::Num));
    }
    Ok(truncated as i32)
}

fn ensure_finite(value: f64) -> Result<f64, BesselConvertEvalError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(BesselConvertEvalError::Domain(WorksheetErrorCode::Num))
    }
}

fn bessi0(x: f64) -> f64 {
    let ax = x.abs();
    if ax < 3.75 {
        let y = (x / 3.75).powi(2);
        horner(
            y,
            &[
                1.0,
                3.515_622_9,
                3.089_942_4,
                1.206_749_2,
                0.265_973_2,
                0.036_076_8,
                0.004_581_3,
            ],
        )
    } else {
        let y = 3.75 / ax;
        let poly = horner(
            y,
            &[
                0.398_942_28,
                0.013_285_92,
                0.002_253_19,
                -0.001_575_65,
                0.009_162_81,
                -0.020_577_06,
                0.026_355_37,
                -0.016_476_33,
                0.003_923_77,
            ],
        );
        (ax.exp() / ax.sqrt()) * poly
    }
}

fn bessi1(x: f64) -> f64 {
    let ax = x.abs();
    let ans = if ax < 3.75 {
        let y = (x / 3.75).powi(2);
        ax * horner(
            y,
            &[
                0.5,
                0.878_905_94,
                0.514_988_69,
                0.150_849_34,
                0.026_587_33,
                0.003_015_32,
                0.000_324_11,
            ],
        )
    } else {
        let y = 3.75 / ax;
        let poly = horner(
            y,
            &[
                0.398_942_28,
                -0.039_880_24,
                -0.003_620_18,
                0.001_638_01,
                -0.010_315_55,
                0.022_829_67,
                -0.028_953_12,
                0.017_876_54,
                -0.004_200_59,
            ],
        );
        (ax.exp() / ax.sqrt()) * poly
    };
    if x < 0.0 { -ans } else { ans }
}

fn bessj0(x: f64) -> f64 {
    let ax = x.abs();
    if ax < 8.0 {
        let y = x * x;
        let ans1 = horner(
            y,
            &[
                57_568_490_574.0,
                -13_362_590_354.0,
                651_619_640.7,
                -11_214_424.18,
                77_392.330_17,
                -184.905_245_6,
            ],
        );
        let ans2 = horner(
            y,
            &[
                57_568_490_411.0,
                1_029_532_985.0,
                9_494_680.718,
                59_272.648_53,
                267.853_271_2,
                1.0,
            ],
        );
        ans1 / ans2
    } else {
        let z = 8.0 / ax;
        let y = z * z;
        let xx = ax - 0.785_398_164;
        // Excel's J-side and Y-side asymptotic tables are separately-typed copies
        // with different typos: J0's P0 y^1 coefficient transposes NR's …628627
        // into …628267, while its Q0 keeps NR's -0.934935152e-7 (Y0's copy has
        // …945152 instead — see bessy0). Do not "unify" these tables.
        let ans1 = horner(
            y,
            &[
                1.0,
                -0.001_098_628_267,
                0.000_027_345_104_07,
                -0.000_002_073_370_639,
                0.000_000_209_388_721_1,
            ],
        );
        let ans2 = horner(
            y,
            &[
                -0.015_624_999_95,
                0.000_143_048_876_5,
                -0.000_006_911_147_651,
                0.000_000_762_109_516_1,
                -0.000_000_093_493_515_2,
            ],
        );
        (NR_2_OVER_PI / ax).sqrt() * (xx.cos() * ans1 - z * xx.sin() * ans2)
    }
}

fn bessj1(x: f64) -> f64 {
    let ax = x.abs();
    let ans = if ax < 8.0 {
        let y = x * x;
        let ans1 = x * horner(
            y,
            &[
                72_362_614_232.0,
                -7_895_059_235.0,
                242_396_853.1,
                -2_972_611.439,
                15_704.482_60,
                -30.160_366_06,
            ],
        );
        let ans2 = horner(
            y,
            &[
                144_725_228_442.0,
                2_300_535_178.0,
                18_583_304.74,
                99_447.433_94,
                376.999_139_7,
                1.0,
            ],
        );
        ans1 / ans2
    } else {
        let z = 8.0 / ax;
        let y = z * z;
        let xx = ax - 2.356_194_491;
        // Excel's J1 P-table is six entries: an inserted duplicate of the y^2
        // coefficient shifts NR's remaining terms down a slot (Y1's copy has the
        // plain five-entry NR table — see bessy1). This costs Excel ~8e-6 absolute
        // accuracy near x=8; match Excel, not the math.
        let ans1 = horner(
            y,
            &[
                1.0,
                0.001_831_05,
                -0.000_035_163_964_96,
                -0.000_035_163_964_96,
                0.000_002_457_520_174,
                -0.000_000_240_337_019,
            ],
        );
        // 0.8449199096e-5 = 8.449199096e-6; transcribing it as 8.4…e-5 (10x) drifted
        // BESSELY(10,1) by 4.7e10 ULP through the shared Y1 table.
        let ans2 = horner(
            y,
            &[
                0.046_874_999_95,
                -0.000_200_269_087_3,
                0.000_008_449_199_096,
                -0.000_000_882_289_87,
                0.000_000_105_787_412,
            ],
        );
        (NR_2_OVER_PI / ax).sqrt() * (xx.cos() * ans1 - z * xx.sin() * ans2)
    };
    if x < 0.0 { -ans } else { ans }
}

fn bessy0(x: f64) -> f64 {
    if x < 8.0 {
        let y = x * x;
        let ans1 = horner(
            y,
            &[
                -2_957_821_389.0,
                7_062_834_065.0,
                -512_359_803.6,
                10_879_881.29,
                -86_327.927_57,
                228.462_273_3,
            ],
        );
        let ans2 = horner(
            y,
            &[
                40_076_544_269.0,
                745_249_964.8,
                7_189_466.438,
                47_447.264_70,
                226.103_024_4,
                1.0,
            ],
        );
        // Excel associates the reflection term as c*(J0*ln x); (c*J0)*ln x rounds
        // differently (1 ULP at x=1.5 and x=3).
        ans1 / ans2 + NR_2_OVER_PI * (bessj0(x) * x.ln())
    } else {
        let z = 8.0 / x;
        let y = z * z;
        let xx = x - 0.785_398_164;
        let ans1 = horner(
            y,
            &[
                1.0,
                -0.001_098_628_627,
                0.000_027_345_104_07,
                -0.000_002_073_370_639,
                0.000_000_209_388_721_1,
            ],
        );
        // Excel's Y0 copy of the Q0 table ends in -0.934945152e-7 (…945…), unlike
        // its J0 copy (…935152, the NR value); …935152 here drifts BESSELY(10,0)
        // by 4773 ULP.
        let ans2 = horner(
            y,
            &[
                -0.015_624_999_95,
                0.000_143_048_876_5,
                -0.000_006_911_147_651,
                0.000_000_762_109_516_1,
                -0.000_000_093_494_515_2,
            ],
        );
        (NR_2_OVER_PI / x).sqrt() * (xx.sin() * ans1 + z * xx.cos() * ans2)
    }
}

fn bessy1(x: f64) -> f64 {
    if x < 8.0 {
        let y = x * x;
        let ans1 = x * horner(
            y,
            &[
                -4.900_604_943e12,
                1.275_274_39e12,
                -5.153_438_139e10,
                7.349_264_551e8,
                -4.237_922_726e6,
                8_511.937_935,
            ],
        );
        // The Y1 denominator is degree 6: dropping the 0.3549632885e3 term drifts
        // BESSELY(2.5,1) by 4.9e8 ULP and grows ~x^10 (the BUG-FUNC-024 witness).
        let ans2 = horner(
            y,
            &[
                2.499_580_57e13,
                4.244_419_664e11,
                3.733_650_367e9,
                2.245_904_002e7,
                1.020_426_05e5,
                354.963_288_5,
                1.0,
            ],
        );
        ans1 / ans2 + NR_2_OVER_PI * (bessj1(x) * x.ln() - 1.0 / x)
    } else {
        let z = 8.0 / x;
        let y = z * z;
        let xx = x - 2.356_194_491;
        let ans1 = horner(
            y,
            &[
                1.0,
                0.001_831_05,
                -0.000_035_163_964_96,
                0.000_002_457_520_174,
                -0.000_000_240_337_019,
            ],
        );
        // Same Q1 table as bessj1: 0.8449199096e-5, not 8.4…e-5.
        let ans2 = horner(
            y,
            &[
                0.046_874_999_95,
                -0.000_200_269_087_3,
                0.000_008_449_199_096,
                -0.000_000_882_289_87,
                0.000_000_105_787_412,
            ],
        );
        (NR_2_OVER_PI / x).sqrt() * (xx.sin() * ans1 + z * xx.cos() * ans2)
    }
}

fn bessk0(x: f64) -> f64 {
    if x <= 2.0 {
        let y = x * x / 4.0;
        let poly = horner(
            y,
            &[
                -0.577_215_66,
                0.422_784_20,
                0.230_697_56,
                0.034_885_90,
                0.002_626_98,
                0.000_107_50,
                0.000_007_40,
            ],
        );
        -((x / 2.0).ln()) * bessi0(x) + poly
    } else {
        let y = 2.0 / x;
        let poly = horner(
            y,
            &[
                1.253_314_14,
                -0.078_323_58,
                0.021_895_68,
                -0.010_624_46,
                0.005_878_72,
                -0.002_515_40,
                0.000_532_08,
            ],
        );
        ((-x).exp() / x.sqrt()) * poly
    }
}

fn bessk1(x: f64) -> f64 {
    if x <= 2.0 {
        let y = x * x / 4.0;
        let poly = horner(
            y,
            &[
                1.0,
                0.154_431_44,
                -0.672_785_79,
                -0.181_568_97,
                -0.019_194_02,
                -0.001_104_04,
                -0.000_046_86,
            ],
        );
        (x / 2.0).ln() * bessi1(x) + (poly / x)
    } else {
        let y = 2.0 / x;
        let poly = horner(
            y,
            &[
                1.253_314_14,
                0.234_986_19,
                -0.036_556_20,
                0.015_042_68,
                -0.007_803_53,
                0.003_256_14,
                -0.000_682_45,
            ],
        );
        ((-x).exp() / x.sqrt()) * poly
    }
}

pub fn besseli_kernel(x: f64, order: f64) -> Result<f64, BesselConvertEvalError> {
    if !x.is_finite() {
        return Err(BesselConvertEvalError::Domain(WorksheetErrorCode::Value));
    }
    let n = trunc_order(order)?;
    if n == 0 {
        return ensure_finite(bessi0(x));
    }
    if n == 1 {
        return ensure_finite(bessi1(x));
    }
    if x == 0.0 {
        return Ok(0.0);
    }

    let tox = 2.0 / x.abs();
    let mut bip = 0.0;
    let mut bi = 1.0;
    let mut ans = 0.0;
    let m = 2 * (n + (ACC * n as f64).sqrt() as i32);
    for j in (1..=m).rev() {
        let bim = bip + j as f64 * tox * bi;
        bip = bi;
        bi = bim;
        if bi.abs() > BIGNO {
            ans *= BIGNI;
            bi *= BIGNI;
            bip *= BIGNI;
        }
        if j == n {
            ans = bip;
        }
    }
    ans *= bessi0(x) / bi;
    if x < 0.0 && n % 2 == 1 {
        ans = -ans;
    }
    ensure_finite(ans)
}

pub fn besselj_kernel(x: f64, order: f64) -> Result<f64, BesselConvertEvalError> {
    if !x.is_finite() {
        return Err(BesselConvertEvalError::Domain(WorksheetErrorCode::Value));
    }
    let n = trunc_order(order)?;
    if n == 0 {
        return ensure_finite(bessj0(x));
    }
    if n == 1 {
        return ensure_finite(bessj1(x));
    }
    if x == 0.0 {
        return Ok(0.0);
    }

    let ax = x.abs();
    let mut ans;
    if ax > n as f64 {
        let tox = 2.0 / ax;
        let mut bjm = bessj0(ax);
        let mut bj = bessj1(ax);
        for j in 1..n {
            let bjp = j as f64 * tox * bj - bjm;
            bjm = bj;
            bj = bjp;
        }
        ans = bj;
    } else {
        let tox = 2.0 / ax;
        let m = 2 * ((n + (ACC * n as f64).sqrt() as i32) / 2);
        let mut jsum = false;
        let mut sum = 0.0;
        let mut bjp = 0.0;
        let mut bj = 1.0;
        ans = 0.0;
        for j in (1..=m).rev() {
            let bjm = j as f64 * tox * bj - bjp;
            bjp = bj;
            bj = bjm;
            if bj.abs() > BIGNO {
                bj *= BIGNI;
                bjp *= BIGNI;
                ans *= BIGNI;
                sum *= BIGNI;
            }
            if jsum {
                sum += bj;
            }
            jsum = !jsum;
            if j == n {
                ans = bjp;
            }
        }
        sum = 2.0 * sum - bj;
        ans /= sum;
    }
    if x < 0.0 && n % 2 == 1 {
        ans = -ans;
    }
    ensure_finite(ans)
}

pub fn besselk_kernel(x: f64, order: f64) -> Result<f64, BesselConvertEvalError> {
    if !x.is_finite() {
        return Err(BesselConvertEvalError::Domain(WorksheetErrorCode::Value));
    }
    let n = trunc_order(order)?;
    if x <= 0.0 {
        return Err(BesselConvertEvalError::Domain(WorksheetErrorCode::Num));
    }
    if n == 0 {
        return ensure_finite(bessk0(x));
    }
    if n == 1 {
        return ensure_finite(bessk1(x));
    }

    let tox = 2.0 / x;
    let mut bkm = bessk0(x);
    let mut bk = bessk1(x);
    for j in 1..n {
        let bkp = bkm + j as f64 * tox * bk;
        bkm = bk;
        bk = bkp;
    }
    ensure_finite(bk)
}

pub fn bessely_kernel(x: f64, order: f64) -> Result<f64, BesselConvertEvalError> {
    if !x.is_finite() {
        return Err(BesselConvertEvalError::Domain(WorksheetErrorCode::Value));
    }
    let n = trunc_order(order)?;
    if x <= 0.0 {
        return Err(BesselConvertEvalError::Domain(WorksheetErrorCode::Num));
    }
    if n == 0 {
        return ensure_finite(bessy0(x));
    }
    if n == 1 {
        return ensure_finite(bessy1(x));
    }

    let tox = 2.0 / x;
    let mut bym = bessy0(x);
    let mut by = bessy1(x);
    for j in 1..n {
        let byp = j as f64 * tox * by - bym;
        bym = by;
        by = byp;
    }
    ensure_finite(by)
}

fn eval_numeric(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
    meta: &FunctionMeta,
    kernel: impl FnOnce(&[CalcValue]) -> Result<f64, BesselConvertEvalError>,
) -> Result<CalcValue, BesselConvertEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(arity_error(meta, args.len()));
    }
    run_values_only_prepared(
        args,
        resolver,
        |prepared| kernel(prepared).map(CalcValue::number),
        BesselConvertEvalError::Coercion,
    )
}

pub fn eval_besseli_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BesselConvertEvalError> {
    eval_numeric(args, resolver, &BESSELI_META, |prepared| {
        besseli_kernel(number_arg(prepared, 0)?, number_arg(prepared, 1)?)
    })
}

pub fn eval_besselj_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BesselConvertEvalError> {
    eval_numeric(args, resolver, &BESSELJ_META, |prepared| {
        besselj_kernel(number_arg(prepared, 0)?, number_arg(prepared, 1)?)
    })
}

pub fn eval_besselk_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BesselConvertEvalError> {
    eval_numeric(args, resolver, &BESSELK_META, |prepared| {
        besselk_kernel(number_arg(prepared, 0)?, number_arg(prepared, 1)?)
    })
}

pub fn eval_bessely_surface(
    args: &[CalcValue],
    resolver: &(impl ReferenceSystemProvider + ?Sized),
) -> Result<CalcValue, BesselConvertEvalError> {
    eval_numeric(args, resolver, &BESSELY_META, |prepared| {
        bessely_kernel(number_arg(prepared, 0)?, number_arg(prepared, 1)?)
    })
}

pub fn map_bessel_convert_error_to_ws(error: &BesselConvertEvalError) -> WorksheetErrorCode {
    match error {
        BesselConvertEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        BesselConvertEvalError::Coercion(CoercionError::WorksheetError(code)) => *code,
        BesselConvertEvalError::Coercion(_) => WorksheetErrorCode::Value,
        BesselConvertEvalError::Domain(code) => *code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1.0e-7,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn meta_ids_match_expected_function_ids() {
        assert_eq!(BESSELI_META.function_id, "FUNC.BESSELI");
        assert_eq!(BESSELJ_META.function_id, "FUNC.BESSELJ");
        assert_eq!(BESSELK_META.function_id, "FUNC.BESSELK");
        assert_eq!(BESSELY_META.function_id, "FUNC.BESSELY");
    }

    #[test]
    fn besseli_matches_seed_row() {
        assert_close(
            besseli_kernel(1.5, 1.0).expect("besseli seed should succeed"),
            0.981_666_428,
        );
    }

    #[test]
    fn besselj_matches_seed_row() {
        assert_close(
            besselj_kernel(1.9, 2.0).expect("besselj seed should succeed"),
            0.329_925_829,
        );
    }

    #[test]
    fn besselk_matches_seed_row() {
        assert_close(
            besselk_kernel(1.5, 1.0).expect("besselk seed should succeed"),
            0.277_387_804,
        );
    }

    #[test]
    fn bessely_matches_seed_row() {
        assert_close(
            bessely_kernel(2.5, 1.0).expect("bessely seed should succeed"),
            0.145_918_138,
        );
    }

    #[test]
    fn bessely0_small_x_bit_exact_after_nr_constant() {
        // BUG-FUNC-024: Excel's BESSELY uses NR's truncated 2/π = 0.636619772; with it the
        // order-0 x<8 path matches live Excel 16.0 b20026 bit-for-bit — BESSELY(0.5,0) was
        // 4.3e6 ULP off under full-precision 2/π. Bits from the elem-probe ledger.
        assert_eq!(
            bessely_kernel(0.5, 0.0).unwrap().to_bits(),
            0xbfdc_72fe_b3fe_171f
        );
    }

    #[test]
    fn bessely_matches_live_excel_bits_across_all_lanes() {
        // BUG-FUNC-024: live Excel 16.0 b19929 witnesses from the W097 R-E cell-ref
        // oracle grid (smart-fuzzer/runs/W097-R-E-bessely-cellref), covering every
        // kernel lane: order 0/1 rational (x<8), order 0/1 asymptotic (x>=8), and
        // the upward recurrence for order >= 2 on both sides of the x=8 branch.
        let witnesses: &[(f64, f64, u64)] = &[
            (1.5, 0.0, 0x3fd8_7a0b_0e10_64fe),
            (3.0, 0.0, 0x3fd8_1e4f_8696_66f6),
            (10.0, 0.0, 0x3fac_80ee_66e4_640e),
            (20.0, 0.0, 0x3fb0_0936_d237_7e6e),
            (100.0, 0.0, 0xbfb3_c648_87ab_4ab3),
            (0.5, 1.0, 0xbff7_8b26_a280_973f),
            (2.5, 1.0, 0x3fc2_ad72_0e3e_e754),
            (5.0, 1.0, 0x3fc2_ed2d_eb0c_cd22),
            (10.0, 1.0, 0x3fcf_dfbc_c743_ab01),
            (100.0, 1.0, 0xbf94_dc7a_b5ff_838f),
            (2.5, 2.0, 0xbfd8_67ce_7975_e982),
            (5.0, 3.0, 0x3fc2_b8e1_e56e_51a6),
            (10.0, 5.0, 0x3fc1_54e3_1704_f65c),
            (5.0, 10.0, 0xc039_210d_577f_18c6),
            (100.0, 10.0, 0x3fad_dda2_d78f_9299),
        ];
        for &(x, n, excel_bits) in witnesses {
            let actual = bessely_kernel(x, n).expect("witness should succeed").to_bits();
            assert_eq!(
                actual, excel_bits,
                "BESSELY({x},{n}): local 0x{actual:016x} != Excel 0x{excel_bits:016x}"
            );
        }
    }

    #[test]
    fn besselj_matches_live_excel_bits_across_all_lanes() {
        // BUG-FUNC-024 companion: live Excel 16.0 b19929 witnesses (elem-probe,
        // cell-ref plumbing, 2026-07-02) covering the J-specific asymptotic tables
        // (P0 transposition, six-entry P1) plus upward/downward recurrence lanes.
        // BESSELJ(50,0)/BESSELJ(150,0) are deliberately absent: Excel's COS is
        // 1 ULP off ours at those reduced arguments (large-argument trig stream),
        // which J0 inherits at full weight.
        let witnesses: &[(f64, f64, u64)] = &[
            (8.0, 0.0, 0x3fc5_f8a7_55e1_788c),
            (8.75, 0.0, 0xbf9a_9256_3e96_315c),
            (9.5, 0.0, 0xbfc8_d2a8_3e5a_7208),
            (12.0, 0.0, 0x3fa8_6abb_bb51_85ea),
            (20.0, 0.0, 0x3fc5_6110_6f86_4aa3),
            (100.0, 0.0, 0x3f94_772b_b4df_490f),
            (400.0, 0.0, 0xbfa3_e0e4_e9d2_dd2c),
            (8.0, 1.0, 0x3fce_084d_8f61_6cc8),
            (9.0, 1.0, 0x3fcf_663b_b752_811e),
            (13.0, 1.0, 0xbfb2_005a_9b46_ef4e),
            (25.0, 1.0, 0xbfc0_0b7a_105f_17f2),
            (60.0, 1.0, 0x3fa7_dbbe_4cfb_f0ce),
            (400.0, 1.0, 0xbf82_e303_b9a6_704d),
            (5.0, 2.0, 0x3fa7_d762_1bc7_9bd8),
            (10.0, 2.0, 0x3fd0_4bdc_86d3_3573),
            (20.0, 5.0, 0x3fc3_5987_db6a_c779),
            (100.0, 3.0, 0x3fb3_875c_878f_ce53),
            (10.0, 10.0, 0x3fca_8ee7_9d2f_09d3),
        ];
        for &(x, n, excel_bits) in witnesses {
            let actual = besselj_kernel(x, n).expect("witness should succeed").to_bits();
            assert_eq!(
                actual, excel_bits,
                "BESSELJ({x},{n}): local 0x{actual:016x} != Excel 0x{excel_bits:016x}"
            );
        }
    }

    #[test]
    fn truncates_order_toward_zero() {
        let truncated = besselj_kernel(1.9, 2.9).expect("truncated order should succeed");
        let exact = besselj_kernel(1.9, 2.0).expect("exact order should succeed");
        assert_close(truncated, exact);
    }

    #[test]
    fn negative_order_is_num_error() {
        assert_eq!(
            besseli_kernel(1.0, -1.0),
            Err(BesselConvertEvalError::Domain(WorksheetErrorCode::Num))
        );
    }

    #[test]
    fn besselk_and_bessely_reject_non_positive_x() {
        assert_eq!(
            besselk_kernel(0.0, 1.0),
            Err(BesselConvertEvalError::Domain(WorksheetErrorCode::Num))
        );
        assert_eq!(
            bessely_kernel(-1.0, 1.0),
            Err(BesselConvertEvalError::Domain(WorksheetErrorCode::Num))
        );
    }

    #[test]
    fn besselj_handles_negative_x_parity() {
        let pos = besselj_kernel(2.0, 3.0).expect("positive x should succeed");
        let neg = besselj_kernel(-2.0, 3.0).expect("negative x should succeed");
        assert_close(neg, -pos);
    }
}
