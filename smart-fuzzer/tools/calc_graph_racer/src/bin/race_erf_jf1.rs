//! joint-866 vs fd P[0]+4 two-mode overlap on 4 LOW. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const CW_RU: u16 = CW_PC64_RN | 0x0800;
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
fn one_w(z: f64) -> Ext80 {
    let mut a = AS0;
    a[0] = poke(AS0[0], 4);
    a[1] = poke(AS0[1], -2);
    a[2] = poke(AS0[2], -5);
    a[3] = poke(AS0[3], 1);
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_add(&ef(1.0), &acc, CW)
}
fn one_y(inner: f64) -> Ext80 {
    let mut p = PP;
    p[0] = poke(PP[0], 4);
    let ze = ef(inner);
    let zz = ext_mul(&ze, &ze, CW);
    let mut r = ef(0.0);
    for &c in p.iter().rev() {
        r = ext_add(&ext_mul(&r, &zz, CW), &ef(c), CW);
    }
    let mut s = ef(0.0);
    for &c in QQ.iter().rev() {
        s = ext_add(&ext_mul(&s, &zz, CW), &ef(c), CW);
    }
    let den = ext_add(&ef(1.0), &ext_mul(&zz, &s, CW), CW);
    let y = ext_div(&r, &den, CW);
    ext_add(&ef(1.0), &y, CW)
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
    let jn = |z: f64| ext_to_f64(&ext_mul(&ef(z), &one_w(z), CW), CW);
    let ru = |z: f64| ext_to_f64(&ext_mul(&ef(z.next_up()), &one_y(z), CW_RU), CW_RU);
    let tf = |z: f64| {
        let ow = ext_to_f64(&one_y(z), CW).next_up();
        ext_to_f64(&ext_mul(&ef(z.next_up()), &ef(ow), CW), CW)
    };
    let mut both = 0usize;
    let mut only_j = 0usize;
    let mut only_f = 0usize;
    let mut only_f_zs = Vec::new();
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let ej = ulp_distance(jn(z), t).unwrap_or(99) == 0;
        let efm = ulp_distance(ru(z), t).unwrap_or(99) == 0
            || ulp_distance(tf(z), t).unwrap_or(99) == 0;
        match (ej, efm) {
            (true, true) => both += 1,
            (true, false) => only_j += 1,
            (false, true) => {
                only_f += 1;
                only_f_zs.push(z);
            }
            _ => {}
        }
    }
    println!("4 LOW joint vs fd two-mode:");
    let mut hit_j = 0usize;
    let mut hit_f = 0usize;
    let mut hit_u = 0usize;
    for &lz in &LOWS {
        let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
            continue;
        };
        let t = f64::from_bits(*bits);
        let ej = ulp_distance(jn(lz), t).unwrap_or(99);
        let eru = ulp_distance(ru(lz), t).unwrap_or(99) == 0;
        let etf = ulp_distance(tf(lz), t).unwrap_or(99) == 0;
        let efm = eru || etf;
        if ej == 0 {
            hit_j += 1;
        }
        if efm {
            hit_f += 1;
        }
        if ej == 0 || efm {
            hit_u += 1;
        }
        println!(
            "  z={lz:.16} joint_ulp={ej} RU={eru} TF={etf} any={}",
            ej == 0 || efm
        );
    }
    println!(
        "overlap both={both} only_joint={only_j} only_fd2={only_f} union={}/{} leftover J={hit_j}/4 FD={hit_f}/4 union={hit_u}/4",
        both + only_j + only_f,
        rows.len()
    );
    print!("only_fd2 z");
    for z in &only_f_zs {
        print!(" {z:.16}");
    }
    println!(" count={}", only_f_zs.len());
}
