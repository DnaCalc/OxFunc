//! W109 ERF.PRECISE clean-room race for the TOMS-654 `gratio` branch 190.
//!
//! This is research tooling only.  It deliberately reads the legacy discovery
//! banks listed in `DISCOVERY_BANKS`; `answers-b9heldout.json` is neither named
//! nor accepted.  The candidate family is source-backed branch-190 arithmetic
//! plus independently observed Excel primitive graphs.  In particular, it
//! tests the distribution-site raw pow graph
//!
//!     exp(RN53(RN64(0.5 * RN53(ln(x)))))
//!
//! against the older register-continuous x87 hypothesis.
//!
//! Usage:
//!   race_erf_precise_pow_substrate <G3-01-dist-directory>
//!   race_erf_precise_pow_substrate <OxFunc-root> gauss
//!   race_erf_precise_pow_substrate <OxFunc-root> gauss-q
//!   race_erf_precise_pow_substrate <OxFunc-root> gauss-input
//!   race_erf_precise_pow_substrate <OxFunc-root> gauss-series
//!   race_erf_precise_pow_substrate <OxFunc-root> gauss-route
//!   race_erf_precise_pow_substrate <OxFunc-root> gauss-route-capture
//!   race_erf_precise_pow_substrate <OxFunc-root> gauss-composite
//!
//! Every GAUSS mode names only the frozen discovery answer.  The sealed
//! heldout path is deliberately absent from this source.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC64_RN, Ext80, ext_abs, ext_add, ext_chs, ext_div, ext_f2xm1, ext_from_f64, ext_fyl2x,
    ext_l2e, ext_ln2, ext_mul, ext_one, ext_rndint, ext_scale, ext_sqrt, ext_sub, ext_to_f64,
};
use std::collections::BTreeMap;

const CW: u16 = CW_PC64_RN;
const DISCOVERY_BANKS: [&str; 7] = [
    "answers-b9train.json",
    "answers-erfp.json",
    "answers-erfm.json",
    "answers-b8erf.json",
    "answers-b7erf.json",
    "answers-b11.json",
    "answers-b10.json",
];

const GP: [f64; 7] = [
    0.577215664901533e+00,
    -0.409078193005776e+00,
    -0.230975380857675e+00,
    0.597275330452234e-01,
    0.766968181649490e-02,
    -0.514889771323592e-02,
    0.589597428611429e-03,
];
const GQ: [f64; 5] = [
    1.0,
    0.427569613095214e+00,
    0.158451672430138e+00,
    0.261132021441447e-01,
    0.423244297896961e-02,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}

fn dbl(x: &Ext80) -> f64 {
    ext_to_f64(x, CW)
}

fn ln_ext(x: &Ext80) -> Ext80 {
    ext_fyl2x(&ext_ln2(), x, CW)
}

fn exp_ext(x: &Ext80) -> Ext80 {
    let t = ext_mul(x, &ext_l2e(), CW);
    let k = ext_rndint(&t, CW);
    let f = ext_sub(&t, &k, CW);
    let neg = dbl(&f) < 0.0;
    let w = ext_f2xm1(&ext_abs(&f, CW), CW);
    let mut m = ext_add(&w, &ext_one(), CW);
    if neg {
        m = ext_div(&ext_one(), &m, CW);
    }
    ext_scale(&m, &k, CW)
}

