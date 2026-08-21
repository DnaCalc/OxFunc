//! Test internal-exp publication variants for u in the Kahan, on clean 46-oracle.
use oxfunc_core::excel_numeric::research as rx;
const CW: u16 = rx::CW_PC64_RN;
const CW_RZ: u16 = rx::CW_PC64_RN | 0x0C00;
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
// exp chain, arg f64, with selectable final store rounding
fn exp_store(t: f64, cw_store: u16) -> f64 {
    let te = e(t);
    let z = rx::ext_mul(&te, &rx::ext_l2e(), CW);
    let k = rx::ext_rndint(&z, CW);
    let f = rx::ext_sub(&z, &k, CW);
    let neg = tf(&f) < 0.0;
    let w = rx::ext_f2xm1(&rx::ext_abs(&f, CW), CW);
    let mut m = rx::ext_add(&w, &rx::ext_one(), CW);
    if neg {
        m = rx::ext_div(&rx::ext_one(), &m, CW);
    }
    rx::ext_to_f64(&rx::ext_scale(&m, &k, CW), cw_store)
}
fn kahan(u: f64, t: f64) -> f64 {
    if u == 1.0 {
        t
    } else if t.abs() < 1.0 {
        (u - 1.0) * t / rx::excel_ln(u)
    } else {
        u - 1.0
    }
}
fn main() {
    let rows = load("../../work/w109/G6-solvers/pmt_em_hdf_oracle.json");
    let variants: [(&str, fn(f64) -> f64); 3] = [
        ("u=exp RN(ref)", |t| rx::excel_exp(t)),
        ("u=exp RZ(chop)", |t| exp_store(t, CW_RZ)),
        ("u=exp_rz api", |t| rx::excel_exp_rz(t)),
    ];
    for (name, ef) in variants {
        let mut ok = 0;
        let mut miss = Vec::new();
        for (r, n, em) in &rows {
            let t = -(*n as f64) * rx::excel_log1p(*r);
            let u = ef(t);
            let val = kahan(u, t);
            if val.to_bits() == em.to_bits() {
                ok += 1;
            } else {
                miss.push((*r, *n, val.to_bits() as i64 - em.to_bits() as i64));
            }
        }
        print!("  {:20} {:2}/46", name, ok);
        if ok >= 38 {
            print!("  miss:");
            for (r, n, d) in &miss {
                print!(" (2^{},{}):{:+}", (r.log2().round()) as i32, n, d);
            }
        }
        println!();
    }
    // Also: is the divide in the Kahan perhaps x87 double-rounded while product SSE, etc?
    // And: what if lnu uses exp's EXTENDED result (u_ext) but u-1 uses f64 u?
    println!();
    let mut ok = 0;
    let mut miss = Vec::new();
    for (r, n, em) in &rows {
        let t = -(*n as f64) * rx::excel_log1p(*r);
        // exp extended u
        let te = e(t);
        let z = rx::ext_mul(&te, &rx::ext_l2e(), CW);
        let k = rx::ext_rndint(&z, CW);
        let f = rx::ext_sub(&z, &k, CW);
        let neg = tf(&f) < 0.0;
        let w = rx::ext_f2xm1(&rx::ext_abs(&f, CW), CW);
        let mut m = rx::ext_add(&w, &rx::ext_one(), CW);
        if neg {
            m = rx::ext_div(&rx::ext_one(), &m, CW);
        }
        let ue = rx::ext_scale(&m, &k, CW);
        let u = tf(&ue);
        let val = if u == 1.0 {
            t
        } else if t.abs() < 1.0 {
            let lnu_ext = tf(&rx::ext_fyl2x(&rx::ext_ln2(), &ue, CW)); // ln of EXTENDED u, stored
            (u - 1.0) * t / lnu_ext
        } else {
            u - 1.0
        };
        if val.to_bits() == em.to_bits() {
            ok += 1;
        } else {
            miss.push((*r, *n, val.to_bits() as i64 - em.to_bits() as i64));
        }
    }
    print!("  {:20} {:2}/46", "lnu=ln(u_ext)", ok);
    if ok >= 38 {
        print!("  miss:");
        for (r, n, d) in &miss {
            print!(" (2^{},{}):{:+}", (r.log2().round()) as i32, n, d);
        }
    }
    println!();
}
