//! agent-P: focused decode of the ROBUST large-|t| lane against GOLD pv=1.
//! Split |t|>2 by sign of t. t<0 => em near -1 (robust to combine op). Decode:
//!  (1) t = -n*log1p(r) delivery (product rounding + log1p), and
//!  (2) em = expm1(t) form (u-1 branch).
use oxfunc_core::excel_numeric::research as rx;
use std::collections::BTreeMap;

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
fn load(path: &str) -> Vec<(f64, i64, f64)> {
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
    if n == 0 {
        println!("  {:36} (0)", name);
        return;
    }
    let exact = *m.get(&0).unwrap_or(&0);
    print!("  {:36} {:5}/{:5} ({:5.1}%)  ", name, exact, n, 100.0 * exact as f64 / n as f64);
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
fn log1p_ext(r: f64) -> rx::Ext80 {
    rx::ext_fyl2xp1(&rx::ext_ln2(), &e(r), CW)
}
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

// t-delivery variants returning f64
fn t_port_mul(r: f64, n: f64) -> f64 {
    -n * rx::excel_log1p(r)
}
fn t_x87mul(r: f64, n: f64) -> f64 {
    // x87 double-rounded product RN53(RN64(-n*log1p_port))
    tf(&rx::ext_mul(&e(-n), &e(rx::excel_log1p(r)), CW))
}
fn t_ext_prod(r: f64, n: f64) -> rx::Ext80 {
    // -n * fyl2xp1(r), extended (log1p not stored)
    rx::ext_mul(&e(-n), &log1p_ext(r), CW)
}

fn main() {
    let rows = load("../../work/w109/G6-solvers/pmt_em_gold_pv1.json");
    // tau = -n*log1p(r) is the expm1 argument. tau<-2 => em near -1 (ROBUST).
    let mut neg: Vec<(f64, i64, f64)> = Vec::new(); // tau<-2, em near -1 (robust)
    let mut pos: Vec<(f64, i64, f64)> = Vec::new(); // tau>2,  em large (sensitive)
    for row in &rows {
        let tau = -(row.1 as f64) * rx::excel_log1p(row.0);
        if tau < -2.0 {
            neg.push(*row);
        } else if tau > 2.0 {
            pos.push(*row);
        }
    }
    println!("large |tau|>2:  tau<-2 (em near -1, ROBUST): {}   tau>2 (em large): {}", neg.len(), pos.len());

    // ---- ROBUST lane: t<0, |t|>2, em = expm1(t) via u-1 branch ----
    println!("\n=== ROBUST t<0,|t|>2: t-delivery x expm1-form vs gold ===");
    // for each t-delivery, test em = u-1 (f64) and RN53(u_ext-1)
    let deliveries: Vec<(&str, Box<dyn Fn(f64, f64) -> (f64, rx::Ext80)>)> = vec![
        (
            "t=f64(-n*port)",
            Box::new(|r, n| {
                let t = t_port_mul(r, n);
                (t, e(t))
            }),
        ),
        (
            "t=x87dr(-n*port)",
            Box::new(|r, n| {
                let t = t_x87mul(r, n);
                (t, e(t))
            }),
        ),
        (
            "t=ext(-n*fyl2xp1)",
            Box::new(|r, n| {
                let te = t_ext_prod(r, n);
                (tf(&te), te)
            }),
        ),
    ];
    for (dn, df) in &deliveries {
        let mut r_uf = Vec::new(); // exp(t_f64) stored, u-1 f64
        let mut r_ue = Vec::new(); // exp on extended t, u stored f64, u-1 f64
        let mut r_uee = Vec::new(); // exp on ext t, RN53(u_ext-1)
        for (r, n, em_x) in &neg {
            let (tf64, text) = df(*r, *n as f64);
            let uf = rx::excel_exp(tf64);
            r_uf.push((uf - 1.0).to_bits() as i64 - em_x.to_bits() as i64);
            let uext = exp_ext(&text);
            let ust = tf(&uext);
            r_ue.push((ust - 1.0).to_bits() as i64 - em_x.to_bits() as i64);
            r_uee.push(tf(&rx::ext_sub(&uext, &rx::ext_one(), CW)).to_bits() as i64 - em_x.to_bits() as i64);
        }
        println!(" delivery {}", dn);
        hist("   exp(t_f64) stored; u-1 f64", &r_uf);
        hist("   exp(t_ext); u stored; u-1 f64", &r_ue);
        hist("   exp(t_ext); RN53(u_ext-1)", &r_uee);
    }
}
