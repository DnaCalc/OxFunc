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
    fn neg(self) -> V {
        V::new(0.0).sub(self)
    }
}

// 2^t via f2xm1 + fscale, extended. Returns (P, P-1) where P-1 is the ACCURATE
// expm1: for |t|<1 (k=0) it is F2XM1(t) directly (no P then minus-1
// cancellation); otherwise P-1 = scale(1+f2xm1(f),k) - 1. Used by the
// log-domain pow forms; `expm1_pm1` selects the accurate P-1 vs plain P.sub(1).
fn two_pow2(t: V, store_t: bool, expm1_pm1: bool) -> (V, V) {
    let t = t.st(store_t);
    let k = rx::ext_rndint(&t.0, CW);
    let f = rx::ext_sub(&t.0, &k, CW);
    // f2xm1 needs |f| in [0,1]; here |f|<=0.5 by construction of rndint.
    let neg = rx::ext_to_f64(&f, CW) < 0.0;
    let wv = rx::ext_f2xm1(&rx::ext_abs(&f, CW), CW); // 2^|f| - 1
    let mut fm1 = wv; // 2^f - 1
    if neg {
        // 2^(-|f|)-1 = -(2^|f|-1)/2^|f|
        let two_f = rx::ext_add(&wv, &rx::ext_one(), CW);
        fm1 = rx::ext_div(&rx::ext_sub(&rx::ext_from_f64(0.0), &wv, CW), &two_f, CW);
    }
    let p2f = rx::ext_add(&fm1, &rx::ext_one(), CW); // 2^f
    let p = V(rx::ext_scale(&p2f, &k, CW)); // 2^t
    let kf = rx::ext_to_f64(&k, CW);
    let pm1 = if expm1_pm1 && kf == 0.0 {
        V(fm1) // exact expm1: 2^t - 1 with t = f (k==0)
    } else {
        p.sub(V::new(1.0))
    };
    (p, pm1)
}

// FYL2XP1 is only valid for |rate| <= 1 - sqrt(1/2) ~ 0.29289; outside that
// Excel must fall back to fyl2x(1+rate).
const FYL2XP1_DOM: f64 = 0.292893218813452476;

// Returns (P, P-1). For the log-domain pforms 4/5 the P-1 is the accurate
// F2XM1-based expm1 (killing the small-rate cancellation); pform 3 keeps P.sub(1).
fn pow_pm1(w: V, rate: f64, n: u64, m: u32, pform: u8) -> (V, V) {
    let bit = |i: u32| m & (1 << i) != 0;
    let mul_pm1 = |p: V| (p, p.sub(V::new(1.0)));
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
            mul_pm1(p)
        }
        1 => {
            let mut p = w;
            for _ in 1..n {
                p = p.mul(w).st(bit(0));
            }
            mul_pm1(p)
        }
        2 => mul_pm1(V::new(rx::excel_pow_positive(w.f(), n as f64))),
        3 => {
            // fused x87 pow chain via fyl2x(1+rate): P = 2^(n*log2 w); plain P-1
            let t = V(rx::ext_fyl2x(&rx::ext_from_f64(n as f64), &w.0, CW));
            two_pow2(t, bit(0), false)
        }
        4 => {
            // log1p provider: P = 2^(n*log2(1+rate)) via FYL2XP1(rate) directly;
            // P-1 via F2XM1 (accurate expm1 for small rate).
            let t = V(rx::ext_fyl2xp1(
                &rx::ext_from_f64(n as f64),
                &rx::ext_from_f64(rate),
                CW,
            ));
            two_pow2(t, bit(0), true)
        }
        _ => {
            // branched provider: FYL2XP1 in-domain (with expm1 P-1), else fyl2x.
            if rate.abs() <= FYL2XP1_DOM {
                let t = V(rx::ext_fyl2xp1(
                    &rx::ext_from_f64(n as f64),
                    &rx::ext_from_f64(rate),
                    CW,
                ));
                two_pow2(t, bit(0), true)
            } else {
                let t = V(rx::ext_fyl2x(&rx::ext_from_f64(n as f64), &w.0, CW));
                two_pow2(t, bit(0), false)
            }
        }
    }
}

