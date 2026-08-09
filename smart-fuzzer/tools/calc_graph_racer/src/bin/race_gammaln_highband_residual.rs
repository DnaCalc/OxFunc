//! Offline W109 G3-02 GAMMALN high-band residual scorer.
//!
//! This is research-only tooling. It consumes an already-captured worksheet
//! answer bank and races clean-room arithmetic graphs assembled from the
//! public fdlibm Stirling coefficients and the previously identified x87
//! FYL2X worksheet-LN primitive. It never starts Excel or uses COM.

use oxfunc_core::excel_numeric::research as rx;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::sync::OnceLock;

const LS2PI: f64 = f64::from_bits(0x3FED_67F1_C864_BEB5);
const W: [f64; 6] = [
    f64::from_bits(0xBF5A_B89D_0B9E_43E4),
    f64::from_bits(0x3F4B_67BA_4CDA_D5D1),
    f64::from_bits(0xBF43_80CB_8C0F_E741),
    f64::from_bits(0x3F4A_019F_98CF_38B6),
    f64::from_bits(0xBF66_C16C_16B0_2E5C),
    f64::from_bits(0x3FB5_5555_5555_553B),
];

const BERNOULLI: [f64; 6] = [
    -691.0 / 360_360.0,
    1.0 / 1188.0,
    -1.0 / 1680.0,
    1.0 / 1260.0,
    -1.0 / 360.0,
    1.0 / 12.0,
];

const CEPHES_A5: [f64; 5] = [
    8.11614167470508450300e-4,
    -5.95061904284301438324e-4,
    7.93650340457716943945e-4,
    -2.77777777730099687205e-3,
    8.33333333333331927722e-2,
];

#[derive(Deserialize)]
struct AnswerBank {
    witnesses: Vec<Witness>,
}

#[derive(Deserialize)]
struct Witness {
    id: String,
    args: Vec<String>,
    expected_bits: String,
}

#[derive(Clone)]
struct Row {
    id: String,
    x: f64,
    expected: u64,
}

#[derive(Clone, Copy, Debug)]
enum BinaryKind {
    Native,
    Dr,
    OneStore,
    Fma,
}

#[derive(Clone, Copy, Debug)]
struct Dd {
    hi: f64,
    lo: f64,
}

impl Dd {
    fn from_f64(value: f64) -> Self {
        Self { hi: value, lo: 0.0 }
    }

    fn renorm(hi: f64, lo: f64) -> Self {
        let sum = hi + lo;
        let err = lo - (sum - hi);
        Self { hi: sum, lo: err }
    }

    fn add(self, other: Self) -> Self {
        let sum = self.hi + other.hi;
        let bb = sum - self.hi;
        let err = (self.hi - (sum - bb)) + (other.hi - bb) + self.lo + other.lo;
        Self::renorm(sum, err)
    }

    fn neg(self) -> Self {
        Self {
            hi: -self.hi,
            lo: -self.lo,
        }
    }

    fn sub(self, other: Self) -> Self {
        self.add(other.neg())
    }

    fn mul(self, other: Self) -> Self {
        let product = self.hi * other.hi;
        let err = self.hi.mul_add(other.hi, -product)
            + self.hi * other.lo
            + self.lo * other.hi
            + self.lo * other.lo;
        Self::renorm(product, err)
    }

    fn div(self, other: Self) -> Self {
        let q1 = self.hi / other.hi;
        let r1 = self.sub(other.mul(Self::from_f64(q1)));
        let q2 = r1.hi / other.hi;
        let r2 = r1.sub(other.mul(Self::from_f64(q2)));
        let q3 = r2.hi / other.hi;
        Self::from_f64(q1)
            .add(Self::from_f64(q2))
            .add(Self::from_f64(q3))
    }

    fn scale(self, value: f64) -> Self {
        self.mul(Self::from_f64(value))
    }

    fn to_ext(self) -> rx::Ext80 {
        rx::ext_add(
            &rx::ext_from_f64(self.hi),
            &rx::ext_from_f64(self.lo),
            rx::CW_PC64_RN,
        )
    }
}

fn dd_log_reduced(m: f64) -> Dd {
    let one = Dd::from_f64(1.0);
    let mm = Dd::from_f64(m);
    let z = mm.sub(one).div(mm.add(one));
    let z2 = z.mul(z);
    let mut term = z;
    let mut sum = z;
    let mut denominator = 3_u32;
    loop {
        term = term.mul(z2);
        let add = term.scale(1.0 / denominator as f64);
        sum = sum.add(add);
        if add.hi.abs() < 1.0e-38 || denominator > 1001 {
            break;
        }
        denominator += 2;
    }
    sum.scale(2.0)
}

fn dd_log(x: f64) -> Dd {
    static LN2: OnceLock<Dd> = OnceLock::new();
    let ln2 = *LN2.get_or_init(|| dd_log_reduced(2.0));
    let bits = x.to_bits();
    let mut exponent = ((bits >> 52) & 0x7ff) as i32 - 1023;
    let mut mantissa = f64::from_bits((bits & 0x000f_ffff_ffff_ffff) | (1023_u64 << 52));
    if mantissa > std::f64::consts::SQRT_2 {
        mantissa *= 0.5;
        exponent += 1;
    }
    dd_log_reduced(mantissa).add(ln2.scale(exponent as f64))
}

