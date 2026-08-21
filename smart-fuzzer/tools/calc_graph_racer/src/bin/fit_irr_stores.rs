//! W109 Phase-4: exhaustive store-mask fit of the IRR one-step chain.
//!
//! The schedule skeleton is identified (FD-Newton in v = 1/(1+r),
//! h = 1e-6*v, publish r = 1/v - 1). The ladder rows are single
//! photographed steps; this fit enumerates every store/extended toggle
//! along the chain, including f0/f1 flowing EXTENDED into the
//! cancellation den = f1 - f0 (inline-loop semantics), which the earlier
//! racers forced to stored doubles.
//!
//! Mask bits (bit=1 -> STORED to binary64; bit=0 -> stays extended):
//!   b0  NPV t-update            b1  NPV term product (c*t / s*v)
//!   b2  NPV accumulator step    b3  f0/f1 on "return" (call semantics)
//!   b4  h = 1e-6*v              b5  vh = v + h
//!   b6  den = f1 - f0           b7  dv numerator (or slope, per assoc)
//!   b8  dv                      b9  v1 = v - dv
//!   b10 reciprocal 1/v1         b11 (1+guess) in the v0 conversion
//! Structural axes: NPV form {seq, horner}; dv association
//!   {(f0*h)/den, f0*(h/den), f0/(den/h)}.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::Ext80;

const CW: u16 = 0x133F;

