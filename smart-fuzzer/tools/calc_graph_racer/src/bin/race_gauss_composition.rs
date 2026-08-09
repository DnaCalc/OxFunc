//! W109 GAUSS clean-room composition race.
//!
//! This research-only tool reads the six legacy build-19929/CV2 broad-scalar
//! cell-reference discovery captures.  They are deliberately named below;
//! no held-out bank is accepted.  The race asks whether GAUSS publishes the
//! mathematically equivalent direct odd form or a normal-CDF-minus-one-half
//! form, and whether `x/sqrt(2)` is formed with binary64 or x87 staging.
//!
//! Usage:
//!   race_gauss_composition <OxFunc repository root>
//!       [legacy|current|tiny|tiny-route|crossview|decode]
//!
//! Current-build modes name only the frozen discovery answer.  The sealed
//! heldout path is deliberately absent from this source.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::special_math_common::gratio;
use rx::{
    CW_PC64_RN, Ext80, ext_add, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_sub, ext_to_f64,
};
use serde::Deserialize;
use std::collections::BTreeMap;

const DISCOVERY_RUNS: [&str; 6] = [
    "broad-scalar-cycle-010-cellref",
    "broad-scalar-cycle-011-cellref",
    "broad-scalar-cycle-012-cellref",
    "broad-scalar-cycle-013-cellref",
    "broad-scalar-cycle-014-cellref",
    "broad-scalar-cycle-015-cellref",
];

const CW: u16 = CW_PC64_RN;

#[derive(Deserialize)]
struct Outcome {
    bits_hex: Option<String>,
}

#[derive(Deserialize)]
struct CaptureRow {
    function_id: String,
    formula_text: String,
    local_outcome: Outcome,
    excel_outcome: Outcome,
}

