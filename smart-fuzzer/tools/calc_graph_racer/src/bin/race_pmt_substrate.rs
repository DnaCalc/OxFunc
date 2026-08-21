//! W109 G6-01 PMT substrate race (2026-07-21). The prior `fit_pmt_stores` zoo
//! raced the RAW BASE-2 hardware chain (`2^(n·log2(1+r))` via fyl2xp1/f2xm1)
//! and hit a ~57% held-out ceiling. This bin races the DISCOUNT form on the
//! NATURAL-base CRT chain instead — `excel_ln`/`excel_exp`/`excel_expm1_internal`,
//! the pow/exp/ln substrate already proven bit-exact in bonds (G6-03d), the G3
//! distribution lane, and worksheet POWER. Body stays plain SSE2 f64; only the
//! transcendentals are x87/CRT. No store-mask fishing — pure provider ID,
//! RANKED BY HELD-OUT (the overfit killer, per the standing rule).
//!
//! PMT(rate, nper, pv, fv, type); rate==0 -> -(pv+fv)/n.
//! Discount identity: em = expm1(-n·log1p(r)); v = 1+em (=(1+r)^-n);
//!   pmt = (pv + fv·v)·r / (tf·em),  tf = 1 + r·type.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

const CW: u16 = rx::CW_PC64_RN;

// ---- natural ln(1+r) providers -------------------------------------------
#[derive(Clone, Copy)]
enum LP {
    Log1pCR,
    LnOf1PlusR,
    Fyl2xp1Nat,
}
fn log1p_of(lp: LP, r: f64) -> f64 {
    match lp {
        LP::Log1pCR => rx::excel_log1p(r), // portable correctly-rounded log1p
        LP::LnOf1PlusR => rx::excel_ln(1.0 + r), // form 1+r (SSE2) then CRT ln (fyl2x)
        // hardware-native natural log1p: ln2 · log2(1+r) in one FYL2XP1 (accurate small-r)
        LP::Fyl2xp1Nat => rx::ext_to_f64(
            &rx::ext_fyl2xp1(&rx::ext_ln2(), &rx::ext_from_f64(r), CW),
            CW,
        ),
    }
}
fn lp_name(lp: LP) -> &'static str {
    match lp {
        LP::Log1pCR => "log1pCR",
        LP::LnOf1PlusR => "ln(1+r)",
        LP::Fyl2xp1Nat => "fyl2xp1",
    }
}

// ---- natural expm1(t) providers ------------------------------------------
#[derive(Clone, Copy)]
enum EP {
    Expm1CR,
    Expm1Internal,
    ExpMinus1,
}
fn expm1_of(ep: EP, t: f64) -> f64 {
    match ep {
        EP::Expm1CR => rx::excel_expm1(t), // portable correctly-rounded expm1
        EP::Expm1Internal => rx::excel_expm1_internal(t), // identified internal (Kahan-corrected exp)
        EP::ExpMinus1 => rx::excel_exp(t) - 1.0, // CRT exp then -1 (cancellation for small t)
    }
}
fn ep_name(ep: EP) -> &'static str {
    match ep {
        EP::Expm1CR => "expm1CR",
        EP::Expm1Internal => "expm1Int",
        EP::ExpMinus1 => "exp-1",
    }
}

// ---- t = -n·L staging ----------------------------------------------------
#[derive(Clone, Copy)]
enum TR {
    X87Mul,
    Sse2,
}
fn t_of(tr: TR, n: f64, l: f64) -> f64 {
    match tr {
        TR::X87Mul => rx::x87_mul(-n, l),
        TR::Sse2 => (-n) * l,
    }
}
fn tr_name(tr: TR) -> &'static str {
    match tr {
        TR::X87Mul => "x87*",
        TR::Sse2 => "sse2*",
    }
}

// ---- outer arrangement ---------------------------------------------------
#[derive(Clone, Copy)]
enum AR {
    NumROverDen,
    NumOverDenR,
    NumROverTfOverEm,
    LibreSplit,
    VIndependent,
}
fn ar_name(ar: AR) -> &'static str {
    match ar {
        AR::NumROverDen => "numR/den",
        AR::NumOverDenR => "num/den*r",
        AR::NumROverTfOverEm => "numR/tf/em",
        AR::LibreSplit => "libreSplit",
        AR::VIndependent => "vIndep",
    }
}

fn model(lp: LP, ep: EP, tr: TR, ar: AR, rate: f64, n: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    if rate == 0.0 {
        return -(pv + fv) / n;
    }
    let l = log1p_of(lp, rate);
    let t = t_of(tr, n, l); // -n·log1p(r)
    let em = expm1_of(ep, t); // (1+r)^-n - 1
    let v = 1.0 + em; // (1+r)^-n
    let tf = 1.0 + rate * ty;
    match ar {
        AR::NumROverDen => {
            let num = pv + fv * v;
            (num * rate) / (tf * em)
        }
        AR::NumOverDenR => {
            let num = pv + fv * v;
            (num / (tf * em)) * rate
        }
        AR::NumROverTfOverEm => {
            let num = pv + fv * v;
            num * rate / tf / em
        }
        AR::VIndependent => {
            // v computed independently as exp(t) (production-style), em separate.
            let vind = rx::excel_exp(t);
            let num = pv + fv * vind;
            (num * rate) / (tf * em)
        }
        AR::LibreSplit => {
            // LibreOffice ScGetPMT: fZw·r/(P-1) + fBw·r/(1-1/P), then /(1+r) if type.
            // P = (1+r)^n forward = exp(n·L); 1-1/P = 1-v = -em.
            let p = rx::excel_exp(t_of(tr, -n, l)); // exp(+n·L)
            let term_fv = fv * rate / (p - 1.0);
            let term_pv = pv * rate / (1.0 - v);
            let tf1 = if ty != 0.0 { 1.0 + rate } else { 1.0 };
            -(term_fv + term_pv) / tf1
        }
    }
}

