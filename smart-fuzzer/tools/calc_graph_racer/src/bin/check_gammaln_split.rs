//! W109: classify live GAMMALN rows by branch family — for each x, does ANY
//! small-path store-mask or ANY Stirling store-mask reproduce the live bits?
//! Locates Excel's actual small/asymptotic split and the surviving masks.

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
    fn new(x: f64) -> V { V(rx::ext_from_f64(x)) }
    fn store(self, yes: bool) -> V {
        if yes { V::new(rx::ext_to_f64(&self.0, CW)) } else { self }
    }
    fn f(self) -> f64 { rx::ext_to_f64(&self.0, CW) }
    fn add(self, o: V) -> V { V(rx::ext_add(&self.0, &o.0, CW)) }
    fn sub(self, o: V) -> V { V(rx::ext_sub(&self.0, &o.0, CW)) }
    fn mul(self, o: V) -> V { V(rx::ext_mul(&self.0, &o.0, CW)) }
    fn div(self, o: V) -> V { V(rx::ext_div(&self.0, &o.0, CW)) }
    fn ln(self, input_stored: bool, output_stored: bool) -> V {
        let arg = self.store(input_stored);
        V(rx::ext_fyl2x(&rx::ext_ln2(), &arg.0, CW)).store(output_stored)
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

/// Small path; toggles b0 z-loop, b1 ln input, b2 ln output, b3 Horner,
/// b4 q store, b5 q intermediate, b6 q association x*(num/den) vs (x*num)/den.
fn small(x0: f64, m: u32) -> f64 {
    let bit = |i: u32| m & (1 << i) != 0;
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
        return z.ln(bit(1), true).f();
    }
    let x = x0 + (p - 2.0);
    let xv = V::new(x);
    let num = polevl(xv, &B, bit(3));
    let den = p1evl(xv, &C, bit(3));
    let q = if bit(6) {
        xv.mul(num.div(den).store(bit(5))).store(bit(4))
    } else {
        xv.mul(num).store(bit(5)).div(den).store(bit(4))
    };
    z.ln(bit(1), bit(2)).add(q).f()
}

/// Stirling; toggles b0 ln out, b1 q0, b2 q0 per-op, b3 p, b4 t, b5 t/x,
/// b6 A-Horner steps.
fn stirling(x0: f64, m: u32) -> f64 {
    let bit = |i: u32| m & (1 << i) != 0;
    let xv = V::new(x0);
    let lnx = xv.ln(false, bit(0));
    let q0 = xv
        .sub(V::new(0.5)).store(bit(2))
        .mul(lnx).store(bit(2))
        .sub(xv).store(bit(2))
        .add(V::new(LS2PI)).store(bit(1));
    if x0 > 1.0e8 {
        return q0.f();
    }
    let p = V::new(1.0).div(xv.mul(xv)).store(bit(3));
    let t = if x0 >= 1000.0 {
        V::new(7.9365079365079365079365e-4)
            .mul(p)
            .sub(V::new(2.7777777777777777777778e-3)).store(bit(6))
            .mul(p)
            .add(V::new(0.0833333333333333333333)).store(bit(4))
    } else {
        polevl(p, &A, bit(6)).store(bit(4))
    };
    q0.add(t.div(xv).store(bit(5))).f()
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
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    rows.dedup_by(|a, b| a.0.to_bits() == b.0.to_bits());
    println!("{} rows", rows.len());

    let mut boundary: Vec<(f64, bool, bool)> = Vec::new(); // (x, small-any, stir-any)
    for &(x, want) in &rows {
        let s_any = (0..128u32).any(|m| small(x, m).to_bits() == want.to_bits());
        let t_any = (0..128u32).any(|m| stirling(x, m).to_bits() == want.to_bits());
        boundary.push((x, s_any, t_any));
    }
    let mut prev = (true, true);
    for &(x, s, t) in &boundary {
        let cur = (s, t);
        if cur != prev {
            println!("  x={x:<12.6} small={} stirling={}", s as u8, t as u8);
            prev = cur;
        }
    }
    let neither: Vec<f64> = boundary.iter().filter(|r| !r.1 && !r.2).map(|r| r.0).collect();
    println!("neither-family rows: {} {:?}", neither.len(), &neither[..neither.len().min(20)]);
    let mut bt = (0.0f64, usize::MAX);
    for t in [9.75f64, 10.0, 10.5, 11.0, 11.5, 12.0, 12.5, 13.0] {
        let bad = boundary
            .iter()
            .filter(|&&(x, s, st)| if x < t { !s } else { !st })
            .count();
        if bad < bt.1 {
            bt = (t, bad);
        }
        println!("threshold {t}: {bad} unexplained");
    }
    let t = bt.0;
    let mut small_masks: Vec<u32> = (0..128).collect();
    let mut stir_masks: Vec<u32> = (0..128).collect();
    for &(x, want) in &rows {
        if x < t {
            small_masks.retain(|&m| small(x, m).to_bits() == want.to_bits());
        } else {
            stir_masks.retain(|&m| stirling(x, m).to_bits() == want.to_bits());
        }
    }
    println!("threshold {t}: surviving small masks {small_masks:?}");
    println!("threshold {t}: surviving stirling masks {stir_masks:?}");
    let sm = small_masks.first().copied();
    let tm = stir_masks.first().copied();
    for &(x, want) in &rows {
        let v = if x < t { sm.map(|m| small(x, m)) } else { tm.map(|m| stirling(x, m)) };
        let fam = if x < t { "small" } else { "stir" };
        if let Some(v) = v {
            if v.to_bits() != want.to_bits() {
                println!(
                    "  MISS {fam} x={x:.9e} got {:016x} want {:016x} ulp {}",
                    v.to_bits(),
                    want.to_bits(),
                    ulp_distance(v, want).unwrap_or(u64::MAX)
                );
            }
        }
    }
}
