//! agent-P: enumerate every available expm1 form against the PINNED em oracle
//! for n=1, report residual hist + per-form exact counts + union coverage, and
//! dump the cases NO form covers.
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

const CW: u16 = rx::CW_PC64_RN;

fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}
fn load_pinned(path: &str) -> Vec<(f64, i64, f64)> {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("json");
    let mut out = Vec::new();
    for (k, val) in v.as_object().unwrap() {
        if val.is_null() {
            continue;
        }
        let (rh, nh) = k.split_once('|').unwrap();
        out.push((fb(rh), nh.parse().unwrap(), fb(val.as_str().unwrap())));
    }
    out
}
fn hist(name: &str, v: &[i64]) {
    let mut m: BTreeMap<i64, u32> = BTreeMap::new();
    for &x in v {
        *m.entry(x).or_default() += 1;
    }
    let n = v.len();
    let exact = *m.get(&0).unwrap_or(&0);
    print!(
        "  {:30} {:4}/{:4} ({:5.1}%)  ",
        name,
        exact,
        n,
        100.0 * exact as f64 / n as f64
    );
    let mut big = 0;
    for (k, c) in &m {
        if k.abs() <= 3 {
            print!("{}:{} ", k, c);
        } else {
            big += c;
        }
    }
    if big > 0 {
        print!("|>3|:{}", big);
    }
    println!();
}
fn exp_ext(arg: &rx::Ext80) -> rx::Ext80 {
    let z = rx::ext_mul(arg, &rx::ext_l2e(), CW);
    let k = rx::ext_rndint(&z, CW);
    let f = rx::ext_sub(&z, &k, CW);
    let neg = rx::ext_to_f64(&f, CW) < 0.0;
    let w = rx::ext_f2xm1(&rx::ext_abs(&f, CW), CW);
    let mut m = rx::ext_add(&w, &rx::ext_one(), CW);
    if neg {
        m = rx::ext_div(&rx::ext_one(), &m, CW);
    }
    rx::ext_scale(&m, &k, CW)
}

fn main() {
    let rows = load_pinned("../../work/w109/G6-solvers/pmt_em_pinned.json");
    let n1: Vec<_> = rows.iter().filter(|(_, n, _)| *n == 1).cloned().collect();

    // each form: name -> fn(r)->em
    type F = Box<dyn Fn(f64) -> f64>;
    let forms: Vec<(&str, F)> = vec![
        (
            "kahan (u-1)*t/lnu",
            Box::new(|r: f64| {
                let t = -rx::excel_log1p(r);
                let u = rx::excel_exp(t);
                if u == 1.0 {
                    t
                } else if t.abs() < 1.0 {
                    (u - 1.0) * t / rx::excel_ln(u)
                } else {
                    u - 1.0
                }
            }),
        ),
        (
            "portable expm1(t)",
            Box::new(|r: f64| rx::excel_expm1(-rx::excel_log1p(r))),
        ),
        (
            "RN53(u_ext-1)",
            Box::new(|r: f64| {
                let t = -rx::excel_log1p(r);
                rx::ext_to_f64(
                    &rx::ext_sub(&exp_ext(&rx::ext_from_f64(t)), &rx::ext_one(), CW),
                    CW,
                )
            }),
        ),
        (
            "kahan on ext-exp u",
            Box::new(|r: f64| {
                let t = -rx::excel_log1p(r);
                let ue = exp_ext(&rx::ext_from_f64(t));
                let u = rx::ext_to_f64(&ue, CW);
                if u == 1.0 {
                    return t;
                }
                let um1 = rx::ext_to_f64(&rx::ext_sub(&ue, &rx::ext_one(), CW), CW); // extended u-1 stored
                if t.abs() < 1.0 {
                    um1 * t / rx::excel_ln(u)
                } else {
                    um1
                }
            }),
        ),
    ];

    println!("=== n=1 all expm1 forms vs pinned ===");
    let mut cover = vec![false; n1.len()];
    let mut oks: Vec<Vec<bool>> = Vec::new();
    for (name, f) in &forms {
        let mut res = Vec::new();
        let mut ok = Vec::new();
        for (r, _, em_x) in &n1 {
            let em = f(*r);
            res.push(em.to_bits() as i64 - em_x.to_bits() as i64);
            ok.push(em.to_bits() == em_x.to_bits());
        }
        hist(name, &res);
        for (i, &b) in ok.iter().enumerate() {
            cover[i] |= b;
        }
        oks.push(ok);
    }
    let covn = cover.iter().filter(|&&b| b).count();
    println!("\nunion coverage of all forms: {}/{}", covn, n1.len());

    // dump the UNCOVERED cases with detail
    println!("\n=== UNCOVERED n=1 cases (no form matches) ===");
    println!("  r  em_excel  kahan  du_kahan  u  t  lnu");
    let mut cnt = 0;
    for (i, (r, _, em_x)) in n1.iter().enumerate() {
        if cover[i] {
            continue;
        }
        cnt += 1;
        let t = -rx::excel_log1p(*r);
        let u = rx::excel_exp(t);
        let lnu = rx::excel_ln(u);
        let kah = (u - 1.0) * t / lnu;
        println!(
            "  r=0x{:016x} em=0x{:016x} kah=0x{:016x} du={:+} u=0x{:016x} t=0x{:016x} lnu=0x{:016x}",
            r.to_bits(),
            em_x.to_bits(),
            kah.to_bits(),
            kah.to_bits() as i64 - em_x.to_bits() as i64,
            u.to_bits(),
            t.to_bits(),
            lnu.to_bits()
        );
    }
    println!("  total uncovered: {}", cnt);
}
