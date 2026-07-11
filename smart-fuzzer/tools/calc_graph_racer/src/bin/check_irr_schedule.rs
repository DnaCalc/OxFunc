//! W109 Phase-4: bit-exact race of the identified IRR schedule —
//! FD-Newton in v = 1/(1+r), h = 1e-6*v, publish r = 1/v - 1 —
//! over NPV stagings {plain, x87 spill} x {seq, horner} and the
//! remaining schedule detail (h sign/scale, tol, cap, check order).

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::Ext80;

const CW: u16 = 0x133F;

#[derive(Clone, Copy, PartialEq)]
enum Stage {
    Plain,
    Spill,
}

#[derive(Clone, Copy)]
struct V {
    e: Ext80,
    m: Stage,
}
impl V {
    fn new(x: f64, m: Stage) -> V {
        V { e: rx::ext_from_f64(x), m }
    }
    fn f(self) -> f64 {
        rx::ext_to_f64(&self.e, CW)
    }
    fn op(self) -> V {
        match self.m {
            Stage::Plain => V::new(self.f(), self.m),
            Stage::Spill => self,
        }
    }
    fn st(self) -> V {
        V::new(self.f(), self.m)
    }
    fn add(self, o: V) -> V {
        V { e: rx::ext_add(&self.e, &o.e, CW), m: self.m }.op()
    }
    fn sub(self, o: V) -> V {
        V { e: rx::ext_sub(&self.e, &o.e, CW), m: self.m }.op()
    }
    fn mul(self, o: V) -> V {
        V { e: rx::ext_mul(&self.e, &o.e, CW), m: self.m }.op()
    }
    fn div(self, o: V) -> V {
        V { e: rx::ext_div(&self.e, &o.e, CW), m: self.m }.op()
    }
}

/// NPV in v: seq = running power accumulate; horner = Horner in v.
fn npv(cf: &[f64], v: f64, m: Stage, horner: bool) -> f64 {
    let vv = V::new(v, m);
    if horner {
        let mut s = V::new(*cf.last().unwrap(), m);
        for c in cf[..cf.len() - 1].iter().rev() {
            s = s.mul(vv).add(V::new(*c, m)).st();
        }
        s.f()
    } else {
        let mut s = V::new(cf[0], m);
        let mut t = V::new(1.0, m);
        for c in &cf[1..] {
            t = t.mul(vv).st();
            s = s.add(V::new(*c, m).mul(t)).st();
        }
        s.f()
    }
}

#[derive(Clone, Copy)]
struct Sched {
    stage: Stage,
    horner: bool,
    hrel: f64,
    tol: f64,
    tol_rel: bool,
    cap: u32,
    apply_last: bool,
    recheck_zero: bool, // stop when recomputed f == 0 after step
}

fn sim(cf: &[f64], guess: f64, s: Sched) -> f64 {
    let m = s.stage;
    // v = 1/(1+guess)
    let v0 = V::new(1.0, m).div(V::new(1.0, m).add(V::new(guess, m))).st();
    let mut v = v0.f();
    let mut f0 = npv(cf, v, m, s.horner);
    if f0 == 0.0 {
        return guess; // observed passthrough returns guess bits unchanged
    }
    for _ in 0..s.cap {
        let h = V::new(s.hrel, m).mul(V::new(v, m)).st().f();
        let f1 = npv(cf, v + h, m, s.horner);
        let den = f1 - f0;
        if den == 0.0 {
            break;
        }
        // dv = f0*h/den staged
        let dv = V::new(f0, m).mul(V::new(h, m)).div(V::new(den, m)).st().f();
        let lim = if s.tol_rel { s.tol * v.abs() } else { s.tol };
        if s.tol > 0.0 && dv.abs() < lim {
            if s.apply_last {
                v -= dv;
            }
            break;
        }
        v -= dv;
        f0 = npv(cf, v, m, s.horner);
        if s.recheck_zero && f0 == 0.0 {
            break;
        }
    }
    // r = 1/v - 1
    let r = V::new(1.0, m).div(V::new(v, m)).st().sub(V::new(1.0, m)).st();
    r.f()
}

fn main() {
    let files = [
        "../../work/w109/G6-solvers/answers-irr-r0.json",
        "../../work/w109/G6-solvers/answers-irr-r1.json",
        "../../work/w109/G6-solvers/answers-irr-r2.json",
    ];
    let mut obs: Vec<(Vec<f64>, f64, u64)> = Vec::new();
    for f in files {
        let ws: WitnessSet =
            serde_json::from_str(&std::fs::read_to_string(f).expect("read")).expect("parse");
        for w in &ws.witnesses {
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
                obs.push((cf, g, want.to_bits()));
            }
        }
    }
    println!("{} observations", obs.len());

    let mut results: Vec<(u32, String)> = Vec::new();
    for stage in [Stage::Plain, Stage::Spill] {
        for horner in [false, true] {
            for hrel in [1e-6, -1e-6, 1e-5, 1e-7] {
                for tol in [1e-5, 1e-7, 1e-4, 0.0] {
                    for tol_rel in [false, true] {
                        for cap in [20u32, 100, 1000] {
                            for apply_last in [true, false] {
                                for rz in [true, false] {
                                    let s = Sched {
                                        stage, horner, hrel, tol, tol_rel, cap,
                                        apply_last, recheck_zero: rz,
                                    };
                                    let mut sc = 0u32;
                                    for (cf, g, want) in &obs {
                                        let r = sim(cf, *g, s);
                                        if r.to_bits() == *want {
                                            sc += 1;
                                        }
                                    }
                                    let sn = if stage == Stage::Plain { "plain" } else { "spill" };
                                    results.push((sc, format!(
                                        "{sn}/{}/h{hrel}/tol{tol}{}/c{cap}/al{}/rz{}",
                                        if horner { "horner" } else { "seq" },
                                        if tol_rel { "rel" } else { "" },
                                        apply_last as u8, rz as u8
                                    )));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    results.sort_by(|a, b| b.0.cmp(&a.0));
    for (sc, key) in results.iter().take(15) {
        println!("{sc:4}/{}  {key}", obs.len());
    }
}
// (miss profile printed by rerunning the winner in main via env var W109_IRR_MISSES=1)
