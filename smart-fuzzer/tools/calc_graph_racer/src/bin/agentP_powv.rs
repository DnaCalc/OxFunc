//! Test em = v-1 where v=(1+r)^-n via the POW chain exp(-n*ln(1+r)) [1+r exact
//! for po2 r], vs expm1-Kahan. Clean 46-oracle. Various product/store choices.
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
fn exp_ext_arg(arg: &rx::Ext80) -> rx::Ext80 {
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
    let miss = [(-5, 2), (-5, 3), (-5, 5), (-5, 12), (-5, 24), (-4, 2)];
    let variants: Vec<(&str, Box<dyn Fn(f64, i64) -> f64>)> = vec![
        // pow chain: t2 = RN53(-n * ln(1+r)) [double-rounded x87 product], v=exp(t2), em=v-1
        (
            "v=exp(RN(-n*ln(1+r)))-1",
            Box::new(|r, n| {
                let l = rx::excel_ln(1.0 + r);
                let t2 = rx::x87_mul(-(n as f64), l);
                rx::excel_exp(t2) - 1.0
            }),
        ),
        // pow chain SSE product
        (
            "v=exp(SSE -n*ln(1+r))-1",
            Box::new(|r, n| {
                let l = rx::excel_ln(1.0 + r);
                let t2 = -(n as f64) * l;
                rx::excel_exp(t2) - 1.0
            }),
        ),
        // ln(1+r) EXTENDED, *-n extended, exp extended, v-1 f64
        (
            "v=exp_ext(-n*lnext(1+r)) f64 -1",
            Box::new(|r, n| {
                let le = rx::ext_fyl2x(&rx::ext_ln2(), &e(1.0 + r), CW);
                let t2 = rx::ext_mul(&e(-(n as f64)), &le, CW);
                let v = tf(&exp_ext_arg(&t2));
                v - 1.0
            }),
        ),
        // ln(1+r) EXTENDED, exp extended, RN53(v_ext-1)
        (
            "RN53(exp_ext(-n*lnext)-1)",
            Box::new(|r, n| {
                let le = rx::ext_fyl2x(&rx::ext_ln2(), &e(1.0 + r), CW);
                let t2 = rx::ext_mul(&e(-(n as f64)), &le, CW);
                let ve = exp_ext_arg(&t2);
                tf(&rx::ext_sub(&ve, &rx::ext_one(), CW))
            }),
        ),
        // log1p (fyl2xp1) extended, exp extended, RN53(v_ext-1) -- the em stable chain but v-1 not Kahan
        (
            "RN53(exp_ext(-n*log1pext)-1)",
            Box::new(|r, n| {
                let le = rx::ext_fyl2xp1(&rx::ext_ln2(), &e(r), CW);
                let t2 = rx::ext_mul(&e(-(n as f64)), &le, CW);
                let ve = exp_ext_arg(&t2);
                tf(&rx::ext_sub(&ve, &rx::ext_one(), CW))
            }),
        ),
    ];
    for (name, f) in &variants {
        let mut ok = 0;
        let mut d6 = Vec::new();
        for (r, n, em) in &rows {
            let d = f(*r, *n).to_bits() as i64 - em.to_bits() as i64;
            if d == 0 {
                ok += 1;
            }
            let k = (r.log2().round()) as i32;
            if miss.contains(&(k, *n)) {
                d6.push(((k, *n), d));
            }
        }
        print!("  {:34} {:2}/46  on-6:", name, ok);
        for ((k, n), d) in &d6 {
            print!(" (2^{},{}):{:+}", k, n, d);
        }
        println!();
    }
}
