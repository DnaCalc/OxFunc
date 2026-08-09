//! W109 G4-03 ACOTH calculation-graph racer.
//!
//! This binary is deliberately oracle-offline.  It scores explicit reciprocal,
//! logarithm, and publication-store graphs against previously captured Excel
//! witness sets.  New answer files are picked up only when they exist, so the
//! same scorer can be used for discovery and frozen held-out gates.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const CW: u16 = 0x133f;
const CRT_LOG1P_BOUND: f64 = 0.292_893_218_813_452_5;
const CRT_ACOTH_SWITCH: f64 = 2.0 + std::f64::consts::SQRT_2;

fn ext(x: f64) -> rx::Ext80 {
    rx::ext_from_f64(x)
}

fn to_f64(x: &rx::Ext80) -> f64 {
    rx::ext_to_f64(x, CW)
}

fn x87_add(a: f64, b: f64) -> f64 {
    to_f64(&rx::ext_add(&ext(a), &ext(b), CW))
}

fn x87_sub(a: f64, b: f64) -> f64 {
    to_f64(&rx::ext_sub(&ext(a), &ext(b), CW))
}

fn x87_div(a: f64, b: f64) -> f64 {
    to_f64(&rx::ext_div(&ext(a), &ext(b), CW))
}

fn half_ext(value: &rx::Ext80) -> f64 {
    to_f64(&rx::ext_mul(value, &ext(0.5), CW))
}

fn ln_ext(x: f64) -> rx::Ext80 {
    rx::ext_fyl2x(&rx::ext_ln2(), &ext(x), CW)
}

fn ln1p_ext(x: f64) -> rx::Ext80 {
    rx::ext_fyl2xp1(&rx::ext_ln2(), &ext(x), CW)
}

fn ratio_direct(a: f64, mask: u8) -> f64 {
    let numerator = if mask & 1 != 0 {
        x87_add(a, 1.0)
    } else {
        a + 1.0
    };
    let denominator = if mask & 2 != 0 {
        x87_sub(a, 1.0)
    } else {
        a - 1.0
    };
    let ratio = if mask & 4 != 0 {
        x87_div(numerator, denominator)
    } else {
        numerator / denominator
    };
    half_ext(&ln_ext(ratio))
}

fn reciprocal(a: f64, x87: bool) -> f64 {
    if x87 {
        x87_div(1.0, a)
    } else {
        1.0 / a
    }
}

/// Superseded W109 discovery hypothesis: reciprocal followed by two FYL2XP1
/// instructions, with the difference and half-scale retained in Ext80.
fn pair_fyl2xp1(a: f64, x87_recip: bool, store_logs: bool, stored_diff: bool) -> f64 {
    let t = reciprocal(a, x87_recip);
    let mut positive = ln1p_ext(t);
    let mut negative = ln1p_ext(-t);
    if store_logs {
        positive = ext(to_f64(&positive));
        negative = ext(to_f64(&negative));
    }
    let difference = rx::ext_sub(&positive, &negative, CW);
    if stored_diff {
        0.5 * to_f64(&difference)
    } else {
        half_ext(&difference)
    }
}

fn pair_fyl2xp1_register_recip(a: f64, cw: u16, store_logs: bool) -> f64 {
    let t = rx::ext_div(&rx::ext_one(), &ext(a), cw);
    let mut positive = rx::ext_fyl2xp1(&rx::ext_ln2(), &t, cw);
    let mut negative = rx::ext_fyl2xp1(&rx::ext_ln2(), &rx::ext_chs(&t, cw), cw);
    if store_logs {
        positive = ext(rx::ext_to_f64(&positive, cw));
        negative = ext(rx::ext_to_f64(&negative, cw));
    }
    let difference = rx::ext_sub(&positive, &negative, cw);
    rx::ext_to_f64(&rx::ext_mul(&difference, &ext(0.5), cw), cw)
}

fn pair_fyl2xp1_stored_recip_cw(a: f64, x87_recip: bool, cw: u16) -> f64 {
    let t = reciprocal(a, x87_recip);
    let positive = rx::ext_fyl2xp1(&rx::ext_ln2(), &ext(t), cw);
    let negative = rx::ext_fyl2xp1(&rx::ext_ln2(), &ext(-t), cw);
    let difference = rx::ext_sub(&positive, &negative, cw);
    rx::ext_to_f64(&rx::ext_mul(&difference, &ext(0.5), cw), cw)
}

/// Reciprocal pair formed as LN(1+t)-LN(1-t).  `sum_mask` chooses ordinary
/// binary64 (0) or x87 RN53(RN64(op)) (1) staging of both wrapper operations;
/// `store_logs` chooses whether each logarithm is published before subtraction.
fn pair_ln(a: f64, x87_recip: bool, sum_mask: bool, store_logs: bool, stored_diff: bool) -> f64 {
    let t = reciprocal(a, x87_recip);
    let plus = if sum_mask { x87_add(1.0, t) } else { 1.0 + t };
    let minus = if sum_mask { x87_sub(1.0, t) } else { 1.0 - t };
    let mut positive = ln_ext(plus);
    let mut negative = ln_ext(minus);
    if store_logs {
        positive = ext(to_f64(&positive));
        negative = ext(to_f64(&negative));
    }
    let difference = rx::ext_sub(&positive, &negative, CW);
    if stored_diff {
        0.5 * to_f64(&difference)
    } else {
        half_ext(&difference)
    }
}

