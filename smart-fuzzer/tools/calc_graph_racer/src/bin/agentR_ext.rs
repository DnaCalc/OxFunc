//! Test numerator/combine STAGING (double per-op vs fma vs fully-extended x87
//! register chain) on the fv=+1 resolution group, plus tf handling for ty=1.
use oxfunc_core::excel_numeric::research as rx;
use serde_json::Value;
use std::collections::BTreeMap;

const CW: u16 = rx::CW_PC64_RN;
fn e(x: f64) -> rx::Ext80 {
    rx::ext_from_f64(x)
}
fn t64(x: &rx::Ext80) -> f64 {
    rx::ext_to_f64(x, CW)
}
fn ea(a: &rx::Ext80, b: &rx::Ext80) -> rx::Ext80 {
    rx::ext_add(a, b, CW)
}
fn es(a: &rx::Ext80, b: &rx::Ext80) -> rx::Ext80 {
    rx::ext_sub(a, b, CW)
}
fn em_(a: &rx::Ext80, b: &rx::Ext80) -> rx::Ext80 {
    rx::ext_mul(a, b, CW)
}
fn ed(a: &rx::Ext80, b: &rx::Ext80) -> rx::Ext80 {
    rx::ext_div(a, b, CW)
}
fn one() -> rx::Ext80 {
    rx::ext_one()
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
fn invf(r: f64, n: f64) -> f64 {
    rx::excel_exp(-(n * rx::excel_log1p(r)))
}

// Variant catalog for pmt given (invF, r, pv, fv, ty)
type F = fn(f64, f64, f64, f64, f64) -> f64;

// double per-op (x87 double-rounded scalar), num=pv+fv*invF
fn d_num_dbl(iv: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let tf = if ty == 0.0 {
        one()
    } else {
        ea(&one(), &em_(&e(r), &e(ty)))
    };
    let em = es(&e(iv), &one());
    let num = ea(&e(pv), &em_(&e(fv), &e(iv)));
    // stage each to double then re-lift (per-op double)
    let numd = e(t64(&num));
    let emd = e(t64(&em));
    let tfd = e(t64(&tf));
    let q1 = e(t64(&ed(&numd, &emd)));
    let q2 = e(t64(&ed(&q1, &tfd)));
    t64(&em_(&q2, &e(r)))
}
// fully extended: everything Ext80, single final store
fn d_num_ext(iv: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let tf = if ty == 0.0 {
        one()
    } else {
        ea(&one(), &em_(&e(r), &e(ty)))
    };
    let em = es(&e(iv), &one());
    let num = ea(&e(pv), &em_(&e(fv), &e(iv)));
    let q1 = ed(&num, &em);
    let q2 = ed(&q1, &tf);
    t64(&em_(&q2, &e(r)))
}
// num extended into division only, rest double
fn d_num_extdiv(iv: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let tf = if ty == 0.0 {
        one()
    } else {
        ea(&one(), &em_(&e(r), &e(ty)))
    };
    let em = es(&e(iv), &one());
    let num = ea(&e(pv), &em_(&e(fv), &e(iv))); // extended num
    let q1 = e(t64(&ed(&num, &em))); // store q1
    let tfd = e(t64(&tf));
    let q2 = e(t64(&ed(&q1, &tfd)));
    t64(&em_(&q2, &e(r)))
}
// num via fma (single round), rest double per-op
fn d_num_fma(iv: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let tf = if ty == 0.0 { 1.0 } else { 1.0 + r * ty };
    let em = iv - 1.0;
    let num = fv.mul_add(iv, pv);
    (num / em) / tf * r
}
// num=pv+fv*invF SSE, combine SSE (pure)
fn d_sse(iv: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
    let tf = if ty == 0.0 { 1.0 } else { 1.0 + r * ty };
    let em = iv - 1.0;
    let num = pv + fv * iv;
    (num / em) / tf * r
}

fn score(rows: &[Row], f: F, fv: f64, ty: f64) -> (usize, usize, BTreeMap<i64, usize>) {
    let mut ok = 0;
    let mut c = 0;
    let mut h = BTreeMap::new();
    for row in rows.iter().filter(|r| r.fv == fv && r.ty == ty) {
        let iv = invf(row.r, row.n);
        let d = f(iv, row.r, row.pv, row.fv, row.ty).to_bits() as i64 - row.want as i64;
        *h.entry(d).or_insert(0) += 1;
        if d == 0 {
            ok += 1;
        }
        c += 1;
    }
    (ok, c, h)
}

fn main() {
    let res = load("../../work/w109/G6-solvers/answers-pmt-fv1sweep.json");
    let r360: Vec<Row> = res
        .iter()
        .cloned()
        .filter(|r| (r.n - 360.0).abs() < 0.5)
        .collect();
    let groups = [(-1.0, 0.0), (-1.0, 1.0), (1.0, 0.0), (1.0, 1.0)];
    let fns: Vec<(&str, F)> = vec![
        ("num_dbl per-op   ", d_num_dbl),
        ("num_ext full-x87 ", d_num_ext),
        ("num_extdiv       ", d_num_extdiv),
        ("num_fma          ", d_num_fma),
        ("pure SSE         ", d_sse),
    ];
    for (nm, f) in &fns {
        let mut line = format!("  {:18}", nm);
        let mut tot = 0;
        for (fv, ty) in groups {
            let (ok, _c, _h) = score(&r360, *f, fv, ty);
            line.push_str(&format!(" ({:+},{}):{:3}", fv as i32, ty as i32, ok));
            tot += ok;
        }
        println!("{}  tot={}", line, tot);
    }
    println!("\n=== hist fv=+1 ty=0 per staging ===");
    for (nm, f) in &fns {
        let (ok, c, h) = score(&r360, *f, 1.0, 0.0);
        println!("  {:18} {}/{} {:?}", nm, ok, c, h);
    }
    println!("\n=== hist fv=+1 ty=1 per staging ===");
    for (nm, f) in &fns {
        let (ok, c, h) = score(&r360, *f, 1.0, 1.0);
        println!("  {:18} {}/{} {:?}", nm, ok, c, h);
    }
}
