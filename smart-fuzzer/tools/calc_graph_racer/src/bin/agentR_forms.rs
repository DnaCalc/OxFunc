//! agentR forms: pin the PMT combine op-graph on the RESOLUTION corpus (fv1sweep,
//! fv=+-1) where pv is NOT swamped, then check constancy/match on fvsweep (fv=1000).
//! Legacy x87 spill-loop model (every op RN53(RN64)), FV factor via pow/binexp/exp.
use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::power_fn::power_kernel;
use serde_json::Value;

const CW: u16 = rx::CW_PC64_RN;
fn e(x: f64) -> rx::Ext80 { rx::ext_from_f64(x) }
fn tf64(x: &rx::Ext80) -> f64 { rx::ext_to_f64(x, CW) }
fn xadd(a: f64, b: f64) -> f64 { tf64(&rx::ext_add(&e(a), &e(b), CW)) }
fn xsub(a: f64, b: f64) -> f64 { tf64(&rx::ext_sub(&e(a), &e(b), CW)) }
fn xmul(a: f64, b: f64) -> f64 { tf64(&rx::ext_mul(&e(a), &e(b), CW)) }
fn xdiv(a: f64, b: f64) -> f64 { tf64(&rx::ext_div(&e(a), &e(b), CW)) }

fn binexp_sse(base: f64, mut n: u64) -> f64 {
    let mut acc = 1.0; let mut b = base;
    while n > 0 { if n & 1 == 1 { acc *= b; } n >>= 1; if n > 0 { b *= b; } }
    acc
}
fn binexp_x87(base: f64, mut n: u64) -> f64 {
    let mut acc = 1.0; let mut b = base;
    while n > 0 { if n & 1 == 1 { acc = xmul(acc, b); } n >>= 1; if n > 0 { b = xmul(b, b); } }
    acc
}

#[derive(Clone, Copy)]
struct Row { r: f64, n: f64, pv: f64, fv: f64, ty: f64, want: u64 }
fn load(path: &str) -> Vec<Row> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut o = Vec::new();
    for w in v["witnesses"].as_array().unwrap() {
        let a = w["args"].as_array().unwrap();
        let g = |i: usize| f64::from_bits(u64::from_str_radix(a[i].as_str().unwrap().trim_start_matches("0x"), 16).unwrap());
        o.push(Row { r: g(0), n: g(1), pv: g(2), fv: g(3), ty: g(4),
            want: u64::from_str_radix(w["expected_bits"].as_str().unwrap().trim_start_matches("0x"), 16).unwrap() });
    }
    o
}

// P source
#[derive(Clone, Copy)]
enum PS { Pow, BinSse, BinX87, ExpFwd }
fn pfactor(ps: PS, r: f64, n: f64) -> f64 {
    let base = xadd(1.0, r);
    match ps {
        PS::Pow => power_kernel(base, n).unwrap(),                 // integer -> binexp SSE inside
        PS::BinSse => binexp_sse(base, n as u64),
        PS::BinX87 => binexp_x87(base, n as u64),
        PS::ExpFwd => rx::excel_exp(n * rx::excel_log1p(r)),       // (1+r)^n via exp
    }
}
fn invfactor(r: f64, n: f64) -> f64 { rx::excel_exp(-(n * rx::excel_log1p(r))) } // (1+r)^-n

// op mode
#[derive(Clone, Copy, PartialEq)]
enum M { Sse, X87 }
fn mul(m: M, a: f64, b: f64) -> f64 { if m == M::Sse { a * b } else { xmul(a, b) } }
fn div(m: M, a: f64, b: f64) -> f64 { if m == M::Sse { a / b } else { xdiv(a, b) } }
fn add(m: M, a: f64, b: f64) -> f64 { if m == M::Sse { a + b } else { xadd(a, b) } }
fn sub(m: M, a: f64, b: f64) -> f64 { if m == M::Sse { a - b } else { xsub(a, b) } }
fn tf_of(m: M, r: f64, ty: f64) -> f64 { if ty == 0.0 { 1.0 } else { add(m, 1.0, mul(m, r, ty)) } }

