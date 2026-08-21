//! W109 sweep: race ATANH/ACOTH x87 fyl2xp1-pair candidates against fresh
//! live corpora. ATANH = 0.5*(ln1p(x) - ln1p(-x)); ACOTH = same on 1/x or
//! the (x+1)/(x-1) ratio forms.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::Ext80;

const CW: u16 = 0x133F;

fn e(x: f64) -> Ext80 {
    rx::ext_from_f64(x)
}
fn tof(x: &Ext80) -> f64 {
    rx::ext_to_f64(x, CW)
}

/// 0.5*(ln1p(x) - ln1p(-x)) fully extended; `pair_scaled`: use y = ln2/2 in
/// fyl2xp1 (exact halving of the ROM constant) instead of a final *0.5.
fn atanh_pair(x: f64, pair_scaled: bool, store_each: bool) -> f64 {
    let ln2 = rx::ext_ln2();
    let y = if pair_scaled {
        // ln2/2: exponent decrement, exact on Ext80 via scale by -1
        rx::ext_scale(&ln2, &e(-1.0), CW)
    } else {
        ln2
    };
    let mut t1 = rx::ext_fyl2xp1(&y, &e(x), CW);
    let mut t2 = rx::ext_fyl2xp1(&y, &e(-x), CW);
    if store_each {
        t1 = e(tof(&t1));
        t2 = e(tof(&t2));
    }
    let d = rx::ext_sub(&t1, &t2, CW);
    if pair_scaled {
        tof(&d)
    } else {
        tof(&rx::ext_mul(&d, &e(0.5), CW))
    }
}

/// ratio form under x87: 0.5*fFLN((1+x)/(1-x)) with staging bits.
fn atanh_ratio(x: f64, ext_ratio: bool) -> f64 {
    let num = rx::ext_add(&rx::ext_one(), &e(x), CW);
    let den = rx::ext_sub(&rx::ext_one(), &e(x), CW);
    let r = rx::ext_div(&num, &den, CW);
    let arg = if ext_ratio { r } else { e(tof(&r)) };
    let l = rx::ext_fyl2x(&rx::ext_ln2(), &arg, CW);
    tof(&rx::ext_mul(&l, &e(0.5), CW))
}

fn load(path: &str) -> Vec<(f64, u64)> {
    let ws: WitnessSet =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("parse");
    ws.witnesses
        .iter()
        .filter_map(|w| {
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s)?,
                _ => return None,
            };
            parse_bits_hex(&w.expected_bits).map(|v| (x, v.to_bits()))
        })
        .collect()
}

fn score(name: &str, rows: &[(f64, u64)], f: &dyn Fn(f64) -> f64) {
    let mut ok = 0;
    let mut miss: Vec<(f64, i64)> = Vec::new();
    for (x, want) in rows {
        let v = f(*x);
        if v.to_bits() == *want {
            ok += 1;
        } else if miss.len() < 5 {
            miss.push((*x, v.to_bits() as i64 - *want as i64));
        }
    }
    println!("{name:36} {ok}/{}  {miss:?}", rows.len());
}

fn main() {
    let at = load("../../work/w109/G4-hyp-answers-atanh.json");
    let ac = load("../../work/w109/G4-hyp-answers-acoth.json");
    println!("{} ATANH, {} ACOTH rows", at.len(), ac.len());
    score("atanh pair ext (final*0.5)", &at, &|x| {
        atanh_pair(x, false, false)
    });
    score("atanh pair ext (y=ln2/2)", &at, &|x| {
        atanh_pair(x, true, false)
    });
    score("atanh pair stored halves", &at, &|x| {
        atanh_pair(x, false, true)
    });
    score("atanh ratio ext", &at, &|x| atanh_ratio(x, true));
    score("atanh ratio stored", &at, &|x| atanh_ratio(x, false));
    // ACOTH candidates: same chains on 1/x (stored or extended reciprocal)
    score("acoth pair(1/x stored)", &ac, &|x| {
        let r = tof(&rx::ext_div(&rx::ext_one(), &e(x), CW));
        atanh_pair(r, false, false)
    });
    score("acoth ratio ((x+1)/(x-1)) ext", &ac, &|x| {
        let num = rx::ext_add(&e(x), &rx::ext_one(), CW);
        let den = rx::ext_sub(&e(x), &rx::ext_one(), CW);
        let q = rx::ext_div(&num, &den, CW);
        let l = rx::ext_fyl2x(&rx::ext_ln2(), &q, CW);
        tof(&rx::ext_mul(&l, &e(0.5), CW))
    });
    score("acoth ratio stored", &ac, &|x| {
        let num = x + 1.0;
        let den = x - 1.0;
        let q = num / den;
        let l = rx::ext_fyl2x(&rx::ext_ln2(), &e(q), CW);
        tof(&rx::ext_mul(&l, &e(0.5), CW))
    });
    // fyl2xp1 on 2/(x-1) form
    score("acoth halfln1p(2/(x-1)) ext", &ac, &|x| {
        let t = rx::ext_div(&e(2.0), &rx::ext_sub(&e(x), &rx::ext_one(), CW), CW);
        let l = rx::ext_fyl2xp1(&rx::ext_ln2(), &t, CW);
        tof(&rx::ext_mul(&l, &e(0.5), CW))
    });
}
