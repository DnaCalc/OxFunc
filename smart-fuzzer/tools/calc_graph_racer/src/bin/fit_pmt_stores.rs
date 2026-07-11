//! W109 Phase-4: PMT identification. FV and PV closed as plain-double over
//! the binexp annuity kernel; PMT resists all plain compositions -> race
//! the same composition zoo under x87 store-mask staging and P-form
//! variants (binexp spill, loop multiply, x87 CRT pow chain).
//!
//! PMT(rate, nper, pv, fv, type); rate==0 -> -(pv+fv)/n.
//! Mask bits (1 = stored to double, 0 = extended):
//!   b0 P-chain steps, b1 w=1+rate, b2 P, b3 P-1, b4 q=(P-1)/rate,
//!   b5 den, b6 num, b7 pre-negation quotient, b8 tf=1+rate*type.

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

fn pow_n(w: V, n: u64, m: u32, pform: u8) -> V {
    let bit = |i: u32| m & (1 << i) != 0;
    match pform {
        0 => {
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
        2 => V::new(rx::excel_pow_positive(w.f(), n as f64)),
        _ => {
            // fused x87 pow chain: P = 2^(n*log2 w), extended end-to-end;
            // bit(0) optionally stores t = n*log2(w) (the POWER staging)
            let t = V(rx::ext_fyl2x(&rx::ext_from_f64(n as f64), &w.0, CW)).st(bit(0));
            let k = rx::ext_rndint(&t.0, CW);
            let f = rx::ext_sub(&t.0, &k, CW);
            let neg = rx::ext_to_f64(&f, CW) < 0.0;
            let wv = rx::ext_f2xm1(&rx::ext_abs(&f, CW), CW);
            let mut m = rx::ext_add(&wv, &rx::ext_one(), CW);
            if neg {
                m = rx::ext_div(&rx::ext_one(), &m, CW);
            }
            V(rx::ext_scale(&m, &k, CW))
        }
    }
}

fn pmt(rate: f64, n: f64, pv: f64, fvv: f64, ty: f64, m: u32, pform: u8, comp: u8) -> f64 {
    let bit = |i: u32| m & (1 << i) != 0;
    if rate == 0.0 {
        return -(pv + fvv) / n;
    }
    let w = V::new(1.0).add(V::new(rate)).st(bit(1));
    let p = pow_n(w, n as u64, m, pform).st(bit(2));
    let pm1 = p.sub(V::new(1.0)).st(bit(3));
    let tf = V::new(1.0).add(V::new(rate).mul(V::new(ty))).st(bit(8));
    match comp {
        0 => {
            // -(pv*P + fv)/(tf*q), q = (P-1)/rate
            let q = pm1.div(V::new(rate)).st(bit(4));
            let den = tf.mul(q).st(bit(5));
            let num = V::new(pv).mul(p).add(V::new(fvv)).st(bit(6));
            num.div(den).st(bit(7)).neg().f()
        }
        1 => {
            // split: -(pv*P/(tf*q) + fv/(tf*q))
            let q = pm1.div(V::new(rate)).st(bit(4));
            let den = tf.mul(q).st(bit(5));
            let t1 = V::new(pv).mul(p).div(den).st(bit(6));
            let t2 = V::new(fvv).div(den).st(bit(7));
            t1.add(t2).neg().f()
        }
        2 => {
            // (-(pv*P + fv)/(P-1)) * (rate/tf)
            let num = V::new(pv).mul(p).add(V::new(fvv)).st(bit(6));
            let a = num.div(pm1).st(bit(4));
            let b = V::new(rate).div(tf).st(bit(5));
            a.mul(b).st(bit(7)).neg().f()
        }
        3 => {
            // -(pv*P + fv)*rate/((P-1)*tf)
            let num = V::new(pv).mul(p).add(V::new(fvv)).st(bit(6));
            let den = pm1.mul(tf).st(bit(5));
            num.mul(V::new(rate)).st(bit(4)).div(den).st(bit(7)).neg().f()
        }
        4 => {
            // via reciprocal: -(pv + fv/P)*(rate*P/(P-1))/tf
            let vn = V::new(fvv).div(p).st(bit(4));
            let num = V::new(pv).add(vn).st(bit(6));
            let fac = V::new(rate).mul(p).div(pm1).st(bit(5));
            num.mul(fac).div(tf).st(bit(7)).neg().f()
        }
        _ => {
            // PV-side dual: -(pv + fv/P)/(tf*(1 - 1/P)/rate)
            let pinv = V::new(1.0).div(p).st(bit(4));
            let apvf = V::new(1.0).sub(pinv).st(bit(3)).div(V::new(rate)).st(bit(5));
            let num = V::new(pv).add(V::new(fvv).mul(pinv).st(bit(6))).st(bit(6));
            num.div(tf.mul(apvf).st(bit(5))).st(bit(7)).neg().f()
        }
    }
}

fn main() {
    let ws: WitnessSet = serde_json::from_str(
        &std::fs::read_to_string("../../work/w109/G6-solvers/answers-pmt-r0.json").expect("read"),
    )
    .expect("parse");
    let mut obs: Vec<(Vec<f64>, u64)> = Vec::new();
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
            obs.push((a, want.to_bits()));
        }
    }
    println!("{} PMT rows", obs.len());
    let mut results: Vec<(u32, u32, u8, u8)> = Vec::new();
    for pform in 0u8..4 {
        for comp in 0u8..6 {
            for m in 0u32..(1 << 9) {
                let sc = obs
                    .iter()
                    .filter(|(a, want)| pmt(a[0], a[1], a[2], a[3], a[4], m, pform, comp).to_bits() == *want)
                    .count() as u32;
                results.push((sc, m, pform, comp));
            }
        }
    }
    results.sort_by(|a, b| b.0.cmp(&a.0));
    for (sc, m, pform, comp) in results.iter().take(10) {
        println!("{sc:3}/{}  pform{pform} comp{comp} mask {m:09b}", obs.len());
    }
    let (_, m, pform, comp) = results[0];
    let mut shown = 0;
    for (a, want) in &obs {
        let got = pmt(a[0], a[1], a[2], a[3], a[4], m, pform, comp);
        if got.to_bits() != *want && shown < 10 {
            shown += 1;
            println!(
                "  MISS rate={:.6e} n={} pv={} fv={} ty={} {:+} ulp",
                a[0], a[1], a[2], a[3], a[4],
                got.to_bits() as i64 - *want as i64
            );
        }
    }
}
