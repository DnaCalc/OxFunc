//! W109 Phase-4: identify the worksheet NPV kernel directly (the IRR f is
//! documented as cf0 + NPV(rate, cf1..)). Race accumulation forms x store
//! masks against direct NPV answers.
//!
//! Forms:
//!   0: t = t/(1+rate) per term, s += c*t          (division chain)
//!   1: t = t*(v) per term with v = 1/(1+rate)     (reciprocal-multiply)
//!   2: s += c/pow_chain (w^i recomputed per term via repeated mul)
//!   3: horner in v = 1/(1+rate)
//! Mask bits: b0 t-update store, b1 term product store, b2 accumulate store,
//!            b3 w = 1+rate store, b4 final store (always applied at publish).

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::Ext80;

const CW: u16 = 0x133F;

#[derive(Clone, Copy)]
struct V(Ext80);
impl V {
    fn new(x: f64) -> V { V(rx::ext_from_f64(x)) }
    fn st(self, yes: bool) -> V {
        if yes { V::new(self.f()) } else { self }
    }
    fn f(self) -> f64 { rx::ext_to_f64(&self.0, CW) }
    fn add(self, o: V) -> V { V(rx::ext_add(&self.0, &o.0, CW)) }
    fn mul(self, o: V) -> V { V(rx::ext_mul(&self.0, &o.0, CW)) }
    fn div(self, o: V) -> V { V(rx::ext_div(&self.0, &o.0, CW)) }
}

fn npv(rate: f64, cf: &[f64], m: u32, form: u8) -> f64 {
    let bit = |i: u32| m & (1 << i) != 0;
    let w = V::new(1.0).add(V::new(rate)).st(bit(3));
    match form {
        0 => {
            let mut s = V::new(0.0);
            let mut t = V::new(1.0);
            for c in cf {
                t = t.div(w).st(bit(0));
                s = s.add(V::new(*c).mul(t).st(bit(1))).st(bit(2));
            }
            s.st(true).f()
        }
        1 => {
            let v = V::new(1.0).div(w).st(true);
            let mut s = V::new(0.0);
            let mut t = V::new(1.0);
            for c in cf {
                t = t.mul(v).st(bit(0));
                s = s.add(V::new(*c).mul(t).st(bit(1))).st(bit(2));
            }
            s.st(true).f()
        }
        2 => {
            let mut s = V::new(0.0);
            for (i, c) in cf.iter().enumerate() {
                let mut p = w;
                for _ in 0..i {
                    p = p.mul(w).st(bit(0));
                }
                s = s.add(V::new(*c).div(p).st(bit(1))).st(bit(2));
            }
            s.st(true).f()
        }
        3 => {
            let v = V::new(1.0).div(w).st(true);
            let mut s = V::new(0.0);
            for c in cf.iter().rev() {
                s = s.add(V::new(*c)).st(bit(1)).mul(v).st(bit(2));
            }
            s.st(true).f()
        }
        4 => { // c / POWER(w, i+1), binexp LSB-first, spill per bit(0)
            let mut s = V::new(0.0);
            for (i, c) in cf.iter().enumerate() {
                let mut p = V::new(1.0);
                let mut b = w;
                let mut e = i + 1;
                while e > 0 {
                    if e & 1 == 1 {
                        p = p.mul(b).st(bit(0));
                    }
                    e >>= 1;
                    if e > 0 {
                        b = b.mul(b).st(bit(0));
                    }
                }
                s = s.add(V::new(*c).div(p).st(bit(1))).st(bit(2));
            }
            s.st(true).f()
        }
        _ => { // reverse-order division chain: s = (s + c)/w from last term
            let mut s = V::new(0.0);
            for c in cf.iter().rev() {
                s = s.add(V::new(*c)).st(bit(1)).div(w).st(bit(2));
            }
            s.st(true).f()
        }
    }
}

fn main() {
    let mut obs: Vec<(f64, Vec<f64>, u64)> = Vec::new();
    for file in ["../../work/w109/G6-solvers/answers-npv-r0.json",
                 "../../work/w109/G6-solvers/answers-npv-r1.json"] {
    let ws: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string(file).expect("read"),
    )
    .expect("parse");
    for w in &ws.witnesses {
        let rate = match &w.args[0] {
            WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
            _ => continue,
        };
        let cf: Vec<f64> = match &w.args[1] {
            WitnessArg::Array(items) => items.iter().map(|s| parse_bits_hex(s).unwrap()).collect(),
            _ => continue,
        };
        if let Some(want) = parse_bits_hex(&w.expected_bits) {
            obs.push((rate, cf, want.to_bits()));
        }
    }
    }
    println!("{} NPV rows", obs.len());
    let mut results: Vec<(u32, u32, u8)> = Vec::new();
    for form in 0u8..6 {
        for m in 0u32..16 {
            let sc = obs
                .iter()
                .filter(|(r, cf, want)| npv(*r, cf, m, form).to_bits() == *want)
                .count() as u32;
            results.push((sc, m, form));
        }
    }
    results.sort_by(|a, b| b.0.cmp(&a.0));
    for (sc, m, form) in results.iter().take(10) {
        println!("{sc:3}/{}  form{form} mask {m:04b}", obs.len());
    }
    let (_, m, form) = results[0];
    let mut shown = 0;
    for (r, cf, want) in &obs {
        let got = npv(*r, cf, m, form);
        if got.to_bits() != *want && shown < 10 {
            shown += 1;
            println!(
                "  MISS rate={r:.6e} n={} got-want {:+} ulp",
                cf.len(),
                got.to_bits() as i64 - *want as i64
            );
        }
    }
}