fn load_obs(path: &str) -> Vec<(Vec<f64>, u64)> {
    let ws: WitnessSet =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("parse");
    let mut obs = Vec::new();
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

fn sord(u: u64) -> i128 {
    if u < 1 << 63 {
        u as i128
    } else {
        ((1u128 << 63) as i128) - (u as i128 - (1i128 << 63))
    }
}

const LPS: [LP; 3] = [LP::Log1pCR, LP::LnOf1PlusR, LP::Fyl2xp1Nat];
const EPS: [EP; 3] = [EP::Expm1CR, EP::Expm1Internal, EP::ExpMinus1];
const TRS: [TR; 2] = [TR::X87Mul, TR::Sse2];
const ARS: [AR; 5] = [
    AR::NumROverDen,
    AR::NumOverDenR,
    AR::NumROverTfOverEm,
    AR::LibreSplit,
    AR::VIndependent,
];

fn score(obs: &[(Vec<f64>, u64)], lp: LP, ep: EP, tr: TR, ar: AR) -> u32 {
    obs.iter()
        .filter(|(a, want)| model(lp, ep, tr, ar, a[0], a[1], a[2], a[3], a[4]).to_bits() == *want)
        .count() as u32
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let train = argv
        .get(1)
        .cloned()
        .unwrap_or_else(|| "../../work/w109/G6-solvers/answers-pmt-r0.json".into());
    let held = argv
        .get(2)
        .cloned()
        .unwrap_or_else(|| "../../work/w109/G6-solvers/answers-pmt-heldout.json".into());
    let tr_obs = load_obs(&train);
    let ho_obs = load_obs(&held);
    println!(
        "train {} rows ({train})  held {} rows ({held})",
        tr_obs.len(),
        ho_obs.len()
    );

    let mut cand: Vec<(u32, u32, LP, EP, TR, AR)> = Vec::new();
    for &lp in &LPS {
        for &ep in &EPS {
            for &tr in &TRS {
                for &ar in &ARS {
                    cand.push((
                        score(&ho_obs, lp, ep, tr, ar),
                        score(&tr_obs, lp, ep, tr, ar),
                        lp,
                        ep,
                        tr,
                        ar,
                    ));
                }
            }
        }
    }
    cand.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
    println!("\nTOP 16 by HELD-OUT ({}):", ho_obs.len());
    for (h, tr_s, lp, ep, tr, ar) in cand.iter().take(16) {
        println!(
            "  held {:4}/{}  train {:3}/{}  L={:8} E={:8} t={:6} arr={}",
            h,
            ho_obs.len(),
            tr_s,
            tr_obs.len(),
            lp_name(*lp),
            ep_name(*ep),
            tr_name(*tr),
            ar_name(*ar)
        );
    }

    // n=1 isolation lane: pure test of expm1(-log1p(r)). Report exact-count per provider.
    println!("\n=== n=1 ISOLATION LANE (pure em provider test) ===");
    let n1: Vec<_> = ho_obs
        .iter()
        .filter(|(a, _)| a[1] == 1.0)
        .cloned()
        .collect();
    let mut n1c: Vec<(u32, LP, EP, TR, AR)> = Vec::new();
    for &lp in &LPS {
        for &ep in &EPS {
            for &tr in &TRS {
                for &ar in &ARS {
                    n1c.push((score(&n1, lp, ep, tr, ar), lp, ep, tr, ar));
                }
            }
        }
    }
    n1c.sort_by(|a, b| b.0.cmp(&a.0));
    println!("n=1 rows: {}", n1.len());
    for (s, lp, ep, tr, ar) in n1c.iter().take(8) {
        println!(
            "  {:4}/{}  L={:8} E={:8} t={:6} arr={}",
            s,
            n1.len(),
            lp_name(*lp),
            ep_name(*ep),
            tr_name(*tr),
            ar_name(*ar)
        );
    }

    // residual structure for the held-out champion
    let (_, _, lp, ep, tr, ar) = cand[0];
    println!(
        "\n=== CHAMPION residual on held-out: L={} E={} t={} arr={} ===",
        lp_name(lp),
        ep_name(ep),
        tr_name(tr),
        ar_name(ar)
    );
    let mut hist: std::collections::BTreeMap<i64, u32> = std::collections::BTreeMap::new();
    let mut by_n: std::collections::BTreeMap<i64, (u32, u32)> = std::collections::BTreeMap::new();
    for (a, want) in &ho_obs {
        let got = model(lp, ep, tr, ar, a[0], a[1], a[2], a[3], a[4]).to_bits();
        let d = (sord(got) - sord(*want)) as i64;
        *hist.entry(d.clamp(-6, 6)).or_insert(0) += 1;
        let e = by_n.entry(a[1] as i64).or_insert((0, 0));
        if d == 0 { e.0 += 1 } else { e.1 += 1 }
    }
    println!("ulp hist (clamped +-6): {:?}", hist);
    println!("by n (exact/off):");
    for (n, (ex, off)) in &by_n {
        println!("  n={:5}  exact {:4}  off {:4}", n, ex, off);
    }
}
