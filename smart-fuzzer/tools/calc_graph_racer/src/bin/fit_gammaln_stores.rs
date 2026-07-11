//! W109 Phase-5: exhaustive store-mask fit of Cephes lgam against the 93-row
//! published-GAMMALN corpus. Eight statement-boundary toggles (bit=1 -> the
//! intermediate is STORED to binary64; bit=0 -> it stays extended):
//!   b0 z-loop steps        b1 ln(z) result        b2 Horner steps (B/C/A)
//!   b3 rational q value    b4 (small) final sum is always stored (return)
//!   b4 Stirling q0         b5 Stirling p=1/x^2    b6 Stirling correction t
//!   b7 ln(x) result (Stirling)

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet, ulp_distance};
use oxfunc_core::excel_numeric::research as rx;
use rx::Ext80;

const A: [f64; 5] = [
    8.11614167470508450300e-4,
    -5.95061904284301438324e-4,
    7.93650340457716943945e-4,
    -2.77777777730099687205e-3,
    8.33333333333331927722e-2,
];
const B: [f64; 6] = [
    -1.37825152569120859100e3,
    -3.88016315134637840924e4,
    -3.31612992738871184744e5,
    -1.16237097492762307383e6,
    -1.72173700820839662146e6,
    -8.53555664245765465627e5,
];
const C: [f64; 6] = [
    -3.51815701436523470549e2,
    -1.70642106651881159223e4,
    -2.20528590553854454839e5,
    -1.13933444367982507207e6,
    -2.53252307177582951285e6,
    -2.01889141433532773231e6,
];
const LS2PI: f64 = 0.91893853320467274178;
const CW: u16 = 0x133F;

#[derive(Clone, Copy)]
struct V(Ext80);
impl V {
    fn new(x: f64) -> V {
        V(rx::ext_from_f64(x))
    }
    fn store(self, yes: bool) -> V {
        if yes {
            V::new(rx::ext_to_f64(&self.0, CW))
        } else {
            self
        }
    }
    fn f(self) -> f64 {
        rx::ext_to_f64(&self.0, CW)
    }
    fn add(self, o: V) -> V {
        V(rx::ext_add(&self.0, &o.0, CW))
    }
    fn sub(self, o: V) -> V {
        V(rx::ext_sub(&self.0, &o.0, CW))
    }
    fn mul(self, o: V) -> V {
        V(rx::ext_mul(&self.0, &o.0, CW))
    }
    fn div(self, o: V) -> V {
        V(rx::ext_div(&self.0, &o.0, CW))
    }
    fn ln(self, stored: bool) -> V {
        // x87 fldln2+fyl2x; `stored` models the CRT double return.
        V(rx::ext_fyl2x(&rx::ext_ln2(), &self.0, CW)).store(stored)
    }
}

fn polevl(x: V, c: &[f64], step_store: bool) -> V {
    let mut r = V::new(c[0]);
    for k in &c[1..] {
        r = r.mul(x).add(V::new(*k)).store(step_store);
    }
    r
}
fn p1evl(x: V, c: &[f64], step_store: bool) -> V {
    let mut r = x.add(V::new(c[0])).store(step_store);
    for k in &c[1..] {
        r = r.mul(x).add(V::new(*k)).store(step_store);
    }
    r
}

fn lgam(x0: f64, m: u32) -> f64 {
    let bit = |i: u32| m & (1 << i) != 0;
    if x0 < 13.0 {
        let mut z = V::new(1.0);
        let mut p = 0.0f64;
        let mut u = x0;
        while u >= 3.0 {
            p -= 1.0;
            u = x0 + p;
            z = z.mul(V::new(u)).store(bit(0));
        }
        while u < 2.0 {
            z = z.div(V::new(u)).store(bit(0));
            p += 1.0;
            u = x0 + p;
        }
        let z = V(rx::ext_abs(&z.0, CW));
        if u == 2.0 {
            return z.ln(true).f();
        }
        let x = x0 + (p - 2.0);
        let xv = V::new(x);
        let num = polevl(xv, &B, bit(2));
        let den = p1evl(xv, &C, bit(2));
        let q = xv.mul(num).div(den).store(bit(3));
        return z.ln(bit(1)).add(q).f();
    }
    let xv = V::new(x0);
    let lnx = xv.ln(bit(7));
    let q0 = xv
        .sub(V::new(0.5))
        .mul(lnx)
        .sub(xv)
        .add(V::new(LS2PI))
        .store(bit(4));
    if x0 > 1.0e8 {
        return q0.f();
    }
    let p = V::new(1.0).div(xv.mul(xv)).store(bit(5));
    let t = if x0 >= 1000.0 {
        V::new(7.9365079365079365079365e-4)
            .mul(p)
            .sub(V::new(2.7777777777777777777778e-3))
            .mul(p)
            .add(V::new(0.0833333333333333333333))
            .store(bit(6))
    } else {
        polevl(p, &A, bit(2)).store(bit(6))
    };
    q0.add(t.div(xv)).f()
}

fn main() {
    let files: Vec<String> = std::env::args().skip(1).collect();
    let mut rows: Vec<(f64, f64)> = Vec::new();
    for file in &files {
        let ws: WitnessSet =
            serde_json::from_str(&std::fs::read_to_string(file).expect("read")).expect("parse");
        for w in &ws.witnesses {
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            if let Some(v) = parse_bits_hex(&w.expected_bits) {
                rows.push((x, v));
            }
        }
    }
    let mut best: Vec<(u32, u32, u64)> = Vec::new(); // (mask, exact, max_ulp)
    for m in 0u32..256 {
        let (mut exact, mut max_ulp) = (0u32, 0u64);
        for &(x, want) in &rows {
            let v = lgam(x, m);
            if v.to_bits() == want.to_bits() {
                exact += 1;
            } else {
                max_ulp = max_ulp.max(ulp_distance(v, want).unwrap_or(u64::MAX));
            }
        }
        best.push((m, exact, max_ulp));
    }
    best.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)));
    for (m, exact, max_ulp) in best.iter().take(10) {
        println!("mask {m:08b}: {exact}/{} exact, max_ulp {max_ulp}", rows.len());
    }
}
