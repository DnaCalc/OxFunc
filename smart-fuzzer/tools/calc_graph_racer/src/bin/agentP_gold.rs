//! agent-P: decode em against the GOLD pv=1 oracle (pmt_em_gold_pv1.json).
//! Binned by |t|=|n*log1p(r)|. Large-|t| (>2) is robust to the combine op and
//! is decoded FIRST/cleanly: t = -n*log1p(r) delivery + expm1 (u-1 branch).
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

// log1p deliveries (return f64 for the "stored" forms)
fn log1p_port(r: f64) -> f64 {
    rx::excel_log1p(r)
}
fn log1p_fyl2xp1_ext(r: f64) -> rx::Ext80 {
    rx::ext_fyl2xp1(&rx::ext_ln2(), &e(r), CW)
}

// t = -n*log1p(r) variants
#[derive(Clone, Copy)]
enum TDelivery {
    F64PortMul,    // t = RN53(-n * log1p_port(r)), both f64
    F64Fyl2xp1Mul, // t = RN53(-n * RN53(fyl2xp1)), f64 product
    ExtFyl2xp1Mul, // t = -n * fyl2xp1 all extended, stored to f64
    ExtKept,       // extended t kept (returns via ext); represented as f64 store here
}
fn t_of(td: TDelivery, r: f64, n: f64) -> f64 {
    match td {
        TDelivery::F64PortMul => -n * log1p_port(r),
        TDelivery::F64Fyl2xp1Mul => -n * tf(&log1p_fyl2xp1_ext(r)),
        TDelivery::ExtFyl2xp1Mul | TDelivery::ExtKept => {
            tf(&rx::ext_mul(&e(-n), &log1p_fyl2xp1_ext(r), CW))
        }
    }
}
fn t_ext_of(r: f64, n: f64) -> rx::Ext80 {
    rx::ext_mul(&e(-n), &log1p_fyl2xp1_ext(r), CW)
}

// expm1 forms given t (f64)
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

fn hist(name: &str, v: &[i64]) {
    let mut m: BTreeMap<i64, u32> = BTreeMap::new();
    for &x in v {
        *m.entry(x).or_default() += 1;
    }
    let n = v.len();
    if n == 0 {
        println!("  {:34} (no rows)", name);
        return;
    }
    let exact = *m.get(&0).unwrap_or(&0);
    print!(
        "  {:34} {:5}/{:5} ({:5.1}%)  ",
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

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../../work/w109/G6-solvers/pmt_em_gold_pv1.json".into());
    let rows = load_pinned(&path);
    // split by |t|
    let mut big: Vec<(f64, i64, f64)> = Vec::new();
    let mut small: Vec<(f64, i64, f64)> = Vec::new();
    for row in &rows {
        let t = (row.1 as f64 * rx::excel_log1p(row.0)).abs();
        if t > 2.0 {
            big.push(*row);
        } else {
            small.push(*row);
        }
    }
    println!(
        "gold rows: {}  |t|>2: {}  |t|<=2: {}",
        rows.len(),
        big.len(),
        small.len()
    );

    // ===== Reference full model over ALL rows (kahan + u-1 branch), several t deliveries =====
    let tds = [
        ("t=f64(-n*port)", TDelivery::F64PortMul),
        ("t=f64(-n*fyl2xp1)", TDelivery::F64Fyl2xp1Mul),
        ("t=ext(-n*fyl2xp1)->f64", TDelivery::ExtFyl2xp1Mul),
    ];
    println!("\n=== FULL model (kahan |t|<1, plain u-1 else) vs gold, ALL rows ===");
    for (tn, td) in tds {
        let mut res = Vec::new();
        for (r, n, em_x) in &rows {
            let t = t_of(td, *r, *n as f64);
            let u = rx::excel_exp(t);
            let em = if u == 1.0 {
                t
            } else if t.abs() < 1.0 {
                (u - 1.0) * t / rx::excel_ln(u)
            } else {
                u - 1.0
            };
            res.push(em.to_bits() as i64 - em_x.to_bits() as i64);
        }
        hist(&format!("ALL {}", tn), &res);
    }

    // ===== LARGE-|t| focused: em = expm1(t), |t|>2 so u-1 branch. Decode cleanly. =====
    println!("\n=== LARGE-|t| (>2): expm1 form decode (robust lane) ===");
    // candidate em forms for large |t|
    for (tn, td) in tds {
        // A: u - 1 (f64 exp stored)
        let mut ra = Vec::new();
        // B: RN53(u_ext - 1) extended exp minus 1
        let mut rb = Vec::new();
        // C: portable expm1(t)
        let mut rc = Vec::new();
        for (r, n, em_x) in &big {
            let t = t_of(td, *r, *n as f64);
            let u = rx::excel_exp(t);
            ra.push((u - 1.0).to_bits() as i64 - em_x.to_bits() as i64);
            let uext = exp_ext(&e(t));
            rb.push(
                tf(&rx::ext_sub(&uext, &rx::ext_one(), CW)).to_bits() as i64
                    - em_x.to_bits() as i64,
            );
            rc.push(rx::excel_expm1(t).to_bits() as i64 - em_x.to_bits() as i64);
        }
        println!(" [{}]", tn);
        hist("  bigt u-1 (f64 exp)", &ra);
        hist("  bigt RN53(u_ext-1)", &rb);
        hist("  bigt portable expm1(t)", &rc);
    }

    // Also test t delivered fully extended into exp for large |t|
    println!("\n=== LARGE-|t|: t EXTENDED into exp (no store of t), u-1 ===");
    let mut re = Vec::new();
    let mut rf = Vec::new();
    for (r, n, em_x) in &big {
        let tex = t_ext_of(*r, *n as f64);
        let uext = exp_ext(&tex);
        // u stored to f64, then u-1 f64
        let u = tf(&uext);
        re.push((u - 1.0).to_bits() as i64 - em_x.to_bits() as i64);
        // extended u-1, single store
        rf.push(
            tf(&rx::ext_sub(&uext, &rx::ext_one(), CW)).to_bits() as i64 - em_x.to_bits() as i64,
        );
    }
    hist("bigt extT->exp, u(f64)-1", &re);
    hist("bigt extT->exp, RN53(u_ext-1)", &rf);
}
