//! B[3]−1 (keep 789) plus one ±1 vs 4 LOW. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
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
    let mut bbase = BB;
    bbase[3] = poke(BB[3], -1);
    let keep = |a: &[f64; 5], b: &[f64; 4]| -> usize {
        rows.iter()
            .filter(|&&(z, bits)| ulp_distance(cody_ab(z, a, b), f64::from_bits(bits)).unwrap_or(99) == 0)
            .count()
    };
    let hit4 = |a: &[f64; 5], b: &[f64; 4]| -> (usize, Vec<f64>) {
        let mut h = 0usize;
        let mut zs = Vec::new();
        for &lz in &LOWS {
            let Some((_, bits)) = rows.iter().find(|(z, _)| (*z - lz).abs() < 1e-14) else {
                continue;
            };
            if ulp_distance(cody_ab(lz, a, b), f64::from_bits(*bits)).unwrap_or(99) == 0 {
                h += 1;
                zs.push(lz);
            }
        }
        (h, zs)
    };
    let k0 = keep(&AA, &bbase);
    let (h0, _) = hit4(&AA, &bbase);
    println!("B[3]-1 keep={k0} hit={h0}/4 (bar CR 203)");
    println!("extra ±1 (print hit>0 or keep>=k0):");
    let mut best_h = h0;
    let mut best_k = k0;
    let mut lab = "B[3]-1".to_string();
    for i in 0..5 {
        for k in [-1i32, 1] {
            let mut a = AA;
            a[i] = poke(AA[i], k);
            let kk = keep(&a, &bbase);
            let (h, zs) = hit4(&a, &bbase);
            if h > 0 || kk >= k0 {
                println!("  A[{i}] {k:+} keep={kk} hit={h}/4 {:?}", zs);
            }
            if h > best_h || (h == best_h && kk > best_k) {
                best_h = h;
                best_k = kk;
                lab = format!("B[3]-1 A[{i}] {k:+}");
            }
        }
    }
    for i in 0..3 {
        for k in [-1i32, 1] {
            let mut b = bbase;
            b[i] = poke(BB[i], k);
            let kk = keep(&AA, &b);
            let (h, zs) = hit4(&AA, &b);
            if h > 0 || kk >= k0 {
                println!("  B[{i}] {k:+} keep={kk} hit={h}/4 {:?}", zs);
            }
            if h > best_h || (h == best_h && kk > best_k) {
                best_h = h;
                best_k = kk;
                lab = format!("B[3]-1 B[{i}] {k:+}");
            }
        }
    }
    println!("best-by-hit {lab} keep={best_k} hit={best_h}");
}
