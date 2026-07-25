//! agent-P FINAL LANE: decode the exact expm1 op-graph reproducing the CLEAN
//! (combine-decoupled) em oracle pmt_em_hdf_oracle.json (46 r=2^-k points).
//! 35 match CR; 11 are +-1 mixed. All 11 are Kahan-branch (|tau|<1). Enumerate
//! Kahan intermediate-rounding variants against ALL 46, flag the 11.
use oxfunc_core::excel_numeric::research as rx;

const CW: u16 = rx::CW_PC64_RN;

fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}
fn e(x: f64) -> rx::Ext80 {
    rx::ext_from_f64(x)
}
fn tf(x: &rx::Ext80) -> f64 {
    rx::ext_to_f64(x, CW)
}
// x87 double-rounded binary ops RN53(RN64(a op b)): compute at PC64 then store RN53.
fn dr_mul(a: f64, b: f64) -> f64 {
    tf(&rx::ext_mul(&e(a), &e(b), CW))
}
fn dr_div(a: f64, b: f64) -> f64 {
    tf(&rx::ext_div(&e(a), &e(b), CW))
}
fn load(path: &str) -> Vec<(f64, i64, f64)> {
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("json");
    let mut out = Vec::new();
    for (k, val) in v.as_object().unwrap() {
        let (rh, nh) = k.split_once('|').unwrap();
        out.push((fb(rh), nh.parse().unwrap(), fb(val.as_str().unwrap())));
    }
    out.sort_by(|a, b| (a.0, a.1).partial_cmp(&(b.0, b.1)).unwrap());
    out
}
// exp chain kept extended (fFEXP), arg extended
fn exp_ext(arg: &rx::Ext80) -> rx::Ext80 {
    let z = rx::ext_mul(arg, &rx::ext_l2e(), CW);
    let k = rx::ext_rndint(&z, CW);
    let f = rx::ext_sub(&z, &k, CW);
    let neg = tf(&f) < 0.0;
    let w = rx::ext_f2xm1(&rx::ext_abs(&f, CW), CW);
    let mut m = rx::ext_add(&w, &rx::ext_one(), CW);
    if neg {
        m = rx::ext_div(&rx::ext_one(), &m, CW);
    }
    rx::ext_scale(&m, &k, CW)
}