/// Difference of original-scale logarithms, avoiding formation of either the
/// ratio or reciprocal: `0.5 * (ln(a + 1) - ln(a - 1))`.
fn pair_ln_original(a: f64, x87_sum: bool, store_logs: bool, stored_diff: bool) -> f64 {
    let plus = if x87_sum { x87_add(a, 1.0) } else { a + 1.0 };
    let minus = if x87_sum { x87_sub(a, 1.0) } else { a - 1.0 };
    let mut positive = ln_ext(plus);
    let mut negative = ln_ext(minus);
    if store_logs {
        positive = ext(to_f64(&positive));
        negative = ext(to_f64(&negative));
    }
    let difference = rx::ext_sub(&positive, &negative, CW);
    if stored_diff {
        0.5 * to_f64(&difference)
    } else {
        half_ext(&difference)
    }
}

/// Published legacy-log1p pair: use FYL2XP1 only inside its architectural
/// accuracy interval and otherwise FYL2X(1+x).  Each call returns binary64,
/// mirroring a separately-called helper rather than one fused pair routine.
fn crt_log1p_stored(x: f64, x87_sum: bool) -> f64 {
    if x.abs() < CRT_LOG1P_BOUND {
        to_f64(&ln1p_ext(x))
    } else {
        let u = if x87_sum {
            if x.is_sign_negative() {
                x87_sub(1.0, -x)
            } else {
                x87_add(1.0, x)
            }
        } else {
            1.0 + x
        };
        to_f64(&ln_ext(u))
    }
}

fn pair_crt_stored(a: f64, x87_recip: bool, x87_sum: bool, x87_diff: bool) -> f64 {
    let t = reciprocal(a, x87_recip);
    let positive = crt_log1p_stored(t, x87_sum);
    let negative = crt_log1p_stored(-t, x87_sum);
    let difference = if x87_diff {
        x87_sub(positive, negative)
    } else {
        positive - negative
    };
    0.5 * difference
}

fn l1p_kahan(x: f64) -> f64 {
    let u = 1.0 + x;
    if u == 1.0 {
        x
    } else {
        rx::excel_ln(u) * x / (u - 1.0)
    }
}

fn l1p_kahan2(x: f64) -> f64 {
    let u = 1.0 + x;
    if u == 1.0 {
        x
    } else {
        rx::excel_ln(u) * (x / (u - 1.0))
    }
}

fn pair_stored_l1p(a: f64, x87_recip: bool, x87_diff: bool, l1p: fn(f64) -> f64) -> f64 {
    let t = reciprocal(a, x87_recip);
    let positive = l1p(t);
    let negative = l1p(-t);
    let difference = if x87_diff {
        x87_sub(positive, negative)
    } else {
        positive - negative
    };
    0.5 * difference
}

fn crt_log1p_ext(x: f64, far_sum_stored: bool) -> rx::Ext80 {
    if x.abs() < CRT_LOG1P_BOUND {
        ln1p_ext(x)
    } else if far_sum_stored {
        ln_ext(1.0 + x)
    } else {
        let u = rx::ext_add(&rx::ext_one(), &ext(x), CW);
        rx::ext_fyl2x(&rx::ext_ln2(), &u, CW)
    }
}

/// fdlibm/musl-family ATANH reduction on the positive reciprocal.  For
/// `t < 0.5`, the cancellation-resistant argument is
/// `2*t + 2*t*t/(1-t)` rather than the algebraically collapsed `2*t/(1-t)`.
/// `op_mask` selects x87 spill staging for add/sub/mul/div in that order.
fn atanh_reduced_arg(t: f64, op_mask: u8) -> f64 {
    let sub = |a, b| {
        if op_mask & 2 != 0 {
            x87_sub(a, b)
        } else {
            a - b
        }
    };
    let mul = |a, b| {
        if op_mask & 4 != 0 {
            to_f64(&rx::ext_mul(&ext(a), &ext(b), CW))
        } else {
            a * b
        }
    };
    let div = |a, b| {
        if op_mask & 8 != 0 {
            x87_div(a, b)
        } else {
            a / b
        }
    };
    let add = |a, b| {
        if op_mask & 1 != 0 {
            x87_add(a, b)
        } else {
            a + b
        }
    };
    if t < 0.5 {
        let twice = mul(2.0, t);
        let square_term = mul(twice, t);
        add(twice, div(square_term, sub(1.0, t)))
    } else {
        mul(2.0, div(t, sub(1.0, t)))
    }
}