// Each variant: fn(M, P, invF, r, n, pv, fv, ty) -> f64
type VF = fn(M, f64, f64, f64, f64, f64) -> f64; // (m, P, invF, r, pv, fv, ty via closure? ) simpler below

fn main() {
    let res = load("../../work/w109/G6-solvers/answers-pmt-fv1sweep.json");
    let big = load("../../work/w109/G6-solvers/answers-pmt-fvsweep.json");
    // resolution groups: n=360 only (accurate em). keys (fv,ty)
    let res360: Vec<Row> = res.iter().cloned().filter(|r| (r.n - 360.0).abs() < 0.5).collect();
    let big360: Vec<Row> = big.iter().cloned().filter(|r| (r.n - 360.0).abs() < 0.5).collect();
    let groups = [(-1.0, 0.0), (-1.0, 1.0), (1.0, 0.0), (1.0, 1.0)];
    println!("resolution rows n=360: {}, big(fv1000) rows n=360: {}", res360.len(), big360.len());

    // Variant catalog. Each closure computes pmt for one row given P, invF and ops.
    struct Var { name: &'static str, ps: PS, m: M,
        f: fn(M, f64, f64, f64, f64, f64, f64) -> f64 } // (m,P,invF,r,pv,fv,ty)
    // Forward FV forms
    fn fwd_a(m: M, p: f64, _invf: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
        // pmt = -(fv+pv*P)/(tf*(P-1)/r)
        let tf = tf_of(m, r, ty);
        let pm1 = sub(m, p, 1.0);
        let fvifa = div(m, pm1, r);
        let den = mul(m, tf, fvifa);
        let num = add(m, fv, mul(m, pv, p));
        -div(m, num, den)
    }
    fn fwd_b(m: M, p: f64, _invf: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
        // pmt = -(fv+pv*P)*r/(tf*(P-1))
        let tf = tf_of(m, r, ty);
        let pm1 = sub(m, p, 1.0);
        let den = mul(m, tf, pm1);
        let num = add(m, fv, mul(m, pv, p));
        -div(m, mul(m, num, r), den)
    }
    fn fwd_c(m: M, p: f64, _invf: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
        // num=pv*P+fv order; pmt=-num*r/((P-1)*tf)
        let tf = tf_of(m, r, ty);
        let pm1 = sub(m, p, 1.0);
        let num = add(m, mul(m, pv, p), fv);
        -div(m, mul(m, num, r), mul(m, pm1, tf))
    }
    // Discount forms (current production style)
    fn disc_a(m: M, _p: f64, invf: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
        // pmt = -(pv+fv*invF)*r/(tf*(1-invF))
        let tf = tf_of(m, r, ty);
        let denom = sub(m, 1.0, invf);
        let num = add(m, pv, mul(m, fv, invf));
        -div(m, mul(m, num, r), mul(m, tf, denom))
    }
    fn disc_b(m: M, _p: f64, invf: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
        // quotient-first established: ((num/em)/tf)*r ; em=invF-1 (neg)
        let tf = tf_of(m, r, ty);
        let em = sub(m, invf, 1.0);
        let num = add(m, pv, mul(m, fv, invf));
        mul(m, div(m, div(m, num, em), tf), r)
    }
    // Two-term (LibreOffice)
    fn two_a(m: M, p: f64, _invf: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
        // pmt = -(fv*r/(P-1) + pv*r/(1-1/P)) [/(1+r) if ty]
        let fvt = div(m, mul(m, fv, r), sub(m, p, 1.0));
        let onem = sub(m, 1.0, div(m, 1.0, p));
        let pvt = div(m, mul(m, pv, r), onem);
        let s = add(m, fvt, pvt);
        let neg = -s;
        if ty == 0.0 { neg } else { div(m, neg, xadd(1.0, r)) }
    }
    fn two_b(m: M, p: f64, invf: f64, r: f64, pv: f64, fv: f64, ty: f64) -> f64 {
        // two-term but onem via invF (=1/P from exp): 1-invF
        let fvt = div(m, mul(m, fv, r), sub(m, p, 1.0));
        let onem = sub(m, 1.0, invf);
        let pvt = div(m, mul(m, pv, r), onem);
        let s = add(m, fvt, pvt);
        let neg = -s;
        if ty == 0.0 { neg } else { div(m, neg, xadd(1.0, r)) }
    }

    let mut cat: Vec<Var> = Vec::new();
    for &(nm, f) in &[("fwd_a", fwd_a as fn(M,f64,f64,f64,f64,f64,f64)->f64), ("fwd_b", fwd_b), ("fwd_c", fwd_c),
                      ("disc_a", disc_a), ("disc_b", disc_b), ("two_a", two_a), ("two_b", two_b)] {
        for &ps in &[PS::Pow, PS::BinSse, PS::BinX87, PS::ExpFwd] {
            for &m in &[M::X87, M::Sse] {
                // leak name
                let full = Box::leak(format!("{} P={} {}", nm,
                    match ps { PS::Pow=>"pow", PS::BinSse=>"binSSE", PS::BinX87=>"binX87", PS::ExpFwd=>"expF" },
                    if m==M::X87 {"x87"} else {"sse"}).into_boxed_str());
                cat.push(Var { name: full, ps, m, f });
            }
        }
    }

    // Score each variant on resolution groups
    println!("\n=== resolution scores (per group /count) ; sorted by total ===");
    let mut scored: Vec<(String, usize, [usize;4])> = Vec::new();
    for v in &cat {
        let mut per = [0usize;4]; let mut tot = 0usize;
        for (gi, &(fv, ty)) in groups.iter().enumerate() {
            for row in res360.iter().filter(|r| r.fv == fv && r.ty == ty) {
                let p = pfactor(v.ps, row.r, row.n);
                let invf = invfactor(row.r, row.n);
                let got = (v.f)(v.m, p, invf, row.r, row.pv, row.fv, row.ty).to_bits();
                if got == row.want { per[gi] += 1; tot += 1; }
            }
        }
        scored.push((v.name.to_string(), tot, per));
    }
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    let gcount: Vec<usize> = groups.iter().map(|&(fv,ty)| res360.iter().filter(|r| r.fv==fv && r.ty==ty).count()).collect();
    println!("group counts (fv-1ty0, fv-1ty1, fv1ty0, fv1ty1) = {:?}", gcount);
    for (nm, tot, per) in scored.iter().take(20) {
        println!("  [{:4}] {:22} per={:?}", tot, nm, per);
    }

    // For the best few, check fvsweep (fv=1000) constancy + match
    println!("\n=== fvsweep (fv=1000) distinct-got + match for top variants ===");
    for (nm, _tot, _per) in scored.iter().take(8) {
        let v = cat.iter().find(|v| v.name == nm).unwrap();
        for ty in [0.0f64, 1.0] {
            let rows: Vec<&Row> = big360.iter().filter(|r| r.ty == ty && r.fv == 1000.0).collect();
            use std::collections::BTreeSet;
            let mut gotset = BTreeSet::new(); let mut ok = 0;
            for row in &rows {
                let p = pfactor(v.ps, row.r, row.n);
                let invf = invfactor(row.r, row.n);
                let got = (v.f)(v.m, p, invf, row.r, row.pv, row.fv, row.ty).to_bits();
                gotset.insert(got);
                if got == row.want { ok += 1; }
            }
            println!("  {:22} ty={}: distinct_got={} match={}/{}", nm, ty, gotset.len(), ok, rows.len());
        }
    }
}