#[derive(Clone, Debug)]
struct Row {
    x: f64,
    expected: u64,
    run: &'static str,
    formula: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArgMode {
    NativeDivide,
    NativeMultiply,
    X87DivideStore,
    X87MultiplyStore,
    X87SqrtDivideStore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BodyMode {
    Erf,
    ErfcNegative,
    ErfcPositive,
    GratioErf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PublishMode {
    /// 0.5*erf(x/sqrt(2)): the stable direct odd identity.
    DirectHalf,
    /// 0.5*(1+erf)-0.5: explicit CDF publication then cancellation.
    HalfOfOnePlusThenSubtract,
    /// (0.5+0.5*erf)-0.5: alternate source association.
    HalfPlusHalfThenSubtract,
    /// 0.5*erfc(-x/sqrt(2))-0.5.
    HalfErfcThenSubtract,
    /// 0.5-0.5*erfc(x/sqrt(2)): direct positive-side complement.
    HalfMinusHalfErfc,
    /// (1-0.5*erfc(x/sqrt(2)))-0.5: explicit CDF publication.
    OneMinusHalfErfcThenSubtract,
    /// Keep the wrapper arithmetic register-continuous in Ext80.
    ExtendedWrapper,
    /// Round every wrapper arithmetic node through binary64 after x87.
    X87DoubleRoundedWrapper,
    /// Public normal-CDF sign split: use the lower-tail erfc form for x<0
    /// and the accurately complemented upper-tail form for x>0.
    SignSplitErfc,
    /// Sign-split publication around the public TOMS-654-derived GRATIO P body.
    SignSplitGratioErf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TinyMode {
    None,
    /// If CDF-minus-one-half cancels to zero, use x*RN(1/sqrt(2*pi)).
    NativePhiIfZero,
    /// Same fallback with the PHI-proven x87 double-rounded multiply.
    X87PhiIfZero,
    /// Explicit machine-epsilon switch, native multiply.
    NativePhiBelowEpsilon,
    /// Explicit machine-epsilon switch, PHI-proven x87 multiply.
    X87PhiBelowEpsilon,
    /// Fold the observed ERF zero-limit slope with 1/sqrt(2) and 0.5
    /// before multiplying by x.
    ErfSlopeFoldedBelowEpsilon,
}

#[derive(Clone, Copy, Debug)]
struct Cfg {
    arg: ArgMode,
    body: BodyMode,
    publish: PublishMode,
    tiny: TinyMode,
}

#[derive(Debug)]
struct Score {
    exact_all: usize,
    exact_finite: usize,
    exact_small: usize,
    exact_regular_small: usize,
    max_ulp_regular_small: u64,
    sum_ulp_regular_small: u64,
    cfg: Cfg,
}

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}

fn store(x: &Ext80) -> f64 {
    ext_to_f64(x, CW)
}

fn parse_hex(text: &str) -> Option<u64> {
    u64::from_str_radix(text.strip_prefix("0x")?, 16).ok()
}

fn parse_x(formula: &str) -> Option<f64> {
    formula
        .strip_prefix("=GAUSS(")?
        .strip_suffix(')')?
        .parse()
        .ok()
}

fn load_rows(root: &str) -> Vec<Row> {
    let mut rows = Vec::new();
    for run in DISCOVERY_RUNS {
        let path =
            format!("{root}/smart-fuzzer/runs/{run}/comparisons/excel_sample_comparisons.jsonl");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read discovery capture {path}: {e}"));
        for line in text.lines() {
            let capture: CaptureRow = serde_json::from_str(line)
                .unwrap_or_else(|e| panic!("failed to parse {path}: {e}"));
            if capture.function_id != "FUNC.GAUSS" {
                continue;
            }
            // Requiring both numeric outcomes rejects kind-drift rows.  The
            // local bits are intentionally not used for candidate scoring.
            let Some(_local) = capture
                .local_outcome
                .bits_hex
                .as_deref()
                .and_then(parse_hex)
            else {
                continue;
            };
            let Some(expected) = capture
                .excel_outcome
                .bits_hex
                .as_deref()
                .and_then(parse_hex)
            else {
                continue;
            };
            let x = parse_x(&capture.formula_text)
                .unwrap_or_else(|| panic!("unexpected GAUSS formula: {}", capture.formula_text));
            rows.push(Row {
                x,
                expected,
                run,
                formula: capture.formula_text,
            });
        }
    }
    rows
}

fn load_current_discovery_rows(root: &str) -> Vec<Row> {
    // Intentionally name only the discovery answer.  There is no held-out
    // path in this reader, so the sealed gate cannot be scored accidentally.
    const ANSWERS: &str = "answers-gauss-exact-discovery-v1.json";
    let path = format!("{root}/smart-fuzzer/work/w109/G3-07-gauss/{ANSWERS}");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read current discovery answer {path}: {e}"));
    let bank: WitnessSet =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("failed to parse {path}: {e}"));
    assert_eq!(bank.function, "GAUSS", "unexpected function in {path}");
    bank.witnesses
        .iter()
        .filter_map(|witness| {
            let x = match &witness.args[0] {
                WitnessArg::Scalar(bits) => parse_bits_hex(bits)?,
                WitnessArg::Array(_) => return None,
            };
            let expected = parse_bits_hex(&witness.expected_bits)?;
            Some(Row {
                x,
                expected: expected.to_bits(),
                run: "current-discovery-v1",
                formula: format!("=GAUSS(0x{:016x})", x.to_bits()),
            })
        })
        .collect()
}

fn load_route_discovery_rows(root: &str) -> Vec<Row> {
    // The independently frozen route heldout is sealed and intentionally
    // absent from this reader.
    const ANSWERS: &str = "answers-gauss-route-discovery-v1.json";
    let path = format!("{root}/smart-fuzzer/work/w109/G3-07-gauss/{ANSWERS}");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read route discovery answer {path}: {e}"));
    let bank: WitnessSet =
        serde_json::from_str(&text).unwrap_or_else(|e| panic!("failed to parse {path}: {e}"));
    assert_eq!(bank.function, "GAUSS", "unexpected function in {path}");
    assert_eq!(bank.witnesses.len(), 1_024, "route discovery count drifted");
    bank.witnesses
        .iter()
        .filter_map(|witness| {
            let x = match &witness.args[0] {
                WitnessArg::Scalar(bits) => parse_bits_hex(bits)?,
                WitnessArg::Array(_) => return None,
            };
            let expected = parse_bits_hex(&witness.expected_bits)?;
            Some(Row {
                x,
                expected: expected.to_bits(),
                run: "route-discovery-v1",
                formula: format!("=GAUSS(0x{:016x})", x.to_bits()),
            })
        })
        .collect()
}

fn arg(x: f64, mode: ArgMode) -> f64 {
    match mode {
        ArgMode::NativeDivide => x / std::f64::consts::SQRT_2,
        ArgMode::NativeMultiply => x * std::f64::consts::FRAC_1_SQRT_2,
        ArgMode::X87DivideStore => store(&ext_div(&ef(x), &ef(std::f64::consts::SQRT_2), CW)),
        ArgMode::X87MultiplyStore => {
            store(&ext_mul(&ef(x), &ef(std::f64::consts::FRAC_1_SQRT_2), CW))
        }
        ArgMode::X87SqrtDivideStore => {
            let root = ext_sqrt(&ef(2.0), CW);
            store(&ext_div(&ef(x), &root, CW))
        }
    }
}

fn body(t: f64, mode: BodyMode) -> f64 {
    match mode {
        BodyMode::Erf => libm::erf(t),
        BodyMode::ErfcNegative => libm::erfc(-t),
        BodyMode::ErfcPositive => libm::erfc(t),
        BodyMode::GratioErf => gratio(0.5, t * t).0,
    }
}

fn dr_add(a: f64, b: f64) -> f64 {
    store(&ext_add(&ef(a), &ef(b), CW))
}

fn dr_sub(a: f64, b: f64) -> f64 {
    store(&ext_sub(&ef(a), &ef(b), CW))
}

fn dr_mul(a: f64, b: f64) -> f64 {
    store(&ext_mul(&ef(a), &ef(b), CW))
}

fn compatible(body: BodyMode, publish: PublishMode) -> bool {
    match publish {
        PublishMode::DirectHalf
        | PublishMode::HalfOfOnePlusThenSubtract
        | PublishMode::HalfPlusHalfThenSubtract => {
            body == BodyMode::Erf || body == BodyMode::GratioErf
        }
        PublishMode::HalfErfcThenSubtract => body == BodyMode::ErfcNegative,
        PublishMode::HalfMinusHalfErfc | PublishMode::OneMinusHalfErfcThenSubtract => {
            body == BodyMode::ErfcPositive
        }
        PublishMode::ExtendedWrapper | PublishMode::X87DoubleRoundedWrapper => true,
        PublishMode::SignSplitErfc => body == BodyMode::ErfcPositive,
        PublishMode::SignSplitGratioErf => body == BodyMode::GratioErf,
    }
}

fn publish(v: f64, body_mode: BodyMode, mode: PublishMode) -> f64 {
    match mode {
        PublishMode::DirectHalf => 0.5 * v,
        PublishMode::HalfOfOnePlusThenSubtract => 0.5 * (1.0 + v) - 0.5,
        PublishMode::HalfPlusHalfThenSubtract => (0.5 + 0.5 * v) - 0.5,
        PublishMode::HalfErfcThenSubtract => 0.5 * v - 0.5,
        PublishMode::HalfMinusHalfErfc => 0.5 - 0.5 * v,
        PublishMode::OneMinusHalfErfcThenSubtract => (1.0 - 0.5 * v) - 0.5,
        PublishMode::ExtendedWrapper => {
            let ve = ef(v);
            let out = match body_mode {
                BodyMode::Erf | BodyMode::GratioErf => {
                    let cdf = ext_mul(&ef(0.5), &ext_add(&ef(1.0), &ve, CW), CW);
                    ext_sub(&cdf, &ef(0.5), CW)
                }
                BodyMode::ErfcNegative => {
                    let cdf = ext_mul(&ef(0.5), &ve, CW);
                    ext_sub(&cdf, &ef(0.5), CW)
                }
                BodyMode::ErfcPositive => {
                    let cdf = ext_sub(&ef(1.0), &ext_mul(&ef(0.5), &ve, CW), CW);
                    ext_sub(&cdf, &ef(0.5), CW)
                }
            };
            store(&out)
        }
        PublishMode::X87DoubleRoundedWrapper => match body_mode {
            BodyMode::Erf | BodyMode::GratioErf => dr_sub(dr_mul(0.5, dr_add(1.0, v)), 0.5),
            BodyMode::ErfcNegative => dr_sub(dr_mul(0.5, v), 0.5),
            BodyMode::ErfcPositive => dr_sub(dr_sub(1.0, dr_mul(0.5, v)), 0.5),
        },
        PublishMode::SignSplitErfc | PublishMode::SignSplitGratioErf => {
            unreachable!("sign split needs the transformed argument")
        }
    }
}

fn eval(x: f64, cfg: Cfg) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    let t = arg(x, cfg.arg);
    let base = if cfg.publish == PublishMode::SignSplitErfc {
        if x < 0.0 {
            0.5 * libm::erfc(-t) - 0.5
        } else {
            (1.0 - 0.5 * libm::erfc(t)) - 0.5
        }
    } else if cfg.publish == PublishMode::SignSplitGratioErf {
        let erf_abs = gratio(0.5, t * t).0;
        let erfc_abs = 1.0 - erf_abs;
        if x < 0.0 {
            0.5 * erfc_abs - 0.5
        } else {
            (1.0 - 0.5 * erfc_abs) - 0.5
        }
    } else {
        publish(body(t, cfg.body), cfg.body, cfg.publish)
    };
    const INV_SQRT_2PI: f64 = f64::from_bits(0x3fd9_8845_33d4_3651);
    match cfg.tiny {
        TinyMode::None => base,
        TinyMode::NativePhiIfZero if base == 0.0 => x * INV_SQRT_2PI,
        TinyMode::X87PhiIfZero if base == 0.0 => rx::x87_mul(x, INV_SQRT_2PI),
        TinyMode::NativePhiBelowEpsilon if x.abs() <= f64::EPSILON => x * INV_SQRT_2PI,
        TinyMode::X87PhiBelowEpsilon if x.abs() <= f64::EPSILON => rx::x87_mul(x, INV_SQRT_2PI),
        TinyMode::ErfSlopeFoldedBelowEpsilon if x.abs() <= f64::EPSILON => {
            // Cross-view black-box limit: ERF(z)/z tends to CR(2/sqrt(pi))+1 ULP.
            const ERF_SLOPE: f64 = f64::from_bits(0x3ff2_0dd7_5042_9b6e);
            x * ((std::f64::consts::FRAC_1_SQRT_2 * ERF_SLOPE) * 0.5)
        }
        _ => base,
    }
}

fn ordered(bits: u64) -> i128 {
    let signed = bits as i64;
    if signed < 0 {
        (!signed) as i128
    } else {
        signed as i128
    }
}

fn distance(a: u64, b: u64) -> u64 {
    ordered(a).abs_diff(ordered(b)) as u64
}

fn score_tiny_linear(rows: &[Row]) {
    const CENTER: u64 = 0x3fd9_8845_33d4_3651;
    let tiny: Vec<_> = rows.iter().filter(|row| row.x.abs() <= 1e-15).collect();
    let mut scores: Vec<(usize, u64, u64, i64, bool)> = Vec::new();
    for offset in -32i64..=32 {
        let c = f64::from_bits((CENTER as i64 + offset) as u64);
        for x87 in [false, true] {
            let (mut exact, mut max_ulp, mut sum_ulp) = (0usize, 0u64, 0u64);
            for row in &tiny {
                let mut got = if x87 {
                    rx::x87_mul(row.x, c)
                } else {
                    row.x * c
                };
                if got.abs() < f64::MIN_POSITIVE {
                    got = 0.0;
                }
                let d = distance(got.to_bits(), row.expected);
                exact += usize::from(d == 0);
                max_ulp = max_ulp.max(d);
                sum_ulp = sum_ulp.saturating_add(d);
            }
            scores.push((exact, max_ulp, sum_ulp, offset, x87));
        }
    }
    scores.sort_by_key(|row| (usize::MAX - row.0, row.1, row.2));
    println!(
        "GAUSS inclusive-1e-15 direct-route linear race: {} rows, RN(1/sqrt(2pi)) +/-32 f64 ULP x native/x87, with canonical subnormal flush",
        tiny.len()
    );
    for (rank, &(exact, max_ulp, sum_ulp, offset, x87)) in scores.iter().take(24).enumerate() {
        println!(
            "#{:02} exact={}/{} max={} sum={} offset={:+} constant=0x{:016x} multiply={}",
            rank + 1,
            exact,
            tiny.len(),
            max_ulp,
            sum_ulp,
            offset,
            (CENTER as i64 + offset) as u64,
            if x87 { "x87-rn64-rn53" } else { "native53" }
        );
    }

    // Public GRATIO branch 190 has the zero-limit graph
    // 0.5 * (x/sqrt(2)) * (1 + gam1(0.5)). Include the empirically decoded
    // sub-double effective-g center and the nearest values produced by mixed
    // spill evaluations of the published GAM1 rational.
    const G_MANTISSAS: [u64; 13] = [
        0x906e_ba82_14db_6c6f,
        0x906e_ba82_14db_6c36,
        0x906e_ba82_14db_6c29,
        0x906e_ba82_14db_6c28,
        0x906e_ba82_14db_6c00,
        0x906e_ba82_14db_6bd6,
        0x906e_ba82_14db_6bd5,
        0x906e_ba82_14db_6bd0,
        0x906e_ba82_14db_6bca,
        0x906e_ba82_14db_6bc9,
        0x906e_ba82_14db_6bc8,
        0x906e_ba82_14db_6bc3,
        0x906e_ba82_14db_6b6a,
    ];
    let spill = |value: Ext80, yes: bool| if yes { ef(store(&value)) } else { value };
    let mut graphs: Vec<(usize, u64, u64, u64, u8, u8)> = Vec::new();
    for mantissa in G_MANTISSAS {
        let mut bytes = [0u8; 10];
        bytes[..8].copy_from_slice(&mantissa.to_le_bytes());
        bytes[8] = 0xff;
        bytes[9] = 0x3f;
        let g = Ext80(bytes);
        for order in 0u8..3 {
            for mask in 0u8..8 {
                let (mut exact, mut max_ulp, mut sum_ulp) = (0usize, 0u64, 0u64);
                for row in &tiny {
                    let x = ef(row.x);
                    let c = ef(std::f64::consts::FRAC_1_SQRT_2);
                    let half = ef(0.5);
                    let out = match order {
                        0 => {
                            let a = spill(ext_mul(&x, &c, CW), mask & 1 != 0);
                            let b = spill(ext_mul(&a, &g, CW), mask & 2 != 0);
                            spill(ext_mul(&b, &half, CW), mask & 4 != 0)
                        }
                        1 => {
                            let a = spill(ext_mul(&c, &g, CW), mask & 1 != 0);
                            let b = spill(ext_mul(&a, &half, CW), mask & 2 != 0);
                            spill(ext_mul(&x, &b, CW), mask & 4 != 0)
                        }
                        _ => {
                            let a = spill(ext_mul(&x, &g, CW), mask & 1 != 0);
                            let b = spill(ext_mul(&c, &half, CW), mask & 2 != 0);
                            spill(ext_mul(&a, &b, CW), mask & 4 != 0)
                        }
                    };
                    let mut got = store(&out);
                    if got.abs() < f64::MIN_POSITIVE {
                        got = 0.0;
                    }
                    let d = distance(got.to_bits(), row.expected);
                    exact += usize::from(d == 0);
                    max_ulp = max_ulp.max(d);
                    sum_ulp = sum_ulp.saturating_add(d);
                }
                graphs.push((exact, max_ulp, sum_ulp, mantissa, order, mask));
            }
        }
    }
    graphs.sort_by_key(|row| (usize::MAX - row.0, row.1, row.2));
    println!(
        "GAUSS tiny branch-190 limit race: {} g values x 3 associations x 8 spill masks",
        G_MANTISSAS.len()
    );
    for (rank, &(exact, max_ulp, sum_ulp, mantissa, order, mask)) in
        graphs.iter().take(24).enumerate()
    {
        println!(
            "#{:02} exact={}/{} max={} sum={} g=0x{mantissa:016x} order={} mask=0b{mask:03b}",
            rank + 1,
            exact,
            tiny.len(),
            max_ulp,
            sum_ulp,
            order
        );
    }

    const G_SCAN_CENTER: u64 = 0x906e_ba82_14db_6bd6;
    let mut g_scan = Vec::new();
    for offset in -4096i32..=4096 {
        let mantissa = (G_SCAN_CENTER as i128 + offset as i128) as u64;
        let mut bytes = [0u8; 10];
        bytes[..8].copy_from_slice(&mantissa.to_le_bytes());
        bytes[8] = 0xff;
        bytes[9] = 0x3f;
        let g = Ext80(bytes);
        let (mut exact, mut max_ulp, mut sum_ulp) = (0usize, 0u64, 0u64);
        for row in &tiny {
            // The discrete graph race above pins order=2, mask bit 0: store
            // x*g, keep the folded (1/sqrt(2))*0.5 factor extended.
            let xg = ef(store(&ext_mul(&ef(row.x), &g, CW)));
            let folded = ext_mul(&ef(std::f64::consts::FRAC_1_SQRT_2), &ef(0.5), CW);
            let mut got = store(&ext_mul(&xg, &folded, CW));
            if got.abs() < f64::MIN_POSITIVE {
                got = 0.0;
            }
            let d = distance(got.to_bits(), row.expected);
            exact += usize::from(d == 0);
            max_ulp = max_ulp.max(d);
            sum_ulp = sum_ulp.saturating_add(d);
        }
        g_scan.push((exact, max_ulp, sum_ulp, offset, mantissa));
    }
    g_scan.sort_by_key(|row| (usize::MAX - row.0, row.1, row.2));
    println!("GAUSS tiny effective-g scan (+/-4096 Ext80 mantissa units):");
    for (rank, &(exact, max_ulp, sum_ulp, offset, mantissa)) in g_scan.iter().take(24).enumerate() {
        println!(
            "#{:02} exact={}/{} max={} sum={} offset={:+} g=0x{mantissa:016x}",
            rank + 1,
            exact,
            tiny.len(),
            max_ulp,
            sum_ulp,
            offset
        );
    }
}

fn load_scalar_answer_map(root: &str, names: &[&str]) -> BTreeMap<u64, u64> {
    let mut map = BTreeMap::new();
    for name in names {
        let path = format!("{root}/smart-fuzzer/work/w109/G3-01-dist/{name}");
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read discovery bank {path}: {e}"));
        let bank: WitnessSet =
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("failed to parse {path}: {e}"));
        for witness in bank.witnesses {
            let x = match &witness.args[0] {
                WitnessArg::Scalar(bits) => parse_bits_hex(bits).expect("scalar input"),
                WitnessArg::Array(_) => continue,
            };
            let Some(expected) = parse_bits_hex(&witness.expected_bits) else {
                continue;
            };
            if let Some(old) = map.insert(x.to_bits(), expected.to_bits()) {
                assert_eq!(old, expected.to_bits(), "conflicting answer for {x:?}");
            }
        }
    }
    map
}