fn fd_atanh_crt(
    a: f64,
    x87_recip: bool,
    op_mask: u8,
    far_sum_stored: bool,
    log_stored: bool,
) -> f64 {
    let t = reciprocal(a, x87_recip);
    let arg = atanh_reduced_arg(t, op_mask);
    let logarithm = crt_log1p_ext(arg, far_sum_stored);
    if log_stored {
        0.5 * to_f64(&logarithm)
    } else {
        half_ext(&logarithm)
    }
}

fn collapsed_atanh_crt(a: f64, x87_recip: bool, far_sum_stored: bool) -> f64 {
    let t = reciprocal(a, x87_recip);
    let arg = (2.0 * t) / (1.0 - t);
    half_ext(&crt_log1p_ext(arg, far_sum_stored))
}

/// ACOTH written as one cancellation-free `log1p` call.  These are distinct
/// graphs even though both are algebraically equal to ACOTH for positive `a`:
///
///   0.5 * log1p( 2 / (a - 1))
///  -0.5 * log1p(-2 / (a + 1))
///
/// `op_mask` selects x87 spill staging for the wrapper add/sub (bit 0), divide
/// (bit 1), and final half-scale (bit 2).  `log_kind` is raw FYL2XP1 (0), the
/// legacy conditional FYL2XP1/FYL2X helper (1), or the portable faithful
/// binary64 log1p core (2).
fn acoth_one_sided(a: f64, upper: bool, op_mask: u8, log_kind: u8) -> f64 {
    let denominator = if upper {
        if op_mask & 1 != 0 {
            x87_sub(a, 1.0)
        } else {
            a - 1.0
        }
    } else if op_mask & 1 != 0 {
        x87_add(a, 1.0)
    } else {
        a + 1.0
    };
    let numerator = if upper { 2.0 } else { -2.0 };
    let argument = if op_mask & 2 != 0 {
        x87_div(numerator, denominator)
    } else {
        numerator / denominator
    };
    let logarithm = match log_kind {
        0 => to_f64(&ln1p_ext(argument)),
        1 => to_f64(&crt_log1p_ext(argument, true)),
        2 => rx::excel_log1p(argument),
        _ => unreachable!(),
    };
    let signed_logarithm = if upper { logarithm } else { -logarithm };
    if op_mask & 4 != 0 {
        to_f64(&rx::ext_mul(&ext(signed_logarithm), &ext(0.5), CW))
    } else {
        0.5 * signed_logarithm
    }
}

// Cephes 2.8 ATANH minimax coefficients (published Netlib source, June 2000).
// The small body is x + x^3 P(x^2)/Q(x^2), |x| < 0.5.
const CEPHES_P: [f64; 5] = [
    -8.540_743_319_296_693e-1,
    1.204_268_613_840_723_8e1,
    -4.612_528_841_987_327e1,
    6.545_667_286_765_443e1,
    -3.090_925_393_798_669_4e1,
];
const CEPHES_Q: [f64; 5] = [
    -1.956_388_493_769_116_5e1,
    1.089_380_921_471_402_6e2,
    -2.498_394_013_258_936e2,
    2.520_066_756_913_445_6e2,
    -9.272_776_181_396_011e1,
];

fn polevl(z: f64, coefficients: &[f64]) -> f64 {
    coefficients
        .iter()
        .skip(1)
        .fold(coefficients[0], |value, coefficient| {
            value * z + coefficient
        })
}

fn p1evl(z: f64, coefficients: &[f64]) -> f64 {
    coefficients
        .iter()
        .skip(1)
        .fold(z + coefficients[0], |value, coefficient| {
            value * z + coefficient
        })
}

fn cephes_atanh_recip(a: f64, x87_recip: bool) -> f64 {
    let t = reciprocal(a, x87_recip);
    if t < 1.0e-7 {
        return t;
    }
    if t < 0.5 {
        let z = t * t;
        return t + t * z * (polevl(z, &CEPHES_P) / p1evl(z, &CEPHES_Q));
    }
    ratio_direct(a, 0)
}

fn cephes_atanh_recip_ext(a: f64, x87_recip: bool) -> f64 {
    let t = reciprocal(a, x87_recip);
    if t < 1.0e-7 {
        return t;
    }
    if t >= 0.5 {
        return ratio_direct(a, 0);
    }
    let te = ext(t);
    let z = rx::ext_mul(&te, &te, CW);
    let mut p = ext(CEPHES_P[0]);
    for coefficient in &CEPHES_P[1..] {
        p = rx::ext_add(&rx::ext_mul(&p, &z, CW), &ext(*coefficient), CW);
    }
    let mut q = rx::ext_add(&z, &ext(CEPHES_Q[0]), CW);
    for coefficient in &CEPHES_Q[1..] {
        q = rx::ext_add(&rx::ext_mul(&q, &z, CW), &ext(*coefficient), CW);
    }
    let rational = rx::ext_div(&p, &q, CW);
    let correction = rx::ext_mul(&rx::ext_mul(&te, &z, CW), &rational, CW);
    to_f64(&rx::ext_add(&te, &correction, CW))
}

