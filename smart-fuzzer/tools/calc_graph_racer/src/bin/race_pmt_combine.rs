//! W109 G6-01 PMT combine (2026-07-21, coordinator). Decide the shared per-(r,n)
//! intermediate: DISCOUNT em (pmt=(pv+fv*v)*r/(tf*em)) vs FORWARD P,q
//! (pmt=-(pv*P+fv)/(tf*q)). Real x87 transcendentals matching agent-P's decode:
//! L=CR log1p(r) stored f64; tau=+/-n*L stored f64; expm1=internal-Kahan; exp=fFEXP.
//! Score PER-PV on the pv-ladder (fv=0/ty=0) — the discriminator that falsified the
//! naive discount combine model-free.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

// transcendentals for (r,n)
struct T {
    v: f64,
    em: f64,
    p: f64,
    q: f64,
    pm1: f64,
}
fn trans(r: f64, n: f64) -> T {
    let l = rx::excel_log1p(r); // CR log1p, stored f64
    let tneg = (-n) * l; // stored f64 (SSE==x87 per agent-P)
    let tpos = n * l;
    let v = rx::excel_exp(tneg); // (1+r)^-n
    let em = rx::excel_expm1_internal(tneg); // (1+r)^-n - 1
    let p = rx::excel_exp(tpos); // (1+r)^n
    let pm1 = rx::excel_expm1_internal(tpos); // (1+r)^n - 1
    let q = pm1 / r; // annuity factor (P-1)/r
    T { v, em, p, q, pm1 }
}

#[derive(Clone, Copy)]
enum F {
    DiscDivRR, // RN(RN(pv*r)/em)               [agent-Q's, known-fail]
    DiscFull,  // (pv+fv*v)*r/(tf*em)  general
    FwdPq,     // -(pv*P+fv)/(tf*q)
    FwdPqRR,   // -RN(pv*P)/q  (fv=0,ty=0)
    FwdPm1,    // -(pv*P+fv)*r/(tf*pm1)
    FwdPm1RR,  // -RN(pv*P)*r/pm1  (fv=0,ty=0)
    Recip,     // -(pv+fv*v)*r*recip, recip=1/(tf*(1-v))  [production order]
}
fn fname(f: F) -> &'static str {
    match f {
        F::DiscDivRR => "disc RN(pv*r)/em",
        F::DiscFull => "disc (pv+fvv)r/(tf em)",
        F::FwdPq => "fwd -(pvP+fv)/(tf q)",
        F::FwdPqRR => "fwd -RN(pvP)/q",
        F::FwdPm1 => "fwd -(pvP+fv)r/(tf pm1)",
        F::FwdPm1RR => "fwd -RN(pvP)r/pm1",
        F::Recip => "recip prod-order",
    }
}

fn combine(f: F, pv: f64, fv: f64, ty: f64, r: f64, t: &T) -> f64 {
    let tf = 1.0 + r * ty;
    match f {
        F::DiscDivRR => (pv * r) / t.em,
        F::DiscFull => {
            let num = pv + fv * t.v;
            (num * r) / (tf * t.em)
        }
        F::FwdPq => -(pv * t.p + fv) / (tf * t.q),
        F::FwdPqRR => -(pv * t.p) / (tf * t.q),
        F::FwdPm1 => -(pv * t.p + fv) * r / (tf * t.pm1),
        F::FwdPm1RR => -(pv * t.p) * r / (tf * t.pm1),
        F::Recip => {
            let num = pv + fv * t.v;
            let recip = 1.0 / (tf * (1.0 - t.v));
            -num * r * recip
        }
    }
}

fn load(path: &str) -> Vec<(f64, f64, f64, f64, f64, u64)> {
    let ws: WitnessSet =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("parse");
    let mut o = Vec::new();
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
            o.push((a[0], a[1], a[2], a[3], a[4], want.to_bits()))
        }
    }
    o
}

fn main() {
    let ladder = load("../../work/w109/G6-solvers/answers-pmt-pvladder.json");
    println!("pv-ladder rows: {}", ladder.len());
    let forms = [
        F::DiscDivRR,
        F::DiscFull,
        F::FwdPq,
        F::FwdPqRR,
        F::FwdPm1,
        F::FwdPm1RR,
        F::Recip,
    ];
    // per-pv match for each form (fv=0/ty=0 ladder)
    for f in forms {
        let mut perpv: BTreeMap<u64, (u32, u32)> = BTreeMap::new();
        for (r, n, pv, fv, ty, want) in &ladder {
            let t = trans(*r, *n);
            let g = combine(f, *pv, *fv, *ty, *r, &t).to_bits();
            let e = perpv.entry(pv.to_bits()).or_insert((0, 0));
            if g == *want { e.0 += 1 } else { e.1 += 1 }
        }
        print!("{:26}:", fname(f));
        for (pvb, (ok, off)) in &perpv {
            print!(
                "  pv{:.3}={:3}%",
                f64::from_bits(*pvb),
                100 * ok / (ok + off)
            );
        }
        println!();
    }
    // full per-form total on ladder
    println!("\ntotals on pv-ladder:");
    for f in forms {
        let ok = ladder
            .iter()
            .filter(|(r, n, pv, fv, ty, want)| {
                let t = trans(*r, *n);
                combine(f, *pv, *fv, *ty, *r, &t).to_bits() == *want
            })
            .count();
        println!(
            "  {:26} {}/{} ({:.1}%)",
            fname(f),
            ok,
            ladder.len(),
            100.0 * ok as f64 / ladder.len() as f64
        );
    }
}
