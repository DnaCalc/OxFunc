//! Firehorse ERFC-body campaign: named F, last-bit NSWC coeffs, then
//! mask-sensitive x87 Horner-stage store cubes (NSWC P/Q, R, AA/BB, Cody C/D).
//! Checkpoints after every chunk. Heldouts unnamed. Does not land kernels.
//!
//! Usage:
//!   campaign_erfc_body --dir G3-01-dist --out erfc-campaign --threads 12 --max-hours 96

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::special_dist_family::erfc_precise_kernel;
use rayon::prelude::*;
use rx::{
    excel_exp, ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC53_RN,
    CW_PC64_RN,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const FRAC_1_SQRT_2: f64 = f64::from_bits(0x3fe6a09e667f3bcd);
const ERFC_BANKS: [&str; 5] = [
    "answers-erfcp.json",
    "answers-erfcm.json",
    "answers-b7erfc.json",
    "answers-b8erfc.json",
    "answers-b11c.json",
];
const PIN_Z: [f64; 5] = [0.75, 1.28125, 1.875, 2.125, 5.0];
const CHUNK: u32 = 4096;
/// Packed NSWC PQR + t-formation store mask:
/// bits 0-6 P stages, 7-13 Q stages, 14-22 R stages,
/// 23 spill(y-3.75), 24 spill(y+3.75), 25 spill(t=num/den).
const PQR_MASK_LIM: u32 = 1 << 26;
/// AA stages 0-7, BB stages 8-18.
const AABB_MASK_LIM: u32 = 1 << 19;
const CODY_MASK_LIM: u32 = 1 << 16;
/// Named-F / assoc-race mid-band bar (X87Continuous + x87-DR + store u/v +
/// cuts 0.5/1.5/3.5): 3332/7741. Cubes try to beat this, not land it.
const NAMED_MID_BAR: usize = 3332;

const P: [f64; 8] = [
    0.16506148041280876191828601e-03,
    0.15471455377139313353998665e-03,
    0.44852548090298868465196794e-04,
    -0.49177280017226285450486205e-05,
    -0.69353602078656412367801676e-05,
    -0.20508667787746282746857743e-05,
    -0.28982842617824971177267380e-06,
    -0.17272433544836633301127174e-07,
];
const Q: [f64; 8] = [
    1.0,
    0.16272656776533322859856317e+01,
    0.12040996037066026106794322e+01,
    0.52400246352158386907601472e+00,
    0.14497345252798672362384241e+00,
    0.25592517111042546492590736e-01,
    0.26869088293991371028123158e-02,
    0.13133767840925681614496481e-03,
];
const R: [f64; 9] = [
    0.145589721275038539045668824025,
    -0.273421931495426482902320421863,
    0.226008066916621506788789064272,
    -0.163571895523923805648814425592,
    0.102604312032193978662297299832,
    -0.548023266949835519254211506880e-01,
    0.241432239725390106956523668160e-01,
    -0.822062115403915116036874169600e-02,
    0.180296241564687154310619200000e-02,
];
const AA: [f64; 9] = [
    -0.45894433406309678202825375e-03,
    -0.12281298722544724287816236e-01,
    -0.91144359512342900801764781e-01,
    -0.28412489223839285652511367e-01,
    0.14083827189977123530129812e+01,
    0.11532175281537044570477189e+01,
    -0.72170903389442152112483632e+01,
    -0.19685597805218214001309225e+01,
    0.93846891504541841150916038e+01,
];
const BB: [f64; 12] = [
    1.0,
    0.25136329960926527692263725e+02,
    0.15349442087145759184067981e+03,
    -0.29971215958498680905476402e+03,
    -0.33876477506888115226730368e+04,
    0.28301829314924804988873701e+04,
    0.22979620942196507068034887e+05,
    -0.24280681522998071562462041e+05,
    -0.36680620673264731899504580e+05,
    0.42278731622295627627042436e+05,
    0.28834257644413614344549790e+03,
    0.70226293775648358646587341e+03,
];
const NSWC_A: [f64; 21] = [
    0.1283791670955125738961589031215e+00,
    -0.3761263890318375246320529677070e+00,
    0.1128379167095512573896158902931e+00,
    -0.2686617064513125175943235372542e-01,
    0.5223977625442187842111812447877e-02,
    -0.8548327023450852832540164081187e-03,
    0.1205533298178966425020717182498e-03,
    -0.1492565035840625090430728526820e-04,
    0.1646211436588924261080723578109e-05,
    -0.1636584469123468757408968429674e-06,
    0.1480719281587021715400818627811e-07,
    -0.1229055530145120140800510155331e-08,
    0.9422759058437197017313055084212e-10,
    -0.6711366740969385085896257227159e-11,
    0.4463222608295664017461758843550e-12,
    -0.2783497395542995487275065856998e-13,
    0.1634095572365337143933023780777e-14,
    -0.9052845786901123985710019387938e-16,
    0.4708274559689744439341671426731e-17,
    -0.2187159356685015949749948252160e-18,
    0.7043407712019701609635599701333e-20,
];
const E0: f64 = 0.540464821348814822409610122136;
const E1: f64 = -0.261515522487415653487049835220e-01;
const E2: f64 = -0.288573438386338758794591212600e-02;
const CODY_C: [f64; 9] = [
    0.564188496988670089,
    8.88314979438837594,
    66.1191906371416295,
    298.635138197400131,
    881.95222124176909,
    1712.04761263407058,
    2051.07837782607147,
    1230.33935479799725,
    2.15311535474403846e-8,
];
const CODY_D: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];
const CODY_SQRPI: f64 = 0.56418958354775628695;

const MONITOR: &str = r#"# ERFC campaign monitor