/// AMD ACML 4-era scalar ATANH [5,5] minimax, from the published 2008--2009
/// `open64_libacml_mv` source.  This is a historically plausible Windows/x64
/// substrate and is kept as a named clean-room candidate rather than fitted.
fn amd_atanh_recip(a: f64, x87_recip: bool) -> f64 {
    let x = reciprocal(a, x87_recip);
    if x < f64::from_bits(0x3e30_0000_0000_0000) {
        return if x < f64::MIN_POSITIVE { 0.0 } else { x };
    }
    if x >= 0.5 {
        return ratio_direct(a, 0);
    }
    let t = x * x;
    let numerator = 0.474_825_735_897_473_56
        + (-1.102_835_679_784_634_2
            + (0.884_681_425_365_016_5
                + (-0.281_802_109_617_808_14
                    + (0.028_728_638_600_548_515 - 0.000_104_681_588_927_531_37 * t) * t)
                    * t)
                * t)
            * t;
    let denominator = 1.424_477_207_692_420_6
        + (-4.163_193_363_969_355
            + (4.541_470_062_608_451
                + (-2.260_888_374_898_849
                    + (0.495_611_965_555_031 - 0.035_861_554_370_169_54 * t) * t)
                    * t)
                * t)
            * t;
    x + x * t * (numerator / denominator)
}

fn amd_atanh_recip_ext(a: f64, x87_recip: bool) -> f64 {
    let x = reciprocal(a, x87_recip);
    if x < f64::from_bits(0x3e30_0000_0000_0000) {
        return if x < f64::MIN_POSITIVE { 0.0 } else { x };
    }
    if x >= 0.5 {
        return ratio_direct(a, 0);
    }
    let xe = ext(x);
    let t = rx::ext_mul(&xe, &xe, CW);
    let numerator_coefficients = [
        -0.000_104_681_588_927_531_37,
        0.028_728_638_600_548_515,
        -0.281_802_109_617_808_14,
        0.884_681_425_365_016_5,
        -1.102_835_679_784_634_2,
        0.474_825_735_897_473_56,
    ];
    let denominator_coefficients = [
        -0.035_861_554_370_169_54,
        0.495_611_965_555_031,
        -2.260_888_374_898_849,
        4.541_470_062_608_451,
        -4.163_193_363_969_355,
        1.424_477_207_692_420_6,
    ];
    let mut numerator = ext(numerator_coefficients[0]);
    let mut denominator = ext(denominator_coefficients[0]);
    for coefficient in &numerator_coefficients[1..] {
        numerator = rx::ext_add(&rx::ext_mul(&numerator, &t, CW), &ext(*coefficient), CW);
    }
    for coefficient in &denominator_coefficients[1..] {
        denominator = rx::ext_add(&rx::ext_mul(&denominator, &t, CW), &ext(*coefficient), CW);
    }
    let polynomial = rx::ext_div(&numerator, &denominator, CW);
    let correction = rx::ext_mul(&rx::ext_mul(&xe, &t, CW), &polynomial, CW);
    to_f64(&rx::ext_add(&xe, &correction, CW))
}

const AOCL_ATANH_A: [f64; 12] = [
    f64::from_bits(0x3fde_638b_7bbe_a45e),
    f64::from_bits(0xbff1_a537_0698_9746),
    f64::from_bits(0x3fec_4f4f_6baa_48ff),
    f64::from_bits(0xbfd2_090b_b730_2592),
    f64::from_bits(0x3f9d_6b0a_4cfd_e8fc),
    f64::from_bits(0xbf1b_7110_00f5_a53b),
    f64::from_bits(0x3ff6_caa8_9cce_fb46),
    f64::from_bits(0xc010_a71c_2944_b0bf),
    f64::from_bits(0x4012_2a77_20ca_aa5d),
    f64::from_bits(0xc002_164c_a4f0_c6f3),
    f64::from_bits(0x3fdf_b81b_3fe4_2b33),
    f64::from_bits(0xbfa2_5c72_1668_3eca),
];

fn aocl_even_10(x: f64, coefficients: &[f64]) -> f64 {
    let r2 = x * x;
    let r4 = r2 * r2;
    let r6 = r4 * r2;
    let a0 = coefficients[2] * r2 + coefficients[1];
    let a1 = a0 * r2 + coefficients[0];
    let a2 = (coefficients[3] * r2 + coefficients[4] * r4 + coefficients[5] * r6) * r4;
    a1 + a2
}

fn amd_atanh_recip_estrin(a: f64, x87_recip: bool) -> f64 {
    let x = reciprocal(a, x87_recip);
    if x < f64::from_bits(0x3e30_0000_0000_0000) {
        return if x < f64::MIN_POSITIVE { 0.0 } else { x };
    }
    if x >= 0.5 {
        return ratio_direct(a, 0);
    }
    let cube = x * x * x;
    let numerator = aocl_even_10(x, &AOCL_ATANH_A[..6]);
    let denominator = aocl_even_10(x, &AOCL_ATANH_A[6..]);
    x + cube * (numerator / denominator)
}