fn score_existing_crossview(root: &str, rows: &[Row]) {
    const ERF_BANKS: [&str; 6] = [
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b8erf.json",
        "answers-b7erf.json",
        "answers-b10.json",
        "answers-b11.json",
    ];
    const ERFC_BANKS: [&str; 5] = [
        "answers-erfcp.json",
        "answers-erfcm.json",
        "answers-b8erfc.json",
        "answers-b7erfc.json",
        "answers-b11c.json",
    ];
    let erf = load_scalar_answer_map(root, &ERF_BANKS);
    let erfc = load_scalar_answer_map(root, &ERFC_BANKS);
    println!(
        "existing discovery cross-view maps: ERF={} ERFC={}; historical heldout absent",
        erf.len(),
        erfc.len()
    );
    for arg_mode in [
        ArgMode::NativeMultiply,
        ArgMode::NativeDivide,
        ArgMode::X87MultiplyStore,
        ArgMode::X87DivideStore,
        ArgMode::X87SqrtDivideStore,
    ] {
        let (mut p_overlap, mut p_direct, mut p_cdf, mut p_split) = (0, 0, 0, 0);
        let (mut q_overlap, mut q_split) = (0, 0);
        for row in rows {
            let z = arg(row.x.abs(), arg_mode);
            if let Some(&p_bits) = erf.get(&z.to_bits()) {
                p_overlap += 1;
                let p = f64::from_bits(p_bits);
                let signed_p = if row.x < 0.0 { -p } else { p };
                let direct = (0.5 * signed_p).to_bits();
                let cdf = (0.5 * (1.0 + signed_p) - 0.5).to_bits();
                let q = 1.0 - p;
                let split = if row.x < 0.0 {
                    0.5 * q - 0.5
                } else {
                    (1.0 - 0.5 * q) - 0.5
                }
                .to_bits();
                p_direct += usize::from(direct == row.expected);
                p_cdf += usize::from(cdf == row.expected);
                p_split += usize::from(split == row.expected);
            }
            if let Some(&q_bits) = erfc.get(&z.to_bits()) {
                q_overlap += 1;
                let q = f64::from_bits(q_bits);
                let split = if row.x < 0.0 {
                    0.5 * q - 0.5
                } else {
                    (1.0 - 0.5 * q) - 0.5
                }
                .to_bits();
                q_split += usize::from(split == row.expected);
            }
        }
        println!(
            "  {:?}: ERF overlap={} direct={} cdf={} signsplit(1-P)={}; ERFC overlap={} signsplit(Q)={}",
            arg_mode, p_overlap, p_direct, p_cdf, p_split, q_overlap, q_split
        );
    }
}