Host: dna-firehorse. Out dir on the host:
`/data/projects/DnaCalc/OxFunc/smart-fuzzer/work/w109/erfc-campaign/`

From the Windows box:

```
scp dna-firehorse:/data/projects/DnaCalc/OxFunc/smart-fuzzer/work/w109/erfc-campaign/STATUS.md .
scp dna-firehorse:/data/projects/DnaCalc/OxFunc/smart-fuzzer/work/w109/erfc-campaign/REGION_MAP.md .
ssh dna-firehorse -- tmux capture-pane -t oxfunc-erfc-campaign -p | tail
```

Stop after the current chunk: drop an empty file named `STOP` in the out dir.
Resume: same `run-erfc-campaign.sh` (completed progress keys are skipped).

This campaign does not land an ERFC body. HIT_ALL_MID, if present, is a
discovery alert — not a production identity.
"#;

#[derive(Clone, Copy, Debug)]
enum Arith {
    Native,
    X87Cont,
    X87Every,
    X87Pc53,
    X87Stage,
}

#[derive(Clone, Copy, Debug)]
enum Ratio {
    Cont,
    Store,
    Recip,
}

impl Ratio {
    fn tag(self) -> &'static str {
        match self {
            Ratio::Cont => "uvC",
            Ratio::Store => "uvS",
            Ratio::Recip => "uvR",
        }
    }
}