fn pmt(rate: f64, n: f64, pv: f64, fvv: f64, ty: f64, m: u32, pform: u8, comp: u8) -> f64 {
    let bit = |i: u32| m & (1 << i) != 0;
    if rate == 0.0 {
        return -(pv + fvv) / n;
    }
    let w = V::new(1.0).add(V::new(rate)).st(bit(1));
    let (p_raw, pm1_raw) = pow_pm1(w, rate, n as u64, m, pform);
    let p = p_raw.st(bit(2));
    let pm1 = pm1_raw.st(bit(3));
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
            num.mul(V::new(rate))
                .st(bit(4))
                .div(den)
                .st(bit(7))
                .neg()
                .f()
        }
        4 => {
            // via reciprocal: -(pv + fv/P)*(rate*P/(P-1))/tf
            let vn = V::new(fvv).div(p).st(bit(4));
            let num = V::new(pv).add(vn).st(bit(6));
            let fac = V::new(rate).mul(p).div(pm1).st(bit(5));
            num.mul(fac).div(tf).st(bit(7)).neg().f()
        }
        5 => {
            // PV-side dual: -(pv + fv/P)/(tf*(1 - 1/P)/rate)
            let pinv = V::new(1.0).div(p).st(bit(4));
            let apvf = V::new(1.0)
                .sub(pinv)
                .st(bit(3))
                .div(V::new(rate))
                .st(bit(5));
            let num = V::new(pv).add(V::new(fvv).mul(pinv).st(bit(6))).st(bit(6));
            num.div(tf.mul(apvf).st(bit(5))).st(bit(7)).neg().f()
        }
        6 => {
            // den = (tf/rate)*(P-1)  [tf/rate grouped first]
            let num = V::new(pv).mul(p).add(V::new(fvv)).st(bit(6));
            let tr = tf.div(V::new(rate)).st(bit(4));
            let den = tr.mul(pm1).st(bit(5));
            num.div(den).st(bit(7)).neg().f()
        }
        7 => {
            // seq: (num/(P-1))/tf*rate
            let num = V::new(pv).mul(p).add(V::new(fvv)).st(bit(6));
            let a = num.div(pm1).st(bit(4)).div(tf).st(bit(5));
            a.mul(V::new(rate)).st(bit(7)).neg().f()
        }
        8 => {
            // seq: (num*rate)/tf/(P-1)
            let num = V::new(pv).mul(p).add(V::new(fvv)).st(bit(6));
            let a = num.mul(V::new(rate)).st(bit(4)).div(tf).st(bit(5));
            a.div(pm1).st(bit(7)).neg().f()
        }
        _ => {
            // (num/tf)*(rate/(P-1))
            let num = V::new(pv).mul(p).add(V::new(fvv)).st(bit(6));
            let a = num.div(tf).st(bit(4));
            let b = V::new(rate).div(pm1).st(bit(5));
            a.mul(b).st(bit(7)).neg().f()
        }
    }
}

fn load_obs(path: &str) -> Vec<(Vec<f64>, u64)> {
    let ws: WitnessSet =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("parse");
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
    obs
}

fn score(obs: &[(Vec<f64>, u64)], m: u32, pform: u8, comp: u8) -> u32 {
    obs.iter()
        .filter(|(a, want)| pmt(a[0], a[1], a[2], a[3], a[4], m, pform, comp).to_bits() == *want)
        .count() as u32
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let train_path = argv
        .get(1)
        .cloned()
        .unwrap_or_else(|| "../../work/w109/G6-solvers/answers-pmt-r0.json".to_string());
    let heldout_path = argv.get(2).cloned();
    let obs = load_obs(&train_path);
    println!("{} PMT train rows ({train_path})", obs.len());
    // held-out validation mode: fit on train, RANK by held-out score (overfit killer)
    if let Some(hp) = heldout_path {
        let ho = load_obs(&hp);
        println!("{} PMT held-out rows ({hp})", ho.len());
        let mut all: Vec<(u32, u32, u8, u8)> = Vec::new();
        for pform in 0u8..6 {
            for comp in 0u8..10 {
                for m in 0u32..(1 << 9) {
                    all.push((score(&obs, m, pform, comp), m, pform, comp));
                }
            }
        }
        let best_train = all.iter().map(|r| r.0).max().unwrap();
        // rank ALL candidates by held-out -> the model family's ceiling on fresh data
        let mut cand: Vec<(u32, u32, u32, u8, u8)> = all
            .iter()
            .map(|&(tr, m, pf, cp)| (score(&ho, m, pf, cp), tr, m, pf, cp))
            .collect();
        cand.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
        println!("top by HELD-OUT over ALL candidates (best_train={best_train}):");
        for (h, tr, m, pf, cp) in cand.iter().take(16) {
            println!(
                "  held {h:3}/{}  train {tr:2}/{}  pform{pf} comp{cp} mask {m:09b}",
                ho.len(),
                obs.len()
            );
        }
        let (_, _, m, pf, cp) = cand[0];
        println!("BEST-ON-HELDOUT pform{pf} comp{cp} mask {m:09b}:");
        for (a, want) in &ho {
            let got = pmt(a[0], a[1], a[2], a[3], a[4], m, pf, cp);
            if got.to_bits() != *want {
                println!(
                    "  MISS rate={:.6e} n={} pv={} fv={} ty={} {:+} ulp",
                    a[0],
                    a[1],
                    a[2],
                    a[3],
                    a[4],
                    got.to_bits() as i64 - *want as i64
                );
            }
        }
        return;
    }
    let mut results: Vec<(u32, u32, u8, u8)> = Vec::new();
    for pform in 0u8..6 {
        for comp in 0u8..10 {
            for m in 0u32..(1 << 9) {
                let sc = score(&obs, m, pform, comp);
                results.push((sc, m, pform, comp));
            }
        }
    }
    results.sort_by(|a, b| b.0.cmp(&a.0));
    for (sc, m, pform, comp) in results.iter().take(10) {
        println!("{sc:3}/{}  pform{pform} comp{comp} mask {m:09b}", obs.len());
    }
    let (best_sc, m, pform, comp) = results[0];
    println!(
        "CHAMPION {best_sc}/{}  pform{pform} comp{comp} mask {m:09b}",
        obs.len()
    );
    for (a, want) in &obs {
        let got = pmt(a[0], a[1], a[2], a[3], a[4], m, pform, comp);
        if got.to_bits() != *want {
            println!(
                "  MISS rate={:.6e} n={} pv={} fv={} ty={} {:+} ulp",
                a[0],
                a[1],
                a[2],
                a[3],
                a[4],
                got.to_bits() as i64 - *want as i64
            );
        }
    }
}
