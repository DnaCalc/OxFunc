//! W109 observability byproduct: does OxFunc's EFFECT / RRI match the fresh live-Excel grids?
use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::financial_time_value_family as fin;
use oxfunc_core::functions::power_fn::power_kernel;
use serde_json::Value;

fn bits(s: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap())
}

fn load(batch: &str, ans: &str) -> Vec<(Vec<f64>, u64)> {
    let b: Value = serde_json::from_str(&std::fs::read_to_string(batch).unwrap()).unwrap();
    let a: Value = serde_json::from_str(&std::fs::read_to_string(ans).unwrap()).unwrap();
    assert_eq!(
        b["function"], a["function"],
        "batch/answer function mismatch: {batch} vs {ans}"
    );
    let probes = b["probes"].as_array().unwrap();
    let wits = a["witnesses"].as_array().unwrap();
    assert_eq!(
        probes.len(),
        wits.len(),
        "batch/answer row-count mismatch: {batch} vs {ans}"
    );
    probes
        .iter()
        .zip(wits)
        .map(|(p, w)| {
            let probe = &p["probe"];
            assert_eq!(
                probe["id"], w["id"],
                "batch/answer witness-id mismatch: {batch} vs {ans}"
            );
            assert_eq!(
                probe["args"], w["args"],
                "batch/answer witness-argument mismatch: {batch} vs {ans}"
            );
            let args: Vec<f64> = probe["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| bits(x.as_str().unwrap()))
                .collect();
            let exp = bits(w["expected_bits"].as_str().unwrap()).to_bits();
            (args, exp)
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TypedOutcome {
    Number(u64),
    Error(String),
}

fn load_typed(batch: &str, ans: &str) -> Vec<(Vec<f64>, TypedOutcome)> {
    let b: Value = serde_json::from_str(&std::fs::read_to_string(batch).unwrap()).unwrap();
    let a: Value = serde_json::from_str(&std::fs::read_to_string(ans).unwrap()).unwrap();
    assert_eq!(b["function"], a["function"]);
    let probes = b["probes"].as_array().unwrap();
    let witnesses = a["witnesses"].as_array().unwrap();
    assert_eq!(probes.len(), witnesses.len());
    probes
        .iter()
        .zip(witnesses)
        .map(|(probe_wrapper, witness)| {
            let probe = &probe_wrapper["probe"];
            assert_eq!(probe["id"], witness["id"]);
            assert_eq!(probe["args"], witness["args"]);
            let args = probe["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| bits(value.as_str().unwrap()))
                .collect();
            let expected = witness["expected_bits"].as_str().unwrap();
            let outcome = if let Some(hex) = expected.strip_prefix("0x") {
                TypedOutcome::Number(u64::from_str_radix(hex, 16).unwrap())
            } else if let Some(error) = expected.strip_prefix("error:") {
                TypedOutcome::Error(error.to_owned())
            } else {
                panic!("unsupported typed outcome {expected}");
            };
            (args, outcome)
        })
        .collect()
}

fn binexp_lsb(base: f64, mut n: u64) -> f64 {
    let mut acc = 1.0;
    let mut b = base;
    while n > 0 {
        if n & 1 == 1 {
            acc *= b;
        }
        n >>= 1;
        if n > 0 {
            b *= b;
        }
    }
    acc
}

fn x87_binexp_msb(base: f64, n: u64) -> f64 {
    if n == 0 {
        return 1.0;
    }
    let mut acc = 1.0;
    let top = 63 - n.leading_zeros();
    for bit in (0..=top).rev() {
        acc = rx::x87_mul(acc, acc);
        if (n >> bit) & 1 == 1 {
            acc = rx::x87_mul(acc, base);
        }
    }
    acc
}

fn x87_loop_mul(base: f64, n: u64) -> f64 {
    let mut acc = 1.0;
    for _ in 0..n {
        acc = rx::x87_mul(acc, base);
    }
    acc
}

fn x87_sub(a: f64, b: f64) -> f64 {
    let v = rx::ext_sub(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN);
    rx::ext_to_f64(&v, rx::CW_PC53_RN)
}

fn x87_add(a: f64, b: f64) -> f64 {
    let v = rx::ext_add(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN);
    rx::ext_to_f64(&v, rx::CW_PC53_RN)
}

fn x87_div(a: f64, b: f64) -> f64 {
    let v = rx::ext_div(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN);
    rx::ext_to_f64(&v, rx::CW_PC53_RN)
}

fn x87_binexp_lsb(base: f64, mut n: u64) -> f64 {
    let mut acc = 1.0;
    let mut b = base;
    while n > 0 {
        if n & 1 == 1 {
            acc = rx::x87_mul(acc, b);
        }
        n >>= 1;
        if n > 0 {
            b = rx::x87_mul(b, b);
        }
    }
    acc
}

fn pow2_from_ext(z: &rx::Ext80) -> f64 {
    let k = rx::ext_rndint(z, rx::CW_PC64_RN);
    let f = rx::ext_sub(z, &k, rx::CW_PC64_RN);
    let negative = rx::ext_to_f64(&f, rx::CW_PC64_RN) < 0.0;
    let w = rx::ext_f2xm1(&rx::ext_abs(&f, rx::CW_PC64_RN), rx::CW_PC64_RN);
    let mut mantissa = rx::ext_add(&w, &rx::ext_one(), rx::CW_PC64_RN);
    if negative {
        mantissa = rx::ext_div(&rx::ext_one(), &mantissa, rx::CW_PC64_RN);
    }
    rx::ext_to_f64(
        &rx::ext_scale(&mantissa, &k, rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}

fn direct_x87_pow_full(base: f64, exponent: f64) -> f64 {
    let z = rx::ext_fyl2x(
        &rx::ext_from_f64(exponent),
        &rx::ext_from_f64(base),
        rx::CW_PC64_RN,
    );
    pow2_from_ext(&z)
}

fn direct_x87_pow_spilled_argument(base: f64, exponent: f64) -> f64 {
    let z = rx::ext_fyl2x(
        &rx::ext_from_f64(exponent),
        &rx::ext_from_f64(base),
        rx::CW_PC64_RN,
    );
    let z = rx::ext_to_f64(&z, rx::CW_PC53_RN);
    pow2_from_ext(&rx::ext_from_f64(z))
}

fn direct_x87_pow_spilled_log2(base: f64, exponent: f64) -> f64 {
    let log2 = rx::ext_fyl2x(&rx::ext_one(), &rx::ext_from_f64(base), rx::CW_PC64_RN);
    let log2 = rx::ext_to_f64(&log2, rx::CW_PC53_RN);
    let z = rx::x87_mul(exponent, log2);
    pow2_from_ext(&rx::ext_from_f64(z))
}

fn score_effect(name: &str, rows: &[(Vec<f64>, u64)], model: fn(f64, f64) -> f64) {
    let mut exact = 0usize;
    let mut misses = Vec::new();
    for (args, expected) in rows {
        let got = model(args[0], args[1]);
        if got.to_bits() == *expected {
            exact += 1;
        } else {
            misses.push((args[0], args[1], got.to_bits() as i64 - *expected as i64));
        }
    }
    println!(
        "  {name:24} {exact:3}/{}  miss={} {:?}",
        rows.len(),
        misses.len(),
        &misses[..misses.len().min(3)]
    );
}

fn score_rri(name: &str, rows: &[(Vec<f64>, u64)], model: fn(f64, f64, f64) -> f64) {
    let mut exact = 0usize;
    let mut misses = Vec::new();
    for (args, expected) in rows {
        let got = model(args[0], args[1], args[2]);
        if got.to_bits() == *expected {
            exact += 1;
        } else {
            misses.push((args[0], args[2], got.to_bits() as i64 - *expected as i64));
        }
    }
    println!(
        "  {name:24} {exact:3}/{}  miss={} {:?}",
        rows.len(),
        misses.len(),
        &misses[..misses.len().min(3)]
    );
}

fn score_nominal(name: &str, rows: &[(Vec<f64>, u64)], model: fn(f64, f64) -> f64) {
    score_effect(name, rows, model);
}

fn score_production_effect(label: &str, rows: &[(Vec<f64>, u64)]) {
    let exact = rows
        .iter()
        .filter(|(args, expected)| {
            fin::effect(args[0], args[1]).is_ok_and(|got| got.to_bits() == *expected)
        })
        .count();
    println!("OxFunc EFFECT {label}: {exact}/{} exact", rows.len());
}

fn verify_production_effect_typed(label: &str, rows: &[(Vec<f64>, TypedOutcome)]) {
    let exact = rows
        .iter()
        .filter(|(args, expected)| {
            let actual = match fin::effect(args[0], args[1]) {
                Ok(value) => TypedOutcome::Number(value.to_bits()),
                Err(error) => TypedOutcome::Error(format!("{error:?}")),
            };
            actual == *expected
        })
        .count();
    println!("OxFunc EFFECT {label}: {exact}/{} exact typed", rows.len());
    assert_eq!(exact, rows.len(), "EFFECT {label} replay regression");
}

fn score_production_rri(label: &str, rows: &[(Vec<f64>, u64)]) {
    let exact = rows
        .iter()
        .filter(|(args, expected)| {
            fin::rri(args[0], args[1], args[2]).is_ok_and(|got| got.to_bits() == *expected)
        })
        .count();
    println!("OxFunc RRI {label}: {exact}/{} exact", rows.len());
}

#[derive(Clone, Copy, Debug)]
enum ExpectedRriOutcome {
    Number(u64),
    Num,
}

impl ExpectedRriOutcome {
    fn display(self) -> String {
        match self {
            Self::Number(bits) => format!("0x{bits:016x}"),
            Self::Num => "#NUM!".to_owned(),
        }
    }
}

fn daz(value: f64) -> f64 {
    if value.abs() < f64::MIN_POSITIVE {
        0.0_f64.copysign(value)
    } else {
        value
    }
}

fn rri_edge_composite(
    periods: f64,
    present_value: f64,
    future_value: f64,
) -> Result<f64, fin::FinancialError> {
    if !periods.is_finite() || !present_value.is_finite() || !future_value.is_finite() {
        return Err(fin::FinancialError::Value);
    }
    // Live edge ordering: positive subnormal periods are rejected, while
    // minimum-normal periods are admitted. This guard precedes the pv==fv
    // short-circuit (RRI(max-subnormal, 1, 1) is #NUM!, not zero).
    if periods < f64::MIN_POSITIVE {
        return Err(fin::FinancialError::Num);
    }

    // The financial body observes DAZ-normalized pv/fv values. Equality is
    // checked before the sign guards, so equal zeros, unequal subnormals that
    // both collapse to zero, and even equal negative normal values return +0.
    let pv = daz(present_value);
    let fv = daz(future_value);
    if pv == fv {
        return Ok(0.0);
    }
    if pv <= 0.0 || fv < 0.0 {
        return Err(fin::FinancialError::Num);
    }

    // A subnormal quotient produced from wholly normal operands is also DAZ
    // before the power call. The zero lane is exact -1 and must bypass the raw
    // x87 log/exp helper, whose contract requires a finite positive base.
    let base = daz(x87_div(fv, pv));
    if base == 0.0 {
        return Ok(-1.0);
    }
    if !base.is_finite() {
        return Err(fin::FinancialError::Num);
    }
    let reciprocal = rx::x87_recip(periods);
    if !reciprocal.is_finite() {
        return Err(fin::FinancialError::Num);
    }
    // The public RRI graph has an exact-period identity lane.  It is narrowly
    // keyed by periods == 1: the adjacent binary64 periods on either side use
    // the raw chain, while periods == 1 preserves even MAX and MAX-1 exactly.
    let powered = if periods == 1.0 {
        base
    } else {
        rx::excel_pow_chain(base, reciprocal)
    };
    let result = x87_sub(powered, 1.0);
    if result.is_finite() {
        Ok(result)
    } else {
        Err(fin::FinancialError::Num)
    }
}

type RriEdgeRow = (&'static str, u64, u64, u64, ExpectedRriOutcome);

fn rri_edge_stage1_rows() -> Vec<RriEdgeRow> {
    use ExpectedRriOutcome::{Num, Number};
    const PZ: u64 = 0x0000_0000_0000_0000;
    const NZ: u64 = 0x8000_0000_0000_0000;
    const MIN_SUB: u64 = 0x0000_0000_0000_0001;
    const MAX_SUB: u64 = 0x000f_ffff_ffff_ffff;
    const MIN_NORMAL: u64 = 0x0010_0000_0000_0000;
    const NEG_MIN_SUB: u64 = 0x8000_0000_0000_0001;
    const NEG_MIN_NORMAL: u64 = 0x8010_0000_0000_0000;
    const ONE: u64 = 0x3ff0_0000_0000_0000;
    const TWO: u64 = 0x4000_0000_0000_0000;
    const MAX_FINITE: u64 = 0x7fef_ffff_ffff_ffff;
    const NEG_ONE: u64 = 0xbff0_0000_0000_0000;
    const RAW_SQRT2_MINUS_1: u64 = 0x3fda_8279_99fc_ef30;
    const RAW_SQRT_HALF_MINUS_1: u64 = 0xbfd2_bec3_3301_8866;
    const RAW_SQRT_MAX_MINUS_1: u64 = 0x5fef_ffff_ffff_ff95;

    vec![
        // 44-row quotient/period cross.
        ("zero-tie/min-sub-period", MIN_SUB, TWO, MIN_SUB, Num),
        (
            "zero-tie/recip-below",
            0x0003_ffff_ffff_ffff,
            TWO,
            MIN_SUB,
            Num,
        ),
        (
            "zero-tie/recip-at",
            0x0004_0000_0000_0000,
            TWO,
            MIN_SUB,
            Num,
        ),
        (
            "zero-tie/recip-above",
            0x0004_0000_0000_0001,
            TWO,
            MIN_SUB,
            Num,
        ),
        (
            "zero-tie/min-normal-period",
            MIN_NORMAL,
            TWO,
            MIN_SUB,
            Number(NEG_ONE),
        ),
        ("zero-tie/n1", ONE, TWO, MIN_SUB, Number(NEG_ONE)),
        ("zero-tie/n2", TWO, TWO, MIN_SUB, Number(NEG_ONE)),
        (
            "zero-tie/max-period",
            MAX_FINITE,
            TWO,
            MIN_SUB,
            Number(NEG_ONE),
        ),
        ("min-sub-base/min-sub-period", MIN_SUB, ONE, MIN_SUB, Num),
        (
            "min-sub-base/recip-above",
            0x0004_0000_0000_0001,
            ONE,
            MIN_SUB,
            Num,
        ),
        ("min-sub-base/n1", ONE, ONE, MIN_SUB, Number(NEG_ONE)),
        ("min-sub-base/n2", TWO, ONE, MIN_SUB, Number(NEG_ONE)),
        (
            "min-sub-base/max-period",
            MAX_FINITE,
            ONE,
            MIN_SUB,
            Number(NEG_ONE),
        ),
        ("max-sub-base/n1", ONE, ONE, MAX_SUB, Number(NEG_ONE)),
        ("max-sub-base/n2", TWO, ONE, MAX_SUB, Number(NEG_ONE)),
        ("min-normal-base/n1", ONE, ONE, MIN_NORMAL, Number(NEG_ONE)),
        ("min-normal-base/n2", TWO, ONE, MIN_NORMAL, Number(NEG_ONE)),
        (
            "min-normal-base/max-period",
            MAX_FINITE,
            ONE,
            MIN_NORMAL,
            Number(PZ),
        ),
        ("half/min-sub-period", MIN_SUB, TWO, ONE, Num),
        ("half/recip-below", 0x0003_ffff_ffff_ffff, TWO, ONE, Num),
        ("half/recip-at", 0x0004_0000_0000_0000, TWO, ONE, Num),
        ("half/recip-above", 0x0004_0000_0000_0001, TWO, ONE, Num),
        (
            "half/min-normal-period",
            MIN_NORMAL,
            TWO,
            ONE,
            Number(NEG_ONE),
        ),
        ("half/n1", ONE, TWO, ONE, Number(0xbfe0_0000_0000_0000)),
        ("half/n2", TWO, TWO, ONE, Number(RAW_SQRT_HALF_MINUS_1)),
        ("half/max-period", MAX_FINITE, TWO, ONE, Number(PZ)),
        ("one/min-sub-period", MIN_SUB, ONE, ONE, Num),
        ("one/recip-at", 0x0004_0000_0000_0000, ONE, ONE, Num),
        ("one/min-normal-period", MIN_NORMAL, ONE, ONE, Number(PZ)),
        ("one/n1", ONE, ONE, ONE, Number(PZ)),
        ("one/max-period", MAX_FINITE, ONE, ONE, Number(PZ)),
        ("two/min-sub-period", MIN_SUB, ONE, TWO, Num),
        ("two/recip-at", 0x0004_0000_0000_0000, ONE, TWO, Num),
        ("two/min-normal-period", MIN_NORMAL, ONE, TWO, Num),
        ("two/n1", ONE, ONE, TWO, Number(ONE)),
        ("two/n2", TWO, ONE, TWO, Number(RAW_SQRT2_MINUS_1)),
        ("two/max-period", MAX_FINITE, ONE, TWO, Number(PZ)),
        ("max-base/n1", ONE, ONE, MAX_FINITE, Number(MAX_FINITE)),
        (
            "max-base/n2",
            TWO,
            ONE,
            MAX_FINITE,
            Number(RAW_SQRT_MAX_MINUS_1),
        ),
        (
            "max-base/max-period",
            MAX_FINITE,
            ONE,
            MAX_FINITE,
            Number(PZ),
        ),
        ("infinite-quotient/n1", ONE, MIN_SUB, MAX_FINITE, Num),
        ("infinite-quotient/n2", TWO, MIN_SUB, MAX_FINITE, Num),
        (
            "infinite-quotient/max-period",
            MAX_FINITE,
            MIN_SUB,
            MAX_FINITE,
            Num,
        ),
        (
            "deep-zero-quotient/n1",
            ONE,
            MAX_FINITE,
            MIN_NORMAL,
            Number(NEG_ONE),
        ),
        // Six-row zero/subnormal threshold follow-up.
        ("explicit-zero-future/n1", ONE, TWO, PZ, Number(NEG_ONE)),
        ("min-sub-over-two/n1", ONE, TWO, MIN_SUB, Number(NEG_ONE)),
        (
            "max-sub-base/max-period-followup",
            MAX_FINITE,
            ONE,
            MAX_SUB,
            Number(NEG_ONE),
        ),
        (
            "min-normal-base/max-period-followup",
            MAX_FINITE,
            ONE,
            MIN_NORMAL,
            Number(PZ),
        ),
        ("max-sub-period/equal-one", MAX_SUB, ONE, ONE, Num),
        (
            "min-normal-period/equal-one",
            MIN_NORMAL,
            ONE,
            ONE,
            Number(PZ),
        ),
        // Ten-row sign/guard-order follow-up.
        ("negative-zero-future", ONE, TWO, NZ, Number(NEG_ONE)),
        (
            "negative-min-sub-future",
            ONE,
            TWO,
            NEG_MIN_SUB,
            Number(NEG_ONE),
        ),
        ("negative-min-normal-future", ONE, TWO, NEG_MIN_NORMAL, Num),
        ("positive-zero-present", ONE, PZ, ONE, Num),
        ("negative-zero-present", ONE, NZ, ONE, Num),
        ("negative-min-sub-present", ONE, NEG_MIN_SUB, ONE, Num),
        ("positive-zero-period/future-zero", PZ, TWO, PZ, Num),
        ("negative-zero-period/future-zero", NZ, TWO, PZ, Num),
        (
            "negative-min-sub-period/future-zero",
            NEG_MIN_SUB,
            TWO,
            PZ,
            Num,
        ),
        ("min-sub-present/future-zero", ONE, MIN_SUB, PZ, Number(PZ)),
    ]
}

fn rri_edge_stage2_rows() -> Vec<RriEdgeRow> {
    use ExpectedRriOutcome::{Num, Number};
    const PZ: u64 = 0x0000_0000_0000_0000;
    const NZ: u64 = 0x8000_0000_0000_0000;
    const MIN_SUB: u64 = 0x0000_0000_0000_0001;
    const MIN_SUB_2: u64 = 0x0000_0000_0000_0002;
    const MAX_SUB: u64 = 0x000f_ffff_ffff_ffff;
    const NEG_MAX_SUB: u64 = 0x800f_ffff_ffff_ffff;
    const MIN_NORMAL: u64 = 0x0010_0000_0000_0000;
    const MIN_NORMAL_NEXT: u64 = 0x0010_0000_0000_0001;
    const NEG_MIN_NORMAL: u64 = 0x8010_0000_0000_0000;
    const ONE: u64 = 0x3ff0_0000_0000_0000;
    const TWO: u64 = 0x4000_0000_0000_0000;
    const NEG_ONE: u64 = 0xbff0_0000_0000_0000;
    const MAX_FINITE: u64 = 0x7fef_ffff_ffff_ffff;

    vec![
        // Equality after input DAZ, before sign guards.
        ("equal +0/+0", ONE, PZ, PZ, Number(PZ)),
        ("equal -0/-0", ONE, NZ, NZ, Number(PZ)),
        ("equal +0/-0", ONE, PZ, NZ, Number(PZ)),
        ("equal -0/+0", ONE, NZ, PZ, Number(PZ)),
        ("equal min-sub", ONE, MIN_SUB, MIN_SUB, Number(PZ)),
        ("equal max-sub", ONE, MAX_SUB, MAX_SUB, Number(PZ)),
        ("DAZ min-sub/max-sub", ONE, MIN_SUB, MAX_SUB, Number(PZ)),
        (
            "DAZ max-sub/negative-max-sub",
            ONE,
            MAX_SUB,
            NEG_MAX_SUB,
            Number(PZ),
        ),
        ("equal min-normal", ONE, MIN_NORMAL, MIN_NORMAL, Number(PZ)),
        (
            "equal negative-min-normal",
            ONE,
            NEG_MIN_NORMAL,
            NEG_MIN_NORMAL,
            Number(PZ),
        ),
        ("equal negative-one", ONE, NEG_ONE, NEG_ONE, Number(PZ)),
        // Input DAZ and unequal sign guards.
        (
            "negative max-sub future",
            ONE,
            ONE,
            NEG_MAX_SUB,
            Number(NEG_ONE),
        ),
        ("negative min-normal future", ONE, ONE, NEG_MIN_NORMAL, Num),
        (
            "positive max-sub future/max-period",
            MAX_FINITE,
            ONE,
            MAX_SUB,
            Number(NEG_ONE),
        ),
        (
            "min-normal future/max-period",
            MAX_FINITE,
            ONE,
            MIN_NORMAL,
            Number(PZ),
        ),
        (
            "positive max-sub present/future-zero",
            ONE,
            MAX_SUB,
            PZ,
            Number(PZ),
        ),
        (
            "negative max-sub present/future-zero",
            ONE,
            NEG_MAX_SUB,
            PZ,
            Number(PZ),
        ),
        (
            "positive max-sub present/min-normal-future",
            ONE,
            MAX_SUB,
            MIN_NORMAL,
            Num,
        ),
        (
            "negative max-sub present/future-one",
            ONE,
            NEG_MAX_SUB,
            ONE,
            Num,
        ),
        // Quotient DAZ from wholly normal operands.
        (
            "ratio tie-to-zero",
            MAX_FINITE,
            0x4340_0000_0000_0000,
            MIN_NORMAL,
            Number(NEG_ONE),
        ),
        (
            "ratio min-sub",
            MAX_FINITE,
            0x4330_0000_0000_0000,
            MIN_NORMAL,
            Number(NEG_ONE),
        ),
        (
            "ratio max-sub",
            MAX_FINITE,
            TWO,
            0x001f_ffff_ffff_fffe,
            Number(NEG_ONE),
        ),
        (
            "ratio min-normal",
            MAX_FINITE,
            TWO,
            0x0020_0000_0000_0000,
            Number(PZ),
        ),
        (
            "ratio min-normal-next",
            MAX_FINITE,
            TWO,
            0x0020_0000_0000_0001,
            Number(PZ),
        ),
        // Exact period cutoff and ordering before equality/zero-future routes.
        ("max-sub period/equal-one", MAX_SUB, ONE, ONE, Num),
        (
            "min-normal period/equal-one",
            MIN_NORMAL,
            ONE,
            ONE,
            Number(PZ),
        ),
        (
            "min-normal-next period/equal-one",
            MIN_NORMAL_NEXT,
            ONE,
            ONE,
            Number(PZ),
        ),
        ("max-sub period/future-zero", MAX_SUB, ONE, PZ, Num),
        (
            "min-normal period/future-zero",
            MIN_NORMAL,
            ONE,
            PZ,
            Number(NEG_ONE),
        ),
        (
            "negative max-sub period/equal-one",
            NEG_MAX_SUB,
            ONE,
            ONE,
            Num,
        ),
        (
            "negative min-normal period/equal-one",
            NEG_MIN_NORMAL,
            ONE,
            ONE,
            Num,
        ),
        // Adjacent subnormal and minimum-normal equality controls.
        (
            "min-sub future/min-sub-2 present",
            ONE,
            MIN_SUB_2,
            MIN_SUB,
            Number(PZ),
        ),
        (
            "min-sub-2 future/min-sub present",
            ONE,
            MIN_SUB,
            MIN_SUB_2,
            Number(PZ),
        ),
        (
            "min-normal future/min-normal-next present",
            MAX_FINITE,
            MIN_NORMAL_NEXT,
            MIN_NORMAL,
            Number(PZ),
        ),
        (
            "min-normal-next future/min-normal present",
            MAX_FINITE,
            MIN_NORMAL,
            MIN_NORMAL_NEXT,
            Number(PZ),
        ),
    ]
}

fn rri_period_one_discriminator_rows() -> Vec<RriEdgeRow> {
    use ExpectedRriOutcome::Number;
    const ONE: u64 = 0x3ff0_0000_0000_0000;
    const TWO: u64 = 0x4000_0000_0000_0000;
    const MAX_FINITE: u64 = 0x7fef_ffff_ffff_ffff;

    vec![
        (
            "max/period-next-down",
            0x3fef_ffff_ffff_ffff,
            ONE,
            MAX_FINITE,
            Number(0x7fef_ffff_ffff_ff2a),
        ),
        ("max/period-one", ONE, ONE, MAX_FINITE, Number(MAX_FINITE)),
        (
            "max/period-next-up",
            0x3ff0_0000_0000_0001,
            ONE,
            MAX_FINITE,
            Number(0x7fef_ffff_ffff_fb2a),
        ),
        (
            "max-prev/period-one",
            ONE,
            ONE,
            0x7fef_ffff_ffff_fffe,
            Number(0x7fef_ffff_ffff_fffe),
        ),
        (
            "max-over-two/period-one",
            ONE,
            TWO,
            MAX_FINITE,
            Number(0x7fdf_ffff_ffff_ffff),
        ),
        (
            "one-plus-ulp/period-one",
            ONE,
            ONE,
            0x3ff0_0000_0000_0001,
            Number(0x3cb0_0000_0000_0000),
        ),
    ]
}

fn write_rri_edge_artifact(root: &str, cohort: &str, rows: &[RriEdgeRow], note: &str) {
    let provenance = serde_json::json!({
        "schema_version": "w109-manual-live-capture-provenance-v1",
        "captured_utc": null,
        "capture_date_utc": "2026-08-09",
        "capture_note": note,
        "environment": {
            "excel_version": "16.0",
            "excel_build": "20228",
            "excel_bitness": "64-bit",
            "workbook_compatibility": "2",
            "excel_operating_system": "Windows (64-bit) NT 10.00",
            "excel_input_plumbing": "cell_value2",
            "formula_interface": "Formula2",
            "workbook_visibility": "hidden"
        },
        "oracle_cache": {
            "mode": "no_cache",
            "root": null,
            "hits": 0,
            "misses": 0
        },
        "serialization": {
            "excel_process_count_before": 0,
            "excel_process_count_after": 0
        },
        "runner": {
            "name": "serialized Excel.Application COM RRI edge capture",
            "version": "w109-rri-edge-manual-v1"
        },
        "materialized_from": "calc_graph_racer/race_effect_rri_check.rs live-observation rows"
    });
    let probes: Vec<Value> = rows
        .iter()
        .enumerate()
        .map(|(index, (label, periods, present, future, _))| {
            let id = format!("{cohort}-{index:03}");
            serde_json::json!({
                "probe": {
                    "id": id,
                    "label": label,
                    "args": [
                        format!("0x{periods:016x}"),
                        format!("0x{present:016x}"),
                        format!("0x{future:016x}")
                    ]
                }
            })
        })
        .collect();
    let witnesses: Vec<Value> = rows
        .iter()
        .enumerate()
        .map(|(index, (label, periods, present, future, expected))| {
            let expected_bits = match expected {
                ExpectedRriOutcome::Number(bits) => format!("0x{bits:016x}"),
                ExpectedRriOutcome::Num => "error:Num".to_owned(),
            };
            serde_json::json!({
                "id": format!("{cohort}-{index:03}"),
                "label": label,
                "args": [
                    format!("0x{periods:016x}"),
                    format!("0x{present:016x}"),
                    format!("0x{future:016x}")
                ],
                "expected_bits": expected_bits
            })
        })
        .collect();
    let batch = serde_json::json!({
        "function": "RRI",
        "row_id": cohort,
        "artifact_provenance": provenance.clone(),
        "probes": probes
    });
    let answers = serde_json::json!({
        "function": "RRI",
        "capture_provenance": provenance,
        "witnesses": witnesses
    });
    let batch_path = format!("{root}/batch-{cohort}.json");
    let answers_path = format!("{root}/answers-{cohort}.json");
    std::fs::write(
        &batch_path,
        format!("{}\n", serde_json::to_string_pretty(&batch).unwrap()),
    )
    .unwrap();
    std::fs::write(
        &answers_path,
        format!("{}\n", serde_json::to_string_pretty(&answers).unwrap()),
    )
    .unwrap();

    let replay = load_typed(&batch_path, &answers_path);
    assert_eq!(replay.len(), rows.len());
    for ((args, actual), (_, periods, present, future, expected)) in replay.iter().zip(rows) {
        assert_eq!(
            args.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
            vec![*periods, *present, *future]
        );
        let expected = match expected {
            ExpectedRriOutcome::Number(bits) => TypedOutcome::Number(*bits),
            ExpectedRriOutcome::Num => TypedOutcome::Error("Num".to_owned()),
        };
        assert_eq!(*actual, expected);
    }
    println!("materialized {cohort}: {} rows", rows.len());
}

fn write_rri_edge_artifacts(root: &str) {
    write_rri_edge_artifact(
        root,
        "rri-edge-stage1-20260809",
        &rri_edge_stage1_rows(),
        "Consolidated serialized live Excel observations from the first RRI edge-domain battery; individual row timestamps were not retained.",
    );
    write_rri_edge_artifact(
        root,
        "rri-edge-stage2-20260809",
        &rri_edge_stage2_rows(),
        "Consolidated serialized live Excel observations from the oracle-blind RRI guard/order disagreement battery; individual row timestamps were not retained.",
    );
    write_rri_edge_artifact(
        root,
        "rri-period-one-discriminator-20260809",
        &rri_period_one_discriminator_rows(),
        "Clean serialized period-one discriminator rerun after a collision-tainted attempt was discarded; the accepted run independently confirmed zero Excel processes before and after.",
    );
}

fn score_rri_edge_rows(
    cohort: &str,
    rows: &[RriEdgeRow],
    label: &str,
    model: fn(f64, f64, f64) -> Result<f64, fin::FinancialError>,
) {
    let mut exact = 0usize;
    let mut misses = Vec::new();
    for (name, n_bits, pv_bits, fv_bits, expected) in rows.iter().copied() {
        let got = model(
            f64::from_bits(n_bits),
            f64::from_bits(pv_bits),
            f64::from_bits(fv_bits),
        );
        let matches = match (got, expected) {
            (Ok(value), ExpectedRriOutcome::Number(bits)) => value.to_bits() == bits,
            (Err(fin::FinancialError::Num), ExpectedRriOutcome::Num) => true,
            _ => false,
        };
        if matches {
            exact += 1;
        } else {
            misses.push(format!(
                "{name}: got={got:?}, expected={}",
                expected.display()
            ));
        }
    }
    println!(
        "RRI {cohort} {label}: {exact}/{} exact; miss={} {:?}",
        rows.len(),
        misses.len(),
        misses
    );
}

fn score_production_rri_edge_domain() {
    // Both cohorts were captured through the public worksheet/cell-Value2 path
    // on Excel 16.0 build 20228 x64, workbook CompatibilityVersion 2
    // (2026-08-09). Typed errors came from cell display strings; numeric results
    // were compared by raw binary64 bits. Stage 1 has 60 observations; stage 2
    // is an oracle-blind 35-row disagreement battery generated after stage 1.
    let stage1 = rri_edge_stage1_rows();
    let stage2 = rri_edge_stage2_rows();
    let period_one = rri_period_one_discriminator_rows();
    for (label, model) in [
        (
            "production",
            fin::rri as fn(f64, f64, f64) -> Result<f64, fin::FinancialError>,
        ),
        (
            "DAZ/equality/quotient composite",
            rri_edge_composite as fn(f64, f64, f64) -> Result<f64, fin::FinancialError>,
        ),
    ] {
        score_rri_edge_rows("edge-stage1", &stage1, label, model);
        score_rri_edge_rows("edge-stage2", &stage2, label, model);
        score_rri_edge_rows("period-one discriminator", &period_one, label, model);
    }
}

fn score_production_nominal(label: &str, rows: &[(Vec<f64>, u64)]) {
    let exact = rows
        .iter()
        .filter(|(args, expected)| {
            fin::nominal(args[0], args[1]).is_ok_and(|got| got.to_bits() == *expected)
        })
        .count();
    println!("OxFunc NOMINAL {label}: {exact}/{} exact", rows.len());
}

fn main() {
    let root = "../../work/w109/G6-solvers";
    if std::env::args().any(|arg| arg == "--write-rri-edge-artifacts") {
        write_rri_edge_artifacts(root);
        return;
    }
    // EFFECT(nominal, npery)
    let eff = load(
        &format!("{root}/batch-effect-grid.json"),
        &format!("{root}/answers-effect-grid.json"),
    );
    let mut ok = 0u32;
    let mut miss: Vec<(f64, f64, i64)> = Vec::new();
    for (args, exp) in &eff {
        if let Ok(v) = fin::effect(args[0], args[1]) {
            if v.to_bits() == *exp {
                ok += 1;
            } else {
                miss.push((args[0], args[1], v.to_bits() as i64 - *exp as i64));
            }
        }
    }
    println!(
        "OxFunc EFFECT vs Excel grid: {}/{} exact; {} miss",
        ok,
        eff.len(),
        eff.len() - ok as usize
    );
    for m in miss.iter().take(12) {
        println!("   nominal={:.8} npery={} ulp={:+}", m.0, m.1, m.2);
    }
    println!("EFFECT candidate graph race:");
    score_effect("lsb binexp - 1", &eff, |nom, npy| {
        binexp_lsb(1.0 + nom / npy.trunc(), npy.trunc() as u64) - 1.0
    });
    score_effect("x87-dr msb binexp - 1", &eff, |nom, npy| {
        x87_binexp_msb(1.0 + nom / npy.trunc(), npy.trunc() as u64) - 1.0
    });
    score_effect("x87-dr repeated mul - 1", &eff, |nom, npy| {
        x87_loop_mul(1.0 + nom / npy.trunc(), npy.trunc() as u64) - 1.0
    });
    score_effect("x87-dr binexp - 1", &eff, |nom, npy| {
        x87_binexp_lsb(1.0 + nom / npy.trunc(), npy.trunc() as u64) - 1.0
    });
    score_effect("x87-spill full graph", &eff, |nom, npy| {
        let n = npy.trunc();
        let base = x87_add(1.0, x87_div(nom, n));
        x87_sub(x87_binexp_lsb(base, n as u64), 1.0)
    });
    score_effect("lsb binexp x87-sub", &eff, |nom, npy| {
        x87_sub(binexp_lsb(1.0 + nom / npy.trunc(), npy.trunc() as u64), 1.0)
    });
    score_effect("POWER kernel - 1", &eff, |nom, npy| {
        power_kernel(1.0 + nom / npy.trunc(), npy.trunc()).unwrap() - 1.0
    });
    score_effect("pow-chain - 1", &eff, |nom, npy| {
        rx::excel_pow_positive(1.0 + nom / npy.trunc(), npy.trunc()) - 1.0
    });
    score_effect("native powf - 1", &eff, |nom, npy| {
        (1.0 + nom / npy.trunc()).powf(npy.trunc()) - 1.0
    });

    // RRI(nper, pv, fv)
    let rri = load(
        &format!("{root}/batch-rri-grid.json"),
        &format!("{root}/answers-rri-grid.json"),
    );
    let mut ok2 = 0u32;
    let mut miss2: Vec<(f64, f64, i64)> = Vec::new();
    for (args, exp) in &rri {
        if let Ok(v) = fin::rri(args[0], args[1], args[2]) {
            if v.to_bits() == *exp {
                ok2 += 1;
            } else {
                miss2.push((args[0], args[2], v.to_bits() as i64 - *exp as i64));
            }
        }
    }
    println!(
        "OxFunc RRI vs Excel grid: {}/{} exact; {} miss",
        ok2,
        rri.len(),
        rri.len() - ok2 as usize
    );
    for m in miss2.iter().take(12) {
        println!("   nper={} fv={:.10} ulp={:+}", m.0, m.1, m.2);
    }
    println!("RRI candidate graph race:");
    score_rri("native powf - 1", &rri, |n, pv, fv| {
        (fv / pv).powf(1.0 / n) - 1.0
    });
    score_rri("POWER kernel - 1", &rri, |n, pv, fv| {
        power_kernel(fv / pv, 1.0 / n).unwrap() - 1.0
    });
    score_rri("pow-chain - 1", &rri, |n, pv, fv| {
        rx::excel_pow_chain(fv / pv, 1.0 / n) - 1.0
    });
    score_rri("exp(ln(base)/n)-1", &rri, |n, pv, fv| {
        rx::excel_exp(rx::excel_ln(fv / pv) / n) - 1.0
    });
    score_rri("exp((1/n)*ln)-1", &rri, |n, pv, fv| {
        rx::excel_exp((1.0 / n) * rx::excel_ln(fv / pv)) - 1.0
    });
    score_rri("raw x87-spill wrapper", &rri, |n, pv, fv| {
        let base = x87_div(fv, pv);
        let reciprocal = rx::x87_recip(n);
        x87_sub(rx::excel_pow_chain(base, reciprocal), 1.0)
    });
    score_rri("edge-domain composite", &rri, |n, pv, fv| {
        rri_edge_composite(n, pv, fv).expect("positive-normal RRI bank row")
    });
    score_production_rri_edge_domain();

    let effect_heldout_batch = format!("{root}/batch-effect-heldout-20260809.json");
    let effect_heldout_answers = format!("{root}/answers-effect-heldout-20260809.json");
    if std::path::Path::new(&effect_heldout_answers).exists() {
        let heldout = load(&effect_heldout_batch, &effect_heldout_answers);
        score_production_effect("fresh held-out", &heldout);
        println!("EFFECT fresh held-out schedule race:");
        score_effect("x87-dr lsb binexp - 1", &heldout, |nom, npy| {
            x87_binexp_lsb(1.0 + nom / npy.trunc(), npy.trunc() as u64) - 1.0
        });
        score_effect("x87-dr msb binexp - 1", &heldout, |nom, npy| {
            x87_binexp_msb(1.0 + nom / npy.trunc(), npy.trunc() as u64) - 1.0
        });
        score_effect("x87-dr repeated mul - 1", &heldout, |nom, npy| {
            x87_loop_mul(1.0 + nom / npy.trunc(), npy.trunc() as u64) - 1.0
        });
    }
    let effect_huge_batch = format!("{root}/batch-effect-huge-domain-scratch.json");
    let effect_huge_answers = format!("{root}/answers-effect-huge-domain-scratch.json");
    if std::path::Path::new(&effect_huge_answers).exists() {
        let huge = load_typed(&effect_huge_batch, &effect_huge_answers);
        verify_production_effect_typed("extreme-domain dispatch", &huge);
    }
    let rri_heldout_batch = format!("{root}/batch-rri-heldout-20260809.json");
    let rri_heldout_answers = format!("{root}/answers-rri-heldout-20260809.json");
    if std::path::Path::new(&rri_heldout_answers).exists() {
        let heldout = load(&rri_heldout_batch, &rri_heldout_answers);
        score_production_rri("fresh held-out", &heldout);
        score_rri("edge-domain composite", &heldout, |n, pv, fv| {
            rri_edge_composite(n, pv, fv).expect("positive-normal RRI held-out row")
        });
    }

    for (label, batch, answers) in [(
        "targeted wrapper staging",
        "batch-effect-wrapper-staging-scratch.json",
        "answers-effect-wrapper-staging-scratch.json",
    )] {
        let answer_path = format!("{root}/{answers}");
        if std::path::Path::new(&answer_path).exists() {
            let rows = load(&format!("{root}/{batch}"), &answer_path);
            score_production_effect(label, &rows);
            score_effect("x87-spill full graph", &rows, |nom, npy| {
                let n = npy.trunc();
                let base = x87_add(1.0, x87_div(nom, n));
                x87_sub(x87_binexp_lsb(base, n as u64), 1.0)
            });
        }
    }
    for (label, batch, answers) in [
        (
            "fresh boundary follow-up",
            "batch-rri-followup-20260809.json",
            "answers-rri-followup-20260809.json",
        ),
        (
            "targeted wrapper staging",
            "batch-rri-wrapper-staging-scratch.json",
            "answers-rri-wrapper-staging-scratch.json",
        ),
    ] {
        let answer_path = format!("{root}/{answers}");
        if std::path::Path::new(&answer_path).exists() {
            let rows = load(&format!("{root}/{batch}"), &answer_path);
            score_production_rri(label, &rows);
            score_rri("raw x87-spill wrapper", &rows, |n, pv, fv| {
                let base = x87_div(fv, pv);
                let reciprocal = rx::x87_recip(n);
                x87_sub(rx::excel_pow_chain(base, reciprocal), 1.0)
            });
            score_rri("edge-domain composite", &rows, |n, pv, fv| {
                rri_edge_composite(n, pv, fv).expect("positive-normal RRI follow-up/staging row")
            });
        }
    }

    for (label, batch, answers) in [
        (
            "fresh adjacent-family",
            "batch-nominal-adjacent-20260809.json",
            "answers-nominal-adjacent-20260809.json",
        ),
        (
            "fresh boundary follow-up",
            "batch-nominal-followup-20260809.json",
            "answers-nominal-followup-20260809.json",
        ),
        (
            "same-effect route boundary",
            "batch-nominal-direct-branch-pair-scratch.json",
            "answers-nominal-direct-branch-pair-scratch.json",
        ),
        (
            "targeted wrapper staging",
            "batch-nominal-wrapper-staging-scratch.json",
            "answers-nominal-wrapper-staging-scratch.json",
        ),
    ] {
        let answer_path = format!("{root}/{answers}");
        if !std::path::Path::new(&answer_path).exists() {
            continue;
        }
        let rows = load(&format!("{root}/{batch}"), &answer_path);
        score_production_nominal(label, &rows);
        println!("NOMINAL {label} candidate graph race:");
        score_nominal("raw chain, plain final", &rows, |effect, npy| {
            let n = npy.trunc();
            n * (rx::excel_pow_chain(1.0 + effect, 1.0 / n) - 1.0)
        });
        score_nominal("POWER wrapper, plain", &rows, |effect, npy| {
            let n = npy.trunc();
            n * (power_kernel(1.0 + effect, 1.0 / n).unwrap() - 1.0)
        });
        score_nominal("native powf, plain", &rows, |effect, npy| {
            let n = npy.trunc();
            n * ((1.0 + effect).powf(1.0 / n) - 1.0)
        });
        score_nominal("expm1(log1p/n), plain", &rows, |effect, npy| {
            let n = npy.trunc();
            n * rx::excel_expm1(rx::excel_log1p(effect) / n)
        });
        score_nominal("internal-expm1(log1p/n)", &rows, |effect, npy| {
            let n = npy.trunc();
            n * rx::excel_expm1_internal(rx::excel_log1p(effect) / n)
        });
        score_nominal("expm1(x87-recip*log1p)", &rows, |effect, npy| {
            let n = npy.trunc();
            n * rx::excel_expm1(rx::x87_mul(rx::x87_recip(n), rx::excel_log1p(effect)))
        });
        score_nominal("internal-expm1(x87 product)", &rows, |effect, npy| {
            let n = npy.trunc();
            n * rx::excel_expm1_internal(rx::x87_mul(rx::x87_recip(n), rx::excel_log1p(effect)))
        });
        score_nominal("direct x87 pow, full", &rows, |effect, npy| {
            let n = npy.trunc();
            n * (direct_x87_pow_full(1.0 + effect, 1.0 / n) - 1.0)
        });
        score_nominal("direct x87 pow, spill arg", &rows, |effect, npy| {
            let n = npy.trunc();
            n * (direct_x87_pow_spilled_argument(1.0 + effect, 1.0 / n) - 1.0)
        });
        score_nominal("direct x87 pow, spill log2", &rows, |effect, npy| {
            let n = npy.trunc();
            n * (direct_x87_pow_spilled_log2(1.0 + effect, 1.0 / n) - 1.0)
        });
        score_nominal("hybrid direct n<=2/raw", &rows, |effect, npy| {
            let n = npy.trunc();
            let powered = if n <= 2.0 {
                rx::excel_pow_x87_direct(x87_add(1.0, effect), 1.0 / n)
            } else {
                rx::excel_pow_chain(x87_add(1.0, effect), 1.0 / n)
            };
            n * (powered - 1.0)
        });
    }
}