fn flush(v: f64) -> f64 {
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn dbl(x: &Ext80) -> f64 {
    ext_to_f64(x, CW_PC64_RN)
}
fn spill(x: Ext80) -> Ext80 {
    ef(dbl(&x))
}
fn cw(a: Arith) -> u16 {
    match a {
        Arith::X87Pc53 => CW_PC53_RN,
        _ => CW_PC64_RN,
    }
}

fn add(a: &Ext80, b: &Ext80, arith: Arith) -> Ext80 {
    match arith {
        Arith::Native => ef(dbl(a) + dbl(b)),
        Arith::X87Every => spill(ext_add(a, b, CW_PC64_RN)),
        _ => ext_add(a, b, cw(arith)),
    }
}
fn sub(a: &Ext80, b: &Ext80, arith: Arith) -> Ext80 {
    match arith {
        Arith::Native => ef(dbl(a) - dbl(b)),
        Arith::X87Every => spill(ext_sub(a, b, CW_PC64_RN)),
        _ => ext_sub(a, b, cw(arith)),
    }
}
fn mul(a: &Ext80, b: &Ext80, arith: Arith) -> Ext80 {
    match arith {
        Arith::Native => ef(dbl(a) * dbl(b)),
        Arith::X87Every => spill(ext_mul(a, b, CW_PC64_RN)),
        _ => ext_mul(a, b, cw(arith)),
    }
}
fn div(a: &Ext80, b: &Ext80, arith: Arith) -> Ext80 {
    match arith {
        Arith::Native => ef(dbl(a) / dbl(b)),
        Arith::X87Every => spill(ext_div(a, b, CW_PC64_RN)),
        _ => ext_div(a, b, cw(arith)),
    }
}

fn horner_native(cs: &[f64], x: f64) -> f64 {
    let mut acc = 0.0;
    for &c in cs.iter().rev() {
        acc = acc * x + c;
    }
    acc
}

fn horner_mask(cs: &[f64], x: &Ext80, mask: u32, bit0: u32, arith: Arith) -> Ext80 {
    let mut acc = ef(*cs.last().unwrap());
    let mut b = bit0;
    for &c in cs.iter().rev().skip(1) {
        acc = add(&mul(&acc, x, arith), &ef(c), arith);
        if matches!(arith, Arith::X87Stage) || mask & (1 << b) != 0 {
            acc = spill(acc);
        }
        b += 1;
    }
    acc
}

fn apply_ratio(u: Ext80, v: Ext80, arith: Arith, ratio: Ratio) -> Ext80 {
    match ratio {
        Ratio::Cont => div(&u, &v, arith),
        Ratio::Store => spill(div(&u, &v, arith)),
        Ratio::Recip => mul(&u, &div(&ef(1.0), &v, arith), arith),
    }
}

fn horner_r(acc0: Ext80, t: Ext80, mask: u32, arith: Arith) -> Ext80 {
    if matches!(arith, Arith::Native) {
        let tt = dbl(&t);
        let mut a = dbl(&acc0);
        for &c in R.iter().rev() {
            a = a * tt + c;
        }
        return ef(a);
    }
    let mut acc = acc0;
    let mut b = 14u32;
    for &c in R.iter().rev() {
        acc = add(&mul(&acc, &t, arith), &ef(c), arith);
        if matches!(arith, Arith::X87Stage) || mask & (1 << b) != 0 {
            acc = spill(acc);
        }
        b += 1;
    }
    acc
}

fn square_y(y: f64, ye: &Ext80, zz_dr: bool, arith: Arith) -> f64 {
    if zz_dr {
        dbl(&spill(ext_mul(ye, ye, cw(arith))))
    } else {
        y * y
    }
}

/// NSWC DERFC0 F-body used by the cubes: P/Q/R below `mid_cut`, AA/BB above.
/// Small `y <= small_cut` uses the published A-series (not libm).
/// `mask_aabb` selects AA/BB Horner bits; otherwise `mask` is the 26-bit PQR+t cube.
fn nswc_body(
    z: f64,
    mask: u32,
    arith: Arith,
    zz_dr: bool,
    ratio: Ratio,
    small_cut: f64,
    mid_cut: f64,
    mask_aabb: bool,
) -> f64 {
    let y = z.abs();
    let ye = ef(y);
    let pqr_mask = if mask_aabb { 0 } else { mask };
    let aabb_mask = if mask_aabb { mask } else { 0 };
    let f = if y <= small_cut {
        let t = square_y(y, &ye, zz_dr, arith);
        let w = if matches!(arith, Arith::Native) {
            ef(horner_native(&NSWC_A, t))
        } else {
            horner_mask(&NSWC_A, &ef(t), 0, 0, arith)
        };
        let inner = mul(&ye, &add(&ef(1.0), &w, arith), arith);
        dbl(&add(&ef(0.5), &sub(&ef(0.5), &inner, arith), arith))
    } else if y <= mid_cut {
        let (u, v) = if matches!(arith, Arith::Native) {
            (ef(horner_native(&P, y)), ef(horner_native(&Q, y)))
        } else {
            (
                horner_mask(&P, &ye, pqr_mask, 0, arith),
                horner_mask(&Q, &ye, pqr_mask, 7, arith),
            )
        };
        let t = if matches!(arith, Arith::Native) {
            ef((y - 3.75) / (y + 3.75))
        } else {
            let mut num = sub(&ye, &ef(3.75), arith);
            if pqr_mask & (1 << 23) != 0 {
                num = spill(num);
            }
            let mut den = add(&ye, &ef(3.75), arith);
            if pqr_mask & (1 << 24) != 0 {
                den = spill(den);
            }
            let mut tt = div(&num, &den, arith);
            if pqr_mask & (1 << 25) != 0 {
                tt = spill(tt);
            }
            tt
        };
        let acc = apply_ratio(u, v, arith, ratio);
        dbl(&horner_r(acc, t, pqr_mask, arith))
    } else {
        let xx = square_y(y, &ye, zz_dr, arith);
        let zv = if matches!(arith, Arith::Native) {
            ef(1.0 / (2.5 + xx))
        } else {
            div(&ef(1.0), &add(&ef(2.5), &ef(xx), arith), arith)
        };
        let t = if matches!(arith, Arith::Native) {
            ef(13.0 * dbl(&zv) - 1.0)
        } else {
            sub(&mul(&ef(13.0), &zv, arith), &ef(1.0), arith)
        };
        let (u, v) = if matches!(arith, Arith::Native) {
            (
                ef(horner_native(&AA, dbl(&zv))),
                ef(horner_native(&BB, dbl(&zv))),
            )
        } else {
            (
                horner_mask(&AA, &zv, aabb_mask, 0, arith),
                horner_mask(&BB, &zv, aabb_mask, 8, arith),
            )
        };
        let mut acc = apply_ratio(u, v, arith, ratio);
        acc = add(&mul(&acc, &t, arith), &ef(E2), arith);
        acc = add(&mul(&acc, &t, arith), &ef(E1), arith);
        acc = add(&mul(&acc, &t, arith), &ef(E0), arith);
        dbl(&div(&acc, &ye, arith))
    };
    let zz = square_y(y, &ye, zz_dr, arith);
    let q = if y <= small_cut {
        flush(f)
    } else {
        flush(excel_exp(-zz) * f)
    };
    flush(if z < 0.0 { 2.0 - q } else { q })
}

fn cody_body(z: f64, mask: u32, arith: Arith, zz_dr: bool, ratio: Ratio) -> f64 {
    let y = z.abs();
    let ye = ef(y);
    let f = if y <= 4.0 {
        if matches!(arith, Arith::Native) {
            let mut xnum = CODY_C[8] * y;
            let mut xden = y;
            for i in 0..7 {
                xnum = (xnum + CODY_C[i]) * y;
                xden = (xden + CODY_D[i]) * y;
            }
            (xnum + CODY_C[7]) / (xden + CODY_D[7])
        } else {
            let num = horner_mask(&CODY_C, &ye, mask, 0, arith);
            let den = horner_mask(&CODY_D, &ye, mask, 8, arith);
            dbl(&apply_ratio(num, den, arith, ratio))
        }
    } else {
        CODY_SQRPI / y
    };
    let zz = square_y(y, &ye, zz_dr, arith);
    flush(excel_exp(-zz) * f)
}

fn nswc_pqr_native_coeffs(z: f64, p: &[f64], q: &[f64], r: &[f64], small_cut: f64) -> f64 {
    let y = z.abs();
    if y <= small_cut {
        return flush(nswc_body(
            z,
            0,
            Arith::Native,
            false,
            Ratio::Cont,
            small_cut,
            4.0,
            false,
        ));
    }
    let u = horner_native(p, y);
    let v = horner_native(q, y);
    let t = (y - 3.75) / (y + 3.75);
    let mut acc = u / v;
    for &c in r.iter().rev() {
        acc = acc * t + c;
    }
    let qv = flush(excel_exp(-(y * y)) * acc);
    flush(if z < 0.0 { 2.0 - qv } else { qv })
}

#[derive(Clone, Serialize, Deserialize)]
struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
}

impl Acc {
    fn add(&mut self, d: u64) {
        self.n += 1;
        if d == 0 {
            self.exact += 1;
        } else {
            self.max_ulp = self.max_ulp.max(d);
        }
    }
}

fn score_rows(rows: &[(f64, u64)], eval: impl Fn(f64) -> f64 + Sync) -> (Acc, Acc, Acc, u32) {
    let mut small = Acc {
        exact: 0,
        n: 0,
        max_ulp: 0,
    };
    let mut mid = Acc {
        exact: 0,
        n: 0,
        max_ulp: 0,
    };
    let mut tail = Acc {
        exact: 0,
        n: 0,
        max_ulp: 0,
    };
    let mut pins = 0u32;
    for &(z, exp) in rows {
        let got = eval(z);
        let d = ulp_distance(got, f64::from_bits(exp)).unwrap_or(u64::MAX);
        if z < 0.5 {
            small.add(d);
        } else if z < 4.0 {
            mid.add(d);
        } else {
            tail.add(d);
        }
        if PIN_Z.iter().any(|&p| p == z) && d == 0 {
            pins += 1;
        }
    }
    (small, mid, tail, pins)
}

