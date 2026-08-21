//! W109 G6-05 RATE (2026-07-21, agent-R schedule + x87 power).
//!
//! CORRECTED SCHEDULE (agent-R, supersedes the earlier secant guess):
//!   RATE is FORWARD-DIFFERENCE NEWTON in r-space -- the r-space sibling of IRR's
//!   v-space FD-Newton. Validated in Python: reproduces the EXACT #NUM! basin on all
//!   116 rate witnesses (rate-B numeric only for guess in [0.02,0.3]; rate-A NUM at
//!   {-0.5,-0.3}) with 0 basin mismatches.
//!     x <- guess
//!     loop (cap ~100; the documented "20" is too small -- rate-B g=0.3 needs ~55
//!           iters, rate-A g5.0 >20, and Excel returns VALUES for those):
//!        f  = balance(x)
//!        h  = 1e-6 * x            (relative FD step; IRR analog was 1e-6*v)
//!        d  = (balance(x+h) - f) / h
//!        if !finite(f|d) || d==0 -> #NUM!
//!        xn = x - f/d
//!        if !finite(xn)          -> #NUM!
//!        if |f| < 1e-7  -> return xn   (stop on RESIDUAL |f|, publish the stepped iterate)
//!        x = xn
//!     cap exceeded / domain(r<=-1) / overflow -> #NUM!
//!   Balance (plain SSE2 double; agent-R proved NOT x87-extended):
//!     y = pv*P + pmt*(1/r+type)*(P-1) + fv,  P=(1+r)^nper ; r=0 limit = pv+pmt*nper+fv.
//!   POWER CONFIRMED x87: X87Chain (exp(nper*ln(1+r)) via real 87tran) gives 15/15 on
//!   the #NUM! basin; BinexpDouble/Powf/BinexpX87 only 10/15. So (1+r)^nper is Excel's
//!   x87 exp*ln chain, NOT plain-double binexp / CRT integer pow. The whole legacy body
//!   is x87. #NUM! basin is 15/15 regardless of Double-vs-Ext80 balance arithmetic.
//!   OPEN (op-graph wall, like GAMMALN/PMT): exact bits still 0-2/101 (w<=32 ~20/101).
//!   Residual = fine op-graph of the balance/FD/step (spill points, op order) + the
//!   knife-edge iterate where |f| first crosses 1e-7 on the not-fully-converged rows.
//!   args=[nper,pmt,pv,fv,type,guess]. Bit-exact per-case incl #NUM!.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

const TOL: f64 = 1e-7;
const CAP: usize = 100;

#[derive(Clone, Copy)]
enum Pow {
    BinexpDouble,
    X87Chain,
    Powf,
    BinexpX87,
    ExcelPow,
}
fn ipow(base: f64, n: f64, m: Pow) -> f64 {
    match m {
        Pow::ExcelPow => {
            if base > 0.0 {
                rx::excel_pow_positive(base, n)
            } else {
                f64::NAN
            }
        }
        Pow::BinexpDouble => {
            if n.fract() == 0.0 && n >= 0.0 && n < 1e9 {
                let mut e = n as u64;
                let mut b = base;
                let mut r = 1.0;
                while e > 0 {
                    if e & 1 == 1 {
                        r *= b;
                    }
                    e >>= 1;
                    if e > 0 {
                        b *= b;
                    }
                }
                r
            } else {
                base.powf(n)
            }
        }
        Pow::BinexpX87 => {
            if n.fract() == 0.0 && n >= 0.0 && n < 1e9 {
                let mut e = n as u64;
                let mut b = base;
                let mut r = 1.0;
                while e > 0 {
                    if e & 1 == 1 {
                        r = rx::x87_mul(r, b);
                    }
                    e >>= 1;
                    if e > 0 {
                        b = rx::x87_mul(b, b);
                    }
                }
                r
            } else {
                base.powf(n)
            }
        }
        Pow::X87Chain => rx::excel_exp(rx::x87_mul(n, rx::excel_ln(base))), // exp(n*ln base) x87 87tran
        Pow::Powf => base.powf(n),
    }
}

