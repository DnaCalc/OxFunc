//! agentR: large-|fv| combine op-order on the ACCURATE-em subset (r=0.008,n=360).
//! Isolate fvsweep n=360 rows (|tau|>=1 so em=u-1 is exact), reconstruct Excel's
//! exact u=exp(tau)/em via the x87 backend, and race every combine op-graph +
//! num-formation. Report per-variant match counts (ty=0 / ty=1 / total).
use oxfunc_core::excel_numeric::research as rx;
use serde_json::Value;

const CW: u16 = rx::CW_PC64_RN;
fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}
fn e(x: f64) -> rx::Ext80 {
    rx::ext_from_f64(x)
}
fn tf64(x: &rx::Ext80) -> f64 {
    rx::ext_to_f64(x, CW)
}

// x87 double-rounded scalar ops: RN53(RN64(a op b)).
fn xadd(a: f64, b: f64) -> f64 {
    tf64(&rx::ext_add(&e(a), &e(b), CW))
}
fn xsub(a: f64, b: f64) -> f64 {
    tf64(&rx::ext_sub(&e(a), &e(b), CW))
}
fn xmul(a: f64, b: f64) -> f64 {
    tf64(&rx::ext_mul(&e(a), &e(b), CW))
}
fn xdiv(a: f64, b: f64) -> f64 {
    tf64(&rx::ext_div(&e(a), &e(b), CW))
}

#[derive(Clone, Copy)]
struct Row {
    r: f64,
    n: f64,
    pv: f64,
    fv: f64,
    ty: f64,
    want: u64,
}

fn load(path: &str) -> Vec<Row> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut o = Vec::new();
    for w in v["witnesses"].as_array().unwrap() {
        let a = w["args"].as_array().unwrap();
        let g = |i: usize| fb(a[i].as_str().unwrap());
        o.push(Row {
            r: g(0),
            n: g(1),
            pv: g(2),
            fv: g(3),
            ty: g(4),
            want: u64::from_str_radix(
                w["expected_bits"]
                    .as_str()
                    .unwrap()
                    .trim_start_matches("0x"),
                16,
            )
            .unwrap(),
        });
    }
    o
}

// op mode
#[derive(Clone, Copy, PartialEq)]
enum M {
    Sse,
    X87,
}
fn mul(m: M, a: f64, b: f64) -> f64 {
    match m {
        M::Sse => a * b,
        M::X87 => xmul(a, b),
    }
}
fn div(m: M, a: f64, b: f64) -> f64 {
    match m {
        M::Sse => a / b,
        M::X87 => xdiv(a, b),
    }
}
fn add(m: M, a: f64, b: f64) -> f64 {
    match m {
        M::Sse => a + b,
        M::X87 => xadd(a, b),
    }
}
fn sub(m: M, a: f64, b: f64) -> f64 {
    match m {
        M::Sse => a - b,
        M::X87 => xsub(a, b),
    }
}

// tau stagings -> (u, em) with em = u-1 branch (|tau|>=1). Returns (u, em_sse, em_x87).
fn u_em(tau_mode: u8, r: f64, n: f64) -> (f64, f64, f64) {
    let l1p = |r: f64| -> f64 {
        match tau_mode {
            0 => rx::excel_log1p(r),    // CR log1p (double)
            1 => rx::excel_ln(1.0 + r), // x87 ln of rounded 1+r
            2 => rx::excel_log1p(r),    // CR log1p, x87 mul for tau (below)
            3 => rx::excel_ln(1.0 + r),
            _ => rx::excel_log1p(r),
        }
    };
    let lp = l1p(r);
    let tau = match tau_mode {
        0 | 1 => -n * lp,      // SSE mul
        2 | 3 => xmul(-n, lp), // x87 DR mul
        _ => -n * lp,
    };
    let u = rx::excel_exp(tau);
    (u, u - 1.0, xsub(u, 1.0))
}