fn score_mid(mid: &[(f64, u64)], eval: impl Fn(f64) -> f64 + Sync) -> (usize, u64, u32) {
    let mut exact = 0usize;
    let mut maxu = 0u64;
    let mut pins = 0u32;
    for &(z, exp) in mid {
        let got = eval(z);
        let d = ulp_distance(got, f64::from_bits(exp)).unwrap_or(u64::MAX);
        if d == 0 {
            exact += 1;
        } else {
            maxu = maxu.max(d);
        }
        if PIN_Z.iter().any(|&p| p == z) && d == 0 {
            pins += 1;
        }
    }
    (exact, maxu, pins)
}

fn load_rows(dir: &str) -> Vec<(f64, u64)> {
    let mut rows = BTreeMap::new();
    for name in ERFC_BANKS {
        assert!(!name.contains("heldout"));
        let path = format!("{dir}/{name}");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet = serde_json::from_str(&text).expect(&path);
        for w in &bank.witnesses {
            let z = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("z"),
                _ => continue,
            };
            let Some(q) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if z.is_finite() && z >= 0.0 {
                rows.entry(z.to_bits()).or_insert(q.to_bits());
            }
        }
    }
    let path = format!("{dir}/answers-b24-normref.json");
    if let Ok(text) = fs::read_to_string(&path) {
        let bank: WitnessSet = serde_json::from_str(&text).expect("normref");
        for w in &bank.witnesses {
            if w.args.len() < 2 {
                continue;
            }
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("x"),
                _ => continue,
            };
            let cum = match &w.args[1] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).expect("c"),
                _ => continue,
            };
            if cum != 1.0 || !x.is_sign_negative() {
                continue;
            }
            let Some(ns) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            let z = x.abs() * FRAC_1_SQRT_2;
            rows.entry(z.to_bits()).or_insert((ns * 2.0).to_bits());
        }
    }
    rows.into_iter()
        .map(|(z, q)| (f64::from_bits(z), q))
        .collect()
}

#[derive(Serialize, Deserialize, Clone)]
struct Checkpoint {
    progress: BTreeMap<String, u32>,
    best_exact: usize,
    #[serde(default)]
    best_mid_exact: usize,
    #[serde(default = "u64_max")]
    best_mid_max_ulp: u64,
    #[serde(default)]
    best_pins: u32,
    best_label: String,
    #[serde(default)]
    best_mid_label: String,
    configs_done: u64,
    started_unix: u64,
    random_stream: u64,
}

fn u64_max() -> u64 {
    u64::MAX
}

#[derive(Serialize, Deserialize)]
struct StatusJson {
    region: String,
    chunk: String,
    configs_done: u64,
    best_exact: usize,
    best_mid_exact: usize,
    best_mid_max_ulp: u64,
    best_pins: u32,
    best_label: String,
    best_mid_label: String,
    named_mid_bar: usize,
    runtime_secs: u64,
    max_hours: f64,
    threads: usize,
    stop_requested: bool,
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn write_atomic(path: &Path, text: &str) {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, text).expect("write tmp");
    fs::rename(&tmp, path).expect("rename");
}

fn region_map_md(ck: &Checkpoint, jobs: &[CubeJob]) -> String {
    let mut s = String::from(
        "# ERFC campaign REGION_MAP\n\n\
         26-bit PQR+t cubes: bits 0-6 P, 7-13 Q, 14-22 R, 23-25 t-formation spills.\n\
         Native / every-op-spill / all-stage-spill are single configs (mask-insensitive).\n\
         First cube is R1m/z0/r0 = x87-DR + store uv + mid_cut 1.5 (assoc-race bar axes).\n\n\
         | id | space | next | note |\n|---|---|---|---|\n",
    );
    let r0 = ck.progress.get("R0").copied().unwrap_or(0);
    s.push_str(&format!(
        "| R0 | named F + implied-F | {} | baselines |\n",
        if r0 >= 1 { "done" } else { "pending" }
    ));
    let r0c = ck.progress.get("R0c").copied().unwrap_or(0);
    s.push_str(&format!(
        "| R0c | NSWC P/Q/R ±1 ULP | {} | last-bit decimals |\n",
        if r0c >= 1 { "done" } else { "pending" }
    ));
    let r1b = ck.progress.get("R1base").copied().unwrap_or(0);
    s.push_str(&format!(
        "| R1base | mask-insensitive arith | {} | Native/Every/Pc53/Stage |\n",
        if r1b >= 1 { "done" } else { "pending" }
    ));
    for j in jobs {
        let v = ck.progress.get(&j.axis).copied().unwrap_or(0);
        let next = if v >= j.mask_lim {
            "done".to_string()
        } else {
            format!("0x{v:07x}/0x{:07x}", j.mask_lim)
        };
        s.push_str(&format!(
            "| {} | 0..0x{:07x} | {next} | {} |\n",
            j.axis, j.mask_lim, j.note
        ));
    }
    s.push_str(&format!(
        "\nbest_mid: {} / bar {NAMED_MID_BAR}  `{}`  max_ulp={}  pins={}\nbest_all: {}  `{}`\n",
        ck.best_mid_exact,
        ck.best_mid_label,
        ck.best_mid_max_ulp,
        ck.best_pins,
        ck.best_exact,
        ck.best_label
    ));
    s
}

