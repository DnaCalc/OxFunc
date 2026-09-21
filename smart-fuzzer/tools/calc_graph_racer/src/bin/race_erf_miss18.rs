//! 18/46 joint-hard that directed 1+w misses, vs named F. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const FDLIBM_PP: [f64; 5] = [
    1.28379167095512558561e-01,
    -3.25042107247001499370e-01,
    -2.84817495755985104766e-02,
    -5.77027029648944159157e-03,
    -2.37630166566501626084e-05,
];
const FDLIBM_QQ: [f64; 5] = [
    3.97917223959155352819e-01,
    6.50222499887672944485e-02,
    5.08130628187576562776e-03,
    1.32494738004321644526e-04,
    -3.96022827877536812320e-06,
];
const CEPHES_T: [f64; 5] = [
    9.60497373987051638749e0,
    9.00260197203842689217e1,
    2.23200534594684319226e3,
    7.00332514112805075473e3,
    5.55923013010394962768e4,
];
const CEPHES_U: [f64; 5] = [
    3.35617141647503099647e1,
    5.21357949780152679795e2,
    4.59432382970980127987e3,
    2.26290000613890934246e4,
    4.92673942608635921086e4,
];
const AA: [f64; 5] = [
    3.16112374387056560,
    113.864154151050156,
    377.485237685302021,
    3209.37758913846947,
    0.185777706184603153,
];
const BB: [f64; 4] = [
    23.6012909523441209,
    244.024637934444173,
    1282.61652607737228,
    2844.23683343917062,
];
const NSWC_A: [f64; 5] = [
    0.771058495001320e-04,
    -0.133733772997339e-02,
    0.323076579225834e-01,
    0.479137145607681e-01,
    0.128379167095513e+00,
];
const NSWC_B: [f64; 3] = [
    0.301048631703895e-02,
    0.538971687740286e-01,
    0.375795757275549e+00,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn horner_lo(cs: &[f64], x: f64) -> f64 {
    let mut a = 0.0;
    for &c in cs.iter().rev() {
        a = a * x + c;
    }
    a
}
fn polevl_hi(cs: &[f64], x: f64) -> f64 {
    let mut a = cs[0];
    for &c in &cs[1..] {
        a = a * x + c;
    }
    a
}
fn p1evl_hi(cs: &[f64], x: f64) -> f64 {
    let mut a = x + cs[0];
    for &c in &cs[1..] {
        a = a * x + c;
    }
    a
}
fn fdlibm_pp(z: f64) -> f64 {
    let zz = z * z;
    let r = horner_lo(&FDLIBM_PP, zz);
    let s = 1.0 + zz * horner_lo(&FDLIBM_QQ, zz);
    z + z * (r / s)
}
fn fdlibm_pp_x87(z: f64) -> f64 {
    let ze = ef(z);
    let zz = ext_mul(&ze, &ze, CW);
    let mut r = ef(0.0);
    for &c in FDLIBM_PP.iter().rev() {
        r = ext_add(&ext_mul(&r, &zz, CW), &ef(c), CW);
    }
    let mut q = ef(0.0);
    for &c in FDLIBM_QQ.iter().rev() {
        q = ext_add(&ext_mul(&q, &zz, CW), &ef(c), CW);
    }
    let s = ext_add(&ef(1.0), &ext_mul(&zz, &q, CW), CW);
    let y = ext_div(&r, &s, CW);
    ext_to_f64(&ext_add(&ze, &ext_mul(&ze, &y, CW), CW), CW)
}
fn cephes_tu(z: f64) -> f64 {
    let zz = z * z;
    z * polevl_hi(&CEPHES_T, zz) / p1evl_hi(&CEPHES_U, zz)
}
fn cody_ab_x87(z: f64) -> f64 {
    let ye = ef(z);
    let ysq = ext_mul(&ye, &ye, CW);
    let mut xnum = ext_mul(&ef(AA[4]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..3 {
        xnum = ext_mul(&ext_add(&xnum, &ef(AA[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(BB[i]), CW), &ysq, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_mul(&ye, &ext_add(&xnum, &ef(AA[3]), CW), CW),
            &ext_add(&xden, &ef(BB[3]), CW),
            CW,
        ),
        CW,
    )
}
fn nswc_erfc1(z: f64) -> f64 {
    let t = z * z;
    z * (horner_lo(&NSWC_A, t) + 1.0) / (1.0 + t * horner_lo(&NSWC_B, t))
}
const AS0: [f64; 21] = [
    0.1283791670955125738961589031215,
    -0.3761263890318375246320529677070,
    0.1128379167095512573896158902931,
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
fn poke(x: f64, k: i32) -> f64 {
    let mut v = x;
    if k > 0 {
        for _ in 0..k {
            v = v.next_up();
        }
    } else {
        for _ in 0..(-k) {
            v = v.next_down();
        }
    }
    v
}
fn erf_a(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    const BANKS: [&str; 7] = [
        "answers-b9train.json",
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b8erf.json",
        "answers-b7erf.json",
        "answers-b11.json",
        "answers-b10.json",
    ];
    let mut map = BTreeMap::new();
    for name in BANKS {
        let path = format!("{dir}/{name}");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet = serde_json::from_str(&text).unwrap();
        for w in &bank.witnesses {
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            let Some(e) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if x > 0.0 && x < 0.5 {
                map.entry(x.to_bits()).or_insert(e.to_bits());
            }
        }
    }
    let rows: Vec<_> = map
        .into_iter()
        .map(|(b, e)| (f64::from_bits(b), e))
        .collect();
    let mut joint = AS0;
    joint[0] = poke(AS0[0], 4);
    joint[1] = poke(AS0[1], -2);
    joint[2] = poke(AS0[2], -5);
    joint[3] = poke(AS0[3], 1);
    let one_w_up = |z: f64| {
        let t = ext_mul(&ef(z), &ef(z), CW);
        let mut acc = ef(0.0);
        for &c in joint.iter().rev() {
            acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
        }
        let s = ext_to_f64(&ext_add(&ef(1.0), &acc, CW), CW).next_up();
        z * s
    };
    let one_w_dn = |z: f64| {
        let t = ext_mul(&ef(z), &ef(z), CW);
        let mut acc = ef(0.0);
        for &c in joint.iter().rev() {
            acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
        }
        let s = ext_to_f64(&ext_add(&ef(1.0), &acc, CW), CW).next_down();
        z * s
    };
    let mut miss: Vec<(f64, u64, bool, u64)> = Vec::new();
    let mut n_hard = 0usize;
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let g = erf_a(z, &joint);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d < 2 {
            continue;
        }
        n_hard += 1;
        let hit = if g < t {
            ulp_distance(one_w_up(z), t).unwrap_or(99) == 0
        } else {
            ulp_distance(one_w_dn(z), t).unwrap_or(99) == 0
        };
        if !hit {
            miss.push((z, bits, g > t, d));
        }
    }
    println!("joint-hard n={n_hard}  1+w-miss n={}", miss.len());
    let graphs: [(&str, fn(f64) -> f64); 6] = [
        ("fdlibm PP native", fdlibm_pp),
        ("fdlibm PP x87", fdlibm_pp_x87),
        ("Cephes T/U", cephes_tu),
        ("Cody A/B", cody_ab_x87),
        ("NSWC 5+3", nswc_erfc1),
        ("libm::erf", libm::erf),
    ];
    for &(z, bits, high, dj) in &miss {
        let t = f64::from_bits(bits);
        print!(
            "  z={z:.16} j_ulp={dj} {}",
            if high { "HIGH" } else { "LOW" }
        );
        for (name, ev) in graphs {
            print!(" {name}={}", ulp_distance(ev(z), t).unwrap_or(99));
        }
        println!();
    }
    for (name, ev) in graphs {
        let mut hit = 0usize;
        let mut u1 = 0usize;
        let mut keep = 0usize;
        for &(z, bits) in &rows {
            if ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99) == 0 {
                keep += 1;
            }
        }
        for &(z, bits, _, _) in &miss {
            let d = ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99);
            if d == 0 {
                hit += 1;
                println!("  HIT {name} z={z:.16}");
            } else if d == 1 {
                u1 += 1;
            }
        }
        println!("{name:18} miss_hit={hit}/{} ulp1={u1} keep={keep}/1508", miss.len());
    }
}