fn main() {
    let rows = load("../../work/w109/G6-solvers/pmt_em_hdf_oracle.json");
    println!("clean oracle rows: {}", rows.len());

    // Precompute per-row t, u (f64 & ext), lnu, um1
    // t = -n * log1p_port(r)  (f64). Also test t via x87-dr product.
    // Variant closures return em for a row.
    type V = Box<dyn Fn(f64, i64) -> f64>;
    let variants: Vec<(&str, V)> = vec![
        // 0 baseline internal-Kahan (SSE f64 (u-1)*t/lnu)
        ("K0 SSE (u-1)*t/lnu", Box::new(|r, n| {
            let t = -(n as f64) * rx::excel_log1p(r);
            let u = rx::excel_exp(t);
            if u == 1.0 { t } else if t.abs() < 1.0 { (u - 1.0) * t / rx::excel_ln(u) } else { u - 1.0 }
        })),
        // 1 x87 PC64 double-round each Kahan op: p=dr(um1*t); em=dr(p/lnu)
        ("K1 x87dr (um1*t)/lnu", Box::new(|r, n| {
            let t = -(n as f64) * rx::excel_log1p(r);
            let u = rx::excel_exp(t);
            if u == 1.0 { return t; }
            if t.abs() < 1.0 { dr_div(dr_mul(u - 1.0, t), rx::excel_ln(u)) } else { u - 1.0 }
        })),
        // 2 x87 extended product kept, divide double-rounded, single store
        ("K2 ext prod, div store", Box::new(|r, n| {
            let t = -(n as f64) * rx::excel_log1p(r);
            let u = rx::excel_exp(t);
            if u == 1.0 { return t; }
            if t.abs() < 1.0 {
                let p = rx::ext_mul(&e(u - 1.0), &e(t), CW);
                tf(&rx::ext_div(&p, &e(rx::excel_ln(u)), CW))
            } else { u - 1.0 }
        })),
        // 3 op-order (u-1)/lnu*t (SSE)
        ("K3 SSE (u-1)/lnu*t", Box::new(|r, n| {
            let t = -(n as f64) * rx::excel_log1p(r);
            let u = rx::excel_exp(t);
            if u == 1.0 { return t; }
            if t.abs() < 1.0 { (u - 1.0) / rx::excel_ln(u) * t } else { u - 1.0 }
        })),
        // 4 op-order t/lnu*(u-1) (SSE)
        ("K4 SSE t/lnu*(u-1)", Box::new(|r, n| {
            let t = -(n as f64) * rx::excel_log1p(r);
            let u = rx::excel_exp(t);
            if u == 1.0 { return t; }
            if t.abs() < 1.0 { t / rx::excel_ln(u) * (u - 1.0) } else { u - 1.0 }
        })),
        // 5 denom = log1p(u-1) via fyl2xp1 (SSE arithmetic)
        ("K5 SSE denom log1p(u-1)", Box::new(|r, n| {
            let t = -(n as f64) * rx::excel_log1p(r);
            let u = rx::excel_exp(t);
            if u == 1.0 { return t; }
            if t.abs() < 1.0 {
                let l = tf(&rx::ext_fyl2xp1(&rx::ext_ln2(), &e(u - 1.0), CW));
                (u - 1.0) * t / l
            } else { u - 1.0 }
        })),
        // 6 u kept EXTENDED from exp: um1 extended, lnu=ln(u_ext) ext, arithmetic ext single store
        ("K6 full-ext spill (u_ext)", Box::new(|r, n| {
            let t = -(n as f64) * rx::excel_log1p(r);
            let ue = exp_ext(&e(t));
            let u = tf(&ue);
            if u == 1.0 { return t; }
            if t.abs() < 1.0 {
                let um1 = rx::ext_sub(&ue, &rx::ext_one(), CW);
                let lnu = rx::ext_fyl2x(&rx::ext_ln2(), &ue, CW);
                tf(&rx::ext_div(&rx::ext_mul(&um1, &e(t), CW), &lnu, CW))
            } else { u - 1.0 }
        })),
        // 7 x87dr but denom is x87 ln (already), product x87dr, divide SSE
        ("K7 x87dr prod, SSE div", Box::new(|r, n| {
            let t = -(n as f64) * rx::excel_log1p(r);
            let u = rx::excel_exp(t);
            if u == 1.0 { return t; }
            if t.abs() < 1.0 { dr_mul(u - 1.0, t) / rx::excel_ln(u) } else { u - 1.0 }
        })),
        // 8 SSE prod, x87dr divide
        ("K8 SSE prod, x87dr div", Box::new(|r, n| {
            let t = -(n as f64) * rx::excel_log1p(r);
            let u = rx::excel_exp(t);
            if u == 1.0 { return t; }
            if t.abs() < 1.0 { dr_div((u - 1.0) * t, rx::excel_ln(u)) } else { u - 1.0 }
        })),
    ];

    for (name, f) in &variants {
        let mut ok = 0;
        let mut miss11 = Vec::new();
        for (r, n, em_x) in &rows {
            let em = f(*r, *n);
            if em.to_bits() == em_x.to_bits() {
                ok += 1;
            } else {
                miss11.push((*r, *n, em.to_bits() as i64 - em_x.to_bits() as i64));
            }
        }
        print!("  {:30} {:2}/46", name, ok);
        if ok >= 40 {
            print!("   misses:");
            for (r, n, d) in &miss11 {
                print!(" (2^{},{}):{:+}", (r.log2().round()) as i32, n, d);
            }
        }
        println!();
    }
}