fn write_status(
    out: &Path,
    ck: &Checkpoint,
    jobs: &[CubeJob],
    region: &str,
    chunk: &str,
    runtime_secs: u64,
    max_hours: f64,
    threads: usize,
    stop: bool,
    extra: &str,
) {
    let hours = runtime_secs as f64 / 3600.0;
    let map = region_map_md(ck, jobs);
    let md = format!(
        "# ERFC campaign STATUS\n\n\
         - region: `{region}`\n\
         - chunk: `{chunk}`\n\
         - configs_done: {}\n\
         - best_exact: {}  `{best}`\n\
         - best_mid: {} / bar {NAMED_MID_BAR}  max_ulp={}  `{midlab}`\n\
         - best_pins: {}\n\
         - runtime_hours: {hours:.2} / {max_hours}\n\
         - threads: {threads}\n\
         - stop_requested: {stop}\n\
         - progress_keys: {}\n\n\
         Stop: `touch STOP` in this directory. Resume: same command.\n\
         Alert files: `HIT_ALL_MID` (all mid rows exact), `HIT_PIN` (a pin went exact).\n\n\
         {extra}\n\n{map}\n",
        ck.configs_done,
        ck.best_exact,
        ck.best_mid_exact,
        ck.best_mid_max_ulp,
        ck.best_pins,
        ck.progress.len(),
        best = ck.best_label,
        midlab = ck.best_mid_label,
    );
    write_atomic(&out.join("STATUS.md"), &md);
    write_atomic(&out.join("REGION_MAP.md"), &map);
    let sj = StatusJson {
        region: region.into(),
        chunk: chunk.into(),
        configs_done: ck.configs_done,
        best_exact: ck.best_exact,
        best_mid_exact: ck.best_mid_exact,
        best_mid_max_ulp: ck.best_mid_max_ulp,
        best_pins: ck.best_pins,
        best_label: ck.best_label.clone(),
        best_mid_label: ck.best_mid_label.clone(),
        named_mid_bar: NAMED_MID_BAR,
        runtime_secs,
        max_hours,
        threads,
        stop_requested: stop,
    };
    write_atomic(
        &out.join("status.json"),
        &serde_json::to_string_pretty(&sj).unwrap(),
    );
    write_atomic(
        &out.join("checkpoint.json"),
        &serde_json::to_string_pretty(ck).unwrap(),
    );
}

fn load_ckpt(out: &Path) -> Checkpoint {
    let p = out.join("checkpoint.json");
    if let Ok(t) = fs::read_to_string(&p) {
        if let Ok(c) = serde_json::from_str(&t) {
            return c;
        }
    }
    Checkpoint {
        progress: BTreeMap::new(),
        best_exact: 0,
        best_mid_exact: 0,
        best_mid_max_ulp: u64::MAX,
        best_pins: 0,
        best_label: String::new(),
        best_mid_label: String::new(),
        configs_done: 0,
        started_unix: now_unix(),
        random_stream: 0xC0FFEE,
    }
}

fn wall_secs(ck: &Checkpoint) -> u64 {
    now_unix().saturating_sub(ck.started_unix)
}

fn timed_out(ck: &Checkpoint, max_hours: f64) -> bool {
    wall_secs(ck) as f64 / 3600.0 >= max_hours
}

fn stop_requested(out: &Path) -> bool {
    out.join("STOP").exists()
}

fn log_leader(out: &Path, exact: usize, max_ulp: u64, pins: u32, label: &str, detail: &str) {
    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(out.join("leaders.jsonl"))
        .and_then(|mut f| {
            writeln!(
                f,
                "{{\"exact\":{exact},\"max_ulp\":{max_ulp},\"pins\":{pins},\"label\":{},\"detail\":{}}}",
                serde_json::to_string(label).unwrap(),
                serde_json::to_string(detail).unwrap()
            )
        });
}

fn consider_all(ck: &mut Checkpoint, out: &Path, label: &str, exact: usize, detail: &str) {
    if exact > ck.best_exact {
        ck.best_exact = exact;
        ck.best_label = label.to_string();
        log_leader(out, exact, 0, 0, label, detail);
    }
}

fn consider_mid(
    ck: &mut Checkpoint,
    out: &Path,
    label: &str,
    exact: usize,
    max_ulp: u64,
    pins: u32,
    mid_n: usize,
    detail: &str,
) {
    let beat =
        exact > ck.best_mid_exact || (exact == ck.best_mid_exact && max_ulp < ck.best_mid_max_ulp);
    if beat {
        ck.best_mid_exact = exact;
        ck.best_mid_max_ulp = max_ulp;
        ck.best_mid_label = label.to_string();
        log_leader(out, exact, max_ulp, pins, label, detail);
    }
    if pins > 0 {
        let _ = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(out.join("pin-hits.jsonl"))
            .and_then(|mut f| {
                writeln!(
                    f,
                    "{{\"pins\":{pins},\"exact\":{exact},\"max_ulp\":{max_ulp},\"label\":{}}}",
                    serde_json::to_string(label).unwrap()
                )
            });
    }
    if pins > ck.best_pins {
        ck.best_pins = pins;
        log_leader(out, exact, max_ulp, pins, label, "pin-best");
        if exact >= NAMED_MID_BAR {
            let _ = fs::write(
                out.join("HIT_PIN"),
                format!("{label} pins={pins} exact={exact} max_ulp={max_ulp}\n"),
            );
        }
    }
    if exact == mid_n && mid_n > 0 {
        let _ = fs::write(
            out.join("HIT_ALL_MID"),
            format!("{label} exact={exact}/{mid_n} max_ulp={max_ulp} pins={pins}\n"),
        );
    }
}