#[derive(Clone, Copy)]
enum Arr {
    InvRPlusType,
    OnePlusRType,
}
#[derive(Clone, Copy)]
enum BalMode {
    Double,
    Ext80,
    PerOp,
}

use rx::{
    CW_PC64_RN as CW, Ext80, ext_add, ext_div, ext_from_f64, ext_mul, ext_one, ext_sub, ext_to_f64,
};
fn e(x: f64) -> Ext80 {
    ext_from_f64(x)
}
// per-op double-rounded x87 ops: RN53(RN64(a op b)) -- the XNPV spill-loop model.
fn pa(a: f64, b: f64) -> f64 {
    ext_to_f64(&ext_add(&e(a), &e(b), CW), CW)
}
fn ps(a: f64, b: f64) -> f64 {
    ext_to_f64(&ext_sub(&e(a), &e(b), CW), CW)
}
fn pmul(a: f64, b: f64) -> f64 {
    ext_to_f64(&ext_mul(&e(a), &e(b), CW), CW)
}
fn pdiv(a: f64, b: f64) -> f64 {
    ext_to_f64(&ext_div(&e(a), &e(b), CW), CW)
}

fn balance_perop(r: f64, nper: f64, pmt: f64, pv: f64, fv: f64, ty: f64, pm: Pow, ar: Arr) -> f64 {
    if r == 0.0 {
        return pa(pa(pv, pmul(pmt, nper)), fv);
    }
    let p = ipow(1.0 + r, nper, pm);
    let pm1 = ps(p, 1.0);
    match ar {
        Arr::InvRPlusType => {
            // pv*P + pmt*(1/r+ty)*(P-1) + fv
            let coef = pa(pdiv(1.0, r), ty);
            let term = pmul(pmul(pmt, coef), pm1);
            pa(pa(pmul(pv, p), term), fv)
        }
        Arr::OnePlusRType => {
            // pv*P + pmt*(1+r*ty)*(P-1)/r + fv
            let tf = pa(1.0, pmul(r, ty));
            let num = pmul(pmul(pmt, tf), pm1);
            let term = pdiv(num, r);
            pa(pa(pmul(pv, p), term), fv)
        }
    }
}

// balance in full x87 extended (80-bit), spill only the final y to f64.
// P (the CRT pow call) is an f64 result reloaded to the stack.
fn balance_x87(r: f64, nper: f64, pmt: f64, pv: f64, fv: f64, ty: f64, pm: Pow, ar: Arr) -> f64 {
    if r == 0.0 {
        // pv + pmt*nper + fv
        let t = ext_add(
            &ext_add(&e(pv), &ext_mul(&e(pmt), &e(nper), CW), CW),
            &e(fv),
            CW,
        );
        return ext_to_f64(&t, CW);
    }
    let p = ipow(1.0 + r, nper, pm);
    let (pe, pme, pve, fve, tye, re) = (e(p), e(pmt), e(pv), e(fv), e(ty), e(r));
    let pm1 = ext_sub(&pe, &ext_one(), CW); // P-1
    let out = match ar {
        Arr::InvRPlusType => {
            // pv*P + pmt*(1/r+ty)*(P-1) + fv
            let invr = ext_div(&ext_one(), &re, CW);
            let coef = ext_add(&invr, &tye, CW);
            let term = ext_mul(&ext_mul(&pme, &coef, CW), &pm1, CW);
            ext_add(&ext_add(&ext_mul(&pve, &pe, CW), &term, CW), &fve, CW)
        }
        Arr::OnePlusRType => {
            // pv*P + pmt*(1+r*ty)*(P-1)/r + fv
            let tf = ext_add(&ext_one(), &ext_mul(&re, &tye, CW), CW);
            let num = ext_mul(&ext_mul(&pme, &tf, CW), &pm1, CW);
            let term = ext_div(&num, &re, CW);
            ext_add(&ext_add(&ext_mul(&pve, &pe, CW), &term, CW), &fve, CW)
        }
    };
    ext_to_f64(&out, CW)
}