fn atanh_taylor_forward(a: f64, x87_recip: bool, terms: usize) -> f64 {
    let x = reciprocal(a, x87_recip);
    let z = x * x;
    let mut power = x;
    let mut sum = x;
    for k in 1..terms {
        power *= z;
        sum += power / ((2 * k + 1) as f64);
    }
    sum
}

fn atanh_taylor_horner(a: f64, x87_recip: bool, terms: usize) -> f64 {
    let x = reciprocal(a, x87_recip);
    let z = x * x;
    let mut polynomial = 1.0 / ((2 * terms - 1) as f64);
    for k in (0..terms - 1).rev() {
        polynomial = 1.0 / ((2 * k + 1) as f64) + z * polynomial;
    }
    x * polynomial
}

fn atanh_taylor_forward_ext(a: f64, register_recip: bool, terms: usize) -> f64 {
    let x = if register_recip {
        rx::ext_div(&rx::ext_one(), &ext(a), CW)
    } else {
        ext(x87_div(1.0, a))
    };
    let z = rx::ext_mul(&x, &x, CW);
    let mut power = x.clone();
    let mut sum = x;
    for k in 1..terms {
        power = rx::ext_mul(&power, &z, CW);
        let term = rx::ext_div(&power, &ext((2 * k + 1) as f64), CW);
        sum = rx::ext_add(&sum, &term, CW);
    }
    to_f64(&sum)
}

fn piecewise(a: f64, switch: f64, ratio_mask: u8, pair: fn(f64) -> f64) -> f64 {
    if a < switch {
        ratio_direct(a, ratio_mask)
    } else {
        pair(a)
    }
}

fn load(path: &Path) -> Vec<(String, f64, u64)> {
    let document: WitnessSet =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read answers"))
            .expect("parse answers");
    document
        .witnesses
        .iter()
        .filter_map(|witness| {
            let input = match &witness.args[0] {
                WitnessArg::Scalar(text) => parse_bits_hex(text)?,
                _ => return None,
            };
            let expected = parse_bits_hex(&witness.expected_bits)?;
            Some((
                witness.id.clone().unwrap_or_else(|| "<missing-id>".into()),
                input,
                expected.to_bits(),
            ))
        })
        .collect()
}

fn score(rows: &[(String, f64, u64)], candidate: impl Fn(f64) -> f64) -> usize {
    rows.iter()
        .filter(|(_, input, expected)| {
            let magnitude = input.abs();
            let value = candidate(magnitude);
            let published = if value == 0.0 {
                0.0
            } else {
                value.copysign(*input)
            };
            published.to_bits() == *expected
        })
        .count()
}