fn ext_le(a: &Ext80, b: &Ext80) -> bool {
    dbl(&ext_sub(a, b, CW)) <= 0.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum XMode {
    /// Keep the result of z*z in an Ext80 temporary.
    Extended,
    /// Publish z*z through ordinary binary64 arithmetic.
    Native53,
    /// Publish z*z through RN53(RN64(z*z)).
    X87DoubleRounded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GamMode {
    /// Published TOMS coefficients, every arithmetic node binary64.
    Binary64,
    /// Published coefficients, register-continuous x87 rational evaluation.
    Extended,
    /// Extended rational evaluation, with gam1's return value stored to f64.
    ExtendedReturn53,
    /// Correct mathematical normalizer, rounded to f64 (control candidate).
    TwoOverSqrtPi53,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WMode {
    /// a*ln(x) and exp remain register-continuous Ext80.
    X87Continuous,
    /// a*ln(x) is stored before the x87 exp chain.
    X87Argument53,
    /// The site-proven distribution raw-pow graph; no 0.5 sqrt shortcut.
    DistributionPow,
    /// Register-continuous pow with a stored binary64 base.
    X87DirectPow,
    /// Separate published LN, native binary64 multiply, published EXP.
    ExcelLnMulExp,
    /// Correctly-rounded hardware sqrt (negative-control wrapper graph).
    Sqrt,
    /// Algebraic sqrt(z*z)=z shortcut (negative-control call-site graph).
    InputZ,
    /// Host libm powf control.
    LibmPow,
}

#[derive(Clone, Copy, Debug)]
enum InnerMode {
    ExtendedCompensated,
    ExtendedDirect,
    Binary64Compensated,
    Binary64Direct,
}

#[derive(Clone, Copy, Debug)]
enum Assoc {
    WgThenInner,
    WThenGInner,
    WInnerThenG,
}

#[derive(Clone, Copy, Debug)]
struct Cfg {
    x: XMode,
    series_53: bool,
    j_53: bool,
    gam: GamMode,
    g_53: bool,
    w: WMode,
    inner: InnerMode,
    assoc: Assoc,
    first_product_53: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GaussArgMode {
    NativeMultiply,
    NativeDivide,
    X87MultiplyStore,
    X87DivideStore,
    X87SqrtDivideStore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GaussInputDelivery {
    NativeMultiplyStored,
    NativeDivideStored,
    ExtendedMultiply,
    ExtendedDivideStoredSqrt,
    ExtendedDivideExtendedSqrt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GaussPublishMode {
    DirectHalf,
    HalfOfOnePlusThenSubtract,
    HalfPlusHalfThenSubtract,
    SignSplitComplement,
}

#[derive(Clone, Copy, Debug)]
struct GaussCfg {
    erf: Cfg,
    arg: GaussArgMode,
    publish: GaussPublishMode,
}

fn x_value(z: f64, mode: XMode) -> Ext80 {
    match mode {
        XMode::Extended => ext_mul(&ef(z), &ef(z), CW),
        XMode::Native53 => ef(z * z),
        XMode::X87DoubleRounded => ef(rx::x87_mul(z, z)),
    }
}

fn gam1_half(mode: GamMode) -> Ext80 {
    if matches!(mode, GamMode::TwoOverSqrtPi53) {
        // g = 1 + gam1(1/2) = 2/sqrt(pi), so return h = g - 1.
        return ef(2.0 / std::f64::consts::PI.sqrt() - 1.0);
    }
    let per_op_53 = matches!(mode, GamMode::Binary64);
    let sp = |v: Ext80| -> Ext80 { if per_op_53 { ef(dbl(&v)) } else { v } };
    let t = ef(0.5);
    let mut top = ef(GP[6]);
    for &c in GP[..6].iter().rev() {
        top = sp(ext_add(&ext_mul(&top, &t, CW), &ef(c), CW));
    }
    let mut bot = ef(GQ[4]);
    for &c in GQ[..4].iter().rev() {
        bot = sp(ext_add(&ext_mul(&bot, &t, CW), &ef(c), CW));
    }
    let w = sp(ext_div(&top, &bot, CW));
    let h = sp(ext_mul(&ef(0.5), &w, CW));
    if matches!(mode, GamMode::ExtendedReturn53) {
        ef(dbl(&h))
    } else {
        h
    }
}

fn horner_half_spill_mask(coefficients: &[f64], mask: u32, bit: &mut u32) -> Ext80 {
    let mut value = ef(*coefficients.last().unwrap());
    for &coefficient in coefficients[..coefficients.len() - 1].iter().rev() {
        value = ext_add(&ext_mul(&value, &ef(0.5), CW), &ef(coefficient), CW);
        if mask & (1 << *bit) != 0 {
            value = ef(dbl(&value));
        }
        *bit += 1;
    }
    value
}

fn gam1_spill_values() -> Vec<(Ext80, u32, u32)> {
    let mut unique: BTreeMap<[u8; 10], (u32, u32)> = BTreeMap::new();
    // Six numerator Horner stages + four denominator stages. A spill after
    // multiplication by exactly 0.5 is equivalent to a spill before the next
    // addition, so one bit per complete stage spans the distinct graph.
    for horner_mask in 0u32..(1 << 10) {
        for tail_mask in 0u32..8 {
            let mut bit = 0;
            let top = horner_half_spill_mask(&GP, horner_mask, &mut bit);
            let bot = horner_half_spill_mask(&GQ, horner_mask, &mut bit);
            debug_assert_eq!(bit, 10);
            let mut ratio = ext_div(&top, &bot, CW);
            if tail_mask & 1 != 0 {
                ratio = ef(dbl(&ratio));
            }
            let mut h = ext_mul(&ef(0.5), &ratio, CW);
            if tail_mask & 2 != 0 {
                h = ef(dbl(&h));
            }
            let mut g = ext_add(&ext_one(), &h, CW);
            if tail_mask & 4 != 0 {
                g = ef(dbl(&g));
            }
            unique.entry(g.0).or_insert((horner_mask, tail_mask));
        }
    }
    unique
        .into_iter()
        .map(|(bytes, (horner_mask, tail_mask))| (Ext80(bytes), horner_mask, tail_mask))
        .collect()
}

fn gam1_spill_search() {
    const TARGET_MANTISSA: u64 = 0x906e_ba82_14db_6c6f;
    let values = gam1_spill_values();
    let target_hits: Vec<_> = values
        .iter()
        .filter(|(g, _, _)| u64::from_le_bytes(g.0[..8].try_into().unwrap()) == TARGET_MANTISSA)
        .collect();
    println!(
        "gam1(1/2) mixed-spill enumeration: {} graphs -> {} distinct Ext80 g values",
        (1 << 10) * 8,
        values.len()
    );
    println!(
        "empirical diagnostic target mantissa 0x{TARGET_MANTISSA:016x}: {} public-graph hits",
        target_hits.len()
    );
    for (g, horner_mask, tail_mask) in target_hits.iter().take(16) {
        println!(
            "  horner_mask=0x{horner_mask:03x} tail_mask=0b{tail_mask:03b} g_f64=0x{:016x} ext={:02x?}",
            dbl(g).to_bits(),
            g.0
        );
    }
    let mut nearest: Vec<(u64, u64, [u8; 10], u32, u32)> = values
        .into_iter()
        .map(|(g, horner_mask, tail_mask)| {
            let mantissa = u64::from_le_bytes(g.0[..8].try_into().unwrap());
            (
                mantissa.abs_diff(TARGET_MANTISSA),
                mantissa,
                g.0,
                horner_mask,
                tail_mask,
            )
        })
        .collect();
    nearest.sort_by_key(|row| row.0);
    println!("nearest distinct values:");
    for (distance, mantissa, bytes, horner_mask, tail_mask) in nearest.iter().take(12) {
        let g = Ext80(*bytes);
        println!(
            "  distance={distance:4} mantissa=0x{mantissa:016x} horner=0x{horner_mask:03x} tail=0b{tail_mask:03b} f64=0x{:016x}",
            dbl(&g).to_bits()
        );
    }
}

fn series_j(x: &Ext80, series_53: bool, j_53: bool) -> Ext80 {
    let a = ef(0.5);
    let sp = |v: Ext80| -> Ext80 { if series_53 { ef(dbl(&v)) } else { v } };
    let mut an = ef(3.0);
    let mut c = *x;
    let mut sum = sp(ext_div(x, &ext_add(&a, &ef(3.0), CW), CW));
    let tol = ext_div(
        &ext_mul(&ef(3.0), &ef(5e-15), CW),
        &ext_add(&a, &ext_one(), CW),
        CW,
    );
    for _ in 0..200 {
        an = sp(ext_add(&an, &ext_one(), CW));
        c = sp(ext_chs(&ext_mul(&c, &ext_div(x, &an, CW), CW), CW));
        let term = sp(ext_div(&c, &ext_add(&a, &an, CW), CW));
        sum = sp(ext_add(&sum, &term, CW));
        if ext_le(&ext_abs(&term, CW), &tol) {
            break;
        }
    }
    let inner_poly = ext_add(
        &ext_mul(
            &ext_sub(
                &ext_div(&sum, &ef(6.0), CW),
                &ext_div(&ef(0.5), &ext_add(&a, &ef(2.0), CW), CW),
                CW,
            ),
            x,
            CW,
        ),
        &ext_div(&ext_one(), &ext_add(&a, &ext_one(), CW), CW),
        CW,
    );
    let j = ext_mul(&ext_mul(&a, x, CW), &inner_poly, CW);
    if j_53 { ef(dbl(&j)) } else { j }
}

fn series_j_spill_mask(x: &Ext80, mask: u16) -> Ext80 {
    let a = ef(0.5);
    let spill = |value: Ext80, bit: u16| {
        if mask & (1 << bit) != 0 {
            ef(dbl(&value))
        } else {
            value
        }
    };
    let mut an = ef(3.0);
    let mut c = *x;
    let mut sum = spill(ext_div(x, &ext_add(&a, &ef(3.0), CW), CW), 0);
    let tol = ext_div(
        &ext_mul(&ef(3.0), &ef(5e-15), CW),
        &ext_add(&a, &ext_one(), CW),
        CW,
    );
    for _ in 0..200 {
        an = spill(ext_add(&an, &ext_one(), CW), 1);
        let ratio = spill(ext_div(x, &an, CW), 2);
        c = spill(ext_chs(&ext_mul(&c, &ratio, CW), CW), 3);
        let term = spill(ext_div(&c, &ext_add(&a, &an, CW), CW), 4);
        sum = spill(ext_add(&sum, &term, CW), 5);
        if ext_le(&ext_abs(&term, CW), &tol) {
            break;
        }
    }
    let sum6 = spill(ext_div(&sum, &ef(6.0), CW), 6);
    let half_over = spill(ext_div(&ef(0.5), &ext_add(&a, &ef(2.0), CW), CW), 7);
    let delta = spill(ext_sub(&sum6, &half_over, CW), 8);
    let scaled = spill(ext_mul(&delta, x, CW), 9);
    let inner_poly = spill(
        ext_add(
            &scaled,
            &ext_div(&ext_one(), &ext_add(&a, &ext_one(), CW), CW),
            CW,
        ),
        10,
    );
    let ax = spill(ext_mul(&a, x, CW), 11);
    spill(ext_mul(&ax, &inner_poly, CW), 12)
}

fn w_value(z: f64, x: &Ext80, mode: WMode) -> Ext80 {
    let base = dbl(x);
    match mode {
        WMode::X87Continuous => {
            let arg = ext_mul(&ef(0.5), &ln_ext(x), CW);
            exp_ext(&arg)
        }
        WMode::X87Argument53 => {
            let arg = dbl(&ext_mul(&ef(0.5), &ln_ext(x), CW));
            ef(rx::excel_exp(arg))
        }
        WMode::DistributionPow => ef(rx::excel_pow_chain(base, 0.5)),
        WMode::X87DirectPow => ef(rx::excel_pow_x87_direct(base, 0.5)),
        WMode::ExcelLnMulExp => ef(rx::excel_exp(0.5 * rx::excel_ln(base))),
        WMode::Sqrt => ef(rx::excel_sqrt(base)),
        WMode::InputZ => ef(z),
        WMode::LibmPow => ef(base.powf(0.5)),
    }
}

fn inner_value(j: &Ext80, mode: InnerMode) -> Ext80 {
    match mode {
        InnerMode::ExtendedCompensated => ext_add(&ef(0.5), &ext_sub(&ef(0.5), j, CW), CW),
        InnerMode::ExtendedDirect => ext_sub(&ext_one(), j, CW),
        InnerMode::Binary64Compensated => {
            let jd = dbl(j);
            ef(0.5 + (0.5 - jd))
        }
        InnerMode::Binary64Direct => ef(1.0 - dbl(j)),
    }
}

fn eval(z: f64, cfg: &Cfg) -> f64 {
    dbl(&eval_ext(z, cfg))
}

fn eval_ext(z: f64, cfg: &Cfg) -> Ext80 {
    let mut g = ext_add(&ext_one(), &gam1_half(cfg.gam), CW);
    if cfg.g_53 {
        g = ef(dbl(&g));
    }
    eval_with_g_ext(z, cfg, g)
}

fn eval_with_g(z: f64, cfg: &Cfg, g: Ext80) -> f64 {
    dbl(&eval_with_g_ext(z, cfg, g))
}

fn eval_with_g_ext(z: f64, cfg: &Cfg, g: Ext80) -> Ext80 {
    let x = x_value(z, cfg.x);
    let j = series_j(&x, cfg.series_53, cfg.j_53);
    let w = w_value(z, &x, cfg.w);
    let inner = inner_value(&j, cfg.inner);
    let sp = |v: Ext80| -> Ext80 { if cfg.first_product_53 { ef(dbl(&v)) } else { v } };
    let ans = match cfg.assoc {
        Assoc::WgThenInner => ext_mul(&sp(ext_mul(&w, &g, CW)), &inner, CW),
        Assoc::WThenGInner => ext_mul(&w, &sp(ext_mul(&g, &inner, CW)), CW),
        Assoc::WInnerThenG => ext_mul(&sp(ext_mul(&w, &inner, CW)), &g, CW),
    };
    ans
}

fn eval_from_ext_input(z: Ext80, cfg: &Cfg) -> Ext80 {
    let mut g = ext_add(&ext_one(), &gam1_half(cfg.gam), CW);
    if cfg.g_53 {
        g = ef(dbl(&g));
    }
    let x = match cfg.x {
        XMode::Extended => ext_mul(&z, &z, CW),
        XMode::Native53 => {
            let zd = dbl(&z);
            ef(zd * zd)
        }
        XMode::X87DoubleRounded => ef(dbl(&ext_mul(&z, &z, CW))),
    };
    let j = series_j(&x, cfg.series_53, cfg.j_53);
    let w = match cfg.w {
        WMode::X87Continuous => {
            let arg = ext_mul(&ef(0.5), &ln_ext(&x), CW);
            exp_ext(&arg)
        }
        WMode::X87Argument53 => {
            let arg = dbl(&ext_mul(&ef(0.5), &ln_ext(&x), CW));
            ef(rx::excel_exp(arg))
        }
        WMode::DistributionPow => ef(rx::excel_pow_chain(dbl(&x), 0.5)),
        WMode::X87DirectPow => ef(rx::excel_pow_x87_direct(dbl(&x), 0.5)),
        WMode::ExcelLnMulExp => ef(rx::excel_exp(0.5 * rx::excel_ln(dbl(&x)))),
        WMode::Sqrt => ef(rx::excel_sqrt(dbl(&x))),
        WMode::InputZ => z,
        WMode::LibmPow => ef(dbl(&x).powf(0.5)),
    };
    let inner = inner_value(&j, cfg.inner);
    let sp = |value: Ext80| {
        if cfg.first_product_53 {
            ef(dbl(&value))
        } else {
            value
        }
    };
    match cfg.assoc {
        Assoc::WgThenInner => ext_mul(&sp(ext_mul(&w, &g, CW)), &inner, CW),
        Assoc::WThenGInner => ext_mul(&w, &sp(ext_mul(&g, &inner, CW)), CW),
        Assoc::WInnerThenG => ext_mul(&sp(ext_mul(&w, &inner, CW)), &g, CW),
    }
}

fn gauss_arg(x: f64, mode: GaussArgMode) -> f64 {
    let x = x.abs();
    match mode {
        GaussArgMode::NativeMultiply => x * std::f64::consts::FRAC_1_SQRT_2,
        GaussArgMode::NativeDivide => x / std::f64::consts::SQRT_2,
        GaussArgMode::X87MultiplyStore => {
            dbl(&ext_mul(&ef(x), &ef(std::f64::consts::FRAC_1_SQRT_2), CW))
        }
        GaussArgMode::X87DivideStore => dbl(&ext_div(&ef(x), &ef(std::f64::consts::SQRT_2), CW)),
        GaussArgMode::X87SqrtDivideStore => {
            let root = ext_sqrt(&ef(2.0), CW);
            dbl(&ext_div(&ef(x), &root, CW))
        }
    }
}

fn gauss_publish(x: f64, erf_abs: f64, mode: GaussPublishMode) -> f64 {
    let signed_erf = if x < 0.0 { -erf_abs } else { erf_abs };
    match mode {
        GaussPublishMode::DirectHalf => 0.5 * signed_erf,
        GaussPublishMode::HalfOfOnePlusThenSubtract => 0.5 * (1.0 + signed_erf) - 0.5,
        GaussPublishMode::HalfPlusHalfThenSubtract => (0.5 + 0.5 * signed_erf) - 0.5,
        GaussPublishMode::SignSplitComplement => {
            let erfc_abs = 1.0 - erf_abs;
            if x < 0.0 {
                0.5 * erfc_abs - 0.5
            } else {
                (1.0 - 0.5 * erfc_abs) - 0.5
            }
        }
    }
}

fn eval_gauss_small(x: f64, cfg: &GaussCfg) -> f64 {
    let z = gauss_arg(x, cfg.arg);
    gauss_publish(x, eval(z, &cfg.erf), cfg.publish)
}

fn eval_literal_gratio_return(z: f64, cfg: &Cfg, spill_mask: u16) -> f64 {
    let spill = |value: Ext80, bit: u16| -> Ext80 {
        if spill_mask & (1 << bit) != 0 {
            ef(dbl(&value))
        } else {
            value
        }
    };
    let x = x_value(z, cfg.x);
    let j = series_j(&x, cfg.series_53, cfg.j_53);
    let h = gam1_half(cfg.gam);
    let mut g = ext_add(&ext_one(), &h, CW);
    if cfg.g_53 {
        g = ef(dbl(&g));
    }
    let w = w_value(z, &x, cfg.w);

    // Literal public GRATIO branch-200 return graph.  It is algebraically
    // w*g*(1-j), but these complement operations are observable under finite
    // precision and therefore must not be collapsed during identification.
    let l_hi = spill(ext_sub(&w, &ef(0.5), CW), 0);
    let l = spill(ext_sub(&l_hi, &ef(0.5), CW), 1);
    let w2_lo = spill(ext_add(&ef(0.5), &l, CW), 2);
    let w2 = spill(ext_add(&ef(0.5), &w2_lo, CW), 3);
    let wj = spill(ext_mul(&w2, &j, CW), 4);
    let pre = spill(ext_sub(&wj, &l, CW), 5);
    let scaled = spill(ext_mul(&pre, &g, CW), 6);
    let qans = spill(ext_sub(&scaled, &h, CW), 7);
    let ans_lo = spill(ext_sub(&ef(0.5), &qans, CW), 8);
    let ans = spill(ext_add(&ef(0.5), &ans_lo, CW), 9);
    dbl(&ans)
}

fn score_literal_gratio_returns(dir: &str) {
    let rows = load_rows(dir);
    let bases = [
        Cfg {
            x: XMode::Native53,
            series_53: false,
            j_53: false,
            gam: GamMode::ExtendedReturn53,
            g_53: false,
            w: WMode::X87Continuous,
            inner: InnerMode::ExtendedCompensated,
            assoc: Assoc::WgThenInner,
            first_product_53: false,
        },
        Cfg {
            x: XMode::Extended,
            series_53: true,
            j_53: true,
            gam: GamMode::Extended,
            g_53: false,
            w: WMode::X87Continuous,
            inner: InnerMode::ExtendedCompensated,
            assoc: Assoc::WThenGInner,
            first_product_53: false,
        },
    ];
    let mut scores: Vec<(usize, u64, u64, usize, u16)> = Vec::new();
    for (base_index, cfg) in bases.iter().enumerate() {
        for mask in 0u16..(1 << 10) {
            let (mut exact, mut max_ulp, mut sum_abs_ulp) = (0usize, 0u64, 0u64);
            for &(z, expected) in &rows {
                let got = eval_literal_gratio_return(z, cfg, mask).to_bits();
                let delta = ordered(expected) - ordered(got);
                let abs = delta.unsigned_abs();
                exact += usize::from(abs == 0);
                max_ulp = max_ulp.max(abs);
                sum_abs_ulp = sum_abs_ulp.saturating_add(abs);
            }
            scores.push((exact, max_ulp, sum_abs_ulp, base_index, mask));
        }
    }
    scores.sort_by_key(|row| (usize::MAX - row.0, row.1, row.2));
    println!(
        "literal GRATIO return race: {} masks x {} base bodies; {} discovery rows; heldout path absent",
        1 << 10,
        bases.len(),
        rows.len()
    );
    for (rank, &(exact, max_ulp, sum, base, mask)) in scores.iter().take(32).enumerate() {
        println!(
            "#{:02} exact={}/{} max={} sum={} base={} mask=0b{:010b} {:?}",
            rank + 1,
            exact,
            rows.len(),
            max_ulp,
            sum,
            base,
            mask,
            bases[base]
        );
    }
}

fn score_gam1_spills(dir: &str) {
    let rows = load_rows(dir);
    let values = gam1_spill_values();
    let mut results: Vec<(usize, i64, u64, u64, u32, u32, Cfg)> = Vec::new();
    for (g, horner_mask, tail_mask) in values {
        let mantissa = u64::from_le_bytes(g.0[..8].try_into().unwrap());
        for x in [XMode::Extended, XMode::Native53, XMode::X87DoubleRounded] {
            for series_53 in [false, true] {
                for j_53 in [false, true] {
                    for inner in [
                        InnerMode::ExtendedCompensated,
                        InnerMode::ExtendedDirect,
                        InnerMode::Binary64Compensated,
                        InnerMode::Binary64Direct,
                    ] {
                        for assoc in [Assoc::WgThenInner, Assoc::WThenGInner, Assoc::WInnerThenG] {
                            for first_product_53 in [false, true] {
                                let cfg = Cfg {
                                    x,
                                    series_53,
                                    j_53,
                                    gam: GamMode::Extended,
                                    g_53: false,
                                    w: WMode::X87Continuous,
                                    inner,
                                    assoc,
                                    first_product_53,
                                };
                                let (mut exact, mut max_ulp, mut sum_abs_ulp) =
                                    (0usize, 0i64, 0u64);
                                for &(z, expected) in &rows {
                                    let got = eval_with_g(z, &cfg, g).to_bits();
                                    let delta = ordered(expected) - ordered(got);
                                    exact += usize::from(delta == 0);
                                    let abs = delta.unsigned_abs();
                                    max_ulp = max_ulp.max(abs as i64);
                                    sum_abs_ulp = sum_abs_ulp.saturating_add(abs);
                                }
                                results.push((
                                    exact,
                                    max_ulp,
                                    sum_abs_ulp,
                                    mantissa,
                                    horner_mask,
                                    tail_mask,
                                    cfg,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    results.sort_by_key(|row| (usize::MAX - row.0, row.1, row.2));
    println!(
        "mixed-gam1 race: {} distinct g values x structured body graphs; {} rows",
        gam1_spill_values().len(),
        rows.len()
    );
    for (rank, row) in results.iter().take(24).enumerate() {
        println!(
            "#{:02} exact={}/{} max_ulp={} sum={} g=0x{:016x} horner=0x{:03x} tail=0b{:03b} {:?}",
            rank + 1,
            row.0,
            rows.len(),
            row.1,
            row.2,
            row.3,
            row.4,
            row.5,
            row.6
        );
    }
}

fn ordered(bits: u64) -> i64 {
    let signed = bits as i64;
    if signed < 0 { !signed } else { signed }
}

fn load_rows(dir: &str) -> Vec<(f64, u64)> {
    let mut rows: BTreeMap<u64, u64> = BTreeMap::new();
    for name in DISCOVERY_BANKS {
        let path = format!("{dir}/{name}");
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("failed to parse {path}: {e}"));
        for witness in &bank.witnesses {
            let z = match &witness.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("scalar bit pattern"),
                _ => continue,
            };
            let Some(expected) = parse_bits_hex(&witness.expected_bits) else {
                continue;
            };
            if z > 0.0 && z < 0.5 {
                if let Some(old) = rows.insert(z.to_bits(), expected.to_bits()) {
                    assert_eq!(
                        old,
                        expected.to_bits(),
                        "conflicting oracle bits for z={z:?}"
                    );
                }
            }
        }
    }
    rows.into_iter()
        .map(|(z, expected)| (f64::from_bits(z), expected))
        .collect()
}

fn load_gauss_discovery_rows(root: &str) -> Vec<(f64, u64)> {
    // The sealed heldout path is intentionally absent.  This reader accepts
    // only the explicitly frozen current-build discovery answer.
    const ANSWER: &str = "smart-fuzzer/work/w109/G3-07-gauss/answers-gauss-exact-discovery-v1.json";
    let path = format!("{root}/{ANSWER}");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read GAUSS discovery {path}: {e}"));
    let bank: WitnessSet =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("failed to parse {path}: {e}"));
    assert_eq!(bank.function, "GAUSS", "unexpected function in {path}");
    assert_eq!(bank.witnesses.len(), 8_192, "GAUSS discovery count drifted");
    bank.witnesses
        .iter()
        .filter_map(|witness| {
            let x = match &witness.args[0] {
                WitnessArg::Scalar(bits) => parse_bits_hex(bits)?,
                WitnessArg::Array(_) => return None,
            };
            let expected = parse_bits_hex(&witness.expected_bits)?;
            Some((x, expected.to_bits()))
        })
        .collect()
}

fn load_gauss_route_discovery_rows(root: &str) -> Vec<(f64, u64)> {
    // The companion route heldout is sealed and intentionally unnameable here.
    const ANSWER: &str = "smart-fuzzer/work/w109/G3-07-gauss/answers-gauss-route-discovery-v1.json";
    let path = format!("{root}/{ANSWER}");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read GAUSS route discovery {path}: {e}"));
    let bank: WitnessSet =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("failed to parse {path}: {e}"));
    assert_eq!(bank.function, "GAUSS", "unexpected function in {path}");
    assert_eq!(
        bank.witnesses.len(),
        1_024,
        "GAUSS route discovery count drifted"
    );
    bank.witnesses
        .iter()
        .filter_map(|witness| {
            let x = match &witness.args[0] {
                WitnessArg::Scalar(bits) => parse_bits_hex(bits)?,
                WitnessArg::Array(_) => return None,
            };
            let expected = parse_bits_hex(&witness.expected_bits)?;
            Some((x, expected.to_bits()))
        })
        .collect()
}

fn gauss_distance(got: u64, expected: u64) -> u64 {
    (ordered(got) as i128).abs_diff(ordered(expected) as i128) as u64
}

fn all_erf_cfgs() -> Vec<Cfg> {
    let mut cfgs = Vec::new();
    for x in [XMode::Extended, XMode::Native53, XMode::X87DoubleRounded] {
        for series_53 in [false, true] {
            for j_53 in [false, true] {
                for gam in [
                    GamMode::Binary64,
                    GamMode::Extended,
                    GamMode::ExtendedReturn53,
                    GamMode::TwoOverSqrtPi53,
                ] {
                    for g_53 in [false, true] {
                        for w in [
                            WMode::X87Continuous,
                            WMode::X87Argument53,
                            WMode::DistributionPow,
                            WMode::X87DirectPow,
                            WMode::ExcelLnMulExp,
                            WMode::Sqrt,
                            WMode::InputZ,
                            WMode::LibmPow,
                        ] {
                            for inner in [
                                InnerMode::ExtendedCompensated,
                                InnerMode::ExtendedDirect,
                                InnerMode::Binary64Compensated,
                                InnerMode::Binary64Direct,
                            ] {
                                for assoc in
                                    [Assoc::WgThenInner, Assoc::WThenGInner, Assoc::WInnerThenG]
                                {
                                    for first_product_53 in [false, true] {
                                        cfgs.push(Cfg {
                                            x,
                                            series_53,
                                            j_53,
                                            gam,
                                            g_53,
                                            w,
                                            inner,
                                            assoc,
                                            first_product_53,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    cfgs
}

fn score_gauss_discovery(root: &str) {
    let mut rows: Vec<_> = load_gauss_discovery_rows(root)
        .into_iter()
        .filter(|(x, _)| x.abs() >= 1e-10 && x.abs() < 0.7)
        .collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert!(!rows.is_empty());

    let sample_count = rows.len().min(128);
    let sample: Vec<_> = (0..sample_count)
        .map(|i| rows[i * (rows.len() - 1) / (sample_count - 1)])
        .collect();
    let args = [
        GaussArgMode::NativeMultiply,
        GaussArgMode::NativeDivide,
        GaussArgMode::X87MultiplyStore,
        GaussArgMode::X87DivideStore,
        GaussArgMode::X87SqrtDivideStore,
    ];
    let publishes = [
        GaussPublishMode::DirectHalf,
        GaussPublishMode::HalfOfOnePlusThenSubtract,
        GaussPublishMode::HalfPlusHalfThenSubtract,
        GaussPublishMode::SignSplitComplement,
    ];

    let mut beam: Vec<(usize, u64, u64, GaussCfg)> = Vec::new();
    for erf in all_erf_cfgs() {
        for arg in args {
            for publish in publishes {
                let cfg = GaussCfg { erf, arg, publish };
                let (mut exact, mut max_ulp, mut sum_ulp) = (0usize, 0u64, 0u64);
                for &(x, expected) in &sample {
                    let d = gauss_distance(eval_gauss_small(x, &cfg).to_bits(), expected);
                    exact += usize::from(d == 0);
                    max_ulp = max_ulp.max(d);
                    sum_ulp = sum_ulp.saturating_add(d);
                }
                beam.push((exact, max_ulp, sum_ulp, cfg));
            }
        }
    }
    beam.sort_by_key(|row| (usize::MAX - row.0, row.1, row.2));
    beam.truncate(512);

    let mut scores: Vec<(usize, u64, u64, GaussCfg)> = Vec::new();
    for &(_, _, _, cfg) in &beam {
        let (mut exact, mut max_ulp, mut sum_ulp) = (0usize, 0u64, 0u64);
        for &(x, expected) in &rows {
            let d = gauss_distance(eval_gauss_small(x, &cfg).to_bits(), expected);
            exact += usize::from(d == 0);
            max_ulp = max_ulp.max(d);
            sum_ulp = sum_ulp.saturating_add(d);
        }
        scores.push((exact, max_ulp, sum_ulp, cfg));
    }
    scores.sort_by_key(|row| (usize::MAX - row.0, row.1, row.2));
    println!(
        "GAUSS source-backed small-body race: {} ERF graphs x {} arg graphs x {} wrappers; {} stratified discovery rows -> top {} rescored on all {} regular-small rows; heldout path absent",
        all_erf_cfgs().len(),
        args.len(),
        publishes.len(),
        sample.len(),
        scores.len(),
        rows.len()
    );
    for (rank, &(exact, max_ulp, sum_ulp, cfg)) in scores.iter().take(32).enumerate() {
        println!(
            "#{:02} exact={}/{} max={} sum={} {:?}",
            rank + 1,
            exact,
            rows.len(),
            max_ulp,
            sum_ulp,
            cfg
        );
    }
    let best = scores[0].3;
    println!("best GAUSS candidate residuals:");
    for &(x, expected) in &rows {
        let got = eval_gauss_small(x, &best).to_bits();
        if got != expected {
            let delta = ordered(expected) - ordered(got);
            println!(
                "  x={x:.17e} bits=0x{:016x} got=0x{got:016x} want=0x{expected:016x} delta={delta}",
                x.to_bits()
            );
        }
    }
}

fn decode_gauss_q_rows(root: &str) -> Vec<(f64, u64)> {
    let rows = load_gauss_discovery_rows(root);
    let by_input: BTreeMap<u64, u64> = rows
        .iter()
        .map(|&(x, expected)| (x.to_bits(), expected))
        .collect();
    let mut decoded: BTreeMap<u64, u64> = BTreeMap::new();
    for &(x, positive_expected) in rows.iter().filter(|(x, _)| *x > f64::EPSILON && *x < 0.7) {
        let Some(&negative_expected) = by_input.get(&(x.to_bits() | (1u64 << 63))) else {
            continue;
        };
        let yp = f64::from_bits(positive_expected);
        let yn = f64::from_bits(negative_expected);
        let center_p = (2.0 * (0.5 - yp)).to_bits();
        let center_n = (2.0 * (yn + 0.5)).to_bits();
        let lo = center_p.min(center_n).saturating_sub(64);
        let hi = center_p.max(center_n).saturating_add(64);
        let candidates: Vec<u64> = (lo..=hi)
            .filter(|&bits| {
                let q = f64::from_bits(bits);
                ((1.0 - 0.5 * q) - 0.5).to_bits() == positive_expected
                    && (0.5 * q - 0.5).to_bits() == negative_expected
            })
            .collect();
        if candidates.len() != 1 {
            continue;
        }
        let z = x * std::f64::consts::FRAC_1_SQRT_2;
        if let Some(old) = decoded.insert(z.to_bits(), candidates[0]) {
            assert_eq!(old, candidates[0], "conflicting Q decode for z={z:?}");
        }
    }
    decoded
        .into_iter()
        .map(|(z, q)| (f64::from_bits(z), q))
        .collect()
}

fn decode_gauss_q_pairs(root: &str) -> Vec<(f64, u64)> {
    let rows = load_gauss_discovery_rows(root);
    let by_input: BTreeMap<u64, u64> = rows
        .iter()
        .map(|&(x, expected)| (x.to_bits(), expected))
        .collect();
    let mut decoded = Vec::new();
    for &(x, positive_expected) in rows.iter().filter(|(x, _)| *x > f64::EPSILON && *x < 0.7) {
        let Some(&negative_expected) = by_input.get(&(x.to_bits() | (1u64 << 63))) else {
            continue;
        };
        let yp = f64::from_bits(positive_expected);
        let yn = f64::from_bits(negative_expected);
        let center_p = (2.0 * (0.5 - yp)).to_bits();
        let center_n = (2.0 * (yn + 0.5)).to_bits();
        let lo = center_p.min(center_n).saturating_sub(64);
        let hi = center_p.max(center_n).saturating_add(64);
        let candidates: Vec<u64> = (lo..=hi)
            .filter(|&bits| {
                let q = f64::from_bits(bits);
                ((1.0 - 0.5 * q) - 0.5).to_bits() == positive_expected
                    && (0.5 * q - 0.5).to_bits() == negative_expected
            })
            .collect();
        if candidates.len() != 1 {
            continue;
        }
        decoded.push((x, candidates[0]));
    }
    decoded
}

fn delivered_gauss_input(x: f64, mode: GaussInputDelivery) -> Ext80 {
    match mode {
        GaussInputDelivery::NativeMultiplyStored => ef(x * std::f64::consts::FRAC_1_SQRT_2),
        GaussInputDelivery::NativeDivideStored => ef(x / std::f64::consts::SQRT_2),
        GaussInputDelivery::ExtendedMultiply => {
            ext_mul(&ef(x), &ef(std::f64::consts::FRAC_1_SQRT_2), CW)
        }
        GaussInputDelivery::ExtendedDivideStoredSqrt => {
            ext_div(&ef(x), &ef(std::f64::consts::SQRT_2), CW)
        }
        GaussInputDelivery::ExtendedDivideExtendedSqrt => {
            ext_div(&ef(x), &ext_sqrt(&ef(2.0), CW), CW)
        }
    }
}

fn score_gauss_input_delivery(root: &str) {
    let mut rows = decode_gauss_q_pairs(root);
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    let sample_count = 128usize;
    let sample: Vec<_> = (0..sample_count)
        .map(|i| rows[i * (rows.len() - 1) / (sample_count - 1)])
        .collect();
    let deliveries = [
        GaussInputDelivery::NativeMultiplyStored,
        GaussInputDelivery::NativeDivideStored,
        GaussInputDelivery::ExtendedMultiply,
        GaussInputDelivery::ExtendedDivideStoredSqrt,
        GaussInputDelivery::ExtendedDivideExtendedSqrt,
    ];
    let mut beam: Vec<(usize, u64, u64, GaussInputDelivery, Cfg)> = Vec::new();
    for cfg in all_erf_cfgs() {
        for delivery in deliveries {
            let (mut exact, mut max_ulp, mut sum_ulp) = (0usize, 0u64, 0u64);
            for &(x, expected) in &sample {
                let p = dbl(&eval_from_ext_input(
                    delivered_gauss_input(x, delivery),
                    &cfg,
                ));
                let got = (1.0 - p).to_bits();
                let d = (ordered(expected) as i128).abs_diff(ordered(got) as i128) as u64;
                exact += usize::from(d == 0);
                max_ulp = max_ulp.max(d);
                sum_ulp = sum_ulp.saturating_add(d);
            }
            beam.push((exact, max_ulp, sum_ulp, delivery, cfg));
        }
    }
    beam.sort_by_key(|row| (usize::MAX - row.0, row.1, row.2));
    beam.truncate(512);
    let mut scores = Vec::new();
    for &(_, _, _, delivery, cfg) in &beam {
        let (mut exact, mut max_ulp, mut sum_ulp) = (0usize, 0u64, 0u64);
        for &(x, expected) in &rows {
            let p = dbl(&eval_from_ext_input(
                delivered_gauss_input(x, delivery),
                &cfg,
            ));
            let got = (1.0 - p).to_bits();
            let d = (ordered(expected) as i128).abs_diff(ordered(got) as i128) as u64;
            exact += usize::from(d == 0);
            max_ulp = max_ulp.max(d);
            sum_ulp = sum_ulp.saturating_add(d);
        }
        scores.push((exact, max_ulp, sum_ulp, delivery, cfg));
    }
    scores.sort_by_key(|row| (usize::MAX - row.0, row.1, row.2));
    println!(
        "GAUSS input-delivery Q race: {} source body graphs x {} delivery graphs; {}-row stratified beam -> {} finalists on {} paired rows; heldout absent",
        all_erf_cfgs().len(),
        deliveries.len(),
        sample.len(),
        scores.len(),
        rows.len()
    );
    for (rank, &(exact, max_ulp, sum_ulp, delivery, cfg)) in scores.iter().take(32).enumerate() {
        println!(
            "#{:02} exact={}/{} max={} sum={} delivery={delivery:?} {cfg:?}",
            rank + 1,
            exact,
            rows.len(),
            max_ulp,
            sum_ulp
        );
    }
    let best = scores[0];
    println!("best input-delivery residuals:");
    for &(x, expected) in &rows {
        let p = dbl(&eval_from_ext_input(
            delivered_gauss_input(x, best.3),
            &best.4,
        ));
        let got = (1.0 - p).to_bits();
        if got != expected {
            println!(
                "  x=0x{:016x} z53=0x{:016x} got=0x{got:016x} want=0x{expected:016x} delta={}",
                x.to_bits(),
                (x * std::f64::consts::FRAC_1_SQRT_2).to_bits(),
                ordered(expected) - ordered(got)
            );
        }
    }
}

fn score_gauss_series_spills(root: &str) {
    let rows = decode_gauss_q_rows(root);
    assert!(!rows.is_empty(), "decoded GAUSS Q corpus is empty");
    let g = ext_add(&ext_one(), &gam1_half(GamMode::Binary64), CW);
    let mut scores: Vec<(usize, u64, u64, u16)> = Vec::new();
    for mask in 0u16..(1 << 13) {
        let (mut exact, mut max_ulp, mut sum_ulp) = (0usize, 0u64, 0u64);
        for &(z, expected) in &rows {
            let x = ext_mul(&ef(z), &ef(z), CW);
            let j = series_j_spill_mask(&x, mask);
            let inner = ef(1.0 - dbl(&j));
            let g_inner = ef(dbl(&ext_mul(&g, &inner, CW)));
            let p = dbl(&ext_mul(&ef(z), &g_inner, CW));
            let got = (1.0 - p).to_bits();
            let d = (ordered(expected) as i128).abs_diff(ordered(got) as i128) as u64;
            exact += usize::from(d == 0);
            max_ulp = max_ulp.max(d);
            sum_ulp = sum_ulp.saturating_add(d);
        }
        scores.push((exact, max_ulp, sum_ulp, mask));
    }
    scores.sort_by_key(|row| (usize::MAX - row.0, row.1, row.2));
    println!(
        "GAUSS decoded-Q branch-190 series race: {} per-site spill masks x {} current-build rows; heldout absent",
        scores.len(),
        rows.len()
    );
    for (rank, &(exact, max_ulp, sum_ulp, mask)) in scores.iter().take(32).enumerate() {
        println!(
            "#{:02} exact={}/{} max={} sum={} mask=0b{mask:013b}",
            rank + 1,
            exact,
            rows.len(),
            max_ulp,
            sum_ulp
        );
    }
    let best = scores[0].3;
    println!("best series-mask residuals:");
    for &(z, expected) in &rows {
        let x = ext_mul(&ef(z), &ef(z), CW);
        let j = series_j_spill_mask(&x, best);
        let inner = ef(1.0 - dbl(&j));
        let g_inner = ef(dbl(&ext_mul(&g, &inner, CW)));
        let p = dbl(&ext_mul(&ef(z), &g_inner, CW));
        let got = (1.0 - p).to_bits();
        if got != expected {
            println!(
                "  z=0x{:016x} got=0x{got:016x} want=0x{expected:016x} delta={}",
                z.to_bits(),
                ordered(expected) - ordered(got)
            );
        }
    }
}

fn gauss_tiny_direct(x: f64) -> f64 {
    // Best source-backed tiny graph: published GAM1 mixed-spill value,
    // (x*g) stored, multiplied by the extended folded 1/(2*sqrt(2)).
    let mantissa = 0x906e_ba82_14db_6bd6u64;
    let mut bytes = [0u8; 10];
    bytes[..8].copy_from_slice(&mantissa.to_le_bytes());
    bytes[8] = 0xff;
    bytes[9] = 0x3f;
    let g = Ext80(bytes);
    let xg = ef(dbl(&ext_mul(&ef(x), &g, CW)));
    let folded = ext_mul(&ef(std::f64::consts::FRAC_1_SQRT_2), &ef(0.5), CW);
    let mut out = dbl(&ext_mul(&xg, &folded, CW));
    if out.abs() < f64::MIN_POSITIVE {
        out = 0.0;
    }
    out
}

fn gauss_public_composite(x: f64) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    // The frozen route-discovery ULP window pins the inclusive switch at the
    // binary64 value of the public decimal literal 1e-15.
    if x.abs() <= 1e-15 {
        return gauss_tiny_direct(x);
    }
    let z = x.abs() * std::f64::consts::FRAC_1_SQRT_2;
    let q = if z < 0.5 {
        let cfg = Cfg {
            x: XMode::Extended,
            series_53: false,
            j_53: false,
            gam: GamMode::Binary64,
            g_53: false,
            w: WMode::X87Continuous,
            inner: InnerMode::Binary64Direct,
            assoc: Assoc::WThenGInner,
            first_product_53: true,
        };
        1.0 - eval(z, &cfg)
    } else {
        libm::erfc(z)
    };
    if x < 0.0 {
        0.5 * q - 0.5
    } else {
        (1.0 - 0.5 * q) - 0.5
    }
}

fn score_gauss_public_composite(root: &str) {
    let rows = load_gauss_discovery_rows(root);
    let mut groups: BTreeMap<&'static str, (usize, usize, u64, u64)> = BTreeMap::new();
    for &(x, expected) in &rows {
        let got = gauss_public_composite(x).to_bits();
        let d = gauss_distance(got, expected);
        let group = if x.abs() <= 1e-15 {
            "tiny-direct"
        } else if x.abs() < std::f64::consts::FRAC_1_SQRT_2 {
            "branch190"
        } else if x.abs() < 9.0 {
            "erfc-tail"
        } else {
            "saturation"
        };
        let entry = groups.entry(group).or_insert((0, 0, 0, 0));
        entry.0 += usize::from(d == 0);
        entry.1 += 1;
        entry.2 = entry.2.max(d);
        entry.3 = entry.3.saturating_add(d);
    }
    let exact: usize = groups.values().map(|row| row.0).sum();
    println!(
        "GAUSS coherent public composite discovery score: {exact}/{} exact; heldout absent",
        rows.len()
    );
    for (group, (group_exact, count, max_ulp, sum_ulp)) in groups {
        println!("  {group}: exact={group_exact}/{count} max={max_ulp} sum={sum_ulp}");
    }
}

fn score_gauss_small_route(root: &str) {
    let rows = load_gauss_discovery_rows(root);
    // The decoded-Q leader, held fixed so this race measures publication
    // routing rather than allowing body changes to absorb wrapper effects.
    let cfg = Cfg {
        x: XMode::Extended,
        series_53: false,
        j_53: false,
        gam: GamMode::Binary64,
        g_53: false,
        w: WMode::X87Continuous,
        inner: InnerMode::Binary64Direct,
        assoc: Assoc::WThenGInner,
        first_product_53: true,
    };
    // count, direct exact, sign-split exact, both exact, neither exact
    let mut bins: BTreeMap<i32, [usize; 5]> = BTreeMap::new();
    let mut boundary_rows = Vec::new();
    for &(x, expected) in rows
        .iter()
        .filter(|(x, _)| x.abs() > f64::EPSILON && x.abs() < 1e-3)
    {
        let z = x.abs() * std::f64::consts::FRAC_1_SQRT_2;
        let p = eval(z, &cfg);
        let signed_p = if x < 0.0 { -p } else { p };
        let direct = (0.5 * signed_p).to_bits();
        let q = 1.0 - p;
        let split = if x < 0.0 {
            0.5 * q - 0.5
        } else {
            (1.0 - 0.5 * q) - 0.5
        }
        .to_bits();
        let direct_exact = direct == expected;
        let split_exact = split == expected;
        let bin = x.abs().log2().floor() as i32;
        if (-51..=-49).contains(&bin) {
            boundary_rows.push((x, expected, direct, split));
        }
        let entry = bins.entry(bin).or_default();
        entry[0] += 1;
        entry[1] += usize::from(direct_exact);
        entry[2] += usize::from(split_exact);
        entry[3] += usize::from(direct_exact && split_exact);
        entry[4] += usize::from(!direct_exact && !split_exact);
    }
    println!("GAUSS discovery small-route race with fixed decoded-Q body; heldout absent");
    println!("  floor(log2|x|): rows direct split both neither");
    for (exponent, counts) in bins {
        println!(
            "  {exponent:4}: {:4} {:4} {:4} {:4} {:4}",
            counts[0], counts[1], counts[2], counts[3], counts[4]
        );
    }
    boundary_rows.sort_by(|a, b| a.0.abs().total_cmp(&b.0.abs()).then(a.0.total_cmp(&b.0)));
    println!("  route-boundary rows (-51<=floor(log2|x|)<=-49):");
    for (x, expected, direct, split) in boundary_rows {
        println!(
            "    x=0x{:016x} ({x:.17e}) want=0x{expected:016x} direct=0x{direct:016x} d={} split=0x{split:016x} d={}",
            x.to_bits(),
            ordered(expected) - ordered(direct),
            ordered(expected) - ordered(split),
        );
    }
}

fn score_gauss_route_capture(root: &str) {
    let rows = load_gauss_route_discovery_rows(root);
    let cfg = Cfg {
        x: XMode::Extended,
        series_53: false,
        j_53: false,
        gam: GamMode::Binary64,
        g_53: false,
        w: WMode::X87Continuous,
        inner: InnerMode::Binary64Direct,
        assoc: Assoc::WThenGInner,
        first_product_53: true,
    };
    let mut by_magnitude: BTreeMap<u64, [usize; 4]> = BTreeMap::new();
    let (mut direct_exact, mut split_exact) = (0usize, 0usize);
    for &(x, expected) in &rows {
        let direct = gauss_tiny_direct(x).to_bits();
        let z = x.abs() * std::f64::consts::FRAC_1_SQRT_2;
        let p = eval(z, &cfg);
        let q = 1.0 - p;
        let split = if x < 0.0 {
            0.5 * q - 0.5
        } else {
            (1.0 - 0.5 * q) - 0.5
        }
        .to_bits();
        let dd = gauss_distance(direct, expected);
        let sd = gauss_distance(split, expected);
        direct_exact += usize::from(dd == 0);
        split_exact += usize::from(sd == 0);
        let entry = by_magnitude.entry(x.abs().to_bits()).or_default();
        entry[0] += 1;
        entry[1] += usize::from(dd < sd);
        entry[2] += usize::from(sd < dd);
        entry[3] += usize::from(sd == dd);
    }
    let last_direct = by_magnitude
        .iter()
        .filter(|(_, counts)| counts[1] > counts[2])
        .next_back();
    let first_split = by_magnitude
        .iter()
        .find(|(_, counts)| counts[2] > counts[1]);
    println!(
        "GAUSS route-discovery classification: {} rows; tiny-direct exact={direct_exact}/{}; sign-split exact={split_exact}/{}; heldout path absent",
        rows.len(),
        rows.len(),
        rows.len()
    );
    if let Some((&bits, counts)) = last_direct {
        println!(
            "  last direct-class magnitude=0x{bits:016x} ({:.17e}) votes={counts:?}",
            f64::from_bits(bits)
        );
    }
    if let Some((&bits, counts)) = first_split {
        println!(
            "  first sign-split-class magnitude=0x{bits:016x} ({:.17e}) votes={counts:?}",
            f64::from_bits(bits)
        );
    }
    for (label, predicate) in [("abs<=1e-15", true), ("abs<1e-15", false)] {
        let (mut exact, mut preferred) = (0usize, 0usize);
        for &(x, expected) in &rows {
            let use_direct = if predicate {
                x.abs() <= 1e-15
            } else {
                x.abs() < 1e-15
            };
            let direct = gauss_tiny_direct(x).to_bits();
            let z = x.abs() * std::f64::consts::FRAC_1_SQRT_2;
            let p = eval(z, &cfg);
            let q = 1.0 - p;
            let split = if x < 0.0 {
                0.5 * q - 0.5
            } else {
                (1.0 - 0.5 * q) - 0.5
            }
            .to_bits();
            let got = if use_direct { direct } else { split };
            exact += usize::from(got == expected);
            preferred += usize::from(
                gauss_distance(got, expected)
                    <= gauss_distance(if use_direct { split } else { direct }, expected),
            );
        }
        println!(
            "  predicate {label}: exact={exact}/{} preferred={preferred}/{}",
            rows.len(),
            rows.len()
        );
    }
}

fn score_gauss_decoded_q(root: &str) {
    let rows = decode_gauss_q_rows(root);
    assert!(!rows.is_empty(), "decoded GAUSS Q corpus is empty");
    let mut scores: Vec<(usize, u64, u64, u8, Cfg)> = Vec::new();
    for cfg in all_erf_cfgs() {
        for q_mode in 0u8..4 {
            let (mut exact, mut max_ulp, mut sum_ulp) = (0usize, 0u64, 0u64);
            for &(z, expected_q) in &rows {
                let p = eval_ext(z, &cfg);
                let got_q = match q_mode {
                    0 => (1.0 - dbl(&p)).to_bits(),
                    1 => dbl(&ext_sub(&ext_one(), &p, CW)).to_bits(),
                    2 => dbl(&ext_add(&ef(0.5), &ext_sub(&ef(0.5), &p, CW), CW)).to_bits(),
                    _ => {
                        let pd = dbl(&p);
                        (0.5 + (0.5 - pd)).to_bits()
                    }
                };
                let d = (ordered(expected_q) as i128).abs_diff(ordered(got_q) as i128) as u64;
                exact += usize::from(d == 0);
                max_ulp = max_ulp.max(d);
                sum_ulp = sum_ulp.saturating_add(d);
            }
            scores.push((exact, max_ulp, sum_ulp, q_mode, cfg));
        }
    }
    scores.sort_by_key(|row| (usize::MAX - row.0, row.1, row.2));
    let libm_exact = rows
        .iter()
        .filter(|(z, expected)| libm::erfc(*z).to_bits() == *expected)
        .count();
    println!(
        "GAUSS symmetric-pair decoded Q race: {} distinct current-build z rows; {} source-backed branch-190 graphs; libm-erfc={}/{}; heldout path absent",
        rows.len(),
        scores.len(),
        libm_exact,
        rows.len()
    );
    for (rank, &(exact, max_ulp, sum_ulp, q_mode, cfg)) in scores.iter().take(32).enumerate() {
        println!(
            "#{:02} exact={}/{} max={} sum={} q_publish={} {:?}",
            rank + 1,
            exact,
            rows.len(),
            max_ulp,
            sum_ulp,
            [
                "stored-p-direct",
                "extended-direct",
                "extended-compensated",
                "stored-p-compensated",
            ][q_mode as usize],
            cfg
        );
    }
    let best_q_mode = scores[0].3;
    let best = scores[0].4;
    println!("best decoded-Q residuals:");
    for &(z, expected) in &rows {
        let p = eval_ext(z, &best);
        let got = match best_q_mode {
            0 => (1.0 - dbl(&p)).to_bits(),
            1 => dbl(&ext_sub(&ext_one(), &p, CW)).to_bits(),
            2 => dbl(&ext_add(&ef(0.5), &ext_sub(&ef(0.5), &p, CW), CW)).to_bits(),
            _ => {
                let pd = dbl(&p);
                (0.5 + (0.5 - pd)).to_bits()
            }
        };
        if got != expected {
            println!(
                "  z=0x{:016x} got=0x{got:016x} want=0x{expected:016x} delta={}",
                z.to_bits(),
                ordered(expected) - ordered(got)
            );
        }
    }

    const G_CENTER: u64 = 0x906e_ba82_14db_6c6f;
    let mut g_scores: Vec<(usize, u64, u64, i32, u64)> = Vec::new();
    for offset in -2048i32..=2048 {
        let mantissa = (G_CENTER as i128 + offset as i128) as u64;
        let mut bytes = [0u8; 10];
        bytes[..8].copy_from_slice(&mantissa.to_le_bytes());
        bytes[8] = 0xff;
        bytes[9] = 0x3f;
        let g = Ext80(bytes);
        let (mut exact, mut max_ulp, mut sum_ulp) = (0usize, 0u64, 0u64);
        for &(z, expected) in &rows {
            let p = dbl(&eval_with_g_ext(z, &best, g));
            let got = (1.0 - p).to_bits();
            let d = (ordered(expected) as i128).abs_diff(ordered(got) as i128) as u64;
            exact += usize::from(d == 0);
            max_ulp = max_ulp.max(d);
            sum_ulp = sum_ulp.saturating_add(d);
        }
        g_scores.push((exact, max_ulp, sum_ulp, offset, mantissa));
    }
    g_scores.sort_by_key(|row| (usize::MAX - row.0, row.1, row.2));
    println!("decoded-Q narrow effective-g race (+/-2048 ext64 mantissa units):");
    for (rank, &(exact, max_ulp, sum_ulp, offset, mantissa)) in g_scores.iter().take(24).enumerate()
    {
        println!(
            "  #{:02} exact={}/{} max={} sum={} offset={:+} g=0x{mantissa:016x}",
            rank + 1,
            exact,
            rows.len(),
            max_ulp,
            sum_ulp,
            offset
        );
    }
}

#[derive(Debug)]
struct Score {
    exact: usize,
    max_ulp: i64,
    sum_abs_ulp: u64,
    cfg: Cfg,
}

fn main() {
    let dir = std::env::args().nth(1).expect("G3-01-dist directory");
    if std::env::args().nth(2).as_deref() == Some("gauss-composite") {
        score_gauss_public_composite(&dir);
        return;
    }
    if std::env::args().nth(2).as_deref() == Some("gauss-series") {
        score_gauss_series_spills(&dir);
        return;
    }
    if std::env::args().nth(2).as_deref() == Some("gauss-route") {
        score_gauss_small_route(&dir);
        return;
    }
    if std::env::args().nth(2).as_deref() == Some("gauss-route-capture") {
        score_gauss_route_capture(&dir);
        return;
    }
    if std::env::args().nth(2).as_deref() == Some("gauss-input") {
        score_gauss_input_delivery(&dir);
        return;
    }
    if std::env::args().nth(2).as_deref() == Some("gauss-q") {
        score_gauss_decoded_q(&dir);
        return;
    }
    if std::env::args().nth(2).as_deref() == Some("gauss") {
        score_gauss_discovery(&dir);
        return;
    }
    if std::env::args().nth(2).as_deref() == Some("gam-spills") {
        gam1_spill_search();
        return;
    }
    if std::env::args().nth(2).as_deref() == Some("gam-score") {
        score_gam1_spills(&dir);
        return;
    }
    if std::env::args().nth(2).as_deref() == Some("literal") {
        score_literal_gratio_returns(&dir);
        return;
    }
    let rows = load_rows(&dir);
    assert!(!rows.is_empty(), "no discovery rows found");
    println!(
        "{} distinct source-backed z<0.5 discovery rows; heldout path is not in the reader",
        rows.len()
    );

    let x_modes = [XMode::Extended, XMode::Native53, XMode::X87DoubleRounded];
    let gam_modes = [
        GamMode::Binary64,
        GamMode::Extended,
        GamMode::ExtendedReturn53,
        GamMode::TwoOverSqrtPi53,
    ];
    let w_modes = [
        WMode::X87Continuous,
        WMode::X87Argument53,
        WMode::DistributionPow,
        WMode::X87DirectPow,
        WMode::ExcelLnMulExp,
        WMode::Sqrt,
        WMode::InputZ,
        WMode::LibmPow,
    ];
    let inner_modes = [
        InnerMode::ExtendedCompensated,
        InnerMode::ExtendedDirect,
        InnerMode::Binary64Compensated,
        InnerMode::Binary64Direct,
    ];
    let assocs = [Assoc::WgThenInner, Assoc::WThenGInner, Assoc::WInnerThenG];

    let mut scores = Vec::new();
    for x in x_modes {
        for series_53 in [false, true] {
            for j_53 in [false, true] {
                for gam in gam_modes {
                    for g_53 in [false, true] {
                        for w in w_modes {
                            for inner in inner_modes {
                                for assoc in assocs {
                                    for first_product_53 in [false, true] {
                                        let cfg = Cfg {
                                            x,
                                            series_53,
                                            j_53,
                                            gam,
                                            g_53,
                                            w,
                                            inner,
                                            assoc,
                                            first_product_53,
                                        };
                                        let (mut exact, mut max_ulp, mut sum_abs_ulp) =
                                            (0usize, 0i64, 0u64);
                                        for &(z, expected) in &rows {
                                            let got = eval(z, &cfg).to_bits();
                                            let delta = ordered(expected) - ordered(got);
                                            if delta == 0 {
                                                exact += 1;
                                            }
                                            let abs = delta.unsigned_abs();
                                            max_ulp = max_ulp.max(abs as i64);
                                            sum_abs_ulp = sum_abs_ulp.saturating_add(abs);
                                        }
                                        scores.push(Score {
                                            exact,
                                            max_ulp,
                                            sum_abs_ulp,
                                            cfg,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    scores.sort_by_key(|s| (usize::MAX - s.exact, s.max_ulp, s.sum_abs_ulp));
    println!("raced {} explicit calculation graphs", scores.len());
    for (rank, score) in scores.iter().take(24).enumerate() {
        println!(
            "#{:02} exact={:4}/{} max_ulp={} sum_abs_ulp={} {:?}",
            rank + 1,
            score.exact,
            rows.len(),
            score.max_ulp,
            score.sum_abs_ulp,
            score.cfg
        );
    }
    println!("best score by w-provider:");
    for w in w_modes {
        let score = scores.iter().find(|score| score.cfg.w == w).unwrap();
        println!(
            "  {:?}: exact={}/{} max_ulp={} sum_abs_ulp={} {:?}",
            w,
            score.exact,
            rows.len(),
            score.max_ulp,
            score.sum_abs_ulp,
            score.cfg
        );
    }
    println!("best score by normalizer evaluation:");
    for gam in gam_modes {
        let score = scores.iter().find(|score| score.cfg.gam == gam).unwrap();
        println!(
            "  {:?}: exact={}/{} max_ulp={} sum_abs_ulp={} {:?}",
            gam,
            score.exact,
            rows.len(),
            score.max_ulp,
            score.sum_abs_ulp,
            score.cfg
        );
    }
}