fn decode_symmetric_gauss_pairs(rows: &[Row]) {
    let by_input: BTreeMap<u64, u64> = rows
        .iter()
        .map(|row| (row.x.to_bits(), row.expected))
        .collect();
    let mut ambiguity: BTreeMap<usize, usize> = BTreeMap::new();
    let (mut pairs, mut decoded_pairs) = (0usize, 0usize);
    let mut decoded_by_z: BTreeMap<u64, u64> = BTreeMap::new();
    let mut q_zero_inputs = Vec::new();
    let mut q_zero_direct_odd = 0usize;
    for row in rows
        .iter()
        .filter(|row| row.x > f64::EPSILON && row.x < 0.7)
    {
        let Some(&negative_expected) = by_input.get(&(row.x.to_bits() | (1u64 << 63))) else {
            continue;
        };
        pairs += 1;
        let positive_expected = row.expected;
        let yp = f64::from_bits(positive_expected);
        let yn = f64::from_bits(negative_expected);
        let center_p = (2.0 * (0.5 - yp)).to_bits();
        let center_n = (2.0 * (yn + 0.5)).to_bits();
        let lo = center_p.min(center_n).saturating_sub(64);
        let hi = center_p.max(center_n).saturating_add(64);
        let candidates: Vec<u64> = (lo..=hi)
            .filter(|&bits| {
                let q = f64::from_bits(bits);
                let got_p = ((1.0 - 0.5 * q) - 0.5).to_bits();
                let got_n = (0.5 * q - 0.5).to_bits();
                got_p == positive_expected && got_n == negative_expected
            })
            .collect();
        *ambiguity.entry(candidates.len()).or_default() += 1;
        if candidates.is_empty() {
            q_zero_inputs.push(row.x);
            q_zero_direct_odd +=
                usize::from(negative_expected == (positive_expected | (1u64 << 63)));
        }
        if candidates.len() != 1 {
            continue;
        }
        decoded_pairs += 1;
        let q_bits = candidates[0];
        let z = row.x * std::f64::consts::FRAC_1_SQRT_2;
        if let Some(old) = decoded_by_z.insert(z.to_bits(), q_bits) {
            assert_eq!(old, q_bits, "conflicting symmetric decode at z={z:?}");
        }
    }

    let (mut libm_exact, mut gratio_exact, mut residual_examples) = (0usize, 0usize, 0usize);
    for (&z_bits, &q_bits) in &decoded_by_z {
        let z = f64::from_bits(z_bits);
        let libm_bits = libm::erfc(z).to_bits();
        let gratio_bits = (1.0 - gratio(0.5, z * z).0).to_bits();
        libm_exact += usize::from(libm_bits == q_bits);
        gratio_exact += usize::from(gratio_bits == q_bits);
        if residual_examples < 40 && libm_bits != q_bits && gratio_bits != q_bits {
            println!(
                "  z=0x{:016x} q=0x{q_bits:016x} libm_delta={} gratio_delta={}",
                z.to_bits(),
                ordered(q_bits) - ordered(libm_bits),
                ordered(q_bits) - ordered(gratio_bits)
            );
            residual_examples += 1;
        }
    }
    println!(
        "symmetric GAUSS discovery decode (EPS<x<0.7): pairs={pairs}, uniquely_decoded_pairs={decoded_pairs}, distinct_z_q={}; libm_erfc_exact={libm_exact}/{}, production_public_gratio_q_exact={gratio_exact}/{}",
        decoded_by_z.len(),
        decoded_by_z.len(),
        decoded_by_z.len()
    );
    println!("  ambiguity histogram (candidate-q count -> pairs): {ambiguity:?}");
    if !q_zero_inputs.is_empty() {
        q_zero_inputs.sort_by(|a, b| a.total_cmp(b));
        let mut exponents: BTreeMap<i32, usize> = BTreeMap::new();
        for &x in &q_zero_inputs {
            *exponents.entry(x.log2().floor() as i32).or_default() += 1;
        }
        println!(
            "  no-Q pairs: count={} direct-odd-compatible={} min=0x{:016x} ({:.17e}) max=0x{:016x} ({:.17e}) floor-log2 histogram={exponents:?}",
            q_zero_inputs.len(),
            q_zero_direct_odd,
            q_zero_inputs[0].to_bits(),
            q_zero_inputs[0],
            q_zero_inputs[q_zero_inputs.len() - 1].to_bits(),
            q_zero_inputs[q_zero_inputs.len() - 1],
        );
        for &x in q_zero_inputs.iter().take(8) {
            println!("    first no-Q x=0x{:016x} ({x:.17e})", x.to_bits());
        }
        for &x in q_zero_inputs.iter().rev().take(8).rev() {
            println!("    last  no-Q x=0x{:016x} ({x:.17e})", x.to_bits());
        }
    }
}

