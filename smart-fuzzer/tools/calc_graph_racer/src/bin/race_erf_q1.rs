//! ERF z∈[0.5,4) vs RN53(1−Q) of unmask Cody fused ∪ F±. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
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
fn cody_mask(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn qmul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
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
            if x >= 0.5 && x < 4.0 {
                map.entry(x.to_bits()).or_insert(e.to_bits());
            }
        }
    }
    let rows: Vec<_> = map
        .into_iter()
        .map(|(b, e)| (f64::from_bits(b), e))
        .collect();
    let mut n0 = 0usize;
    let mut nls = 0usize;
    let mut n_u = 0usize;
    let mut miss = 0usize;
    println!("ERF [0.5,4) RN53(1−Q) unmask Cody ∪ F± n={}", rows.len());
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let w = f::w_rn53(z);
        let ff = cody_mask(z, 0);
        let q0 = qmul(w, ff);
        let p0 = 1.0 - q0;
        let e0 = ulp_distance(p0, t).unwrap_or(99) == 0;
        let els = (-4i32..=4).filter(|&k| k != 0).any(|k| {
            let q = qmul(w, poke(ff, k));
            ulp_distance(1.0 - q, t).unwrap_or(99) == 0
        });
        if e0 {
            n0 += 1;
        }
        if els {
            nls += 1;
        }
        if e0 || els {
            n_u += 1;
        } else {
            miss += 1;
            if miss <= 8 {
                println!(
                    "  MISS z={z:.16} 1-Q={}{}",
                    ulp_distance(p0, t).unwrap_or(99),
                    if p0 < t { "L" } else { "H" }
                );
            }
        }
    }
    println!("fused-1Q={n0} last-store-1Q={nls} union={n_u}/{} miss={miss}", rows.len());
}
