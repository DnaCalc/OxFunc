//! W109: race the single-path lgamma — upward recurrence into Stirling —
//! with extended-value handoff and log-accumulation variants.
//!   lgam(x) = stirling(u) - ln(z),  z = x(x+1)...(u-1), u = x+n >= T
//!   or       stirling(u) - [ln(x) + ln(x+1) + ...]  (lnsum variant)

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

/// Stirling, returns UNSTORED extended value. Mask: bit0 ln out, bit1 q0,
/// bit2 q0 per-op, bit3 p, bit4 t, bit5 t/x, bit6 A-Horner steps,
/// bit7 ln input stored.
fn stirling_v(xv: V, x_hint: f64, m: u32) -> V {
    let bit = |i: u32| m & (1 << i) != 0;
    let lnx = xv.ln(bit(7), bit(0));
    let q0 = xv
        .sub(V::new(0.5))
        .store(bit(2))
        .mul(lnx)
        .store(bit(2))
        .sub(xv)
        .store(bit(2))
        .add(V::new(LS2PI))
        .store(bit(1));
    if x_hint > 1.0e8 {
        return q0;
    }
    let p = V::new(1.0).div(xv.mul(xv)).store(bit(3));
    let t = if x_hint >= 1000.0 {
        V::new(7.9365079365079365079365e-4)
            .mul(p)
            .sub(V::new(2.7777777777777777777778e-3))
            .store(bit(6))
            .mul(p)
            .add(V::new(0.0833333333333333333333))
            .store(bit(4))
    } else {
        polevl(p, &A, bit(6)).store(bit(4))
    };
    q0.add(t.div(xv).store(bit(5)))
}

/// rec bits: r0 prod/lnsum step stored, r1 ln input stored, r2 ln output
/// stored, r3 stirling value stored before subtraction, r4 lnsum variant.
fn lgam(x0: f64, t: f64, rec: u32, stir: u32) -> f64 {
    let rbit = |i: u32| rec & (1 << i) != 0;
    if x0 >= t {
        return stirling_v(V::new(x0), x0, stir | 0x80).f();
    }
    if rbit(4) {
        // lnsum: r = stirling(u); r -= ln(x+k) sequentially
        let mut u = x0;
        let mut n = 0;
        while u < t {
            u += 1.0;
            n += 1;
        }
        let mut r = stirling_v(V::new(u), u, stir | 0x80).store(rbit(3));
        let mut w = x0;
        for _ in 0..n {
            r = r.sub(V::new(w).ln(rbit(1), rbit(2))).store(rbit(0));
            w += 1.0;
        }
        r.f()
    } else {
        let mut z = V::new(x0);
        let mut u = x0 + 1.0;
        while u < t {
            z = z.mul(V::new(u)).store(rbit(0));
            u += 1.0;
        }
        let s = stirling_v(V::new(u), u, stir | 0x80).store(rbit(3));
        let lnz = z.ln(rbit(1), rbit(2));
        s.sub(lnz).f()
    }
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

    let mut results: Vec<(f64, u32, u32, u32, u64)> = Vec::new();
    for ti in [12, 13, 14] {
        let t = ti as f64;
        for rec in 0u32..32 {
            for stir in 0u32..256 {
                let (mut exact, mut max_ulp) = (0u32, 0u64);
                for &(x, want) in &rows {
                    let v = lgam(x, t, rec, stir);
                    if v.to_bits() == want.to_bits() {
                        exact += 1;
                    } else {
                        max_ulp = max_ulp.max(ulp_distance(v, want).unwrap_or(u64::MAX));
                    }
                }
                results.push((t, rec, stir, exact, max_ulp));
            }
        }
    }
    results.sort_by(|a, b| b.3.cmp(&a.3).then(a.4.cmp(&b.4)));
    for (t, rec, stir, exact, max_ulp) in results.iter().take(15) {
        println!(
            "T={t} rec={rec:05b} stir={stir:08b}: {exact}/{} max_ulp {max_ulp}",
            rows.len()
        );
    }
    let (t, rec, stir, _, _) = results[0];
    let mut misses = 0;
    for &(x, want) in &rows {
        let v = lgam(x, t, rec, stir);
        if v.to_bits() != want.to_bits() && misses < 25 {
            misses += 1;
            println!(
                "  MISS x={x:.9e} got {:016x} want {:016x} ulp {}",
                v.to_bits(),
                want.to_bits(),
                ulp_distance(v, want).unwrap_or(u64::MAX)
            );
        }
    }
}