#[cfg(windows)]
type CLog = unsafe extern "C" fn(f64) -> f64;

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryA(name: *const u8) -> *mut core::ffi::c_void;
    fn GetProcAddress(module: *mut core::ffi::c_void, name: *const u8) -> *mut core::ffi::c_void;
}

#[cfg(windows)]
fn legacy_log_functions() -> &'static Vec<(String, CLog)> {
    static LOGS: OnceLock<Vec<(String, CLog)>> = OnceLock::new();
    LOGS.get_or_init(|| {
        let mut out = Vec::new();
        for dll in [
            "msvcrt.dll",
            "msvcr100.dll",
            "msvcr110.dll",
            "msvcr120.dll",
            "ucrtbase.dll",
        ] {
            let mut dll_name = dll.as_bytes().to_vec();
            dll_name.push(0);
            // SAFETY: the documented Windows loader API is called with
            // NUL-terminated static names. The module is intentionally kept
            // loaded for the process lifetime so the function pointer remains
            // valid. `log(double)` is the documented CRT C signature.
            unsafe {
                let module = LoadLibraryA(dll_name.as_ptr());
                if module.is_null() {
                    continue;
                }
                let address = GetProcAddress(module, c"log".as_ptr().cast());
                if !address.is_null() {
                    let function: CLog = core::mem::transmute(address);
                    out.push((dll.trim_end_matches(".dll").to_string(), function));
                }
            }
        }
        out
    })
}

#[cfg(windows)]
fn legacy_tgamma_functions() -> &'static Vec<(String, CLog)> {
    static FUNCTIONS: OnceLock<Vec<(String, CLog)>> = OnceLock::new();
    FUNCTIONS.get_or_init(|| {
        let mut out = Vec::new();
        for dll in [
            "msvcrt.dll",
            "msvcr100.dll",
            "msvcr110.dll",
            "msvcr120.dll",
            "ucrtbase.dll",
        ] {
            let mut dll_name = dll.as_bytes().to_vec();
            dll_name.push(0);
            unsafe {
                let module = LoadLibraryA(dll_name.as_ptr());
                if module.is_null() {
                    continue;
                }
                let address = GetProcAddress(module, c"tgamma".as_ptr().cast());
                if !address.is_null() {
                    let function: CLog = core::mem::transmute(address);
                    out.push((dll.trim_end_matches(".dll").to_string(), function));
                }
            }
        }
        out
    })
}

fn parse_hex(text: &str) -> u64 {
    u64::from_str_radix(text.trim_start_matches("0x"), 16).unwrap()
}

fn ordered(bits: u64) -> i128 {
    if bits >> 63 != 0 {
        -((bits - (1_u64 << 63)) as i128)
    } else {
        bits as i128
    }
}

fn ulp_delta(got: u64, expected: u64) -> i128 {
    ordered(got) - ordered(expected)
}

fn ext_ln(x: f64) -> rx::Ext80 {
    rx::ext_fyl2x(&rx::ext_ln2(), &rx::ext_from_f64(x), rx::CW_PC64_RN)
}

