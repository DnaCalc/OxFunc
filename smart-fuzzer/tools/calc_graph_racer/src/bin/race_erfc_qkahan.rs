//! Compensated / double-double Horner of SPECFUN C/D as Q=w*F.
//! Also last-mul two-prod. Scores the 53 hard direct leftovers.
//! Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const C: [f64; 9] = [
    0.564188496988670089,
    8.88314979438837594,
    66.1191906371416295,
    298.635138197400131,
    881.95222124176909,
    1712.04761263407058,
    2051.07837782607147,
    1230.33935479799725,
    2.15311535474403846e-8,
];
const D: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];
const NUM_A: [f64; 9] = [C[7], C[6], C[5], C[4], C[3], C[2], C[1], C[0], C[8]];
const DEN_A: [f64; 9] = [D[7], D[6], D[5], D[4], D[3], D[2], D[1], D[0], 1.0];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn specfun_x87(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(C[7]), CW), &ext_add(&xden, &ef(D[7]), CW), CW),
        CW,
    )
}
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let v = s - a;
    let e = (a - (s - v)) + (b - v);
    (s, e)
}
fn two_prod(a: f64, b: f64) -> (f64, f64) {
    let p = a * b;
    let e = f64::mul_add(a, b, -p);
    (p, e)
}
fn comp_horner(a: &[f64], x: f64) -> f64 {
    let n = a.len();
    let mut s = a[n - 1];
    let mut r = 0.0;
    for i in (0..n - 1).rev() {
        let (p, pi) = two_prod(s, x);
        let (ns, si) = two_sum(p, a[i]);
        s = ns;
        r = f64::mul_add(r, x, pi + si);
    }
    s + r
}
fn specfun_f64(y: f64) -> f64 {
    let ye = y.abs();
    let mut xnum = C[8] * ye;
    let mut xden = ye;
    for i in 0..7 {
        xnum = (xnum + C[i]) * ye;
        xden = (xden + D[i]) * ye;
    }
    (xnum + C[7]) / (xden + D[7])
}
fn specfun_comp_loop(y: f64) -> f64 {
    let ye = y.abs();
    let (mut xnum, mut en) = two_prod(C[8], ye);
    let (mut xden, mut ed) = (ye, 0.0);
    for i in 0..7 {
        let (sn, cn) = two_sum(xnum, C[i]);
        en += cn;
        let (pn, pin) = two_prod(sn, ye);
        xnum = pn;
        en = f64::mul_add(en, ye, pin);
        let (sd, cd) = two_sum(xden, D[i]);
        ed += cd;
        let (pd, pid) = two_prod(sd, ye);
        xden = pd;
        ed = f64::mul_add(ed, ye, pid);
    }
    let (sn, cn) = two_sum(xnum, C[7]);
    let (sd, cd) = two_sum(xden, D[7]);
    (sn + (en + cn)) / (sd + (ed + cd))
}
fn specfun_comp_poly(y: f64) -> f64 {
    let ye = y.abs();
    comp_horner(&NUM_A, ye) / comp_horner(&DEN_A, ye)
}
fn specfun_dd_div(y: f64) -> f64 {
    let ye = y.abs();
    let n = comp_horner(&NUM_A, ye);
    let d = comp_horner(&DEN_A, ye);
    n / d
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn qw_twoprod(z: f64, ff: f64) -> f64 {
    let w = f::w_rn53(z);
    let (p, e) = two_prod(w, ff);
    let v = p + e;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn qw_fma0(z: f64, ff: f64) -> f64 {
    let v = f64::mul_add(f::w_rn53(z), ff, 0.0);
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut hard = Vec::new();
    let mut exacts = Vec::new();
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let qg = qw(r.z, specfun_x87(r.z));
        let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(99);
        if d == 0 {
            exacts.push(*r);
        } else if d >= 2 {
            hard.push(*r);
        }
    }
    println!("CR direct exacts {} hard {}", exacts.len(), hard.len());

    type Ev = fn(f64) -> f64;
    let graphs: [(&str, Ev); 7] = [
        ("x87 SPECFUN w*F", |z| qw(z, specfun_x87(z))),
        ("f64 SPECFUN w*F", |z| qw(z, specfun_f64(z))),
        ("comp fused-loop w*F", |z| qw(z, specfun_comp_loop(z))),
        ("comp poly N/D w*F", |z| qw(z, specfun_comp_poly(z))),
        ("comp poly + two-prod mul", |z| qw_twoprod(z, specfun_dd_div(z))),
        ("x87 F + two-prod mul", |z| qw_twoprod(z, specfun_x87(z))),
        ("x87 F + fma(w,F,0)", |z| qw_fma0(z, specfun_x87(z))),
    ];
    for (name, ev) in graphs {
        let mut mid_ex = 0usize;
        let mut mid_n = 0usize;
        let mut dmid = 0usize;
        let mut dn = 0usize;
        let mut max_d = 0u64;
        let mut max_a = 0u64;
        let mut keep = 0usize;
        let mut hit = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let qg = ev(r.z);
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            mid_n += 1;
            max_a = max_a.max(d);
            if d == 0 {
                mid_ex += 1;
            }
            if r.direct {
                dn += 1;
                max_d = max_d.max(d);
                if d == 0 {
                    dmid += 1;
                }
            }
        }
        for r in &exacts {
            let qg = ev(r.z);
            if ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                keep += 1;
            }
        }
        for r in &hard {
            let qg = ev(r.z);
            if ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                hit += 1;
            }
        }
        println!(
            "{name:26} {mid_ex}/{mid_n} dmid={dmid}/{dn} maxd={max_d} maxa={max_a} keep={keep}/{} hit={hit}/{}",
            exacts.len(),
            hard.len()
        );
    }

    println!("## direct signed-ULP runs (x87 SPECFUN)");
    let mut prev_s = 0i8;
    let mut run = 0usize;
    let mut nchg = 0usize;
    let mut runs = Vec::new();
    let mut nz = 0usize;
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let qg = qw(r.z, specfun_x87(r.z));
        let qo = f64::from_bits(r.qbits);
        let d = ulp_distance(qg, qo).unwrap_or(0);
        if d == 0 {
            if run > 0 {
                runs.push(run);
                run = 0;
            }
            prev_s = 0;
            continue;
        }
        nz += 1;
        let s = if qg > qo { 1i8 } else { -1 };
        if s == prev_s {
            run += 1;
        } else {
            if run > 0 {
                runs.push(run);
            }
            if prev_s != 0 {
                nchg += 1;
            }
            run = 1;
            prev_s = s;
        }
    }
    if run > 0 {
        runs.push(run);
    }
    runs.sort_unstable();
    let med = if runs.is_empty() {
        0
    } else {
        runs[runs.len() / 2]
    };
    let mx = runs.iter().copied().max().unwrap_or(0);
    println!(
        "direct nonzero {} sign_changes {} runs {} median {} max {}",
        nz,
        nchg,
        runs.len(),
        med,
        mx
    );
}