fn balance(r: f64, nper: f64, pmt: f64, pv: f64, fv: f64, ty: f64, pm: Pow, ar: Arr) -> f64 {
    if r == 0.0 {
        return pv + pmt * nper + fv;
    }
    let p = ipow(1.0 + r, nper, pm);
    match ar {
        Arr::InvRPlusType => pv * p + pmt * (1.0 / r + ty) * (p - 1.0) + fv,
        Arr::OnePlusRType => pv * p + pmt * (1.0 + r * ty) * (p - 1.0) / r + fv,
    }
}

fn bal(
    r: f64,
    nper: f64,
    pmt: f64,
    pv: f64,
    fv: f64,
    ty: f64,
    pm: Pow,
    ar: Arr,
    bm: BalMode,
) -> f64 {
    match bm {
        BalMode::Double => balance(r, nper, pmt, pv, fv, ty, pm, ar),
        BalMode::Ext80 => balance_x87(r, nper, pmt, pv, fv, ty, pm, ar),
        BalMode::PerOp => balance_perop(r, nper, pmt, pv, fv, ty, pm, ar),
    }
}

#[derive(Clone, Copy)]
enum Deriv {
    FdRel1e6,
    FdRel1e7,
    FdAbs1e6,
    Analytic,
}
fn fprime(
    x: f64,
    f: f64,
    nper: f64,
    pmt: f64,
    pv: f64,
    fv: f64,
    ty: f64,
    pm: Pow,
    ar: Arr,
    dv: Deriv,
    bm: BalMode,
) -> f64 {
    match dv {
        Deriv::Analytic => {
            let p = ipow(1.0 + x, nper, pm);
            let pm1 = ipow(1.0 + x, nper - 1.0, pm);
            let dp = nper * pm1;
            pv * dp + pmt * (-1.0 / (x * x) * (p - 1.0) + (1.0 / x + ty) * dp)
        }
        _ => {
            let h = match dv {
                Deriv::FdRel1e6 => {
                    if x != 0.0 {
                        1e-6 * x
                    } else {
                        1e-6
                    }
                }
                Deriv::FdRel1e7 => {
                    if x != 0.0 {
                        1e-7 * x
                    } else {
                        1e-7
                    }
                }
                Deriv::FdAbs1e6 => 1e-6,
                Deriv::Analytic => unreachable!(),
            };
            (bal(x + h, nper, pmt, pv, fv, ty, pm, ar, bm) - f) / h
        }
    }
}

// Excel RATE: forward-difference Newton in r-space. Some(rate) or None(#NUM!).
fn rate_solve(
    nper: f64,
    pmt: f64,
    pv: f64,
    fv: f64,
    ty: f64,
    guess: f64,
    pm: Pow,
    ar: Arr,
    dv: Deriv,
    bm: BalMode,
) -> Option<f64> {
    let mut x = guess;
    for _ in 0..CAP {
        let f = bal(x, nper, pmt, pv, fv, ty, pm, ar, bm);
        let d = fprime(x, f, nper, pmt, pv, fv, ty, pm, ar, dv, bm);
        if !f.is_finite() || !d.is_finite() || d == 0.0 {
            return None;
        }
        let xn = x - f / d;
        if !xn.is_finite() {
            return None;
        }
        if f.abs() < TOL {
            return Some(xn);
        } // stop on residual, publish stepped iterate
        x = xn;
    }
    None
}

fn load(path: &str) -> Vec<(Vec<f64>, Option<u64>, String)> {
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
        if a.len() != 6 {
            continue;
        }
        let want = parse_bits_hex(&w.expected_bits).map(|f| f.to_bits());
        let id = w.id.clone().unwrap_or_default();
        o.push((a, want, id));
    }
    o
}