fn dr_add(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_add(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}

fn dr_sub(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_sub(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}

fn dr_mul(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_mul(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}

fn dr_div(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_div(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}

fn one_store_mul_add(a: f64, b: f64, c: f64) -> f64 {
    let product = rx::ext_mul(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN);
    rx::ext_to_f64(
        &rx::ext_add(&product, &rx::ext_from_f64(c), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}

fn mul_add(a: f64, b: f64, c: f64, kind: BinaryKind) -> f64 {
    match kind {
        BinaryKind::Native => a * b + c,
        BinaryKind::Dr => dr_add(dr_mul(a, b), c),
        BinaryKind::OneStore => one_store_mul_add(a, b, c),
        BinaryKind::Fma => a.mul_add(b, c),
    }
}

fn q_graphs(x: f64) -> Vec<(String, f64)> {
    let ln_rn = rx::excel_ln(x);
    let ln_ext = ext_ln(x);
    let ln_dd = dd_log(x);
    let mut out = Vec::new();

    for (source, ln_value) in [("x87", ln_rn), ("std-ucrt", x.ln()), ("libm", libm::log(x))] {
        out.push((
            format!("q:sse-cephes-{source}"),
            (x - 0.5) * ln_value - x + LS2PI,
        ));
        out.push((
            format!("q:dr-each-cephes-{source}"),
            dr_add(dr_sub(dr_mul(x - 0.5, ln_value), x), LS2PI),
        ));
        for bump in -4_i8..=4 {
            if bump == 0 {
                continue;
            }
            let bumped = if bump < 0 {
                f64::from_bits(ln_value.to_bits() - bump.unsigned_abs() as u64)
            } else {
                f64::from_bits(ln_value.to_bits() + bump as u64)
            };
            out.push((
                format!("q:dr-each-cephes-{source}-lnbump{bump:+}"),
                dr_add(dr_sub(dr_mul(x - 0.5, bumped), x), LS2PI),
            ));
        }
    }
    #[cfg(windows)]
    for (source, log_fn) in legacy_log_functions() {
        // SAFETY: function pointers were resolved from the documented CRT
        // `log(double)` export and their modules remain loaded.
        let ln_value = unsafe { log_fn(x) };
        out.push((
            format!("q:sse-cephes-{source}"),
            (x - 0.5) * ln_value - x + LS2PI,
        ));
        out.push((
            format!("q:dr-each-cephes-{source}"),
            dr_add(dr_sub(dr_mul(x - 0.5, ln_value), x), LS2PI),
        ));
    }

    out.push(("q:sse-cephes".into(), (x - 0.5) * ln_rn - x + LS2PI));
    out.push((
        "q:dr-each-cephes".into(),
        dr_add(dr_sub(dr_mul(x - 0.5, ln_rn), x), LS2PI),
    ));
    out.push((
        "q:sse-xlogminus1".into(),
        x * (ln_rn - 1.0) - 0.5 * ln_rn + LS2PI,
    ));
    out.push((
        "q:dr-xlogminus1".into(),
        dr_add(
            dr_sub(dr_mul(x, dr_sub(ln_rn, 1.0)), dr_mul(0.5, ln_rn)),
            LS2PI,
        ),
    ));
    out.push((
        "q:sse-fdlibm".into(),
        (x - 0.5) * (ln_rn - 1.0) + (LS2PI - 0.5),
    ));
    out.push((
        "q:dr-fdlibm".into(),
        dr_add(dr_mul(x - 0.5, dr_sub(ln_rn, 1.0)), LS2PI - 0.5),
    ));

    // Cephes q with PC=64 arithmetic and every explicit store mask. Both a
    // stored and retained FYL2X output are public x87 realizations.
    for retain_log in [false, true] {
        for mask in 0_u8..8 {
            let lg = if retain_log {
                ln_ext
            } else {
                rx::ext_from_f64(ln_rn)
            };
            let mut q = rx::ext_mul(&rx::ext_from_f64(x - 0.5), &lg, rx::CW_PC64_RN);
            if mask & 1 != 0 {
                q = rx::ext_from_f64(rx::ext_to_f64(&q, rx::CW_PC64_RN));
            }
            q = rx::ext_sub(&q, &rx::ext_from_f64(x), rx::CW_PC64_RN);
            if mask & 2 != 0 {
                q = rx::ext_from_f64(rx::ext_to_f64(&q, rx::CW_PC64_RN));
            }
            q = rx::ext_add(&q, &rx::ext_from_f64(LS2PI), rx::CW_PC64_RN);
            if mask & 4 != 0 {
                q = rx::ext_from_f64(rx::ext_to_f64(&q, rx::CW_PC64_RN));
            }
            out.push((
                format!(
                    "q:pc64-cephes-log{}-mask{mask:03b}",
                    if retain_log { "ext" } else { "rn" }
                ),
                rx::ext_to_f64(&q, rx::CW_PC64_RN),
            ));
        }
    }

    // Correct-log double-double control. This is a clean-room atanh-series
    // implementation, retained through one PC=64 x87 load/add. It answers
    // whether sub-binary64 log bits (rather than a stored-log provider) can
    // reach the worksheet target.
    for mask in 0_u8..8 {
        let mut q = rx::ext_mul(&rx::ext_from_f64(x - 0.5), &ln_dd.to_ext(), rx::CW_PC64_RN);
        if mask & 1 != 0 {
            q = rx::ext_from_f64(rx::ext_to_f64(&q, rx::CW_PC64_RN));
        }
        q = rx::ext_sub(&q, &rx::ext_from_f64(x), rx::CW_PC64_RN);
        if mask & 2 != 0 {
            q = rx::ext_from_f64(rx::ext_to_f64(&q, rx::CW_PC64_RN));
        }
        q = rx::ext_add(&q, &rx::ext_from_f64(LS2PI), rx::CW_PC64_RN);
        if mask & 4 != 0 {
            q = rx::ext_from_f64(rx::ext_to_f64(&q, rx::CW_PC64_RN));
        }
        out.push((
            format!("q:pc64-cephes-logdd-mask{mask:03b}"),
            rx::ext_to_f64(&q, rx::CW_PC64_RN),
        ));
    }

    // PC=53 controls. These are not SSE aliases because the explicit final
    // store can still expose x87 double-rounding at underflow boundaries.
    for retain_log in [false, true] {
        let lg = if retain_log {
            ln_ext
        } else {
            rx::ext_from_f64(ln_rn)
        };
        let q = rx::ext_mul(&rx::ext_from_f64(x - 0.5), &lg, rx::CW_PC53_RN);
        let q = rx::ext_sub(&q, &rx::ext_from_f64(x), rx::CW_PC53_RN);
        let q = rx::ext_add(&q, &rx::ext_from_f64(LS2PI), rx::CW_PC53_RN);
        out.push((
            format!("q:pc53-cephes-log{}", if retain_log { "ext" } else { "rn" }),
            rx::ext_to_f64(&q, rx::CW_PC64_RN),
        ));
    }
    out
}

fn correction(x: f64, z_kind: u8, y_kind: u8, step_code: u16, corr_kind: u8) -> f64 {
    let z = match z_kind {
        0 => 1.0 / x,
        1 => dr_div(1.0, x),
        _ => rx::ext_to_f64(
            &rx::ext_div(&rx::ext_from_f64(1.0), &rx::ext_from_f64(x), rx::CW_PC53_RN),
            rx::CW_PC64_RN,
        ),
    };
    let y = match y_kind {
        0 => z * z,
        1 => dr_mul(z, z),
        _ => z.mul_add(z, 0.0),
    };
    let mut code = step_code;
    let mut w = W[0];
    for coefficient in W.iter().skip(1) {
        let kind = match code & 3 {
            0 => BinaryKind::Native,
            1 => BinaryKind::Dr,
            2 => BinaryKind::OneStore,
            _ => BinaryKind::Fma,
        };
        code >>= 2;
        w = mul_add(w, y, *coefficient, kind);
    }
    match corr_kind {
        0 => z * w,
        1 => dr_mul(z, w),
        2 => z.mul_add(w, 0.0),
        _ => w / x,
    }
}

fn native_tail(x: f64, coefficients: &[f64], divide: bool) -> f64 {
    let z = 1.0 / x;
    let y = z * z;
    let mut w = coefficients[0];
    for coefficient in coefficients.iter().skip(1) {
        w = w * y + coefficient;
    }
    if divide { w / x } else { z * w }
}

fn final_add(q: f64, corr: f64, kind: u8) -> f64 {
    match kind {
        0 => q + corr,
        1 => dr_add(q, corr),
        _ => q.mul_add(1.0, corr),
    }
}

fn base_high_gammaln(x: f64) -> f64 {
    let q = dr_add(dr_sub(dr_mul(x - 0.5, rx::excel_ln(x)), x), LS2PI);
    final_add(q, correction(x, 0, 0, 0, 0), 1)
}

fn combined_specials(x: f64) -> Vec<(String, f64)> {
    let ln_rn = rx::excel_ln(x);
    let corr = correction(x, 0, 0, 0, 0);
    let product = dr_mul(x - 0.5, ln_rn);
    let sub = dr_sub(product, x);
    let mut out = vec![
        (
            "combine:dr-sub+(ls+corr)-native".to_string(),
            dr_add(sub, LS2PI + corr),
        ),
        (
            "combine:dr-sub+(ls+corr)-dr".to_string(),
            dr_add(sub, dr_add(LS2PI, corr)),
        ),
        (
            "combine:(dr-sub+corr)+ls-dr".to_string(),
            dr_add(dr_add(sub, corr), LS2PI),
        ),
        (
            "combine:(dr-sub+ls)+corr-native".to_string(),
            dr_add(sub, LS2PI) + corr,
        ),
    ];

    let log_inputs = [
        ("x87rn", rx::ext_from_f64(ln_rn)),
        ("x87ext", ext_ln(x)),
        ("ddext", dd_log(x).to_ext()),
    ];
    for (log_name, log_ext) in log_inputs {
        for mask in 0_u8..8 {
            let mut value = rx::ext_mul(&rx::ext_from_f64(x - 0.5), &log_ext, rx::CW_PC64_RN);
            if mask & 1 != 0 {
                value = rx::ext_from_f64(rx::ext_to_f64(&value, rx::CW_PC64_RN));
            }
            value = rx::ext_sub(&value, &rx::ext_from_f64(x), rx::CW_PC64_RN);
            if mask & 2 != 0 {
                value = rx::ext_from_f64(rx::ext_to_f64(&value, rx::CW_PC64_RN));
            }
            value = rx::ext_add(&value, &rx::ext_from_f64(LS2PI), rx::CW_PC64_RN);
            if mask & 4 != 0 {
                value = rx::ext_from_f64(rx::ext_to_f64(&value, rx::CW_PC64_RN));
            }
            value = rx::ext_add(&value, &rx::ext_from_f64(corr), rx::CW_PC64_RN);
            out.push((
                format!("combine:pc64-{log_name}-qmask{mask:03b}-corr-cont"),
                rx::ext_to_f64(&value, rx::CW_PC64_RN),
            ));
        }
    }
    out
}

fn public_tail_specials(x: f64) -> Vec<(String, f64)> {
    let ln_rn = rx::excel_ln(x);
    let sub = dr_sub(dr_mul(x - 0.5, ln_rn), x);
    let q = dr_add(sub, LS2PI);
    let families: [(&str, &[f64]); 3] = [
        ("fdlibm", &W),
        ("bernoulli", &BERNOULLI),
        ("cephes-a5", &CEPHES_A5),
    ];
    let mut out = Vec::new();
    for (name, coefficients) in families {
        for divide in [false, true] {
            let corr = native_tail(x, coefficients, divide);
            out.push((
                format!(
                    "tailpub:{name}-{}-native",
                    if divide { "div" } else { "mul" }
                ),
                q + corr,
            ));
            out.push((
                format!("tailpub:{name}-{}-dr", if divide { "div" } else { "mul" }),
                dr_add(q, corr),
            ));
            out.push((
                format!(
                    "tailpub:{name}-{}-prels",
                    if divide { "div" } else { "mul" }
                ),
                dr_add(dr_add(sub, corr), LS2PI),
            ));
            out.push((
                format!("tailpub:{name}-{}-rhs", if divide { "div" } else { "mul" }),
                dr_add(sub, dr_add(LS2PI, corr)),
            ));
        }
    }
    out
}

fn state_op(a: rx::Ext80, b: rx::Ext80, kind: u8, op: u8) -> rx::Ext80 {
    let cw = if kind == 0 {
        rx::CW_PC53_RN
    } else {
        rx::CW_PC64_RN
    };
    let raw = match op {
        0 => rx::ext_add(&a, &b, cw),
        1 => rx::ext_sub(&a, &b, cw),
        2 => rx::ext_mul(&a, &b, cw),
        _ => rx::ext_div(&a, &b, cw),
    };
    if kind == 1 {
        rx::ext_from_f64(rx::ext_to_f64(&raw, rx::CW_PC64_RN))
    } else {
        raw
    }
}

fn trit(code: &mut u32) -> u8 {
    let value = (*code % 3) as u8;
    *code /= 3;
    value
}

fn full_state_graph(x: f64, mut code: u32, log_kind: u8) -> f64 {
    let log = match log_kind {
        0 => rx::ext_from_f64(rx::excel_ln(x)),
        1 => ext_ln(x),
        _ => dd_log(x).to_ext(),
    };
    let mut q = state_op(rx::ext_from_f64(x - 0.5), log, trit(&mut code), 2);
    q = state_op(q, rx::ext_from_f64(x), trit(&mut code), 1);
    q = state_op(q, rx::ext_from_f64(LS2PI), trit(&mut code), 0);

    let z = state_op(
        rx::ext_from_f64(1.0),
        rx::ext_from_f64(x),
        trit(&mut code),
        3,
    );
    let y = state_op(z, z, trit(&mut code), 2);
    let mut w = rx::ext_from_f64(W[0]);
    for coefficient in W.iter().skip(1) {
        let kind = trit(&mut code);
        w = state_op(w, y, kind, 2);
        w = state_op(w, rx::ext_from_f64(*coefficient), kind, 0);
    }
    let corr = state_op(z, w, trit(&mut code), 2);
    let result = state_op(q, corr, trit(&mut code), 0);
    rx::ext_to_f64(&result, rx::CW_PC64_RN)
}

#[derive(Clone)]
struct Score {
    name: String,
    exact: usize,
    worst: i128,
    sum: i128,
    misses: Vec<(String, f64, i128, u64, u64)>,
}

fn score<F>(rows: &[Row], name: String, mut f: F) -> Score
where
    F: FnMut(&Row) -> f64,
{
    let mut exact = 0;
    let mut worst = 0;
    let mut sum = 0;
    let mut misses = Vec::new();
    for row in rows {
        let got = f(row).to_bits();
        let d = ulp_delta(got, row.expected);
        if d == 0 {
            exact += 1;
        } else {
            misses.push((row.id.clone(), row.x, d, got, row.expected));
        }
        worst = worst.max(d.abs());
        sum += d.abs();
    }
    Score {
        name,
        exact,
        worst,
        sum,
        misses,
    }
}

fn rank(scores: &mut [Score]) {
    scores.sort_by_key(|s| (std::cmp::Reverse(s.exact), s.worst, s.sum));
}

fn print_score(score: &Score, n: usize) {
    println!(
        "{}: {}/{} exact worst={} sum={}",
        score.name, score.exact, n, score.worst, score.sum
    );
    for (id, x, d, got, expected) in &score.misses {
        println!("  {id:18} x={x:.17} delta={d:+} got=0x{got:016x} want=0x{expected:016x}");
    }
}

fn run_state_search(rows: &[Row], residual_ids: &[String]) {
    const GRAPH_COUNT: u32 = 531_441; // 3^12
    let residuals: Vec<&Row> = residual_ids
        .iter()
        .filter_map(|id| rows.iter().find(|row| &row.id == id))
        .collect();
    println!(
        "\nfull PC53/PC64-DR/PC64-cont state search: graphs={} log_sources=3 residuals={}",
        GRAPH_COUNT,
        residuals.len()
    );
    for log_kind in 0_u8..3 {
        let log_name = ["x87-rn", "x87-ext", "dd-ext"][log_kind as usize];
        let mut each_hits = vec![0_usize; residuals.len()];
        let mut all_hits = 0_usize;
        let mut top = Vec::<Score>::new();
        for code in 0_u32..GRAPH_COUNT {
            let hit: Vec<bool> = residuals
                .iter()
                .map(|row| full_state_graph(row.x, code, log_kind).to_bits() == row.expected)
                .collect();
            for (index, value) in hit.iter().enumerate() {
                if *value {
                    each_hits[index] += 1;
                }
            }
            if hit.iter().all(|value| *value) {
                all_hits += 1;
            }
            if !hit.iter().any(|value| *value) {
                continue;
            }
            let candidate = score(rows, format!("state:{log_name}:code={code}"), |row| {
                full_state_graph(row.x, code, log_kind)
            });
            top.push(candidate);
            if top.len() >= 128 {
                rank(&mut top);
                top.truncate(24);
            }
        }
        rank(&mut top);
        top.truncate(12);
        println!("state {log_name}: per-residual-hits={each_hits:?} all-residual-hits={all_hits}");
        for item in &top {
            print_score(item, rows.len());
        }
    }
}

fn run_gamma_implications(gamma_path: &str, published_lg: &BTreeMap<u64, u64>) {
    let bank: AnswerBank = serde_json::from_str(&fs::read_to_string(gamma_path).unwrap()).unwrap();
    let rows: Vec<Row> = bank
        .witnesses
        .into_iter()
        .filter_map(|witness| {
            if !witness.expected_bits.starts_with("0x") {
                return None;
            }
            let arg_bits = parse_hex(&witness.args[0]);
            let x = f64::from_bits(arg_bits);
            (x >= 8.0).then(|| Row {
                id: witness.id,
                x,
                expected: parse_hex(&witness.expected_bits),
            })
        })
        .collect();
    println!(
        "\nGAMMA wrapper implications: bank={gamma_path} positive x>=8 rows={} paired-published-lg={}",
        rows.len(),
        rows.iter()
            .filter(|row| published_lg.contains_key(&row.x.to_bits()))
            .count()
    );
    let mut scores = Vec::new();
    scores.push(score(&rows, "gamma:production-kernel".into(), |row| {
        oxfunc_core::functions::special_dist_family::gamma_kernel(row.x).unwrap_or(f64::NAN)
    }));
    scores.push(score(&rows, "gamma:base-lg+excel-exp".into(), |row| {
        rx::excel_exp(base_high_gammaln(row.x))
    }));
    scores.push(score(&rows, "gamma:base-lg+excel-exp-rz".into(), |row| {
        rx::excel_exp_rz(base_high_gammaln(row.x))
    }));
    scores.push(score(&rows, "gamma:base-lg+std-ucrt-exp".into(), |row| {
        base_high_gammaln(row.x).exp()
    }));
    scores.push(score(&rows, "gamma:base-lg+libm-exp".into(), |row| {
        libm::exp(base_high_gammaln(row.x))
    }));
    scores.push(score(&rows, "gamma:libm-tgamma".into(), |row| {
        libm::tgamma(row.x)
    }));
    #[cfg(windows)]
    for (name, function) in legacy_tgamma_functions() {
        scores.push(score(&rows, format!("gamma:{name}-tgamma"), |row| {
            // SAFETY: documented `tgamma(double)` C export, module retained.
            unsafe { function(row.x) }
        }));
    }
    for (name, exp_fn) in [
        ("excel-exp", rx::excel_exp as fn(f64) -> f64),
        ("excel-exp-rz", rx::excel_exp_rz as fn(f64) -> f64),
        ("std-ucrt-exp", f64::exp as fn(f64) -> f64),
        ("libm-exp", libm::exp as fn(f64) -> f64),
    ] {
        let paired: Vec<Row> = rows
            .iter()
            .filter(|row| published_lg.contains_key(&row.x.to_bits()))
            .cloned()
            .collect();
        scores.push(score(
            &paired,
            format!("gamma:excel-published-lg+{name}"),
            |row| exp_fn(f64::from_bits(published_lg[&row.x.to_bits()])),
        ));
    }
    rank(&mut scores);
    for item in &scores {
        print_score(item, rows.len());
    }

    let mut bump_histogram = BTreeMap::<i32, usize>::new();
    let mut skipped = 0_usize;
    let mut outside = Vec::new();
    let mut max_crossing = 0_i32;
    for row in &rows {
        let bits = published_lg
            .get(&row.x.to_bits())
            .copied()
            .unwrap_or_else(|| base_high_gammaln(row.x).to_bits());
        let mut hit = None;
        let mut crossing = None;
        let mut prior = None::<u64>;
        for bump in -4096_i32..=4096 {
            let candidate_bits = if bump < 0 {
                bits - bump.unsigned_abs() as u64
            } else {
                bits + bump as u64
            };
            let got = rx::excel_exp(f64::from_bits(candidate_bits)).to_bits();
            if got == row.expected {
                hit = Some(bump);
                break;
            }
            if got > row.expected {
                crossing = Some((bump, prior, got));
                break;
            }
            prior = Some(got);
        }
        if let Some(bump) = hit {
            *bump_histogram.entry(bump).or_default() += 1;
            max_crossing = max_crossing.max(bump.abs());
        } else if let Some((bump, Some(lower), upper)) = crossing {
            assert!(lower < row.expected && row.expected < upper);
            skipped += 1;
            max_crossing = max_crossing.max(bump.abs());
        } else {
            outside.push(row.id.clone());
        }
    }
    println!(
        "gamma stored-binary64-lg inversion: exact={} skipped-between-adjacent-exp-outputs={} outside-search={} max-crossing-bump={} histogram={bump_histogram:?}; outside={outside:?}",
        bump_histogram.values().sum::<usize>(),
        skipped,
        outside.len(),
        max_crossing
    );
}

fn main() {
    let mut base_only = false;
    let mut state_search = false;
    let mut gamma_path = None::<String>;
    let mut answer_paths = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--base-only" {
            base_only = true;
        } else if arg == "--state-search" {
            state_search = true;
        } else if arg == "--gamma-bank" {
            gamma_path = args.next();
        } else {
            answer_paths.push(arg);
        }
    }
    if answer_paths.is_empty() {
        answer_paths.push(
            "../../work/w109/G3-02-gamma/answers-gammaln-current-build-discovery-v1.json"
                .to_string(),
        );
    }
    let mut row_map = BTreeMap::<u64, Row>::new();
    let mut conflicts = 0_usize;
    for answer_path in &answer_paths {
        let bank: AnswerBank =
            serde_json::from_str(&fs::read_to_string(answer_path).unwrap()).unwrap();
        for witness in bank.witnesses {
            let arg_bits = parse_hex(&witness.args[0]);
            let x = f64::from_bits(arg_bits);
            if x >= 8.0 && witness.expected_bits.starts_with("0x") {
                let expected = parse_hex(&witness.expected_bits);
                if let Some(prior) = row_map.get(&arg_bits) {
                    if prior.expected != expected {
                        conflicts += 1;
                    }
                } else {
                    row_map.insert(
                        arg_bits,
                        Row {
                            id: witness.id,
                            x,
                            expected,
                        },
                    );
                }
            }
        }
    }
    let published_lg: BTreeMap<u64, u64> = row_map
        .iter()
        .map(|(bits, row)| (*bits, row.expected))
        .collect();
    let rows: Vec<Row> = row_map.into_values().collect();
    println!(
        "banks={} x>=8 unique numeric rows={} conflicts={conflicts}",
        answer_paths.len(),
        rows.len()
    );

    let base = score(&rows, "base:dr-q/native-tail/dr-final".into(), |row| {
        let q = dr_add(
            dr_sub(dr_mul(row.x - 0.5, rx::excel_ln(row.x)), row.x),
            LS2PI,
        );
        final_add(q, correction(row.x, 0, 0, 0, 0), 1)
    });
    print_score(&base, rows.len());
    println!("\nbase-residual log sensitivity:");
    for (id, x, _, _, expected) in &base.misses {
        let x87 = rx::excel_ln(*x);
        let std_log = x.ln();
        let portable = libm::log(*x);
        print!(
            "  {id:18} x={x:.17} x87ln=0x{:016x} std-d={} libm-d={}",
            x87.to_bits(),
            ulp_delta(std_log.to_bits(), x87.to_bits()),
            ulp_delta(portable.to_bits(), x87.to_bits())
        );
        #[cfg(windows)]
        for (source, log_fn) in legacy_log_functions() {
            // SAFETY: see the loader comment above.
            let value = unsafe { log_fn(*x) };
            print!(" {source}-d={}", ulp_delta(value.to_bits(), x87.to_bits()));
        }
        println!();
        for bump in -8_i8..=8 {
            let log_bits = if bump < 0 {
                x87.to_bits() - bump.unsigned_abs() as u64
            } else {
                x87.to_bits() + bump as u64
            };
            let q = dr_add(
                dr_sub(dr_mul(*x - 0.5, f64::from_bits(log_bits)), *x),
                LS2PI,
            );
            let got = final_add(q, correction(*x, 0, 0, 0, 0), 1).to_bits();
            let d = ulp_delta(got, *expected);
            print!(" {bump:+}:{d:+}");
        }
        println!();
    }
    let mut constant_scores = Vec::new();
    for bump in -32_i32..=32 {
        let bits = if bump < 0 {
            LS2PI.to_bits() - bump.unsigned_abs() as u64
        } else {
            LS2PI.to_bits() + bump as u64
        };
        let constant = f64::from_bits(bits);
        constant_scores.push(score(&rows, format!("constant-bump{bump:+}"), |row| {
            let q = dr_add(
                dr_sub(dr_mul(row.x - 0.5, rx::excel_ln(row.x)), row.x),
                constant,
            );
            final_add(q, correction(row.x, 0, 0, 0, 0), 1)
        }));
    }
    rank(&mut constant_scores);
    println!("\nLS2PI-neighbor diagnostic:");
    for item in constant_scores.iter().take(12) {
        print_score(item, rows.len());
    }
    if state_search {
        let residual_ids: Vec<String> = base.misses.iter().map(|miss| miss.0.clone()).collect();
        run_state_search(&rows, &residual_ids);
    }
    if let Some(path) = &gamma_path {
        run_gamma_implications(path, &published_lg);
    }
    if base_only {
        return;
    }

    let mut q_scores = Vec::new();
    if let Some(first) = rows.first() {
        let q_names: Vec<String> = q_graphs(first.x).into_iter().map(|p| p.0).collect();
        for (q_index, name) in q_names.into_iter().enumerate() {
            q_scores.push(score(&rows, format!("{name}/tail00000/final-dr"), |row| {
                let q = q_graphs(row.x)[q_index].1;
                final_add(q, correction(row.x, 0, 0, 0, 0), 1)
            }));
        }
    }
    rank(&mut q_scores);
    println!("\nq-only ranking:");
    for item in q_scores.iter().take(12) {
        print_score(item, rows.len());
    }
    for (id, ..) in &base.misses {
        let mut fixing: Vec<&Score> = q_scores
            .iter()
            .filter(|score| !score.misses.iter().any(|miss| &miss.0 == id))
            .collect();
        fixing.sort_by_key(|score| (std::cmp::Reverse(score.exact), score.worst, score.sum));
        println!(
            "q reachability {id}: {} graphs hit; best full-bank score {}",
            fixing.len(),
            fixing
                .first()
                .map(|score| format!("{} exact ({})", score.exact, score.name))
                .unwrap_or_else(|| "none".to_string())
        );
    }

    let mut combined_scores = Vec::new();
    if let Some(first) = rows.first() {
        let names: Vec<String> = combined_specials(first.x)
            .into_iter()
            .map(|pair| pair.0)
            .collect();
        for (index, name) in names.into_iter().enumerate() {
            combined_scores.push(score(&rows, name, |row| combined_specials(row.x)[index].1));
        }
    }
    rank(&mut combined_scores);
    println!("\nq/tail retained-association ranking:");
    for item in combined_scores.iter().take(20) {
        print_score(item, rows.len());
    }

    let mut public_tail_scores = Vec::new();
    if let Some(first) = rows.first() {
        let names: Vec<String> = public_tail_specials(first.x)
            .into_iter()
            .map(|pair| pair.0)
            .collect();
        for (index, name) in names.into_iter().enumerate() {
            public_tail_scores.push(score(&rows, name, |row| {
                public_tail_specials(row.x)[index].1
            }));
        }
    }
    rank(&mut public_tail_scores);
    println!("\npublic-tail family ranking:");
    for item in public_tail_scores.iter().take(20) {
        print_score(item, rows.len());
    }

    let mut tail_scores = Vec::new();
    for z_kind in 0..3 {
        for y_kind in 0..3 {
            for step_code in 0..(1_u16 << 10) {
                for corr_kind in 0..4 {
                    for final_kind in 0..3 {
                        let name = format!(
                            "q-dr/tail-z{z_kind}y{y_kind}s{step_code:05x}c{corr_kind}/f{final_kind}"
                        );
                        tail_scores.push(score(&rows, name, |row| {
                            let q = dr_add(
                                dr_sub(dr_mul(row.x - 0.5, rx::excel_ln(row.x)), row.x),
                                LS2PI,
                            );
                            final_add(
                                q,
                                correction(row.x, z_kind, y_kind, step_code, corr_kind),
                                final_kind,
                            )
                        }));
                    }
                }
            }
        }
    }
    rank(&mut tail_scores);
    println!("\ntail/final ranking ({} graphs):", tail_scores.len());
    for item in tail_scores.iter().take(20) {
        print_score(item, rows.len());
    }
    for (id, ..) in &base.misses {
        let mut fixing: Vec<&Score> = tail_scores
            .iter()
            .filter(|score| !score.misses.iter().any(|miss| &miss.0 == id))
            .collect();
        fixing.sort_by_key(|score| (std::cmp::Reverse(score.exact), score.worst, score.sum));
        println!(
            "tail reachability {id}: {} graphs hit; best full-bank score {}",
            fixing.len(),
            fixing
                .first()
                .map(|score| format!("{} exact ({})", score.exact, score.name))
                .unwrap_or_else(|| "none".to_string())
        );
    }

    // Cross only the bounded top families. This prevents a residual-only
    // overfit: every survivor is ranked over the full current-build bank.
    let q_top: Vec<String> = q_scores.iter().take(12).map(|s| s.name.clone()).collect();
    let mut q_index_by_prefix = BTreeMap::new();
    if let Some(first) = rows.first() {
        for (index, (name, _)) in q_graphs(first.x).into_iter().enumerate() {
            q_index_by_prefix.insert(name, index);
        }
    }
    let mut cross = Vec::new();
    for q_full in q_top {
        let q_name = q_full.split('/').next().unwrap().to_string();
        let q_index = q_index_by_prefix[&q_name];
        for tail in tail_scores.iter().take(64) {
            let fields: Vec<&str> = tail.name.split('/').collect();
            let desc = fields[1];
            let final_kind: u8 = fields[2].trim_start_matches('f').parse().unwrap();
            let z_kind: u8 = desc[6..7].parse().unwrap();
            let y_kind: u8 = desc[8..9].parse().unwrap();
            let s_pos = desc.find('s').unwrap();
            let c_pos = desc.rfind('c').unwrap();
            let step_code = u16::from_str_radix(&desc[s_pos + 1..c_pos], 16).unwrap();
            let corr_kind: u8 = desc[c_pos + 1..].parse().unwrap();
            cross.push(score(
                &rows,
                format!("{q_name}/{desc}/f{final_kind}"),
                |row| {
                    let q = q_graphs(row.x)[q_index].1;
                    final_add(
                        q,
                        correction(row.x, z_kind, y_kind, step_code, corr_kind),
                        final_kind,
                    )
                },
            ));
        }
    }
    rank(&mut cross);
    println!("\ntop-q x top-tail cross ranking:");
    for item in cross.iter().take(20) {
        print_score(item, rows.len());
    }
}
