//! W109 G3-01 *INV lane: race inverter variants against the b14 corpora.
//!
//! agentC verdict: Excel publishes FULLY-CONVERGED near-CR roots of its own
//! forward (DCDFLIB gaminv early-stop schedule ruled out). The production
//! `bisect_inverse` is an early-stop bisection (4*EPS relative width) — this
//! racer measures what full float-lattice convergence buys on the captured
//! corpora, and which publication rule Excel's converged root matches:
//!
//!   V0 early-stop bisect (production baseline)
//!   V1 lattice bisection to adjacent doubles, publish hi (first f(x) >= p)
//!   V2 lattice bisection, publish lo
//!   V3 lattice bisection, publish the endpoint with smaller |f(x)-p| (tie: hi)
//!
//! CHIINV additionally races tail staging: invert P at 1-p (production) vs
//! invert Q directly at p (Excel CHIDIST is the Q surface).
//!
//! Usage: check_inv <work-dir>

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::functions::special_math_common::{
    bisect_inverse, regularized_beta, regularized_gamma_p, regularized_gamma_q,
};

fn ulp_key(x: f64) -> i64 {
    let i = x.to_bits() as i64;
    if i < 0 { !i } else { i }
}

/// Bisection on the float lattice of [lo, hi] (both >= 0) until lo and hi are
/// adjacent doubles, maintaining f(lo) < p <= f(hi). Returns (lo, hi).
fn lattice_bisect(p: f64, mut lo: f64, mut hi: f64, f: &dyn Fn(f64) -> f64) -> (f64, f64) {
    let mut lob = lo.to_bits();
    let mut hib = hi.to_bits();
    while hib - lob > 1 {
        let midb = lob + (hib - lob) / 2;
        let mid = f64::from_bits(midb);
        if f(mid) >= p {
            hib = midb;
        } else {
            lob = midb;
        }
    }
    lo = f64::from_bits(lob);
    hi = f64::from_bits(hib);
    (lo, hi)
}

fn publish(p: f64, lo: f64, hi: f64, f: &dyn Fn(f64) -> f64, rule: u8) -> f64 {
    match rule {
        1 => hi,
        2 => lo,
        _ => {
            let dl = (f(lo) - p).abs();
            let dh = (f(hi) - p).abs();
            if dl < dh { lo } else { hi }
        }
    }
}

struct Variant {
    name: &'static str,
    rule: u8, // 0 = early-stop baseline, else publication rule for lattice
}

const VARIANTS: [Variant; 4] = [
    Variant { name: "V0-earlystop-hi", rule: 0 },
    Variant { name: "V1-lattice-hi", rule: 1 },
    Variant { name: "V2-lattice-lo", rule: 2 },
    Variant { name: "V3-lattice-closest", rule: 3 },
];

fn run_variant(p: f64, lo: f64, hi: f64, f: &dyn Fn(f64) -> f64, rule: u8) -> f64 {
    if rule == 0 {
        return bisect_inverse(p, lo, hi, f);
    }
    let (l, h) = lattice_bisect(p, lo, hi, f);
    publish(p, l, h, f, rule)
}

fn score(
    label: &str,
    rows: &[(String, Vec<f64>, f64)],
    make: &dyn Fn(&[f64]) -> (f64, f64, f64, Box<dyn Fn(f64) -> f64>),
) {
    println!("\n== {label} ({} rows) ==", rows.len());
    for v in &VARIANTS {
        let mut exact = 0usize;
        let mut worst: i64 = 0;
        let mut miss: Vec<(String, i64)> = Vec::new();
        for (id, args, expected) in rows {
            let (p, lo, hi) = {
                let (p, lo, hi, _) = make(args);
                (p, lo, hi)
            };
            let (_, _, _, f) = make(args);
            let x = run_variant(p, lo, hi, f.as_ref(), v.rule);
            let d = ulp_key(x) - ulp_key(*expected);
            if d == 0 {
                exact += 1;
            } else {
                if d.abs() > worst.abs() {
                    worst = d;
                }
                if miss.len() < 12 {
                    miss.push((id.clone(), d));
                }
            }
        }
        print!("  {:20} {:4}/{} worst {:+}", v.name, exact, rows.len(), worst);
        if exact < rows.len() {
            print!("  miss: ");
            for (id, d) in &miss {
                print!("{d:+}@{id} ");
            }
        }
        println!();
    }
}

fn load(dir: &str, name: &str) -> Vec<(String, Vec<f64>, f64)> {
    let Ok(txt) = std::fs::read_to_string(format!("{dir}/{name}")) else {
        println!("(missing {name})");
        return Vec::new();
    };
    let ws: WitnessSet = serde_json::from_str(&txt).unwrap();
    let mut rows = Vec::new();
    for w in &ws.witnesses {
        let id = w.id.clone().unwrap_or_default();
        let mut args = Vec::new();
        let mut ok = true;
        for a in &w.args {
            match a {
                WitnessArg::Scalar(s) => match parse_bits_hex(s) {
                    Some(v) => args.push(v),
                    None => ok = false,
                },
                _ => ok = false,
            }
        }
        let Some(e) = parse_bits_hex(&w.expected_bits) else {
            continue;
        };
        if ok {
            rows.push((id, args, e));
        }
    }
    rows
}

