//! ERF z∈[0.5,4): Cody A/B fused ∪ last-store of z / (1+w). Not an identity.
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
fn spec(z: f64) -> f64 {
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
fn ow(z: f64) -> Ext80 {
    let ye = ef(z);
    let ysq = ext_mul(&ye, &ye, CW);
    let mut xnum = ext_mul(&ef(AA[4]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..3 {
        xnum = ext_mul(&ext_add(&xnum, &ef(AA[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(BB[i]), CW), &ysq, CW);
    }
    ext_div(&ext_add(&xnum, &ef(AA[3]), CW), &ext_add(&xden, &ef(BB[3]), CW), CW)
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
    let mut n_f = 0usize;
    let mut n_ls = 0usize;
    let mut n_u = 0usize;
    let mut miss = 0usize;
    println!("ERF z∈[0.5,4) Cody A/B fused ∪ last-store n={}", rows.len());
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let g = spec(z);
        let efused = ulp_distance(g, t).unwrap_or(99) == 0;
        let o = ow(z);
        let of = ext_to_f64(&o, CW);
        let els = (-4i32..=4).filter(|&k| k != 0).any(|k| {
            ulp_distance(
                ext_to_f64(&ext_mul(&ef(poke(z, k)), &o, CW), CW),
                t,
            )
            .unwrap_or(99)
                == 0
                || ulp_distance(
                    ext_to_f64(&ext_mul(&ef(z), &ef(poke(of, k)), CW), CW),
                    t,
                )
                .unwrap_or(99)
                    == 0
        });
        if efused {
            n_f += 1;
        }
        if els {
            n_ls += 1;
        }
        if efused || els {
            n_u += 1;
        } else {
            miss += 1;
            if miss <= 10 {
                let d = ulp_distance(g, t).unwrap_or(99);
                println!(
                    "  MISS z={z:.16} d={}{}",
                    d,
                    if g < t { "L" } else { "H" }
                );
            }
        }
    }
    println!(
        "fused={n_f} last-store={n_ls} union={n_u}/{} miss={miss}",
        rows.len()
    );
}