fn main() {
    let all = load("../../work/w109/G6-solvers/answers-pmt-fvsweep.json");
    // accurate-em subset: n==360 (r=0.008). Split by ty.
    let sub: Vec<Row> = all
        .iter()
        .cloned()
        .filter(|r| (r.n - 360.0).abs() < 0.5)
        .collect();
    let n0 = sub.iter().filter(|r| r.ty == 0.0).count();
    let n1 = sub.iter().filter(|r| r.ty == 1.0).count();
    println!(
        "accurate-em subset: {} rows (ty0={}, ty1={}), rate={}",
        sub.len(),
        n0,
        n1,
        sub[0].r
    );

    // Pin u/em for the (single) rate/n and print candidate values.
    let r = sub[0].r;
    let n = sub[0].n;
    for tm in 0..4u8 {
        let (u, em0, emx) = u_em(tm, r, n);
        println!(
            "  tau_mode={}: u=0x{:016x} em_sse=0x{:016x} em_x87=0x{:016x} 1+em=0x{:016x}",
            tm,
            u.to_bits(),
            em0.to_bits(),
            emx.to_bits(),
            (1.0 + em0).to_bits()
        );
    }

    // Combine forms: given (num, em, tf, r, mode) -> pmt.  pmt = num*r/(tf*em).
    // A/B/C are op orderings; each op rounded per `mode`.
    let forms: Vec<(&str, fn(M, f64, f64, f64, f64) -> f64)> = vec![
        ("A1 (num/em)/tf*r ", |m, num, em, tf, r| {
            mul(m, div(m, div(m, num, em), tf), r)
        }),
        ("A2 (num/em)*r/tf ", |m, num, em, tf, r| {
            div(m, mul(m, div(m, num, em), r), tf)
        }),
        ("B1 (num*r)/em/tf ", |m, num, em, tf, r| {
            div(m, div(m, mul(m, num, r), em), tf)
        }),
        ("B2 (num*r)/(em*tf)", |m, num, em, tf, r| {
            div(m, mul(m, num, r), mul(m, em, tf))
        }),
        ("B3 (num*r)/tf/em ", |m, num, em, tf, r| {
            div(m, div(m, mul(m, num, r), tf), em)
        }),
        ("C1 num/(em*tf)*r ", |m, num, em, tf, r| {
            mul(m, div(m, num, mul(m, em, tf)), r)
        }),
        ("C2 (num/tf)/em*r ", |m, num, em, tf, r| {
            mul(m, div(m, div(m, num, tf), em), r)
        }),
        ("C3 (num/tf)*r/em ", |m, num, em, tf, r| {
            div(m, mul(m, div(m, num, tf), r), em)
        }),
    ];

    // num formations: given pv, fv, v -> num
    // returns (label, num)
    fn num_forms(pv: f64, fv: f64, v: f64) -> Vec<(&'static str, f64)> {
        vec![
            ("sse pv+fv*v ", pv + fv * v),
            ("fma(fv,v,pv)", fv.mul_add(v, pv)),
            ("x87 pv+fv*v ", xadd(pv, xmul(fv, v))),
            ("sse fv*v+pv ", fv * v + pv), // == pv+fv*v (commutative) sanity
        ]
    }

    // tf formations
    let tf_of = |m: M, r: f64, ty: f64| -> f64 {
        if ty == 0.0 {
            1.0
        } else {
            add(m, 1.0, mul(m, r, ty))
        }
    };

    // Full sweep
    println!(
        "\n=== FULL SWEEP (match counts ty0/ty1/total out of {}/{}/{}) ===",
        n0,
        n1,
        sub.len()
    );
    let mut best: Vec<(String, usize, usize)> = Vec::new();
    for tm in 0..4u8 {
        let (u, em_sse, em_x87) = u_em(tm, r, n);
        for &(vlabel, vsel) in &[("v=u", 0u8), ("v=1+em_sse", 1), ("v=x87(1+em)", 2)] {
            for &(emlabel, emsel) in &[("em_sse", 0u8), ("em_x87", 1)] {
                for nf_idx in 0..4usize {
                    for &(cmode, cml) in &[(M::Sse, "sse"), (M::X87, "x87")] {
                        for &(flabel, ff) in &forms {
                            let mut ok0 = 0usize;
                            let mut ok1 = 0usize;
                            for row in &sub {
                                let em = if emsel == 0 { em_sse } else { em_x87 };
                                let v = match vsel {
                                    0 => u,
                                    1 => 1.0 + em_sse,
                                    _ => xadd(1.0, em_sse),
                                };
                                let num = num_forms(row.pv, row.fv, v)[nf_idx].1;
                                let tf = tf_of(cmode, row.r, row.ty);
                                let got = ff(cmode, num, em, tf, row.r).to_bits();
                                if got == row.want {
                                    if row.ty == 0.0 { ok0 += 1 } else { ok1 += 1 }
                                }
                            }
                            let tot = ok0 + ok1;
                            {
                                let nf_l = num_forms(1.0, 1.0, 1.0)[nf_idx].0;
                                let label = format!(
                                    "tm{} {:11} {:6} num[{}] {:3} {} : {}/{}",
                                    tm, vlabel, emlabel, nf_l, cml, flabel, ok0, ok1
                                );
                                best.push((label, tot, ok1));
                            }
                        }
                    }
                }
            }
        }
    }
    best.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));
    for (l, tot, _) in best.iter().take(40) {
        println!("  [{:4}] {}", tot, l);
    }

    // Residual histogram for the canonical candidate: tm0, v=u, em_sse, num sse, sse A1.
    let (u, em_sse, _) = u_em(0, r, n);
    println!("\n=== residual hist: tm0 v=u num=sse(pv+fv*u) sse A1 (num/em)/tf*r ===");
    for ty_pick in [0.0f64, 1.0] {
        use std::collections::BTreeMap;
        let mut hist: BTreeMap<i64, usize> = BTreeMap::new();
        for row in sub.iter().filter(|r| r.ty == ty_pick) {
            let v = u;
            let num = row.pv + row.fv * v;
            let tf = if row.ty == 0.0 {
                1.0
            } else {
                1.0 + row.r * row.ty
            };
            let got = ((num / em_sse) / tf * row.r).to_bits();
            let d = got as i64 - row.want as i64;
            *hist.entry(d).or_default() += 1;
        }
        println!("  ty={}: {:?}", ty_pick, hist);
    }

    // What v would Excel need? Invert ty=0 rows: pmt=RN(RN(num/em)*r), solve num.
    // For a few rows print want, and num/em/pmt under v=u to see direction.
    println!("\n=== row-level probe (ty=0, first 4 + last 2) ===");
    let rows0: Vec<&Row> = sub.iter().filter(|r| r.ty == 0.0).collect();
    for &row in rows0.iter().take(4).chain(rows0.iter().rev().take(2)) {
        let v = u;
        let num = row.pv + row.fv * v;
        let got = (num / em_sse) * row.r;
        println!(
            "  pv=0x{:016x} num=0x{:016x} want=0x{:016x} got(v=u)=0x{:016x} d={}",
            row.pv.to_bits(),
            num.to_bits(),
            row.want,
            got.to_bits(),
            got.to_bits() as i64 - row.want as i64
        );
    }
}
