//! Pin Excel's exact (1+r)^-n factor used in PMT. fv=+1 ty=0 is invF-sensitive;
//! fv=-1 ty=0 is not. Sweep invF candidates; score disc_b combine per group.
use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::power_fn::power_kernel;
use serde_json::Value;
use std::collections::BTreeMap;

const CW: u16 = rx::CW_PC64_RN;
fn e(x: f64) -> rx::Ext80 {
    rx::ext_from_f64(x)
}
fn tf64(x: &rx::Ext80) -> f64 {
    rx::ext_to_f64(x, CW)
}
fn xadd(a: f64, b: f64) -> f64 {
    tf64(&rx::ext_add(&e(a), &e(b), CW))
}
fn xmul(a: f64, b: f64) -> f64 {
    tf64(&rx::ext_mul(&e(a), &e(b), CW))
}
fn binexp_sse(base: f64, mut n: u64) -> f64 {
    let mut a = 1.0;
    let mut b = base;
    while n > 0 {
        if n & 1 == 1 {
            a *= b;
        }
        n >>= 1;
        if n > 0 {
            b *= b;
        }
    }
    a
}
fn binexp_x87(base: f64, mut n: u64) -> f64 {
    let mut a = 1.0;
    let mut b = base;
    while n > 0 {
        if n & 1 == 1 {
            a = xmul(a, b);
        }
        n >>= 1;
        if n > 0 {
            b = xmul(b, b);
        }
    }
    a
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
        let g = |i: usize| {
            f64::from_bits(
                u64::from_str_radix(a[i].as_str().unwrap().trim_start_matches("0x"), 16).unwrap(),
            )
        };
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

// candidate invF = (1+r)^-n
fn invf_candidates(r: f64, n: f64) -> Vec<(&'static str, f64)> {
    let base = xadd(1.0, r);
    let base_sse = 1.0 + r;
    let log1p = rx::excel_log1p(r);
    let lnb = rx::excel_ln(base); // x87 ln of rounded 1+r
    vec![
        ("exp(-n*log1p)", rx::excel_exp(-(n * log1p))),
        ("1/exp(n*log1p)", 1.0 / rx::excel_exp(n * log1p)),
        (
            "recipX87 exp(n*l1p)",
            rx::x87_recip(rx::excel_exp(n * log1p)),
        ),
        ("exp(-n*lnx87(1+r))", rx::excel_exp(-(n * lnb))),
        ("1/binexpSSE(1+r)", 1.0 / binexp_sse(base_sse, n as u64)),
        (
            "recipX87 binSSE",
            rx::x87_recip(binexp_sse(base_sse, n as u64)),
        ),
        ("1/binexpX87(1+r)", 1.0 / binexp_x87(base, n as u64)),
        ("recipX87 binX87", rx::x87_recip(binexp_x87(base, n as u64))),
        ("1/power_kernel", 1.0 / power_kernel(base, n).unwrap()),
        (
            "recipX87 powkern",
            rx::x87_recip(power_kernel(base, n).unwrap()),
        ),
        ("binexpSSE(1/(1+r))", binexp_sse(1.0 / base_sse, n as u64)),
        (
            "binexpX87(recip)",
            binexp_x87(rx::x87_recip(base), n as u64),
        ),
    ]
}

// disc_b combine: (num/em)/tf*r, num=pv+fv*invF, em=invF-1, all x87 double-rounded
fn xsub(a: f64, b: f64) -> f64 {
    tf64(&rx::ext_sub(&e(a), &e(b), CW))
}
fn xdiv(a: f64, b: f64) -> f64 {
    tf64(&rx::ext_div(&e(a), &e(b), CW))
}
fn disc_b_x87(invf: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let tf = if ty == 0.0 {
        1.0
    } else {
        xadd(1.0, xmul(r, ty))
    };
    let em = xsub(invf, 1.0);
    let num = xadd(pv, xmul(fv, invf));
    xmul(xdiv(xdiv(num, em), tf), r)
}
fn disc_b_sse(invf: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let tf = if ty == 0.0 { 1.0 } else { 1.0 + r * ty };
    let em = invf - 1.0;
    let num = pv + fv * invf;
    (num / em) / tf * r
}

fn main() {
    let res = load("../../work/w109/G6-solvers/answers-pmt-fv1sweep.json");
    let big = load("../../work/w109/G6-solvers/answers-pmt-fvsweep.json");
    let res360: Vec<Row> = res
        .iter()
        .cloned()
        .filter(|r| (r.n - 360.0).abs() < 0.5)
        .collect();
    let big360: Vec<Row> = big
        .iter()
        .cloned()
        .filter(|r| (r.n - 360.0).abs() < 0.5)
        .collect();
    let r0 = res360[0].r;
    let n0 = res360[0].n;
    println!("rate={} n={}", r0, n0);
    for (nm, iv) in invf_candidates(r0, n0) {
        println!("  invF {:22} = 0x{:016x}", nm, iv.to_bits());
    }
    let groups = [
        ("fv-1 ty0", -1.0, 0.0),
        ("fv-1 ty1", -1.0, 1.0),
        ("fv1 ty0", 1.0, 0.0),
        ("fv1 ty1", 1.0, 1.0),
    ];
    println!("\n=== disc_b(x87) scores per invF candidate ===");
    for combo in ["x87", "sse"] {
        println!("-- ops {} --", combo);
        for (nm, _) in invf_candidates(r0, n0) {
            let mut line = format!("  {:22}", nm);
            let mut tot = 0;
            for (gn, fv, ty) in groups {
                let mut ok = 0;
                let mut cnt = 0;
                for row in res360.iter().filter(|r| r.fv == fv && r.ty == ty) {
                    let iv = invf_candidates(row.r, row.n)
                        .into_iter()
                        .find(|(x, _)| *x == nm)
                        .unwrap()
                        .1;
                    let got = if combo == "x87" {
                        disc_b_x87(iv, row.r, row.pv, row.fv, row.ty)
                    } else {
                        disc_b_sse(iv, row.r, row.pv, row.fv, row.ty)
                    };
                    if got.to_bits() == row.want {
                        ok += 1;
                    }
                    cnt += 1;
                }
                let _ = gn;
                line.push_str(&format!(" {}:{:3}", gn, ok));
                tot += ok;
            }
            println!("{}  tot={}", line, tot);
        }
    }

    // For the fvsweep constant (fv=1000), pin invF: invert pmt = disc_b at pv=1.0.
    // Print which candidate best matches the constant + distinct-got.
    println!("\n=== fvsweep fv=1000 match per invF (x87) ===");
    for (nm, _) in invf_candidates(r0, n0) {
        for ty in [0.0f64, 1.0] {
            let rows: Vec<&Row> = big360
                .iter()
                .filter(|r| r.ty == ty && r.fv == 1000.0)
                .collect();
            let mut hist: BTreeMap<i64, usize> = BTreeMap::new();
            let mut ok = 0;
            for row in &rows {
                let iv = invf_candidates(row.r, row.n)
                    .into_iter()
                    .find(|(x, _)| *x == nm)
                    .unwrap()
                    .1;
                let got = disc_b_x87(iv, row.r, row.pv, row.fv, row.ty);
                let d = got.to_bits() as i64 - row.want as i64;
                *hist.entry(d).or_default() += 1;
                if d == 0 {
                    ok += 1;
                }
            }
            println!(
                "  {:22} ty={}: match={}/{} hist={:?}",
                nm,
                ty,
                ok,
                rows.len(),
                hist
            );
        }
    }
}