#[derive(Clone, Copy)]
struct V(Ext80);
impl V {
    fn new(x: f64) -> V {
        V(rx::ext_from_f64(x))
    }
    fn st(self, yes: bool) -> V {
        if yes { V::new(self.f()) } else { self }
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
}

fn npv(cf: &[f64], v: V, m: u32, form: u8) -> V {
    let bit = |i: u32| m & (1 << i) != 0;
    match form {
        0 => {
            // seq multiply in v
            let mut s = V::new(cf[0]);
            let mut t = V::new(1.0);
            for c in &cf[1..] {
                t = t.mul(v).st(bit(0));
                s = s.add(V::new(*c).mul(t).st(bit(1))).st(bit(2));
            }
            s.st(bit(3))
        }
        1 => {
            // horner in v
            let mut s = V::new(*cf.last().unwrap());
            for c in cf[..cf.len() - 1].iter().rev() {
                s = s.mul(v).st(bit(1)).add(V::new(*c)).st(bit(2));
            }
            s.st(bit(3))
        }
        2 => {
            // per-term division by w = 1/v
            let w = V::new(1.0).div(v).st(true);
            let mut s = V::new(cf[0]);
            let mut t = V::new(1.0);
            for c in &cf[1..] {
                t = t.div(w).st(bit(0));
                s = s.add(V::new(*c).mul(t).st(bit(1))).st(bit(2));
            }
            s.st(bit(3))
        }
        _ => {
            // shared NPV(rate) routine: rate = 1/v - 1, w = 1 + rate
            let rate = V::new(1.0).div(v).st(true).sub(V::new(1.0)).st(true);
            let w = V::new(1.0).add(rate).st(true);
            let mut s = V::new(cf[0]);
            let mut t = V::new(1.0);
            for c in &cf[1..] {
                t = t.div(w).st(bit(0));
                s = s.add(V::new(*c).mul(t).st(bit(1))).st(bit(2));
            }
            s.st(bit(3))
        }
    }
}

fn simulate(cf: &[f64], guess: f64, m: u32, form: u8, tol: f64, hneg: bool, assoc: u8) -> f64 {
    let bit = |i: u32| m & (1 << i) != 0;
    let v0 = V::new(1.0)
        .div(V::new(1.0).add(V::new(guess)).st(bit(11)))
        .st(true);
    let mut v = v0;
    let mut f0 = npv(cf, v, m, form);
    if f0.f() == 0.0 {
        return guess;
    }
    let h = if hneg { V::new(-1e-3) } else { V::new(1e-3) }.st(bit(4));
    let mut steps = 0u32;
    for _ in 0..20 {
        let vh = v.add(h).st(bit(5));
        let f1 = npv(cf, vh, m, form);
        let den = f1.sub(f0).st(bit(6));
        if den.f() == 0.0 {
            break;
        }
        let hnum = match assoc {
            0 | 1 | 2 => h,
            _ => vh.sub(v).st(bit(7)), // recomputed effective h
        };
        let dv = match assoc % 3 {
            0 => f0.mul(hnum).st(bit(7)).div(den).st(bit(8)),
            1 => f0.mul(hnum.div(den).st(bit(7))).st(bit(8)),
            _ => f0.div(den.div(hnum).st(bit(7))).st(bit(8)),
        };
        if steps >= 1 && dv.f().abs() < tol {
            v = v.sub(dv).st(bit(9)); // apply-last
            break;
        }
        v = v.sub(dv).st(bit(9));
        steps += 1;
        f0 = npv(cf, v, m, form);
        if f0.f() == 0.0 {
            break;
        }
    }
    V::new(1.0).div(v).st(bit(10)).sub(V::new(1.0)).f()
}

fn main() {
    let files = [
        "../../work/w109/G6-solvers/answers-irr-r0.json",
        "../../work/w109/G6-solvers/answers-irr-r1.json",
        "../../work/w109/G6-solvers/answers-irr-r2.json",
        "../../work/w109/G6-solvers/answers-irr-r3.json",
    ];
    let mut obs: Vec<(Vec<f64>, f64, u64, String)> = Vec::new();
    for f in files {
        let ws: WitnessSet =
            serde_json::from_str(&std::fs::read_to_string(f).expect("read")).expect("parse");
        for w in &ws.witnesses {
            let id = w.id.clone().unwrap_or_default();
            let cf: Vec<f64> = match &w.args[0] {
                WitnessArg::Array(items) => {
                    items.iter().map(|s| parse_bits_hex(s).unwrap()).collect()
                }
                _ => continue,
            };
            let g = match &w.args[1] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            if let Some(want) = parse_bits_hex(&w.expected_bits) {
                obs.push((cf, g, want.to_bits(), id));
            }
        }
    }
    println!("{} rows", obs.len());

    let mut results: Vec<(u32, u32, u8, u8)> = Vec::new(); // (score, mask, form, cfg)
    // cfg: bit0 = h negative; bits1..2 = tol index {1e-7, 1e-8, 1e-9}
    let tols = [1e-7, 1e-8, 1e-9];
    for form in [0u8, 1, 2, 3] {
        for cfg in 0u8..12 {
            let hneg = cfg & 1 != 0;
            let tol = tols[((cfg >> 1) % 3) as usize];
            let assoc = (cfg / 6) as u8;
            for m in 0u32..(1 << 12) {
                let sc = obs
                    .iter()
                    .filter(|(cf, g, want, _)| {
                        simulate(cf, *g, m, form, tol, hneg, assoc).to_bits() == *want
                    })
                    .count() as u32;
                results.push((sc, m, form, cfg));
            }
        }
    }
    results.sort_by(|a, b| b.0.cmp(&a.0));
    for (sc, m, form, cfg) in results.iter().take(12) {
        println!(
            "{sc:3}/{}  mask {m:012b} form{form} hneg{} tol1e-{} assoc{}",
            obs.len(),
            cfg & 1,
            7 + ((cfg >> 1) % 3),
            cfg / 6
        );
    }
    let (_, m, form, cfg) = results[0];
    let sbits_tol = [1e-7, 1e-8, 1e-9][((cfg >> 1) % 3) as usize];
    let sbits_hneg = cfg & 1 != 0;
    let sbits_assoc = (cfg / 6) as u8;
    let _ = sbits_assoc;
    println!("-- winner misses --");
    let mut shown = 0;
    for (cf, g, want, id) in &obs {
        let got = simulate(cf, *g, m, form, sbits_tol, sbits_hneg, sbits_assoc);
        if got.to_bits() != *want && shown < 60 && !id.contains("-g") {
            shown += 1;
            println!(
                "  {id}: got-want {:+} ulp",
                got.to_bits() as i64 - *want as i64
            );
        }
    }
}
