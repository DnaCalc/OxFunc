//! W109 Phase-4: bit-exact race of the identified IRR schedule —
//! FD-Newton in v = 1/(1+r), h = 1e-6*v, publish r from final v —
//! over NPV stagings {plain, x87 spill} x {seq, horner} and the
//! remaining detail axes: dv association, v-update staging,
//! publication association, tolerance form, cap, check order.

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
            Stage::Spill => self,
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
    dv_assoc: u8,  // 0: dv=f0*h/den; 1: dv=(f0/den)*h; 2: v1 = v - f0*h/den fused
    pub_assoc: u8, // 0: 1/v - 1; 1: (1-v)/v
}

fn publish(v: f64, m: Stage, pa: u8) -> f64 {
    match pa {
        0 => V::new(1.0, m)
            .div(V::new(v, m))
            .st()
            .sub(V::new(1.0, m))
            .st()
            .f(),
        _ => V::new(1.0, m)
            .sub(V::new(v, m))
            .st()
            .div(V::new(v, m))
            .st()
            .f(),
    }
}

fn sim(cf: &[f64], guess: f64, s: Sched) -> f64 {
    let m = s.stage;
    let mut v = V::new(1.0, m)
        .div(V::new(1.0, m).add(V::new(guess, m)))
        .st()
        .f();
    let mut f0 = npv(cf, v, m, s.horner);
    if f0 == 0.0 {
        return guess;
    }
    for _ in 0..s.cap {
        let h = V::new(s.hrel, m).mul(V::new(v, m)).st().f();
        let vh = V::new(v, m).add(V::new(h, m)).st().f();
        let f1 = npv(cf, vh, m, s.horner);
        let den = V::new(f1, m).sub(V::new(f0, m)).st().f();
        if den == 0.0 {
            break;
        }
        let dv = match s.dv_assoc {
            0 => V::new(f0, m).mul(V::new(h, m)).div(V::new(den, m)).st().f(),
            1 => V::new(f0, m)
                .div(V::new(den, m))
                .st()
                .mul(V::new(h, m))
                .st()
                .f(),
            _ => {
                // fused update: v1 = v - f0*h/den in one extended expr
                let v1 = V::new(v, m)
                    .sub(V::new(f0, m).mul(V::new(h, m)).div(V::new(den, m)))
                    .st()
                    .f();
                let dv_eff = v - v1; // for the tolerance test (plain diff)
                let lim = if s.tol_rel { s.tol * v.abs() } else { s.tol };
                if s.tol > 0.0 && dv_eff.abs() < lim {
                    if s.apply_last {
                        v = v1;
                    }
                    break;
                }
                v = v1;
                f0 = npv(cf, v, m, s.horner);
                if f0 == 0.0 {
                    break;
                }
                continue;
            }
        };
        let lim = if s.tol_rel { s.tol * v.abs() } else { s.tol };
        if s.tol > 0.0 && dv.abs() < lim {
            if s.apply_last {
                v = V::new(v, m).sub(V::new(dv, m)).st().f();
            }
            break;
        }
        v = V::new(v, m).sub(V::new(dv, m)).st().f();
        f0 = npv(cf, v, m, s.horner);
        if f0 == 0.0 {
            break;
        }
    }
    publish(v, m, s.pub_assoc)
}

struct Obs {
    id: String,
    cf: Vec<f64>,
    g: f64,
    want: u64,
}

fn main() {
    let files = [
        "../../work/w109/G6-solvers/answers-irr-r0.json",
        "../../work/w109/G6-solvers/answers-irr-r1.json",
        "../../work/w109/G6-solvers/answers-irr-r2.json",
    ];
    let mut obs: Vec<Obs> = Vec::new();
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
                obs.push(Obs {
                    id: w.id.clone().unwrap_or_default(),
                    cf,
                    g,
                    want: want.to_bits(),
                });
            }
        }
    }
    println!("{} observations", obs.len());

    let mut results: Vec<(u32, Sched, String)> = Vec::new();
    for stage in [Stage::Plain, Stage::Spill] {
        for horner in [false, true] {
            for hrel in [1e-6, -1e-6, 1e-5] {
                for tol in [1e-7, 1e-5, 1e-4] {
                    for tol_rel in [false, true] {
                        for cap in [20u32, 1000] {
                            for apply_last in [true, false] {
                                for dv_assoc in [0u8, 1, 2] {
                                    for pub_assoc in [0u8, 1] {
                                        let s = Sched {
                                            stage,
                                            horner,
                                            hrel,
                                            tol,
                                            tol_rel,
                                            cap,
                                            apply_last,
                                            dv_assoc,
                                            pub_assoc,
                                        };
                                        let sc = obs
                                            .iter()
                                            .filter(|o| sim(&o.cf, o.g, s).to_bits() == o.want)
                                            .count()
                                            as u32;
                                        let sn = if stage == Stage::Plain {
                                            "plain"
                                        } else {
                                            "spill"
                                        };
                                        results.push((sc, s, format!(
                                            "{sn}/{}/h{hrel}/tol{tol}{}/c{cap}/al{}/dv{dv_assoc}/pub{pub_assoc}",
                                            if horner { "horner" } else { "seq" },
                                            if tol_rel { "rel" } else { "" },
                                            apply_last as u8
                                        )));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    results.sort_by(|a, b| b.0.cmp(&a.0));
    for (sc, _, key) in results.iter().take(10) {
        println!("{sc:4}/{}  {key}", obs.len());
    }
    // Miss profile of the winner
    let (_, s, key) = &results[0];
    println!("-- winner {key} misses --");
    let mut cats: std::collections::BTreeMap<String, (u32, u32)> = Default::default();
    for o in &obs {
        let got = sim(&o.cf, o.g, *s);
        let cat = if o.id.contains("l2") || o.id.contains("lad") {
            format!("ladder-{}", if o.cf.len() == 2 { "B" } else { "A" })
        } else {
            format!("sweep-{}", if o.cf.len() == 2 { "B" } else { "A" })
        };
        let e = cats.entry(cat).or_default();
        e.1 += 1;
        if got.to_bits() == o.want {
            e.0 += 1;
        } else if o.id.contains("lad") || o.id.contains("l2") {
            // ladder misses: show offset
            let d = got.to_bits() as i64 - o.want as i64;
            if e.1 - e.0 <= 6 {
                println!("  {} got-want {d:+} ulp", o.id);
            }
        }
    }
    for (c, (ok, n)) in &cats {
        println!("  {c}: {ok}/{n}");
    }
}