fn main() {
    let root = std::env::args().nth(1).expect("OxFunc repository root");
    let mode = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "legacy".to_string());
    let rows = match mode.as_str() {
        "legacy" => {
            let rows = load_rows(&root);
            assert_eq!(rows.len(), 62, "legacy GAUSS discovery row count drifted");
            println!(
                "{} legacy build-19929/CV2 discovery rows from six explicitly named cell-ref captures; no heldout reader",
                rows.len()
            );
            rows
        }
        "current" | "tiny" | "tiny-route" | "crossview" | "decode" => {
            let rows = load_current_discovery_rows(&root);
            assert_eq!(
                rows.len(),
                8_192,
                "current GAUSS discovery answer row count drifted"
            );
            println!(
                "{} current discovery rows from the one explicitly named answer bank; no heldout reader",
                rows.len()
            );
            if mode == "tiny-route" {
                let mut by_input: BTreeMap<u64, Row> =
                    rows.into_iter().map(|row| (row.x.to_bits(), row)).collect();
                for row in load_route_discovery_rows(&root) {
                    if row.x.abs() <= 1e-15 {
                        if let Some(old) = by_input.insert(row.x.to_bits(), row.clone()) {
                            assert_eq!(old.expected, row.expected, "conflicting route answer");
                        }
                    }
                }
                println!(
                    "{} combined exact-discovery plus direct-side route-discovery rows; both heldout paths absent",
                    by_input.len()
                );
                by_input.into_values().collect()
            } else {
                rows
            }
        }
        _ => panic!("mode must be legacy|current|tiny|tiny-route|crossview|decode"),
    };

    if mode == "tiny" || mode == "tiny-route" {
        score_tiny_linear(&rows);
        return;
    }
    if mode == "crossview" {
        score_existing_crossview(&root, &rows);
        return;
    }
    if mode == "decode" {
        decode_symmetric_gauss_pairs(&rows);
        return;
    }

    let arg_modes = [
        ArgMode::NativeDivide,
        ArgMode::NativeMultiply,
        ArgMode::X87DivideStore,
        ArgMode::X87MultiplyStore,
        ArgMode::X87SqrtDivideStore,
    ];
    let body_modes = [
        BodyMode::Erf,
        BodyMode::ErfcNegative,
        BodyMode::ErfcPositive,
        BodyMode::GratioErf,
    ];
    let publish_modes = [
        PublishMode::DirectHalf,
        PublishMode::HalfOfOnePlusThenSubtract,
        PublishMode::HalfPlusHalfThenSubtract,
        PublishMode::HalfErfcThenSubtract,
        PublishMode::HalfMinusHalfErfc,
        PublishMode::OneMinusHalfErfcThenSubtract,
        PublishMode::ExtendedWrapper,
        PublishMode::X87DoubleRoundedWrapper,
        PublishMode::SignSplitErfc,
        PublishMode::SignSplitGratioErf,
    ];
    let tiny_modes = [
        TinyMode::None,
        TinyMode::NativePhiIfZero,
        TinyMode::X87PhiIfZero,
        TinyMode::NativePhiBelowEpsilon,
        TinyMode::X87PhiBelowEpsilon,
        TinyMode::ErfSlopeFoldedBelowEpsilon,
    ];

    let mut scores = Vec::new();
    for arg in arg_modes {
        for body in body_modes {
            for publish in publish_modes {
                if !compatible(body, publish) {
                    continue;
                }
                for tiny in tiny_modes {
                    let cfg = Cfg {
                        arg,
                        body,
                        publish,
                        tiny,
                    };
                    let mut score = Score {
                        exact_all: 0,
                        exact_finite: 0,
                        exact_small: 0,
                        exact_regular_small: 0,
                        max_ulp_regular_small: 0,
                        sum_ulp_regular_small: 0,
                        cfg,
                    };
                    for row in &rows {
                        let got = eval(row.x, cfg).to_bits();
                        let exact = got == row.expected;
                        score.exact_all += usize::from(exact);
                        if row.x.abs() < 9.0 {
                            score.exact_finite += usize::from(exact);
                        }
                        if row.x.abs() < 1.0 {
                            score.exact_small += usize::from(exact);
                        }
                        if row.x.abs() >= 1e-10 && row.x.abs() < 1.0 {
                            let d = distance(got, row.expected);
                            score.exact_regular_small += usize::from(d == 0);
                            score.max_ulp_regular_small = score.max_ulp_regular_small.max(d);
                            score.sum_ulp_regular_small =
                                score.sum_ulp_regular_small.saturating_add(d);
                        }
                    }
                    scores.push(score);
                }
            }
        }
    }
    scores.sort_by_key(|s| {
        (
            usize::MAX - s.exact_regular_small,
            s.max_ulp_regular_small,
            s.sum_ulp_regular_small,
            usize::MAX - s.exact_all,
        )
    });

    let finite_count = rows.iter().filter(|r| r.x.abs() < 9.0).count();
    let small_count = rows.iter().filter(|r| r.x.abs() < 1.0).count();
    let regular_small_count = rows
        .iter()
        .filter(|r| r.x.abs() >= 1e-10 && r.x.abs() < 1.0)
        .count();
    println!(
        "subsets: finite/non-saturated |x|<9={finite_count}; small |x|<1={small_count}; regular-small 1e-10<=|x|<1={regular_small_count}"
    );
    println!(
        "raced {} explicit public-libm composition graphs",
        scores.len()
    );
    for (rank, s) in scores.iter().take(24).enumerate() {
        println!(
            "#{:02} regular-small={}/{} max={} sum={} small={}/{} finite={}/{} all={}/{} {:?}",
            rank + 1,
            s.exact_regular_small,
            regular_small_count,
            s.max_ulp_regular_small,
            s.sum_ulp_regular_small,
            s.exact_small,
            small_count,
            s.exact_finite,
            finite_count,
            s.exact_all,
            rows.len(),
            s.cfg
        );
    }

    let best = &scores[0];
    println!("best candidate row diagnostics:");
    let mut diagnostics = 0usize;
    for row in &rows {
        let got = eval(row.x, best.cfg).to_bits();
        if got != row.expected && diagnostics < 200 {
            println!(
                "  {} {} got=0x{got:016x} want=0x{:016x} ulp={} {}",
                row.run,
                row.formula,
                row.expected,
                distance(got, row.expected),
                if got == row.expected { "EXACT" } else { "" }
            );
            diagnostics += 1;
        }
    }
    println!(
        "  displayed {diagnostics} of {} mismatches",
        rows.len() - best.exact_all
    );

    // Stable witness requested by the G3-07 catalog row.
    println!("GAUSS(1) candidate bits:");
    for s in scores.iter().take(8) {
        println!("  0x{:016x} {:?}", eval(1.0, s.cfg).to_bits(), s.cfg);
    }

    let mut by_output: BTreeMap<u64, usize> = BTreeMap::new();
    for s in &scores {
        *by_output.entry(eval(1.0, s.cfg).to_bits()).or_default() += 1;
    }
    println!("GAUSS(1) distinct candidate outputs:");
    for (bits, count) in by_output {
        println!("  0x{bits:016x}: {count} graphs");
    }
}
