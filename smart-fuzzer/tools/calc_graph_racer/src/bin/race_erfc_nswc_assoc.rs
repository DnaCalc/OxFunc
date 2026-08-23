//! W109 ERFC body test 1: NSWC DERFC0 Horner association / store-site.
//!
//! Coefficients verbatim from NSWC TR 92/425 DERFC/DERFC0 (Morris 1993,
//! public NSWC library). Arithmetic axes match `race_erf_precise_public_small`.
//! Frozen discovery only; heldouts are not named.
//!
//! Usage (from this crate):
//!   cargo run --release --bin race_erfc_nswc_assoc -- ../../work/w109/G3-01-dist

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    CW_PC53_RN, CW_PC64_RN, Ext80, excel_exp, ext_add, ext_div, ext_from_f64, ext_mul, ext_sub,
    ext_to_f64,
};
use std::collections::BTreeMap;

const FRAC_1_SQRT_2: f64 = f64::from_bits(0x3fe6a09e667f3bcd);
const ERFC_BANKS: [&str; 5] = [
    "answers-erfcp.json",
    "answers-erfcm.json",
    "answers-b7erfc.json",
    "answers-b8erfc.json",
    "answers-b11c.json",
];
const PIN_Z: [f64; 5] = [0.75, 1.28125, 1.875, 2.125, 5.0];

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
    0.145589721275038539045668824025e+00,
    -0.273421931495426482902320421863e+00,
    0.226008066916621506788789064272e+00,
    -0.163571895523923805648814425592e+00,
    0.102604312032193978662297299832e+00,
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
const CC: [f64; 9] = [
    -0.7040906288250128001000086e-04,
    -0.3858822461760510359506941e-02,
    -0.7708202127512212359395078e-01,
    -0.6713655014557429480440263e+00,
    -0.2081992124162995545731882e+01,
    0.2898831421475282558867888e+01,
    0.2199509380600429331650192e+02,
    0.2907064664404115316722996e+01,
    -0.4766208741588182425380950e+02,
];
const DD: [f64; 10] = [
    1.0,
    0.5238852785508439144747174e+02,
    0.9646843357714742409535148e+03,
    0.7007152775135939601804416e+04,
    0.8515386792259821780601162e+04,
    -0.1002360095177164564992134e+06,
    -0.2065250031331232815791912e+06,
    0.5695324805290370358175984e+06,
    0.6589752493461331195697873e+06,
    -0.1192930193156561957631462e+07,
];
const E0: f64 = 0.540464821348814822409610122136e+00;
const E1: f64 = -0.261515522487415653487049835220e-01;
const E2: f64 = -0.288573438386338758794591212600e-02;
const E3: f64 = -0.529353396945788057720258856000e-03;
const S: [f64; 12] = [
    1.0,
    -0.5,
    0.75,
    -1.875,
    6.5625,
    -29.53125,
    162.421875,
    -1055.7421875,
    7918.06640625,
    -67303.564453125,
    639383.8623046875,
    -6713530.55419921875,
];
const S11: f64 = 77205601.373291015625;
const RPINV: f64 = 0.56418958354775628694807945156077259;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Arithmetic {
    X87Continuous,
    X87EveryOp53,
    X87HornerStage53,
    X87Pc53,
    Native53,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TMode {
    Native53,
    X87DoubleRounded,
    X87Pc53,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ratio {
    Continuous,
    StoreUv,
    UTimesInvV,
}

#[derive(Clone, Copy, Debug)]
struct Cfg {
    arith: Arithmetic,
    zz: TMode,
    ratio: Ratio,
    small_cut: f64,
    mid_cut: f64,
    far_cut: f64,
    use_excel_exp: bool,
}

impl Cfg {
    fn name(self) -> String {
        format!(
            "{:?}/{:?}/{:?}/cuts={:.3},{:.1},{:.1}/{}",
            self.arith,
            self.zz,
            self.ratio,
            self.small_cut,
            self.mid_cut,
            self.far_cut,
            if self.use_excel_exp { "xlexp" } else { "hostexp" }
        )
    }
}

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn dbl64(x: &Ext80) -> f64 {
    ext_to_f64(x, CW_PC64_RN)
}
fn spill(x: Ext80) -> Ext80 {
    ef(dbl64(&x))
}
fn flush(v: f64) -> f64 {
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn add(a: &Ext80, b: &Ext80, mode: Arithmetic) -> Ext80 {
    match mode {
        Arithmetic::X87Continuous | Arithmetic::X87HornerStage53 => ext_add(a, b, CW_PC64_RN),
        Arithmetic::X87EveryOp53 => spill(ext_add(a, b, CW_PC64_RN)),
        Arithmetic::X87Pc53 => ext_add(a, b, CW_PC53_RN),
        Arithmetic::Native53 => ef(dbl64(a) + dbl64(b)),
    }
}
fn sub(a: &Ext80, b: &Ext80, mode: Arithmetic) -> Ext80 {
    match mode {
        Arithmetic::X87Continuous | Arithmetic::X87HornerStage53 => ext_sub(a, b, CW_PC64_RN),
        Arithmetic::X87EveryOp53 => spill(ext_sub(a, b, CW_PC64_RN)),
        Arithmetic::X87Pc53 => ext_sub(a, b, CW_PC53_RN),
        Arithmetic::Native53 => ef(dbl64(a) - dbl64(b)),
    }
}
fn mul(a: &Ext80, b: &Ext80, mode: Arithmetic) -> Ext80 {
    match mode {
        Arithmetic::X87Continuous | Arithmetic::X87HornerStage53 => ext_mul(a, b, CW_PC64_RN),
        Arithmetic::X87EveryOp53 => spill(ext_mul(a, b, CW_PC64_RN)),
        Arithmetic::X87Pc53 => ext_mul(a, b, CW_PC53_RN),
        Arithmetic::Native53 => ef(dbl64(a) * dbl64(b)),
    }
}
fn div(a: &Ext80, b: &Ext80, mode: Arithmetic) -> Ext80 {
    match mode {
        Arithmetic::X87Continuous | Arithmetic::X87HornerStage53 => ext_div(a, b, CW_PC64_RN),
        Arithmetic::X87EveryOp53 => spill(ext_div(a, b, CW_PC64_RN)),
        Arithmetic::X87Pc53 => ext_div(a, b, CW_PC53_RN),
        Arithmetic::Native53 => ef(dbl64(a) / dbl64(b)),
    }
}

fn horner_ext(cs: &[f64], x: &Ext80, mode: Arithmetic) -> Ext80 {
    let mut acc = ef(*cs.last().unwrap());
    for &c in cs.iter().rev().skip(1) {
        acc = add(&mul(&acc, x, mode), &ef(c), mode);
        if matches!(mode, Arithmetic::X87HornerStage53) {
            acc = spill(acc);
        }
    }
    acc
}

fn square(y: f64, mode: TMode) -> f64 {
    match mode {
        TMode::Native53 => y * y,
        TMode::X87DoubleRounded => dbl64(&spill(ext_mul(&ef(y), &ef(y), CW_PC64_RN))),
        TMode::X87Pc53 => dbl64(&ext_mul(&ef(y), &ef(y), CW_PC53_RN)),
    }
}

fn uv_then_r(u: Ext80, v: Ext80, t: Ext80, r: &[f64], cfg: Cfg) -> Ext80 {
    let mut acc = match cfg.ratio {
        Ratio::Continuous => div(&u, &v, cfg.arith),
        Ratio::StoreUv => spill(div(&u, &v, cfg.arith)),
        Ratio::UTimesInvV => mul(&u, &div(&ef(1.0), &v, cfg.arith), cfg.arith),
    };
    for &c in r.iter().rev() {
        acc = add(&mul(&acc, &t, cfg.arith), &ef(c), cfg.arith);
        if matches!(cfg.arith, Arithmetic::X87HornerStage53) {
            acc = spill(acc);
        }
    }
    acc
}

fn derfc0_ext(x: f64, cfg: Cfg) -> Ext80 {
    let xe = ef(x);
    let mode = cfg.arith;
    if x <= cfg.mid_cut {
        let u = horner_ext(&P, &xe, mode);
        let v = horner_ext(&Q, &xe, mode);
        let t = div(
            &sub(&xe, &ef(3.75), mode),
            &add(&xe, &ef(3.75), mode),
            mode,
        );
        return uv_then_r(u, v, t, &R, cfg);
    }
    let xx = square(x, cfg.zz);
    let z = div(&ef(1.0), &add(&ef(2.5), &ef(xx), mode), mode);
    let t = sub(&mul(&ef(13.0), &z, mode), &ef(1.0), mode);
    if x <= cfg.far_cut {
        let u = horner_ext(&AA, &z, mode);
        let v = horner_ext(&BB, &z, mode);
        let mut acc = match cfg.ratio {
            Ratio::Continuous => div(&u, &v, mode),
            Ratio::StoreUv => spill(div(&u, &v, mode)),
            Ratio::UTimesInvV => mul(&u, &div(&ef(1.0), &v, mode), mode),
        };
        acc = add(&mul(&acc, &t, mode), &ef(E2), mode);
        acc = add(&mul(&acc, &t, mode), &ef(E1), mode);
        acc = add(&mul(&acc, &t, mode), &ef(E0), mode);
        return div(&acc, &xe, mode);
    }
    if x < 50.0 {
        let u = horner_ext(&CC, &z, mode);
        let v = horner_ext(&DD, &z, mode);
        let mut acc = match cfg.ratio {
            Ratio::Continuous => div(&u, &v, mode),
            Ratio::StoreUv => spill(div(&u, &v, mode)),
            Ratio::UTimesInvV => mul(&u, &div(&ef(1.0), &v, mode), mode),
        };
        acc = add(&mul(&acc, &t, mode), &ef(E3), mode);
        acc = add(&mul(&acc, &t, mode), &ef(E2), mode);
        acc = add(&mul(&acc, &t, mode), &ef(E1), mode);
        acc = add(&mul(&acc, &t, mode), &ef(E0), mode);
        return div(&acc, &xe, mode);
    }
    let rec = div(&ef(1.0), &xe, mode);
    let tt = mul(&rec, &rec, mode);
    let mut acc = ef(S11);
    for &s in S.iter().rev() {
        acc = add(&mul(&acc, &tt, mode), &ef(s), mode);
        if matches!(mode, Arithmetic::X87HornerStage53) {
            acc = spill(acc);
        }
    }
    mul(&ef(RPINV), &div(&acc, &xe, mode), mode)
}

fn nswc_eval(z: f64, cfg: Cfg) -> f64 {
    let y = z.abs();
    let q = if y <= cfg.small_cut {
        let t = square(y, cfg.zz);
        let w = horner_ext(&NSWC_A, &ef(t), cfg.arith);
        let inner = mul(&ef(y), &add(&ef(1.0), &w, cfg.arith), cfg.arith);
        let comp = add(&ef(0.5), &sub(&ef(0.5), &inner, cfg.arith), cfg.arith);
        dbl64(&comp)
    } else if y > 100.0 {
        0.0
    } else {
        let f = dbl64(&derfc0_ext(y, cfg));
        let zz = square(y, cfg.zz);
        let w = if cfg.use_excel_exp {
            excel_exp(-zz)
        } else {
            (-zz).exp()
        };
        flush(w * f)
    };
    flush(if z < 0.0 { 2.0 - q } else { q })
}

struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
    sum_ulp: u128,
}
impl Acc {
    fn new() -> Self {
        Self {
            exact: 0,
            n: 0,
            max_ulp: 0,
            sum_ulp: 0,
        }
    }
    fn add(&mut self, d: u64) {
        self.n += 1;
        if d == 0 {
            self.exact += 1;
        } else {
            self.max_ulp = self.max_ulp.max(d);
            self.sum_ulp += d as u128;
        }
    }
}

fn band_idx(z: f64) -> usize {
    if z < 0.5 {
        0
    } else if z < 4.0 {
        1
    } else {
        2
    }
}

fn score(rows: &[(f64, u64)], cfg: Cfg) -> ([Acc; 3], Acc) {
    let mut bands = [Acc::new(), Acc::new(), Acc::new()];
    let mut all = Acc::new();
    for &(z, expected) in rows {
        let got = nswc_eval(z, cfg);
        let d = ulp_distance(got, f64::from_bits(expected)).unwrap_or(u64::MAX);
        all.add(d);
        bands[band_idx(z)].add(d);
    }
    (bands, all)
}

fn fmt(a: &Acc) -> String {
    if a.n == 0 {
        return "—".into();
    }
    format!("{}/{} max={} sum={}", a.exact, a.n, a.max_ulp, a.sum_ulp)
}

fn load_rows(dir: &str) -> Vec<(f64, u64)> {
    let mut rows = BTreeMap::new();
    for name in ERFC_BANKS {
        let path = format!("{dir}/{name}");
        let Ok(text) = std::fs::read_to_string(&path) else {
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
    if let Ok(text) = std::fs::read_to_string(&path) {
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

fn print_row(label: &str, bands: &[Acc; 3], all: &Acc) {
    println!(
        "{:<72} {:>22} {:>22} {:>22} {:>22}",
        label,
        fmt(&bands[0]),
        fmt(&bands[1]),
        fmt(&bands[2]),
        fmt(all)
    );
}

fn main() {
    let dir = std::env::args().nth(1).expect("G3-01-dist directory");
    assert!(!dir.contains("heldout"));
    let rows = load_rows(&dir);
    println!(
        "{} distinct nonnegative z; heldout absent",
        rows.len()
    );
    println!(
        "{:<72} {:>22} {:>22} {:>22} {:>22}",
        "cfg", "small z<0.5", "mid [0.5,4)", "tail z>=4", "all"
    );

    let baseline = Cfg {
        arith: Arithmetic::Native53,
        zz: TMode::Native53,
        ratio: Ratio::Continuous,
        small_cut: 1.0,
        mid_cut: 2.0,
        far_cut: 4.0,
        use_excel_exp: true,
    };
    let mut best_exact = 0usize;
    let mut best_cfg = baseline;

    println!("--- arithmetic axes (native zz, continuous uv, default cuts, excel_exp) ---");
    for arith in [
        Arithmetic::Native53,
        Arithmetic::X87Continuous,
        Arithmetic::X87EveryOp53,
        Arithmetic::X87HornerStage53,
        Arithmetic::X87Pc53,
    ] {
        let cfg = Cfg { arith, ..baseline };
        let (bands, all) = score(&rows, cfg);
        print_row(&cfg.name(), &bands, &all);
        if all.exact > best_exact {
            best_exact = all.exact;
            best_cfg = cfg;
        }
    }

    println!("--- zz × ratio on best arith ---");
    for zz in [TMode::Native53, TMode::X87DoubleRounded, TMode::X87Pc53] {
        for ratio in [Ratio::Continuous, Ratio::StoreUv, Ratio::UTimesInvV] {
            let cfg = Cfg {
                arith: best_cfg.arith,
                zz,
                ratio,
                ..baseline
            };
            let (bands, all) = score(&rows, cfg);
            print_row(&cfg.name(), &bands, &all);
            if all.exact > best_exact {
                best_exact = all.exact;
                best_cfg = cfg;
            }
        }
    }

    println!("--- cut-points on best arith/zz/ratio ---");
    for small_cut in [0.46875, 0.5, 1.0] {
        for mid_cut in [1.5, 2.0, 2.5] {
            for far_cut in [3.5, 4.0, 5.0] {
                let cfg = Cfg {
                    small_cut,
                    mid_cut,
                    far_cut,
                    ..best_cfg
                };
                let (bands, all) = score(&rows, cfg);
                let interesting = all.exact != best_exact
                    || small_cut != 1.0
                    || mid_cut != 2.0
                    || far_cut != 4.0;
                if interesting {
                    print_row(&cfg.name(), &bands, &all);
                }
                if all.exact > best_exact {
                    best_exact = all.exact;
                    best_cfg = cfg;
                }
            }
        }
    }

    println!();
    println!("best exact={best_exact} cfg={}", best_cfg.name());
    println!("pins vs best:");
    for &z in &PIN_Z {
        if let Some((_, expected)) = rows.iter().find(|(zz, _)| *zz == z) {
            let got = nswc_eval(z, best_cfg);
            let d = ulp_distance(got, f64::from_bits(*expected)).unwrap_or(u64::MAX);
            println!(
                "  z={z} got={:#x} excel={expected:#x} ulp={d}",
                got.to_bits()
            );
        }
    }
}