fn parse_args() -> (String, PathBuf, usize, f64) {
    let mut dir = "../../work/w109/G3-01-dist".to_string();
    let mut out = PathBuf::from("../../work/w109/erfc-campaign");
    let mut threads = 12usize;
    let mut max_hours = 96.0;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--dir" => dir = it.next().expect("--dir"),
            "--out" => out = PathBuf::from(it.next().expect("--out")),
            "--threads" => threads = it.next().unwrap().parse().unwrap(),
            "--max-hours" => max_hours = it.next().unwrap().parse().unwrap(),
            _ => {}
        }
    }
    assert!(!dir.contains("heldout"));
    (dir, out, threads, max_hours)
}

#[derive(Clone, Copy, Debug)]
enum CubeKind {
    NswcPqr,
    NswcAaBb,
    Cody,
}

#[derive(Clone, Debug)]
struct CubeJob {
    axis: String,
    kind: CubeKind,
    zz_dr: bool,
    ratio: Ratio,
    small_cut: f64,
    mid_cut: f64,
    pc53: bool,
    mask_lim: u32,
    note: String,
}

fn cube_jobs() -> Vec<CubeJob> {
    // Store + x87-DR first: that is the assoc-race bar. A few-hour look-in
    // spends its time on the most promising 26-bit cube.
    let ratios = [Ratio::Store, Ratio::Cont, Ratio::Recip];
    let zz = [true, false];
    let mut jobs = Vec::new();
    // R1m: winning cuts 0.5/1.5, 26-bit PQR+t mask below 1.5, AA/BB above.
    for (zi, zz_dr) in zz.iter().enumerate() {
        for (ri, ratio) in ratios.iter().enumerate() {
            jobs.push(CubeJob {
                axis: format!("R1m/z{zi}/r{ri}"),
                kind: CubeKind::NswcPqr,
                zz_dr: *zz_dr,
                ratio: *ratio,
                small_cut: 0.5,
                mid_cut: 1.5,
                pc53: false,
                mask_lim: PQR_MASK_LIM,
                note: format!(
                    "NSWC 26-bit PQR+t mask, mid_cut=1.5 AA/BB above, zz_dr={zz_dr} {}",
                    ratio.tag()
                ),
            });
        }
    }
    // R1: same 26-bit mask, PQR on the whole mid band.
    for (zi, zz_dr) in zz.iter().enumerate() {
        for (ri, ratio) in ratios.iter().enumerate() {
            jobs.push(CubeJob {
                axis: format!("R1/z{zi}/r{ri}"),
                kind: CubeKind::NswcPqr,
                zz_dr: *zz_dr,
                ratio: *ratio,
                small_cut: 0.5,
                mid_cut: 4.0,
                pc53: false,
                mask_lim: PQR_MASK_LIM,
                note: format!(
                    "NSWC 26-bit PQR+t mask, PQR on [0.5,4), zz_dr={zz_dr} {}",
                    ratio.tag()
                ),
            });
        }
    }
    // R4: AA/BB 19-bit with mid_cut=1.5.
    for (zi, zz_dr) in zz.iter().enumerate() {
        for (ri, ratio) in ratios.iter().enumerate() {
            jobs.push(CubeJob {
                axis: format!("R4/z{zi}/r{ri}"),
                kind: CubeKind::NswcAaBb,
                zz_dr: *zz_dr,
                ratio: *ratio,
                small_cut: 0.5,
                mid_cut: 1.5,
                pc53: false,
                mask_lim: AABB_MASK_LIM,
                note: format!(
                    "NSWC AA/BB 19-bit store-mask, mid_cut=1.5, zz_dr={zz_dr} {}",
                    ratio.tag()
                ),
            });
        }
    }
    // R2: Cody C/D 16-bit.
    for (zi, zz_dr) in zz.iter().enumerate() {
        for (ri, ratio) in ratios.iter().enumerate() {
            jobs.push(CubeJob {
                axis: format!("R2/z{zi}/r{ri}"),
                kind: CubeKind::Cody,
                zz_dr: *zz_dr,
                ratio: *ratio,
                small_cut: 0.5,
                mid_cut: 4.0,
                pc53: false,
                mask_lim: CODY_MASK_LIM,
                note: format!("Cody C/D 16-bit store-mask, zz_dr={zz_dr} {}", ratio.tag()),
            });
        }
    }
    jobs.push(CubeJob {
        axis: "R1p/mid15".into(),
        kind: CubeKind::NswcPqr,
        zz_dr: true,
        ratio: Ratio::Store,
        small_cut: 0.5,
        mid_cut: 1.5,
        pc53: true,
        mask_lim: PQR_MASK_LIM,
        note: "NSWC 26-bit PQR+t, X87Pc53, zz_dr, uv store, mid_cut=1.5".into(),
    });
    jobs
}

fn eval_job(j: &CubeJob, mask: u32, z: f64) -> f64 {
    let arith = if j.pc53 {
        Arith::X87Pc53
    } else {
        Arith::X87Cont
    };
    match j.kind {
        CubeKind::NswcPqr => nswc_body(
            z,
            mask,
            arith,
            j.zz_dr,
            j.ratio,
            j.small_cut,
            j.mid_cut,
            false,
        ),
        CubeKind::NswcAaBb => nswc_body(
            z,
            mask,
            arith,
            j.zz_dr,
            j.ratio,
            j.small_cut,
            j.mid_cut,
            true,
        ),
        CubeKind::Cody => cody_body(z, mask, arith, j.zz_dr, j.ratio),
    }
}