fn main() {
    let files = ["r0", "r1", "r2", "min", "neg"];
    let mut all = Vec::new();
    for f in files {
        let p = format!("../../work/w109/G6-solvers/answers-rate-{}.json", f);
        if std::path::Path::new(&p).exists() {
            for row in load(&p) {
                all.push((f, row));
            }
        }
    }
    println!("total rate witnesses: {}", all.len());
    let pows = [
        ("binexpDbl", Pow::BinexpDouble),
        ("x87chain", Pow::X87Chain),
        ("powf", Pow::Powf),
        ("binexpX87", Pow::BinexpX87),
        ("excelPow", Pow::ExcelPow),
    ];
    let arrs = [
        ("(1/r+ty)", Arr::InvRPlusType),
        ("(1+r*ty)/r", Arr::OnePlusRType),
    ];
    let derivs = [
        ("fdRel1e6", Deriv::FdRel1e6),
        ("fdRel1e7", Deriv::FdRel1e7),
        ("fdAbs1e6", Deriv::FdAbs1e6),
        ("analytic", Deriv::Analytic),
    ];
    let bals = [
        ("dbl", BalMode::Double),
        ("x87", BalMode::Ext80),
        ("perop", BalMode::PerOp),
    ];
    let mut best = (0i64, String::new());
    for (bn, bm) in bals {
        for (dn, dv) in derivs {
            for (pn, pm) in pows {
                for (an, ar) in arrs {
                    let (mut exact, mut num_ok, mut val_ok, mut numtot, mut valtot, mut w32) =
                        (0, 0, 0, 0, 0, 0);
                    for (_f, (a, want, _id)) in &all {
                        let got = rate_solve(a[0], a[1], a[2], a[3], a[4], a[5], pm, ar, dv, bm);
                        match (got, want) {
                            (None, None) => {
                                num_ok += 1;
                                numtot += 1;
                            }
                            (Some(g), Some(w)) => {
                                valtot += 1;
                                let d = (g.to_bits() as i64 - *w as i64).abs();
                                if d == 0 {
                                    exact += 1;
                                    val_ok += 1;
                                }
                                if d <= 32 {
                                    w32 += 1;
                                }
                            }
                            (Some(_), None) => {
                                numtot += 1;
                            }
                            (None, Some(_)) => {
                                valtot += 1;
                            }
                        }
                    }
                    println!(
                        "bal={:3} deriv={:9} pow={:9} arr={:11}: exact {:3}/{:3}  w<=32 {:3}  #NUM! {:2}/{:2}",
                        bn, dn, pn, an, val_ok, valtot, w32, num_ok, numtot
                    );
                    if exact as i64 > best.0 {
                        best = (exact as i64, format!("bal={}/{}/{}/{}", bn, dn, pn, an));
                    }
                }
            }
        }
    }
    println!("\nbest exact-config: {} ({} exact)", best.1, best.0);

    // detail for x87 balance / fdRel1e6 / x87chain / (1/r+ty): per-case ULP + basin
    for (bn, bm) in [("x87", BalMode::Ext80), ("dbl", BalMode::Double)] {
        println!(
            "\n--- detail: bal={} / fdRel1e6 / x87chain / (1/r+ty) ---",
            bn
        );
        let mut basin_bad = 0;
        let mut shown = 0;
        for (_f, (a, want, id)) in &all {
            let got = rate_solve(
                a[0],
                a[1],
                a[2],
                a[3],
                a[4],
                a[5],
                Pow::X87Chain,
                Arr::InvRPlusType,
                Deriv::FdRel1e6,
                bm,
            );
            let m = match (got, want) {
                (Some(g), Some(w)) => {
                    if g.to_bits() == *w {
                        None
                    } else {
                        Some(format!("{:+} ulp", g.to_bits() as i64 - *w as i64))
                    }
                }
                (None, None) => None,
                (Some(_), None) => {
                    basin_bad += 1;
                    Some("BASIN got val want NUM".into())
                }
                (None, Some(_)) => {
                    basin_bad += 1;
                    Some("BASIN got NUM want val".into())
                }
            };
            if let Some(s) = m {
                if shown < 20 {
                    println!("  {:16} {}", id, s);
                    shown += 1;
                }
            }
        }
        println!("basin mismatches: {}", basin_bad);
    }

    // NEAR-ROOT one-step ladder (r1/r2 rows with guess within 1e-7 of the RATE-A root):
    // pure one-step maps (|f|<1e-7 at first iterate) -> isolates balance/FD/step op-graph,
    // no trajectory sensitivity. Find the config that best reproduces these clean rows.
    const ROOT_A: f64 = 0.0346015379965336;
    let near: Vec<_> = all
        .iter()
        .filter(|(_f, (a, _w, _i))| (a[5] - ROOT_A).abs() < 1e-7 && a[0] == 10.0)
        .collect();
    println!("\n=== NEAR-ROOT one-step ladder: {} rows ===", near.len());
    let mut rank = Vec::new();
    for (bn, bm) in [
        ("dbl", BalMode::Double),
        ("x87", BalMode::Ext80),
        ("perop", BalMode::PerOp),
    ] {
        for (dn, dv) in derivs {
            for (pn, pm) in pows {
                for (an, ar) in arrs {
                    let (mut ex, mut w4, mut w32) = (0i32, 0i32, 0i32);
                    for (_f, (a, want, _i)) in &near {
                        if let (Some(g), Some(w)) = (
                            rate_solve(a[0], a[1], a[2], a[3], a[4], a[5], pm, ar, dv, bm),
                            want,
                        ) {
                            let d = (g.to_bits() as i64 - *w as i64).abs();
                            if d == 0 {
                                ex += 1;
                            }
                            if d <= 4 {
                                w4 += 1;
                            }
                            if d <= 32 {
                                w32 += 1;
                            }
                        }
                    }
                    rank.push((ex, w4, w32, format!("{}/{}/{}/{}", bn, dn, pn, an)));
                }
            }
        }
    }
    rank.sort_by(|a, b| (b.0, b.1, b.2).cmp(&(a.0, a.1, a.2)));
    for (ex, w4, w32, name) in rank.iter().take(12) {
        println!(
            "  near-root exact {:2}  w<=4 {:2}  w<=32 {:2}   {}",
            ex, w4, w32, name
        );
    }

    // DECOMPOSE near-root residual: is it in f, f', or the step?
    // Use x87chain/(1/r+ty)/fdRel1e6/dbl. Print my_f vs implied_f=(guess-excel_out)*my_d.
    println!("\n--- near-root decomposition (x87chain/(1/r+ty)/fdRel1e6/dbl) ---");
    println!(
        "  {:>6} {:>8} {:>8} {:>8}  {:>14} {:>14} {:>8}",
        "guessΔ", "exOut-g", "myOut-g", "out-ex", "my_f", "implied_f", "fΔulp"
    );
    let (pm, ar, dv, bm) = (
        Pow::X87Chain,
        Arr::InvRPlusType,
        Deriv::FdRel1e6,
        BalMode::Double,
    );
    let root_bits = ROOT_A.to_bits() as i64;
    for (_f, (a, want, _i)) in near.iter().take(24) {
        let g = a[5];
        let gb = g.to_bits() as i64;
        let f = bal(g, a[0], a[1], a[2], a[3], a[4], pm, ar, bm);
        let d = fprime(g, f, a[0], a[1], a[2], a[3], a[4], pm, ar, dv, bm);
        let myout = g - f / d;
        if let Some(w) = want {
            let exout = f64::from_bits(*w);
            let implied_f = (g - exout) * d;
            let fdulp = (f.to_bits() as i64) - (implied_f.to_bits() as i64);
            println!(
                "  {:>6} {:>8} {:>8} {:>8}  {:>14.4e} {:>14.4e} {:>8}",
                gb - root_bits,
                *w as i64 - gb,
                myout.to_bits() as i64 - gb,
                myout.to_bits() as i64 - *w as i64,
                f,
                implied_f,
                fdulp
            );
        }
    }
}
