//! Three-way union joint / fd P[0]+4 / T/U U[4]−1. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
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
const PP: [f64; 5] = [
    1.28379167095512558561e-01,
    -3.25042107247001499370e-01,
    -2.84817495755985104766e-02,
    -5.77027029648944159157e-03,
    -2.37630166566501626084e-05,
];
const QQ: [f64; 5] = [
    3.97917223959155352819e-01,
    6.50222499887672944485e-02,
    5.08130628187576562776e-03,
    1.32494738004321644526e-04,
    -3.96022827877536812320e-06,
];
const T0: [f64; 5] = [
    9.60497373987051638749e0,
    9.00260197203842689217e1,
    2.23200534594684319226e3,
    7.00332514112805075473e3,
    5.55923013010394962768e4,
];
const U0: [f64; 5] = [
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
const LOWS: [f64; 4] = [
    0.23046875,
    0.37109375,
    0.4524739583333333,
    0.4716796875,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
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
fn fd_x87(z: f64, p: &[f64; 5], q: &[f64; 5]) -> f64 {
    let ze = ef(z);
    let zz = ext_mul(&ze, &ze, CW);
    let mut r = ef(0.0);
    for &c in p.iter().rev() {
        r = ext_add(&ext_mul(&r, &zz, CW), &ef(c), CW);
    }
    let mut s = ef(0.0);
    for &c in q.iter().rev() {
        s = ext_add(&ext_mul(&s, &zz, CW), &ef(c), CW);
    }
    let den = ext_add(&ef(1.0), &ext_mul(&zz, &s, CW), CW);
    let y = ext_div(&r, &den, CW);
    ext_to_f64(&ext_add(&ze, &ext_mul(&ze, &y, CW), CW), CW)
}
fn cody_ab(z: f64, a: &[f64; 5], b: &[f64; 4]) -> f64 {
    let ye = ef(z);
    let ysq = ext_mul(&ye, &ye, CW);
    let mut xnum = ext_mul(&ef(a[4]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..3 {
        xnum = ext_mul(&ext_add(&xnum, &ef(a[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(b[i]), CW), &ysq, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_mul(&ye, &ext_add(&xnum, &ef(a[3]), CW), CW),
            &ext_add(&xden, &ef(b[3]), CW),
            CW,
        ),
        CW,
    )
}
fn tu_x87(z: f64, t: &[f64; 5], u: &[f64; 5]) -> f64 {
    let ze = ef(z);
    let zz = ext_mul(&ze, &ze, CW);
    let mut num = ef(t[0]);
    for &c in &t[1..] {
        num = ext_add(&ext_mul(&num, &zz, CW), &ef(c), CW);
    }
    let mut den = ext_add(&zz, &ef(u[0]), CW);
    for &c in &u[1..] {
        den = ext_add(&ext_mul(&den, &zz, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_div(&ext_mul(&ze, &num, CW), &den, CW), CW)
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
    let mut p4 = PP;
    p4[0] = poke(PP[0], 4);
    let mut u4 = U0;
    u4[4] = poke(U0[4], -1);
    let mut b3 = BB;
    b3[3] = poke(BB[3], -1);
    let mut n_j = 0usize;
    let mut n_f = 0usize;
    let mut n_u = 0usize;
    let mut n_b = 0usize;
    let mut n_any3 = 0usize;
    let mut n_any4 = 0usize;
    let mut b_not_jfu = 0usize;
    let mut only_f = 0usize;
    let mut only_u = 0usize;
    let mut only_fu = 0usize;
    let mut fu_not_j = 0usize;
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let ej = ulp_distance(erf_a(z, &joint), t).unwrap_or(99) == 0;
        let efd = ulp_distance(fd_x87(z, &p4, &QQ), t).unwrap_or(99) == 0;
        let eu = ulp_distance(tu_x87(z, &T0, &u4), t).unwrap_or(99) == 0;
        let eb = ulp_distance(cody_ab(z, &AA, &b3), t).unwrap_or(99) == 0;
        if ej {
            n_j += 1;
        }
        if efd {
            n_f += 1;
        }
        if eu {
            n_u += 1;
        }
        if eb {
            n_b += 1;
        }
        if ej || efd || eu {
            n_any3 += 1;
        }
        if ej || efd || eu || eb {
            n_any4 += 1;
        }
        if eb && !(ej || efd || eu) {
            b_not_jfu += 1;
        }
        if efd && !ej && !eu {
            only_f += 1;
        }
        if eu && !ej && !efd {
            only_u += 1;
        }
        if efd && eu && !ej {
            only_fu += 1;
        }
        if (efd || eu) && !ej {
            fu_not_j += 1;
        }
    }
    println!(
        "keep j={n_j} fd={n_f} tu={n_u} B3={n_b} union3={n_any3} union4={n_any4}/{} B3_not_jfu={b_not_jfu} fu_not_j={fu_not_j} only_f={only_f} only_u={only_u} only_fu={only_fu}",
        rows.len()
    );
    println!("4 LOW:");
    for &lz in &LOWS {
        let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
            continue;
        };
        let t = f64::from_bits(*bits);
        print!("  z={lz:.16}");
        for (name, g) in [
            ("j", erf_a(lz, &joint)),
            ("fd", fd_x87(lz, &p4, &QQ)),
            ("tu", tu_x87(lz, &T0, &u4)),
            ("B3", cody_ab(lz, &AA, &b3)),
        ] {
            let d = ulp_distance(g, t).unwrap_or(99);
            let dir = if g < t {
                "L"
            } else if g > t {
                "H"
            } else {
                "="
            };
            print!(" {name}={d}{dir}");
        }
        println!();
    }
}
