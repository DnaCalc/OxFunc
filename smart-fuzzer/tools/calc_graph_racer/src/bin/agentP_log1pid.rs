//! Confirm: em model with log1p = fyl2xp1 EXCEPT +1 ulp at r in {2^-5,2^-4} closes
//! the residual. Also test candidate log1p algorithms (fdlibm poly) for the +1 pattern.
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;
const CW: u16 = rx::CW_PC64_RN;
fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}
fn load(path: &str) -> Vec<(f64, i64, f64)> {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut o = Vec::new();
    for (k, val) in v.as_object().unwrap() {
        let (rh, nh) = k.split_once('|').unwrap();
        o.push((fb(rh), nh.parse().unwrap(), fb(val.as_str().unwrap())));
    }
    o.sort_by(|a, b| (a.0, a.1).partial_cmp(&(b.0, b.1)).unwrap());
    o
}
fn na(x: f64, d: i64) -> f64 {
    f64::from_bits((x.to_bits() as i64 + d) as u64)
}
fn kahan(l: f64, r: f64, n: i64) -> f64 {
    let t = -(n as f64) * l;
    let u = rx::excel_exp(t);
    let lnu = rx::excel_ln(u);
    if u == 1.0 {
        t
    } else if t.abs() < 1.0 {
        (u - 1.0) * t / lnu
    } else {
        u - 1.0
    }
}
// fdlibm log1p (classic s_log1p.c) in f64
fn fdlibm_log1p(x: f64) -> f64 {
    const LN2_HI: f64 = 6.93147180369123816490e-01;
    const LN2_LO: f64 = 1.90821492927058770002e-10;
    const LP1: f64 = 6.666666666666735130e-01;
    const LP2: f64 = 3.999999999940941908e-01;
    const LP3: f64 = 2.857142874366239149e-01;
    const LP4: f64 = 2.222219843214978396e-01;
    const LP5: f64 = 1.818357216161805012e-01;
    const LP6: f64 = 1.531383769920937332e-01;
    const LP7: f64 = 1.479819860511658591e-01;
    if x == 0.0 {
        return x;
    }
    let f = x;
    let f2 = f * f;
    // fdlibm computes: hfsq=0.5*f*f; s=f/(2.0+f); z=s*s; R=z*(LP1+z*(...)); return f-(hfsq-s*(hfsq+R))
    let hfsq = 0.5 * f2;
    let s = f / (2.0 + f);
    let z = s * s;
    let r = z * (LP1 + z * (LP2 + z * (LP3 + z * (LP4 + z * (LP5 + z * (LP6 + z * LP7))))));
    let _ = (LN2_HI, LN2_LO);
    f - (hfsq - s * (hfsq + r))
}
fn main() {
    for (label, path) in [
        ("HDF46", "../../work/w109/G6-solvers/pmt_em_hdf_oracle.json"),
        ("POX", "../../work/w109/G6-solvers/agentP_pox_em.json"),
    ] {
        let rows = load(path);
        // model 1: fyl2xp1 (CR)  ; model 2: +1 at 2^-5,2^-4 ; model 3: fdlibm log1p
        let mut m: BTreeMap<&str, (u32, u32)> = BTreeMap::new();
        let mut perr: BTreeMap<&str, BTreeMap<i32, (u32, u32)>> = BTreeMap::new();
        for (r, n, em) in &rows {
            let k = (r.log2().round()) as i32;
            let l_cr = rx::excel_log1p(*r);
            let l_patch = if k == -5 || k == -4 {
                na(l_cr, 1)
            } else {
                l_cr
            };
            let l_fd = fdlibm_log1p(*r);
            for (nm, l) in [
                ("CR log1p", l_cr),
                ("patch+1@{2^-5,2^-4}", l_patch),
                ("fdlibm log1p", l_fd),
            ] {
                let ok = kahan(l, *r, *n).to_bits() == em.to_bits();
                let e = m.entry(nm).or_insert((0, 0));
                e.1 += 1;
                if ok {
                    e.0 += 1;
                }
                let pe = perr.entry(nm).or_default().entry(k).or_insert((0, 0));
                pe.1 += 1;
                if ok {
                    pe.0 += 1;
                }
            }
        }
        println!("=== {} ===", label);
        for (nm, (o, t)) in &m {
            print!("  {:22} {:3}/{:3}  per-r:", nm, o, t);
            for (k, (a, b)) in &perr[nm] {
                print!(" 2^{}:{}/{}", k, a, b);
            }
            println!();
        }
        // report fdlibm vs CR bit diff per po2 r
        if label == "HDF46" {
            print!("  fdlibm-CR log1p ulp per r:");
            let mut seen = std::collections::BTreeSet::new();
            for (r, _, _) in &rows {
                let k = (r.log2().round()) as i32;
                if seen.insert(k) {
                    let d =
                        fdlibm_log1p(*r).to_bits() as i64 - rx::excel_log1p(*r).to_bits() as i64;
                    print!(" 2^{}:{:+}", k, d);
                }
            }
            println!();
        }
    }
}
