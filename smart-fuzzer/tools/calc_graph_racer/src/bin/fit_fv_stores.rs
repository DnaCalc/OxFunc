//! W109 Phase-4: identify the annuity kernel from direct worksheet FV
//! answers. FV(rate,n,pmt,pv,type) = -(pv*P + pmt*(1+rate*type)*(P-1)/rate),
//! P = (1+rate)^n. Race P-forms x association store masks.
//!
//! P forms: 0 binexp LSB-first, 1 loop multiply, 2 x87 CRT pow (exp/ln chain)
//! Mask bits: b0 P-chain step stores, b1 w = 1+rate store, b2 P store,
//!   b3 (P-1) store, b4 quotient (P-1)/rate store, b5 pmt-term store,
//!   b6 pv-term store, b7 sum store, b8 type-factor store.

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
    fn sub(self, o: V) -> V { V(rx::ext_sub(&self.0, &o.0, CW)) }
    fn mul(self, o: V) -> V { V(rx::ext_mul(&self.0, &o.0, CW)) }
    fn div(self, o: V) -> V { V(rx::ext_div(&self.0, &o.0, CW)) }
    fn neg(self) -> V { V::new(0.0).sub(self) }
}

fn pow_n(w: V, n: u64, m: u32, form: u8) -> V {
    let bit = |i: u32| m & (1 << i) != 0;
    match form {
        0 => {
            // binexp LSB-first
            let mut p = V::new(1.0);
            let mut b = w;
            let mut e = n;
            while e > 0 {
                if e & 1 == 1 {
                    p = p.mul(b).st(bit(0));
                }
                e >>= 1;
                if e > 0 {
                    b = b.mul(b).st(bit(0));
                }
            }
            p
        }
        1 => {
            let mut p = w;
            for _ in 1..n {
                p = p.mul(w).st(bit(0));
            }
            p
        }
        _ => {
            // x87 CRT pow chain (positive base)
            V::new(rx::excel_pow_positive(w.f(), n as f64))
        }
    }
}

fn fv(rate: f64, n: f64, pmt: f64, pv: f64, ty: f64, m: u32, form: u8, tyassoc: u8) -> f64 {
    let bit = |i: u32| m & (1 << i) != 0;
    if rate == 0.0 {
        return -(pv + pmt * n);
    }
    let w = V::new(1.0).add(V::new(rate)).st(bit(1));
    let p = pow_n(w, n as u64, m, form).st(bit(2));
    let pm1 = p.sub(V::new(1.0)).st(bit(3));
    let q = pm1.div(V::new(rate)).st(bit(4));
    let tf = V::new(1.0).add(V::new(rate).mul(V::new(ty))).st(bit(8));
    let pmt_term = match tyassoc {
        0 => V::new(pmt).mul(tf).mul(q).st(bit(5)),
        1 => V::new(pmt).mul(tf.mul(q).st(bit(8))).st(bit(5)),
        2 => V::new(pmt).mul(q).st(bit(8)).mul(tf).st(bit(5)),
        3 => {
            // q' = q*(1+rate*ty) folded into q before pmt
            let q2 = q.mul(tf).st(bit(8));
            V::new(pmt).mul(q2).st(bit(5))
        }
        _ => {
            // annuity-due via w-shift: q_due = (P-1)/rate * w when ty=1
            let q2 = if ty != 0.0 { q.mul(w).st(bit(8)) } else { q };
            V::new(pmt).mul(q2).st(bit(5))
        }
    };
    let pv_term = V::new(pv).mul(p).st(bit(6));
    pv_term.add(pmt_term).st(bit(7)).neg().f()
}

fn main() {
    let ws: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string("../../work/w109/G6-solvers/answers-fv-r0.json").expect("read"),
    )
    .expect("parse");
    let mut obs: Vec<(f64, f64, f64, f64, f64, u64)> = Vec::new();
    for w in &ws.witnesses {
        let a: Vec<f64> = w
            .args
            .iter()
            .filter_map(|x| match x {
                WitnessArg::Scalar(s) => parse_bits_hex(s),
                _ => None,
            })
            .collect();
        if a.len() != 5 {
            continue;
        }
        if let Some(want) = parse_bits_hex(&w.expected_bits) {
            obs.push((a[0], a[1], a[2], a[3], a[4], want.to_bits()));
        }
    }
    println!("{} FV rows", obs.len());
    let mut results: Vec<(u32, u32, u8, u8)> = Vec::new();
    for form in 0u8..3 {
        for tyassoc in 0u8..5 {
            for m in 0u32..(1 << 9) {
                let sc = obs
                    .iter()
                    .filter(|(r, n, pmt, pv, ty, want)| fv(*r, *n, *pmt, *pv, *ty, m, form, tyassoc).to_bits() == *want)
                    .count() as u32;
                results.push((sc, m, form, tyassoc));
            }
        }
    }
    results.sort_by(|a, b| b.0.cmp(&a.0));
    for (sc, m, form, ta) in results.iter().take(10) {
        println!("{sc:3}/{}  form{form} tyassoc{ta} mask {m:09b}", obs.len());
    }
    let (_, m, form, ta) = results[0];
    let mut shown = 0;
    for (r, n, pmt, pv, ty, want) in &obs {
        let got = fv(*r, *n, *pmt, *pv, *ty, m, form, ta);
        if got.to_bits() != *want && shown < 12 {
            shown += 1;
            println!(
                "  MISS rate={r:.6e} n={n} pmt={pmt} pv={pv} ty={ty} got-want {:+} ulp",
                got.to_bits() as i64 - *want as i64
            );
        }
    }
}