fn main() {
    let dir = std::env::args().nth(1).expect("work dir");

    // GAMMA.INV(p, alpha, beta): invert P(alpha, x/beta) over x in [0, hi].
    let gi = load(&dir, "answers-b14-gammainv.json");
    score("GAMMA.INV x-space", &gi, &|a: &[f64]| {
        let (p, alpha, beta) = (a[0], a[1], a[2]);
        let hi = beta * (alpha + 10.0 * alpha.sqrt() + 10.0).max(1.0);
        (
            p,
            0.0,
            hi,
            Box::new(move |x: f64| regularized_gamma_p(alpha, x / beta)) as Box<dyn Fn(f64) -> f64>,
        )
    });
    // z-space staging: invert P(alpha, z), publish beta*z. Distinguishable from
    // x-space only when beta != 1.
    score("GAMMA.INV z-space (beta*z)", &gi, &|a: &[f64]| {
        let (p, alpha, beta) = (a[0], a[1], a[2]);
        let hi = (alpha + 10.0 * alpha.sqrt() + 10.0).max(1.0);
        (
            p,
            0.0,
            hi,
            Box::new(move |z: f64| regularized_gamma_p(alpha, z)) as Box<dyn Fn(f64) -> f64>,
        )
    });
    // NOTE: z-space rows are scored after multiplying by beta.
    // (When beta == 1 the two spaces coincide bit-for-bit.)

    // CHIINV(p, df) is the RIGHT-tail inverse: production inverts P at 1-p.
    let ci = load(&dir, "answers-b14-chiinv.json");
    score("CHIINV via P at 1-p", &ci, &|a: &[f64]| {
        let (p, df) = (a[0], a[1].trunc());
        let q = 1.0 - p;
        let hi = 2.0 * ((df / 2.0) + 10.0 * (df / 2.0).sqrt() + 10.0).max(1.0);
        (
            q,
            0.0,
            hi,
            Box::new(move |x: f64| regularized_gamma_p(df / 2.0, x / 2.0)) as Box<dyn Fn(f64) -> f64>,
        )
    });
    // Q-direct staging: find x with Q(x) <= p (Q decreasing; use -Q to keep the
    // increasing-forward convention: invert -Q at -p).
    score("CHIINV via Q direct", &ci, &|a: &[f64]| {
        let (p, df) = (a[0], a[1].trunc());
        let hi = 2.0 * ((df / 2.0) + 10.0 * (df / 2.0).sqrt() + 10.0).max(1.0);
        (
            -p,
            0.0,
            hi,
            Box::new(move |x: f64| -regularized_gamma_q(df / 2.0, x / 2.0)) as Box<dyn Fn(f64) -> f64>,
        )
    });

    // BETAINV(p, alpha, beta[, A, B]): invert I_z(alpha,beta) over z in [0,1],
    // publish A + z*(B-A) (corpus rows are A=0,B=1 unless args say otherwise).
    let bi = load(&dir, "answers-b14-betainv.json");
    score("BETAINV z-space", &bi, &|a: &[f64]| {
        let (p, alpha, beta) = (a[0], a[1], a[2]);
        (
            p,
            0.0,
            1.0,
            Box::new(move |z: f64| regularized_beta(z, alpha, beta)) as Box<dyn Fn(f64) -> f64>,
        )
    });

    // ---------------- b19 HELD-OUT (fresh rows, never raced before) ----------
    println!("\n########## b19 HELD-OUT ##########");

    let gi9 = load(&dir, "answers-b19-gammainv.json");
    score("b19 GAMMA.INV x-space", &gi9, &|a: &[f64]| {
        let (p, alpha, beta) = (a[0], a[1], a[2]);
        let mut hi = beta * (alpha + 10.0 * alpha.sqrt() + 10.0).max(1.0);
        while regularized_gamma_p(alpha, hi / beta) < p {
            hi *= 2.0;
        }
        (
            p,
            0.0,
            hi,
            Box::new(move |x: f64| regularized_gamma_p(alpha, x / beta)) as Box<dyn Fn(f64) -> f64>,
        )
    });
    // z-space: invert P(alpha, z), publish beta*z — needs post-scaling, so score
    // it manually here rather than through score().
    {
        let mut exact = 0usize;
        let mut worst: i64 = 0;
        for (_, a, e) in &gi9 {
            let (p, alpha, beta) = (a[0], a[1], a[2]);
            let mut hi = (alpha + 10.0 * alpha.sqrt() + 10.0).max(1.0);
            while regularized_gamma_p(alpha, hi) < p {
                hi *= 2.0;
            }
            let f = |z: f64| regularized_gamma_p(alpha, z);
            let (l, h) = lattice_bisect(p, 0.0, hi, &f);
            let x = beta * publish(p, l, h, &f, 1);
            let d = ulp_key(x) - ulp_key(*e);
            if d == 0 {
                exact += 1;
            } else if d.abs() > worst.abs() {
                worst = d;
            }
        }
        println!("  z-space beta*z (V1)  {:4}/{} worst {:+}", exact, gi9.len(), worst);
    }

    let ci9 = load(&dir, "answers-b19-chiinv.json");
    score("b19 CHIINV Q-direct (production)", &ci9, &|a: &[f64]| {
        let (p, df) = (a[0], a[1].trunc());
        let mut hi = df.max(1.0);
        while -regularized_gamma_q(df / 2.0, hi / 2.0) < -p {
            hi *= 2.0;
        }
        (
            -p,
            0.0,
            hi,
            Box::new(move |x: f64| -regularized_gamma_q(df / 2.0, x / 2.0)) as Box<dyn Fn(f64) -> f64>,
        )
    });
    score("b19 CHIINV via P at 1-p (old)", &ci9, &|a: &[f64]| {
        let (p, df) = (a[0], a[1].trunc());
        let q = 1.0 - p;
        let mut hi = df.max(1.0);
        while regularized_gamma_p(df / 2.0, hi / 2.0) < q {
            hi *= 2.0;
        }
        (
            q,
            0.0,
            hi,
            Box::new(move |x: f64| regularized_gamma_p(df / 2.0, x / 2.0)) as Box<dyn Fn(f64) -> f64>,
        )
    });

    let cl9 = load(&dir, "answers-b19-chisqinv.json");
    score("b19 CHISQ.INV P-direct (production)", &cl9, &|a: &[f64]| {
        let (p, df) = (a[0], a[1].trunc());
        let mut hi = df.max(1.0);
        while regularized_gamma_p(df / 2.0, hi / 2.0) < p {
            hi *= 2.0;
        }
        (
            p,
            0.0,
            hi,
            Box::new(move |x: f64| regularized_gamma_p(df / 2.0, x / 2.0)) as Box<dyn Fn(f64) -> f64>,
        )
    });

    let bi9 = load(&dir, "answers-b19-betainv.json");
    score("b19 BETAINV z-space", &bi9, &|a: &[f64]| {
        let (p, alpha, beta) = (a[0], a[1], a[2]);
        (
            p,
            0.0,
            1.0,
            Box::new(move |z: f64| regularized_beta(z, alpha, beta)) as Box<dyn Fn(f64) -> f64>,
        )
    });

    // FINV(p, d1, d2) = right-tail inverse. Production: invert CDF at 1-p.
    // Q-direct: FDIST's accurate complement form Q(x) = I_{d2/(d2+d1 x)}(d2/2, d1/2).
    let fi9 = load(&dir, "answers-b19-finv.json");
    score("b19 FINV via CDF at 1-p (production)", &fi9, &|a: &[f64]| {
        let (p, d1, d2) = (a[0], a[1].trunc(), a[2].trunc());
        let q = 1.0 - p;
        let f = move |x: f64| {
            let z = d1 * x / (d1 * x + d2);
            regularized_beta(z, d1 / 2.0, d2 / 2.0)
        };
        let mut hi = 1.0f64;
        while f(hi) < q {
            hi *= 2.0;
        }
        (q, 0.0, hi, Box::new(f) as Box<dyn Fn(f64) -> f64>)
    });
    score("b19 FINV Q-direct complement-form", &fi9, &|a: &[f64]| {
        let (p, d1, d2) = (a[0], a[1].trunc(), a[2].trunc());
        let f = move |x: f64| {
            let z = d2 / (d2 + d1 * x);
            -regularized_beta(z, d2 / 2.0, d1 / 2.0)
        };
        let mut hi = 1.0f64;
        while f(hi) < -p {
            hi *= 2.0;
        }
        (-p, 0.0, hi, Box::new(f) as Box<dyn Fn(f64) -> f64>)
    });

    // TINV(p, df) = two-tailed inverse. Production: t_cdf at 1 - p/2.
    // 2t-direct: invert the published two-tail surface I_{v/(v+x^2)}(v/2, 1/2) at p.
    let ti9 = load(&dir, "answers-b19-tinv.json");
    score("b19 TINV via CDF at 1-p/2 (production)", &ti9, &|a: &[f64]| {
        let (p, v) = (a[0], a[1].trunc());
        let q = 1.0 - p / 2.0;
        let f = move |x: f64| {
            let xx = v / (v + x * x);
            let ib = regularized_beta(xx, v / 2.0, 0.5);
            1.0 - 0.5 * ib
        };
        let mut hi = 1.0f64;
        while f(hi) < q {
            hi *= 2.0;
        }
        (q, 0.0, hi, Box::new(f) as Box<dyn Fn(f64) -> f64>)
    });
    score("b19 TINV 2t-direct", &ti9, &|a: &[f64]| {
        let (p, v) = (a[0], a[1].trunc());
        let f = move |x: f64| {
            let xx = v / (v + x * x);
            -regularized_beta(xx, v / 2.0, 0.5)
        };
        let mut hi = 1.0f64;
        while f(hi) < -p {
            hi *= 2.0;
        }
        (-p, 0.0, hi, Box::new(f) as Box<dyn Fn(f64) -> f64>)
    });
}
