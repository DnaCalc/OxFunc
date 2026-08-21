//! W109 G6-01 PMT H-DF combine (2026-07-21, Fable breakthrough). The combine is
//! QUOTIENT-FIRST, *rate LAST (VB financial lineage): pmt = RN(RN(num/den)*r),
//! num=pv+fv*(1+em), den=em*tf, tf=1+r*type. Confirmed 256/256 on every
//! consecutive-pv sweep. Remaining unknown = em bit-exact. This bin scores H-DF
//! with the REAL x87 internal-Kahan em on the full corpora, per Fable's 4 spill
//! variants {em f64 / em extended} x {ops RN53 / x87 RN64->RN53}.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;

#[derive(Clone, Copy)]
struct Var {
    ext_q: bool,
    x87_ops: bool,
}

fn kahan_log1p(r: f64, mode: u8) -> f64 {
    let u = 1.0 + r;
    if u == 1.0 {
        return r;
    }
    let um1 = u - 1.0; // exact (Sterbenz) for |r|<1
    let ln = rx::excel_ln(u); // x87 87tran ln
    match mode {
        0 => rx::excel_log1p(r), // baseline CR log1p
        1 => (ln * r) / um1,     // (ln*r)/(u-1)  left-assoc
        2 => ln * (r / um1),     // ln*(r/(u-1))
        3 => (ln / um1) * r,     // (ln/(u-1))*r
        4 => {
            use rx::{CW_PC64_RN as CW, ext_div, ext_from_f64, ext_mul, ext_to_f64}; // all extended, 1 store
            let q = ext_div(
                &ext_mul(&ext_from_f64(ln), &ext_from_f64(r), CW),
                &ext_from_f64(um1),
                CW,
            );
            ext_to_f64(&q, CW)
        }
        5 => {
            let cr = r / um1;
            ln * cr
        } // corr first (SSE2), = mode2
        _ => rx::excel_log1p(r),
    }
}
fn pmt_hdf_m(r: f64, n: f64, pv: f64, fv: f64, ty: f64, v: Var, l1p: u8) -> f64 {
    if r == 0.0 {
        return -(pv + fv) / n;
    }
    let l = kahan_log1p(r, l1p);
    let tau = -n * l;
    let em = rx::excel_expm1_internal(tau); // (1+r)^-n - 1, x87 internal-Kahan
    let vv = 1.0 + em; // (1+r)^-n  (num uses v=1+em)
    let tf = 1.0 + r * ty;
    let num = pv + fv * vv;
    let den = em * tf;
    if v.x87_ops {
        // x87 double-rounded ops (RN64 then RN53 store), quotient optionally held extended
        use rx::{CW_PC64_RN as CW, ext_div, ext_from_f64, ext_mul, ext_to_f64};
        let qext = ext_div(&ext_from_f64(num), &ext_from_f64(den), CW);
        if v.ext_q {
            // keep quotient extended, multiply by r extended, single store
            ext_to_f64(&ext_mul(&qext, &ext_from_f64(r), CW), CW)
        } else {
            let qd = ext_to_f64(&qext, CW); // store quotient to f64
            rx::x87_mul(qd, r) // RN53(RN64(qd*r))
        }
    } else {
        // plain SSE2 ops
        let q = num / den;
        q * r
    }
}

fn load(path: &str) -> Vec<(Vec<f64>, u64)> {
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
            o.push((a, want.to_bits()))
        }
    }
    o
}
fn sord(u: u64) -> i128 {
    if u < 1 << 63 {
        u as i128
    } else {
        ((1u128 << 63) as i128) - (u as i128 - (1i128 << 63))
    }
}

fn main() {
    let corpora = [
        ("heldout", "answers-pmt-heldout.json"),
        ("po2", "answers-pmt-po2.json"),
        ("combsweep", "answers-pmt-combsweep.json"),
        ("em", "answers-pmt-em.json"),
        ("pvladder", "answers-pmt-pvladder.json"),
        ("fvty", "answers-pmt-fvty.json"),
    ];
    let modes = [
        (0u8, "CRlog1p"),
        (1, "(ln*r)/um1"),
        (2, "ln*(r/um1)"),
        (3, "(ln/um1)*r"),
        (4, "ext-1store"),
    ];
    let v = Var {
        ext_q: false,
        x87_ops: false,
    }; // SSE2 body (combine solved as SSE2)
    let loaded: Vec<_> = corpora
        .iter()
        .filter_map(|(cn, cf)| {
            let p = format!("../../work/w109/G6-solvers/{}", cf);
            if std::path::Path::new(&p).exists() {
                Some((*cn, load(&p)))
            } else {
                None
            }
        })
        .collect();
    for (m, mn) in modes {
        print!("log1p={:12}", mn);
        for (cn, rows) in &loaded {
            let mut ex = 0u32;
            for (a, want) in rows {
                let g = pmt_hdf_m(a[0], a[1], a[2], a[3], a[4], v, m).to_bits();
                if (sord(g) - sord(*want)).abs() == 0 {
                    ex += 1;
                }
            }
            print!("  {}:{}/{}", cn, ex, rows.len());
        }
        println!();
    }
}