fn run_cube(
    ck: &mut Checkpoint,
    out: &Path,
    jobs: &[CubeJob],
    job: &CubeJob,
    mid: &[(f64, u64)],
    max_hours: f64,
    threads: usize,
) -> bool {
    let mut c0 = ck.progress.get(&job.axis).copied().unwrap_or(0);
    while c0 < job.mask_lim {
        if timed_out(ck, max_hours) || stop_requested(out) {
            write_status(
                out,
                ck,
                jobs,
                &job.axis,
                &format!("stop@{c0:07x}"),
                wall_secs(ck),
                max_hours,
                threads,
                stop_requested(out),
                &job.note,
            );
            return true;
        }
        let c1 = (c0 + CHUNK).min(job.mask_lim);
        let scores: Vec<(u32, usize, u64, u32)> = (c0..c1)
            .into_par_iter()
            .map(|mask| {
                let (exact, maxu, pins) = score_mid(mid, |z| eval_job(job, mask, z));
                (mask, exact, maxu, pins)
            })
            .collect();
        ck.configs_done += scores.len() as u64;
        for (mask, exact, maxu, pins) in &scores {
            consider_mid(
                ck,
                out,
                &format!("{} /mask={mask:07x}", job.axis),
                *exact,
                *maxu,
                *pins,
                mid.len(),
                &job.note,
            );
        }
        c0 = c1;
        ck.progress.insert(job.axis.clone(), c0);
        let chunk_best = scores.iter().map(|s| s.1).max().unwrap_or(0);
        write_status(
            out,
            ck,
            jobs,
            &job.axis,
            &format!("{c0:07x}"),
            wall_secs(ck),
            max_hours,
            threads,
            false,
            &format!(
                "{}\nlast chunk next=0x{c0:07x} chunk_best_mid={chunk_best}\n",
                job.note
            ),
        );
    }
    false
}

fn implied_f_note(rows: &[(f64, u64)]) -> String {
    let mut s = String::from("## R0 implied-F (Q / excel_exp(−z²) vs NSWC/Cody F)\n\n");
    for &pz in &PIN_Z {
        let Some(&(z, qbits)) = rows.iter().find(|(z, _)| *z == pz) else {
            s.push_str(&format!("- pin {pz}: not in discovery rows\n"));
            continue;
        };
        let q = f64::from_bits(qbits);
        let w = excel_exp(-(z * z));
        let f_or = q / w;
        let f_nswc = {
            let y = z;
            let u = horner_native(&P, y);
            let v = horner_native(&Q, y);
            let t = (y - 3.75) / (y + 3.75);
            let mut acc = u / v;
            for &c in R.iter().rev() {
                acc = acc * t + c;
            }
            acc
        };
        let f_cody = {
            let mut xnum = CODY_C[8] * z;
            let mut xden = z;
            for i in 0..7 {
                xnum = (xnum + CODY_C[i]) * z;
                xden = (xden + CODY_D[i]) * z;
            }
            (xnum + CODY_C[7]) / (xden + CODY_D[7])
        };
        let d_n = ulp_distance(f_nswc, f_or).unwrap_or(u64::MAX);
        let d_c = ulp_distance(f_cody, f_or).unwrap_or(u64::MAX);
        s.push_str(&format!(
            "- z={z}: F_or={f_or:.16e}  NSWC ulp={d_n}  Cody ulp={d_c}\n"
        ));
    }
    s
}

