//! Offline public-kernel inverse seed for the W109 F.TEST variance lane.
//!
//! This utility does not read any witness bank.  It accepts the already
//! observed F.TEST result bits and the two integer degree-of-freedom values,
//! forms the exact half-probability, and reports the OxFunc public FDIST inverse
//! seed.  It is a discovery aid, not an oracle or an implementation claim.

use oxfunc_core::functions::chi_f_t_family::{f_dist_rt_kernel, f_inv_rt_kernel};

// Public-domain Numerical Recipes-style incomplete-beta continued fraction,
// used only as an independent mathematical inverse seed.  It deliberately
// does not reuse the production BRATIO graph.
fn ln_gamma(z: f64) -> f64 {
    const COEFFICIENTS: [f64; 9] = [
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
    if z < 0.5 {
        return std::f64::consts::PI.ln()
            - (std::f64::consts::PI * z).sin().ln()
            - ln_gamma(1.0 - z);
    }
    let z = z - 1.0;
    let mut x = COEFFICIENTS[0];
    for (index, coefficient) in COEFFICIENTS.iter().enumerate().skip(1) {
        x += coefficient / (z + index as f64);
    }
    let t = z + 7.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (z + 0.5) * t.ln() - t + x.ln()
}

fn beta_fraction(a: f64, b: f64, x: f64) -> f64 {
    const MAX_ITERATIONS: usize = 512;
    const EPSILON: f64 = 2.0e-16;
    const FLOOR: f64 = 1.0e-300;
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < FLOOR {
        d = FLOOR;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..=MAX_ITERATIONS {
        let m = m as f64;
        let m2 = 2.0 * m;
        let mut aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < FLOOR {
            d = FLOOR;
        }
        c = 1.0 + aa / c;
        if c.abs() < FLOOR {
            c = FLOOR;
        }
        d = 1.0 / d;
        h *= d * c;
        aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < FLOOR {
            d = FLOOR;
        }
        c = 1.0 + aa / c;
        if c.abs() < FLOOR {
            c = FLOOR;
        }
        d = 1.0 / d;
        let delta = d * c;
        h *= delta;
        if (delta - 1.0).abs() <= EPSILON {
            break;
        }
    }
    h
}

fn regularized_beta(x: f64, a: f64, b: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let front = (ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b) + a * x.ln() + b * (-x).ln_1p()).exp();
    if x < (a + 1.0) / (a + b + 2.0) {
        front * beta_fraction(a, b, x) / a
    } else {
        1.0 - front * beta_fraction(b, a, 1.0 - x) / b
    }
}

fn mathematical_f_tail(ratio: f64, df_hi: f64, df_lo: f64) -> f64 {
    let denominator = df_lo + df_hi * ratio;
    regularized_beta(df_lo / denominator, df_lo * 0.5, df_hi * 0.5)
}

fn mathematical_inverse(target: f64, df_hi: f64, df_lo: f64) -> f64 {
    let (mut lo, mut hi): (f64, f64) = (1.0, 64.0);
    while lo.to_bits() + 1 < hi.to_bits() {
        let mid_bits = lo.to_bits() + (hi.to_bits() - lo.to_bits()) / 2;
        let mid = f64::from_bits(mid_bits);
        if mathematical_f_tail(mid, df_hi, df_lo) > target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let lo_error = (mathematical_f_tail(lo, df_hi, df_lo) - target).abs();
    let hi_error = (mathematical_f_tail(hi, df_hi, df_lo) - target).abs();
    if lo_error <= hi_error { lo } else { hi }
}

fn parse_bits(text: &str) -> Result<u64, String> {
    u64::from_str_radix(
        text.strip_prefix("0x")
            .ok_or_else(|| format!("expected 0x-prefixed bits, got {text}"))?,
        16,
    )
    .map_err(|error| error.to_string())
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let ftest_bits = parse_bits(&args.next().ok_or_else(|| {
        "usage: invert_ftest_variance_target <ftest-bits> <df-hi> <df-lo>".to_string()
    })?)?;
    let df_hi: f64 = args
        .next()
        .ok_or_else(|| "missing df-hi".to_string())?
        .parse::<f64>()
        .map_err(|error| error.to_string())?;
    let df_lo: f64 = args
        .next()
        .ok_or_else(|| "missing df-lo".to_string())?
        .parse::<f64>()
        .map_err(|error| error.to_string())?;
    let evaluation_ratio = args.next().map(|value| parse_bits(&value)).transpose()?;
    if args.next().is_some() {
        return Err("too many arguments".to_string());
    }
    let target_tail = f64::from_bits(ftest_bits) * 0.5;
    let ratio = f_inv_rt_kernel(target_tail, df_hi, df_lo)
        .map_err(|error| format!("FDIST inverse domain error: {error:?}"))?;
    let mathematical_ratio = mathematical_inverse(target_tail, df_hi, df_lo);
    println!(
        "target_tail=0x{:016x} inverse_ratio=0x{:016x} mathematical_ratio=0x{:016x}",
        target_tail.to_bits(),
        ratio.to_bits(),
        mathematical_ratio.to_bits()
    );
    if let Some(ratio_bits) = evaluation_ratio {
        let evaluation_ratio = f64::from_bits(ratio_bits);
        let public_tail = f_dist_rt_kernel(evaluation_ratio, df_hi, df_lo)
            .map_err(|error| format!("FDIST domain error: {error:?}"))?;
        println!(
            "evaluation_ratio=0x{ratio_bits:016x} public_tail=0x{:016x} mathematical_tail=0x{:016x}",
            public_tail.to_bits(),
            mathematical_f_tail(evaluation_ratio, df_hi, df_lo).to_bits()
        );
    }
    Ok(())
}
