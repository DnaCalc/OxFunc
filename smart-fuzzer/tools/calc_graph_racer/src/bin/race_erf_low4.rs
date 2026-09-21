//! 4 P-side LOW 1+w-miss vs 1−(w·C/D). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
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
const C0: [f64; 9] = [
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
const D0: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody(y: f64, c: &[f64; 9], d: &[f64; 8], mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = rx::ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(d[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
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
fn one_minus_q(z: f64, ff: f64) -> f64 {
    let q = f::w_rn53(z) * ff;
    1.0 - q
}
fn one_minus_q80(z: f64, ff: f64) -> f64 {
    let q = ext_mul(&ef(f::w_rn53(z)), &ef(ff), CW);
    ext_to_f64(&ext_sub(&ef(1.0), &q, CW), CW)
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
    let mut jc = C0;
    jc[0] = poke(C0[0], 4);
    jc[1] = poke(C0[1], 1);
    jc[4] = poke(C0[4], -1);
    let mut jd = D0;
    jd[0] = poke(D0[0], -1);
    jd[4] = poke(D0[4], -1);
    let one_w_up = |z: f64| {
        let t = ext_mul(&ef(z), &ef(z), CW);
        let mut acc = ef(0.0);
        for &c in joint.iter().rev() {
            acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
        }
        let s = ext_to_f64(&ext_add(&ef(1.0), &acc, CW), CW).next_up();
        z * s
    };
    let mut lows: Vec<(f64, u64, u64)> = Vec::new();
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let g = erf_a(z, &joint);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d >= 2 && g < t && ulp_distance(one_w_up(z), t).unwrap_or(99) != 0 {
            lows.push((z, bits, d));
        }
    }
    println!("LOW 1+w-miss n={} (expect 4)", lows.len());
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 8] = [
        ("joint", Box::new(|z| erf_a(z, &joint))),
        ("1-w*CR", Box::new(|z| one_minus_q(z, cody(z, &C0, &D0, 0)))),
        ("1-w*0x74", Box::new(|z| one_minus_q(z, cody(z, &C0, &D0, 0x74)))),
        ("1-w*j3432", Box::new(|z| one_minus_q(z, cody(z, &jc, &jd, 0)))),
        ("1-w*j20", Box::new(|z| one_minus_q(z, cody(z, &jc, &jd, 0x20)))),
        ("80(1-w*CR)", Box::new(|z| one_minus_q80(z, cody(z, &C0, &D0, 0)))),
        ("80(1-w*j)", Box::new(|z| one_minus_q80(z, cody(z, &jc, &jd, 0)))),
        ("1-libm erfc", Box::new(|z| 1.0 - libm::erfc(z))),
    ];
    for &(z, bits, dj) in &lows {
        let t = f64::from_bits(bits);
        print!("  z={z:.16} j_ulp={dj}");
        for (name, ev) in &graphs {
            print!(" {name}={}", ulp_distance(ev(z), t).unwrap_or(99));
        }
        println!();
    }
    for (name, ev) in &graphs {
        let mut hit = 0usize;
        let mut u1 = 0usize;
        let mut keep = 0usize;
        for &(z, bits) in &rows {
            if ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99) == 0 {
                keep += 1;
            }
        }
        for &(z, bits, _) in &lows {
            let d = ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99);
            if d == 0 {
                hit += 1;
                println!("  HIT {name} z={z:.16}");
            } else if d == 1 {
                u1 += 1;
            }
        }
        println!("{name:14} hit={hit}/{} ulp1={u1} keep={keep}/1508", lows.len());
    }
    let erf_t = |z: f64, k: i32| {
        let mut tt = ext_to_f64(&ext_mul(&ef(z), &ef(z), CW), CW);
        if k > 0 {
            tt = tt.next_up();
        } else if k < 0 {
            tt = tt.next_down();
        }
        let te = ef(tt);
        let mut acc = ef(0.0);
        for &c in joint.iter().rev() {
            acc = ext_add(&ext_mul(&acc, &te, CW), &ef(c), CW);
        }
        ext_to_f64(&ext_mul(&ef(z), &ext_add(&ef(1.0), &acc, CW), CW), CW)
    };
    println!("t=x² store nudge on LOW 1+w-miss (bar 866):");
    for (name, k) in [("t RN", 0i32), ("t next_up", 1), ("t next_down", -1)] {
        let mut hit = 0usize;
        let mut keep = 0usize;
        for &(z, bits) in &rows {
            if ulp_distance(erf_t(z, k), f64::from_bits(bits)).unwrap_or(99) == 0 {
                keep += 1;
            }
        }
        for &(z, bits, _) in &lows {
            let d = ulp_distance(erf_t(z, k), f64::from_bits(bits)).unwrap_or(99);
            if d == 0 {
                hit += 1;
                println!("  HIT {name} z={z:.16}");
            }
        }
        println!("  {name:12} hit={hit}/4 keep={keep}/1508");
    }
}