fn main() {
    let (dir, out, threads, max_hours) = parse_args();
    fs::create_dir_all(&out).unwrap();
    let _ = fs::write(out.join("MONITOR.md"), MONITOR);
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .ok();
    let rows = load_rows(&dir);
    assert!(!rows.is_empty(), "no discovery rows loaded from {dir}");
    let mid: Vec<(f64, u64)> = rows
        .iter()
        .copied()
        .filter(|(z, _)| *z >= 0.5 && *z < 4.0)
        .collect();
    let jobs = cube_jobs();
    let mut ck = load_ckpt(&out);
    if ck.started_unix == 0 {
        ck.started_unix = now_unix();
    }

    write_status(
        &out,
        &ck,
        &jobs,
        "init",
        "-",
        wall_secs(&ck),
        max_hours,
        threads,
        false,
        &format!("rows={} mid={}", rows.len(), mid.len()),
    );

    // ---- R0 named F + implied-F ----
    if ck.progress.get("R0").copied().unwrap_or(0) == 0 {
        let mut extra = String::from("## R0 named F (full discovery, not a landing)\n\n");
        let graphs: [(&str, fn(f64) -> f64); 4] = [
            ("libm::erfc", |z| flush(libm::erfc(z))),
            ("production", |z| {
                flush(erfc_precise_kernel(z).unwrap_or(f64::NAN))
            }),
            ("nswc_derfc0_native", |z| {
                nswc_body(z, 0, Arith::Native, false, Ratio::Cont, 0.5, 4.0, false)
            }),
            ("cody_unsplit", |z| {
                cody_body(z, 0, Arith::Native, false, Ratio::Cont)
            }),
        ];
        for (name, f) in graphs {
            let (s, m, t, pins) = score_rows(&rows, f);
            extra.push_str(&format!(
                "- {name}: small {}/{} mid {}/{} tail {}/{} pins {pins} max_ulp mid/tail {}/{}\n",
                s.exact, s.n, m.exact, m.n, t.exact, t.n, m.max_ulp, t.max_ulp
            ));
            consider_all(&mut ck, &out, name, s.exact + m.exact + t.exact, &extra);
            consider_mid(
                &mut ck,
                &out,
                name,
                m.exact,
                m.max_ulp,
                pins,
                mid.len(),
                name,
            );
        }
        // Sanity: the assoc-race bar cfg must land near 3332/7741 mid.
        {
            let name = "nswc_x87cont_zzdr_store_mid15";
            let (s, m, t, pins) = score_rows(&rows, |z| {
                nswc_body(z, 0, Arith::X87Cont, true, Ratio::Store, 0.5, 1.5, false)
            });
            extra.push_str(&format!(
                "- {name}: small {}/{} mid {}/{} tail {}/{} pins {pins} (assoc-race bar is mid {NAMED_MID_BAR}/7741)\n",
                s.exact, s.n, m.exact, m.n, t.exact, t.n
            ));
            consider_all(&mut ck, &out, name, s.exact + m.exact + t.exact, name);
            consider_mid(
                &mut ck,
                &out,
                name,
                m.exact,
                m.max_ulp,
                pins,
                mid.len(),
                name,
            );
        }
        extra.push('\n');
        extra.push_str(&implied_f_note(&rows));
        write_atomic(&out.join("R0.md"), &extra);
        ck.progress.insert("R0".into(), 1);
        write_status(
            &out,
            &ck,
            &jobs,
            "R0",
            "done",
            wall_secs(&ck),
            max_hours,
            threads,
            false,
            &extra,
        );
    }

    // ---- R0c NSWC P/Q/R ±1 ULP (Cody last-bit already noise on this corpus) ----
    if ck.progress.get("R0c").copied().unwrap_or(0) == 0
        && !timed_out(&ck, max_hours)
        && !stop_requested(&out)
    {
        let mut extra = String::from("## R0c NSWC P/Q/R ±1 ULP native unsplit exp\n\n");
        let poke = |cs: &[f64], i: usize, up: bool| -> Vec<f64> {
            let mut v = cs.to_vec();
            v[i] = if up {
                cs[i].next_up()
            } else {
                cs[i].next_down()
            };
            v
        };
        for (set, n) in [("P", P.len()), ("Q", Q.len()), ("R", R.len())] {
            for i in 0..n {
                for up in [true, false] {
                    let (p, q, r) = match set {
                        "P" => (poke(&P, i, up), Q.to_vec(), R.to_vec()),
                        "Q" => (P.to_vec(), poke(&Q, i, up), R.to_vec()),
                        _ => (P.to_vec(), Q.to_vec(), poke(&R, i, up)),
                    };
                    let label = format!("R0c/{set}[{i}]/{}", if up { "up" } else { "dn" });
                    let (exact, maxu, pins) =
                        score_mid(&mid, |z| nswc_pqr_native_coeffs(z, &p, &q, &r, 0.5));
                    extra.push_str(&format!(
                        "- {label}: mid {exact}/{} max_ulp={maxu} pins={pins}\n",
                        mid.len()
                    ));
                    consider_mid(&mut ck, &out, &label, exact, maxu, pins, mid.len(), "R0c");
                    ck.configs_done += 1;
                }
            }
        }
        write_atomic(&out.join("R0c.md"), &extra);
        ck.progress.insert("R0c".into(), 1);
        write_status(
            &out,
            &ck,
            &jobs,
            "R0c",
            "done",
            wall_secs(&ck),
            max_hours,
            threads,
            false,
            &extra,
        );
    }

    // ---- R1base: mask-insensitive arith (one config per axis) ----
    if ck.progress.get("R1base").copied().unwrap_or(0) == 0
        && !timed_out(&ck, max_hours)
        && !stop_requested(&out)
    {
        let mut extra = String::from("## R1base mask-insensitive\n\n");
        let ariths = [
            Arith::Native,
            Arith::X87Every,
            Arith::X87Pc53,
            Arith::X87Stage,
        ];
        for arith in ariths {
            for zz_dr in [false, true] {
                for ratio in [Ratio::Cont, Ratio::Store, Ratio::Recip] {
                    for (mid_cut, tag) in [(4.0, "mid4"), (1.5, "mid15")] {
                        let label = format!("R1base/{arith:?}/zz{zz_dr}/{} /{tag}", ratio.tag());
                        let (exact, maxu, pins) = score_mid(&mid, |z| {
                            nswc_body(z, 0, arith, zz_dr, ratio, 0.5, mid_cut, false)
                        });
                        extra.push_str(&format!(
                            "- {label}: mid {exact}/{} max_ulp={maxu} pins={pins}\n",
                            mid.len()
                        ));
                        consider_mid(
                            &mut ck,
                            &out,
                            &label,
                            exact,
                            maxu,
                            pins,
                            mid.len(),
                            "R1base",
                        );
                        ck.configs_done += 1;
                    }
                }
            }
        }
        write_atomic(&out.join("R1base.md"), &extra);
        ck.progress.insert("R1base".into(), 1);
        write_status(
            &out,
            &ck,
            &jobs,
            "R1base",
            "done",
            wall_secs(&ck),
            max_hours,
            threads,
            false,
            &extra,
        );
    }

    for job in &jobs {
        if timed_out(&ck, max_hours) || stop_requested(&out) {
            break;
        }
        if run_cube(&mut ck, &out, &jobs, job, &mid, max_hours, threads) {
            break;
        }
    }

    let exhausted = jobs
        .iter()
        .all(|j| ck.progress.get(&j.axis).copied().unwrap_or(0) >= j.mask_lim);
    write_status(
        &out,
        &ck,
        &jobs,
        if exhausted { "exit-regions" } else { "exit" },
        "-",
        wall_secs(&ck),
        max_hours,
        threads,
        stop_requested(&out),
        if exhausted {
            "all named cubes finished before max-hours. not filling with random 16-bit (already enumerated). resume is a no-op unless you raise max-hours after adding regions."
        } else {
            "campaign process exiting (time, STOP, or interrupt). resume = same command."
        },
    );
}