fn main() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/w109");
    let cohorts = [
        ("legacy", base.join("G4-hyp-answers-acoth.json")),
        (
            "dense-discovery",
            base.join("G4-03-acoth/answers-acoth-dense-discovery-20260809.json"),
        ),
        (
            "graph-discovery",
            base.join("G4-03-acoth/answers-acoth-graph-discovery-20260809.json"),
        ),
        (
            "switch-r1",
            base.join("G4-03-acoth/answers-acoth-switch-r1-20260809.json"),
        ),
        (
            "switch-r2",
            base.join("G4-03-acoth/answers-acoth-switch-r2-20260809.json"),
        ),
        (
            "fresh-heldout",
            base.join("G4-03-acoth/answers-acoth-exact-heldout-20260809.json"),
        ),
    ];
    let mut all = BTreeMap::new();
    for (name, path) in cohorts {
        if !path.exists() {
            continue;
        }
        let rows = load(&path);
        println!("{name:18} {} rows", rows.len());
        for row in rows {
            all.insert(row.1.to_bits(), row);
        }
    }
    let rows: Vec<_> = all.into_values().collect();
    println!("{} distinct rows", rows.len());

    let candidates: Vec<(&str, fn(f64) -> f64)> = vec![
        ("fyl2xp1 r0 logs0 diff0", |a| {
            pair_fyl2xp1(a, false, false, false)
        }),
        ("fyl2xp1 r1 logs0 diff0", |a| {
            pair_fyl2xp1(a, true, false, false)
        }),
        ("fyl2xp1 r0 logs1 diff0", |a| {
            pair_fyl2xp1(a, false, true, false)
        }),
        ("fyl2xp1 r1 logs1 diff0", |a| {
            pair_fyl2xp1(a, true, true, false)
        }),
        ("fyl2xp1 r0 logs1 diff1", |a| {
            pair_fyl2xp1(a, false, true, true)
        }),
        ("fyl2xp1 r1 logs1 diff1", |a| {
            pair_fyl2xp1(a, true, true, true)
        }),
        ("fyl2xp1 register rcp pc64", |a| {
            pair_fyl2xp1_register_recip(a, rx::CW_PC64_RN, false)
        }),
        ("fyl2xp1 register rcp pc64 logs1", |a| {
            pair_fyl2xp1_register_recip(a, rx::CW_PC64_RN, true)
        }),
        ("fyl2xp1 register rcp pc53", |a| {
            pair_fyl2xp1_register_recip(a, rx::CW_PC53_RN, false)
        }),
        ("fyl2xp1 register rcp pc53 logs1", |a| {
            pair_fyl2xp1_register_recip(a, rx::CW_PC53_RN, true)
        }),
        ("fyl2xp1 stored r0 pc53", |a| {
            pair_fyl2xp1_stored_recip_cw(a, false, rx::CW_PC53_RN)
        }),
        ("fyl2xp1 stored r1 pc53", |a| {
            pair_fyl2xp1_stored_recip_cw(a, true, rx::CW_PC53_RN)
        }),
        ("lnpair r0 sums0 logs0 d0", |a| {
            pair_ln(a, false, false, false, false)
        }),
        ("lnpair r1 sums0 logs0 d0", |a| {
            pair_ln(a, true, false, false, false)
        }),
        ("lnpair r0 sums1 logs0 d0", |a| {
            pair_ln(a, false, true, false, false)
        }),
        ("lnpair r1 sums1 logs0 d0", |a| {
            pair_ln(a, true, true, false, false)
        }),
        ("lnpair r0 sums0 logs1 d0", |a| {
            pair_ln(a, false, false, true, false)
        }),
        ("lnpair r1 sums0 logs1 d0", |a| {
            pair_ln(a, true, false, true, false)
        }),
        ("lnpair r0 sums1 logs1 d0", |a| {
            pair_ln(a, false, true, true, false)
        }),
        ("lnpair r1 sums1 logs1 d0", |a| {
            pair_ln(a, true, true, true, false)
        }),
        ("lnpair r0 sums0 logs1 d1", |a| {
            pair_ln(a, false, false, true, true)
        }),
        ("lnpair r1 sums1 logs1 d1", |a| {
            pair_ln(a, true, true, true, true)
        }),
        ("orig-ln sums0 logs0 d0", |a| {
            pair_ln_original(a, false, false, false)
        }),
        ("orig-ln sums1 logs0 d0", |a| {
            pair_ln_original(a, true, false, false)
        }),
        ("orig-ln sums0 logs1 d0", |a| {
            pair_ln_original(a, false, true, false)
        }),
        ("orig-ln sums1 logs1 d0", |a| {
            pair_ln_original(a, true, true, false)
        }),
        ("orig-ln sums0 logs1 d1", |a| {
            pair_ln_original(a, false, true, true)
        }),
        ("orig-ln sums1 logs1 d1", |a| {
            pair_ln_original(a, true, true, true)
        }),
        ("crt r0 sums0 diff0", |a| {
            pair_crt_stored(a, false, false, false)
        }),
        ("crt r1 sums0 diff0", |a| {
            pair_crt_stored(a, true, false, false)
        }),
        ("crt r0 sums1 diff1", |a| {
            pair_crt_stored(a, false, true, true)
        }),
        ("crt r1 sums1 diff1", |a| {
            pair_crt_stored(a, true, true, true)
        }),
        ("std-l1p pair r0 diff0", |a| {
            pair_stored_l1p(a, false, false, f64::ln_1p)
        }),
        ("std-l1p pair r1 diff0", |a| {
            pair_stored_l1p(a, true, false, f64::ln_1p)
        }),
        ("std-l1p pair r0 diff1", |a| {
            pair_stored_l1p(a, false, true, f64::ln_1p)
        }),
        ("std-l1p pair r1 diff1", |a| {
            pair_stored_l1p(a, true, true, f64::ln_1p)
        }),
        ("portable-l1p pair r0 d0", |a| {
            pair_stored_l1p(a, false, false, rx::excel_log1p)
        }),
        ("portable-l1p pair r1 d0", |a| {
            pair_stored_l1p(a, true, false, rx::excel_log1p)
        }),
        ("kahan-l1p pair r0 d0", |a| {
            pair_stored_l1p(a, false, false, l1p_kahan)
        }),
        ("kahan-l1p pair r1 d0", |a| {
            pair_stored_l1p(a, true, false, l1p_kahan)
        }),
        ("kahan2-l1p pair r0 d0", |a| {
            pair_stored_l1p(a, false, false, l1p_kahan2)
        }),
        ("kahan2-l1p pair r1 d0", |a| {
            pair_stored_l1p(a, true, false, l1p_kahan2)
        }),
        ("platform atanh(1/a)", |a| (1.0 / a).atanh()),
        ("excel-atanh recip r0", |a| {
            oxfunc_core::functions::atanh::atanh_kernel(1.0 / a).unwrap()
        }),
        ("excel-atanh recip r1", |a| {
            oxfunc_core::functions::atanh::atanh_kernel(x87_div(1.0, a)).unwrap()
        }),
        ("collapsed crt r0 ext-sum", |a| {
            collapsed_atanh_crt(a, false, false)
        }),
        ("collapsed crt r1 ext-sum", |a| {
            collapsed_atanh_crt(a, true, false)
        }),
        ("collapsed crt r0 stored-sum", |a| {
            collapsed_atanh_crt(a, false, true)
        }),
        ("fd crt r0 mask0000 ext-sum", |a| {
            fd_atanh_crt(a, false, 0, false, false)
        }),
        ("fd crt r1 mask0000 ext-sum", |a| {
            fd_atanh_crt(a, true, 0, false, false)
        }),
        ("fd crt r0 mask1111 ext-sum", |a| {
            fd_atanh_crt(a, false, 15, false, false)
        }),
        ("fd crt r1 mask1111 ext-sum", |a| {
            fd_atanh_crt(a, true, 15, false, false)
        }),
        ("fd crt r0 mask0000 stored-sum", |a| {
            fd_atanh_crt(a, false, 0, true, false)
        }),
        ("fd crt r0 mask0000 stored-log", |a| {
            fd_atanh_crt(a, false, 0, false, true)
        }),
        ("cephes atanh recip r0", |a| cephes_atanh_recip(a, false)),
        ("cephes atanh recip r1", |a| cephes_atanh_recip(a, true)),
        ("cephes atanh ext r0", |a| cephes_atanh_recip_ext(a, false)),
        ("cephes atanh ext r1", |a| cephes_atanh_recip_ext(a, true)),
        ("amd acml atanh recip r0", |a| amd_atanh_recip(a, false)),
        ("amd acml atanh recip r1", |a| amd_atanh_recip(a, true)),
        ("amd acml atanh ext r0", |a| amd_atanh_recip_ext(a, false)),
        ("amd acml atanh ext r1", |a| amd_atanh_recip_ext(a, true)),
        ("amd aocl estrin recip r0", |a| {
            amd_atanh_recip_estrin(a, false)
        }),
        ("amd aocl estrin recip r1", |a| {
            amd_atanh_recip_estrin(a, true)
        }),
    ];

    println!("\npair-only scores:");
    for &(name, candidate) in &candidates {
        println!("  {name:30} {}/{}", score(&rows, candidate), rows.len());
    }

    println!("\none-sided log1p scores:");
    let mut one_sided_scores = Vec::new();
    for upper in [true, false] {
        for log_kind in 0_u8..3 {
            for op_mask in 0_u8..8 {
                let count = score(&rows, |a| acoth_one_sided(a, upper, op_mask, log_kind));
                one_sided_scores.push((count, upper, log_kind, op_mask));
            }
        }
    }
    one_sided_scores.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    for &(count, upper, log_kind, op_mask) in one_sided_scores.iter().take(16) {
        println!(
            "  {count}/{} side={} log={log_kind} mask={op_mask:03b}",
            rows.len(),
            if upper { "a-1" } else { "a+1" }
        );
    }

    println!("\nTaylor-series piecewise scores:");
    let mut series_scores = Vec::new();
    for switch in [CRT_ACOTH_SWITCH, 3.5, 3.75, 4.0] {
        for x87_recip in [false, true] {
            for terms in 2_usize..=32 {
                let forward = score(&rows, |a| {
                    if a < switch {
                        ratio_direct(a, 4)
                    } else {
                        atanh_taylor_forward(a, x87_recip, terms)
                    }
                });
                series_scores.push((forward, switch, x87_recip, terms, "forward"));
                let horner = score(&rows, |a| {
                    if a < switch {
                        ratio_direct(a, 4)
                    } else {
                        atanh_taylor_horner(a, x87_recip, terms)
                    }
                });
                series_scores.push((horner, switch, x87_recip, terms, "horner"));
            }
        }
    }
    series_scores.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    for &(count, switch, x87_recip, terms, kind) in series_scores.iter().take(16) {
        println!(
            "  {count}/{} T={switch:.17e} {kind} terms={terms} recip={}",
            rows.len(),
            if x87_recip { "r1" } else { "r0" }
        );
    }

    println!("\nlegacy residual candidate deltas:");
    for &(input_bits, label) in &[
        (0x4014_0000_0000_0000_u64, "5"),
        (0x4020_3333_35a5_6e96_u64, "8.1-probe"),
    ] {
        let input = f64::from_bits(input_bits);
        let expected = rows
            .iter()
            .find(|(_, x, _)| x.abs().to_bits() == input_bits)
            .map(|(_, _, expected)| *expected)
            .expect("legacy residual");
        println!("  {label} expected=0x{expected:016x}");
        for &(name, candidate) in &candidates {
            let got = candidate(input).to_bits();
            let delta = got as i64 - expected as i64;
            if delta.abs() <= 3 {
                println!("    {name:30} 0x{got:016x} ({delta:+} ulp)");
            }
        }
        for upper in [true, false] {
            for log_kind in 0_u8..3 {
                for op_mask in 0_u8..8 {
                    let got = acoth_one_sided(input, upper, op_mask, log_kind).to_bits();
                    let delta = got as i64 - expected as i64;
                    if delta == 0 {
                        println!(
                            "    one-sided side={} log={log_kind} mask={op_mask:03b} EXACT",
                            if upper { "a-1" } else { "a+1" }
                        );
                    }
                }
            }
        }
    }

    println!("\npiecewise at 2+sqrt(2)={CRT_ACOTH_SWITCH:.17}:");
    let mut best = (0_usize, 0_u8, "");
    for ratio_mask in 0_u8..8 {
        for &(name, candidate) in &candidates {
            let count = score(&rows, |a| {
                piecewise(a, CRT_ACOTH_SWITCH, ratio_mask, candidate)
            });
            if count > best.0 {
                best = (count, ratio_mask, name);
            }
        }
    }
    println!(
        "  best {}/{} ratio-mask={:03b} pair={}",
        best.0,
        rows.len(),
        best.1,
        best.2
    );

    println!("\nbest observed split ratio(<T) | candidate(>=T):");
    let mut positive: Vec<_> = rows
        .iter()
        .filter(|(_, input, _)| input.is_sign_positive())
        .collect();
    positive.sort_by(|left, right| left.1.partial_cmp(&right.1).unwrap());
    let mut ranked = Vec::new();
    for &(name, candidate) in &candidates {
        for ratio_mask in 0_u8..8 {
            let mut suffix = positive
                .iter()
                .filter(|(_, input, expected)| candidate(*input).to_bits() == *expected)
                .count();
            let mut best_local = (suffix, positive[0].1, ratio_mask, name);
            for (index, (_, input, expected)) in positive.iter().enumerate() {
                if candidate(*input).to_bits() == *expected {
                    suffix -= 1;
                }
                if ratio_direct(*input, ratio_mask).to_bits() == *expected {
                    suffix += 1;
                }
                if suffix > best_local.0 {
                    let threshold = positive
                        .get(index + 1)
                        .map_or(f64::INFINITY, |(_, next, _)| *next);
                    best_local = (suffix, threshold, ratio_mask, name);
                }
            }
            ranked.push(best_local);
        }
    }
    ranked.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    for &(count, threshold, ratio_mask, name) in ranked.iter().take(12) {
        println!(
            "  {count}/{} T={threshold:.17e} ratio={ratio_mask:03b} {name}",
            positive.len()
        );
    }

    println!("\nselected positive-region scores:");
    let selected: [(&str, fn(f64) -> f64); 7] = [
        ("ratio-mask100", |a| ratio_direct(a, 4)),
        ("fyl2xp1-r1", |a| pair_fyl2xp1(a, true, false, false)),
        ("platform-atanh", |a| (1.0 / a).atanh()),
        ("cephes-r1", |a| cephes_atanh_recip(a, true)),
        ("amd-r1", |a| amd_atanh_recip(a, true)),
        ("reciprocal-r0", |a| 1.0 / a),
        ("reciprocal-r1", |a| x87_div(1.0, a)),
    ];
    let bands = [
        (1.0, 2.0),
        (2.0, 2.0 + std::f64::consts::SQRT_2),
        (2.0 + std::f64::consts::SQRT_2, 10.0),
        (10.0, 1.0e7),
        (1.0e7, 268_435_456.0),
        (268_435_456.0, 9_007_199_254_740_992.0),
        (9_007_199_254_740_992.0, f64::INFINITY),
    ];
    for (name, candidate) in selected {
        print!("  {name:16}");
        for (lo, hi) in bands {
            let mut count = 0_usize;
            let mut exact = 0_usize;
            for (_, input, expected) in &positive {
                if *input >= lo && *input < hi {
                    count += 1;
                    if candidate(*input).to_bits() == *expected {
                        exact += 1;
                    }
                }
            }
            print!(" {exact}/{count}");
        }
        println!();
    }

    println!("\nprovisional Taylor residuals (positive only):");
    for (id, input, expected) in &positive {
        let got = if *input < 3.75 {
            ratio_direct(*input, 4)
        } else {
            atanh_taylor_forward(*input, true, 13)
        };
        if got.to_bits() != *expected {
            let r0 = atanh_taylor_forward(*input, false, 13).to_bits();
            let horner = atanh_taylor_horner(*input, true, 13).to_bits();
            let ext_stored = atanh_taylor_forward_ext(*input, false, 13).to_bits();
            let ext_register = atanh_taylor_forward_ext(*input, true, 13).to_bits();
            println!(
                "  {id:26} x=0x{:016x} {:.17e} want=0x{:016x} got=0x{:016x} d={:+} r0={r0:016x} horner={horner:016x} extS={ext_stored:016x} extR={ext_register:016x}",
                input.to_bits(),
                input,
                expected,
                got.to_bits(),
                got.to_bits() as i64 - *expected as i64,
            );
        }
    }

    println!("\nratio wrapper masks (ratio globally):");
    for mask in 0_u8..8 {
        println!(
            "  mask {mask:03b} {}/{}",
            score(&rows, |a| ratio_direct(a, mask)),
            rows.len()
        );
    }
}
