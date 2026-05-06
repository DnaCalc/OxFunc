use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_number, run_values_only_prepared,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

const BESSEL_BASE_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.BESSEL_BASE",
    arity: Arity::exact(2),
    determinism: DeterminismClass::Deterministic,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::None,
    thread_safety: ThreadSafetyClass::SafePure,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
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

fn number_arg(args: &[PreparedArgValue], idx: usize) -> Result<f64, BesselConvertEvalError> {
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
        (std::f64::consts::FRAC_2_PI / ax).sqrt() * (xx.cos() * ans1 - z * xx.sin() * ans2)
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
        let ans2 = horner(
            y,
            &[
                0.046_874_999_95,
                -0.000_200_269_087_3,
                0.000_084_491_990_96,
                -0.000_000_882_289_87,
                0.000_000_105_787_412,
            ],
        );
        (std::f64::consts::FRAC_2_PI / ax).sqrt() * (xx.cos() * ans1 - z * xx.sin() * ans2)
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
        ans1 / ans2 + std::f64::consts::FRAC_2_PI * bessj0(x) * x.ln()
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
        (std::f64::consts::FRAC_2_PI / x).sqrt() * (xx.sin() * ans1 + z * xx.cos() * ans2)
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
        let ans2 = horner(
            y,
            &[
                2.499_580_57e13,
                4.244_419_664e11,
                3.733_650_367e9,
                2.245_904_002e7,
                1.020_426_05e5,
                1.0,
            ],
        );
        ans1 / ans2 + std::f64::consts::FRAC_2_PI * (bessj1(x) * x.ln() - 1.0 / x)
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
        let ans2 = horner(
            y,
            &[
                0.046_874_999_95,
                -0.000_200_269_087_3,
                0.000_084_491_990_96,
                -0.000_000_882_289_87,
                0.000_000_105_787_412,
            ],
        );
        (std::f64::consts::FRAC_2_PI / x).sqrt() * (xx.sin() * ans1 + z * xx.cos() * ans2)
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
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
    meta: &FunctionMeta,
    kernel: impl FnOnce(&[PreparedArgValue]) -> Result<f64, BesselConvertEvalError>,
) -> Result<EvalValue, BesselConvertEvalError> {
    if !meta.arity.accepts(args.len()) {
        return Err(arity_error(meta, args.len()));
    }
    run_values_only_prepared(
        args,
        resolver,
        |prepared| kernel(prepared).map(EvalValue::Number),
        BesselConvertEvalError::Coercion,
    )
}

pub fn eval_besseli_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, BesselConvertEvalError> {
    eval_numeric(args, resolver, &BESSELI_META, |prepared| {
        besseli_kernel(number_arg(prepared, 0)?, number_arg(prepared, 1)?)
    })
}

pub fn eval_besselj_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, BesselConvertEvalError> {
    eval_numeric(args, resolver, &BESSELJ_META, |prepared| {
        besselj_kernel(number_arg(prepared, 0)?, number_arg(prepared, 1)?)
    })
}

pub fn eval_besselk_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, BesselConvertEvalError> {
    eval_numeric(args, resolver, &BESSELK_META, |prepared| {
        besselk_kernel(number_arg(prepared, 0)?, number_arg(prepared, 1)?)
    })
}

pub fn eval_bessely_surface(
    args: &[CallArgValue],
    resolver: &(impl ReferenceResolver + ?Sized),
) -> Result<EvalValue, BesselConvertEvalError> {
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
