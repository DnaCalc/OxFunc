//! W109 Phase-4: pin the IRR NPV kernel from the one-step ladder rows.
//! Every irrA ladder row is a single FD step: v1 = v0 - f(v0)*h/(f(v0+h)-f(v0)),
//! published as r = 1/v1 - 1. Race NPV forms x h forms x small staging bits.

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
        V {
            e: rx::ext_from_f64(x),
            m,
        }
    }
    fn f(self) -> f64 {
        rx::ext_to_f64(&self.e, CW)
    }
    fn op(self) -> V {
        match self.m {
            Stage::Plain => V::new(self.f(), self.m),
            _ => self,
        }
    }
    fn st(self) -> V {
        V::new(self.f(), self.m)
    }
    fn add(self, o: V) -> V {
        V {
            e: rx::ext_add(&self.e, &o.e, CW),
            m: self.m,
        }
        .op()
    }
    fn sub(self, o: V) -> V {
        V {
            e: rx::ext_sub(&self.e, &o.e, CW),
            m: self.m,
        }
        .op()
    }
    fn mul(self, o: V) -> V {
        V {
            e: rx::ext_mul(&self.e, &o.e, CW),
            m: self.m,
        }
        .op()
    }
    fn div(self, o: V) -> V {
        V {
            e: rx::ext_div(&self.e, &o.e, CW),
            m: self.m,
        }
        .op()
    }
}

fn npv(cf: &[f64], v: f64, m: Stage, form: u8) -> f64 {
    let vv = V::new(v, m);
    match form {
        0 => {
            // seq forward: t*=v each term
            let mut s = V::new(cf[0], m);
            let mut t = V::new(1.0, m);
            for c in &cf[1..] {
                t = t.mul(vv).st();
                s = s.add(V::new(*c, m).mul(t)).st();
            }
            s.f()
        }
        1 => {
            // horner in v
            let mut s = V::new(*cf.last().unwrap(), m);
            for c in cf[..cf.len() - 1].iter().rev() {
                s = s.mul(vv).add(V::new(*c, m)).st();
            }
            s.f()
        }
        2 => {
            // sum tail then add cf0 last
            let mut s = V::new(0.0, m);
            let mut t = V::new(1.0, m);
            for c in &cf[1..] {
                t = t.mul(vv).st();
                s = s.add(V::new(*c, m).mul(t)).st();
            }
            s.add(V::new(cf[0], m)).st().f()
        }
        3 => {
            // divide by (1+r)-style: terms as c*t with t = t/one_plus_r... here t*v equivalent but division staging
            // t_i = t_{i-1} / (1/v): model as repeated division by w = 1/v stored
            let w = V::new(1.0, m).div(vv).st();
            let mut s = V::new(cf[0], m);
            let mut t = V::new(1.0, m);
            for c in &cf[1..] {
                t = t.div(w).st();
                s = s.add(V::new(*c, m).mul(t)).st();
            }
            s.f()
        }
        _ => {
            // reverse seq: accumulate from last cashflow down
            let mut s = V::new(0.0, m);
            for (i, c) in cf.iter().enumerate().rev() {
                if i == 0 {
                    s = s.add(V::new(*c, m)).st();
                    continue;
                }
                let mut t = V::new(1.0, m);
                for _ in 0..i {
                    t = t.mul(vv).st();
                }
                s = s.add(V::new(*c, m).mul(t)).st();
            }
            s.f()
        }
    }
}

fn one_step(cf: &[f64], guess: f64, m: Stage, form: u8, hform: u8) -> f64 {
    let v0 = V::new(1.0, m)
        .div(V::new(1.0, m).add(V::new(guess, m)))
        .st()
        .f();
    let f0 = npv(cf, v0, m, form);
    if f0 == 0.0 {
        return guess;
    }
    let h = match hform {
        0 => V::new(1e-6, m).mul(V::new(v0, m)).st().f(),
        1 => V::new(v0, m).div(V::new(1e6, m)).st().f(),
        2 => V::new(v0, m).mul(V::new(1e-6, m)).st().f(),
        _ => 1e-6, // absolute
    };
    let vh = V::new(v0, m).add(V::new(h, m)).st().f();
    let f1 = npv(cf, vh, m, form);
    let den = V::new(f1, m).sub(V::new(f0, m)).st().f();
    if den == 0.0 {
        return guess;
    }
    let dv = V::new(f0, m).mul(V::new(h, m)).div(V::new(den, m)).st().f();
    let v1 = V::new(v0, m).sub(V::new(dv, m)).st().f();
    V::new(1.0, m)
        .div(V::new(v1, m))
        .st()
        .sub(V::new(1.0, m))
        .st()
        .f()
}

fn main() {
    let files = [
        "../../work/w109/G6-solvers/answers-irr-r1.json",
        "../../work/w109/G6-solvers/answers-irr-r2.json",
    ];
    let mut obs: Vec<(Vec<f64>, f64, u64, String)> = Vec::new();
    for f in files {
        let ws: WitnessSet =
            serde_json::from_str(&std::fs::read_to_string(f).expect("read")).expect("parse");
        for w in &ws.witnesses {
            let id = w.id.clone().unwrap_or_default();
            if !id.starts_with("irrA") {
                continue;
            }
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
    println!("{} irrA ladder rows (one-step regime)", obs.len());
    let mut results: Vec<(u32, String)> = Vec::new();
    for m in [Stage::Plain, Stage::Spill] {
        for form in 0u8..5 {
            for hform in 0u8..4 {
                let sc = obs
                    .iter()
                    .filter(|(cf, g, want, _)| one_step(cf, *g, m, form, hform).to_bits() == *want)
                    .count() as u32;
                let sn = if m == Stage::Plain { "plain" } else { "spill" };
                results.push((sc, format!("{sn}/form{form}/h{hform}")));
            }
        }
    }
    results.sort_by(|a, b| b.0.cmp(&a.0));
    for (sc, key) in results.iter().take(12) {
        println!("{sc:3}/{}  {key}", obs.len());
    }
}
