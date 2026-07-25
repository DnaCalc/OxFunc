//! agent-P: dump exact f64 bits of the Kahan sub-ops for all n=1 pinned cases,
//! plus extended-precision variants, so Python/mpmath can decode the exact
//! rounding structure. CSV to stdout.
use oxfunc_core::excel_numeric::research as rx;

const CW: u16 = rx::CW_PC64_RN;

fn fb(h: &str) -> f64 {
    f64::from_bits(u64::from_str_radix(h.trim_start_matches("0x"), 16).unwrap())
}
fn hx(x: f64) -> String {
    format!("0x{:016x}", x.to_bits())
}
fn ext_hex(e: &rx::Ext80) -> String {
    let mut s = String::from("0x");
    for b in e.0.iter().rev() {
        s.push_str(&format!("{:02x}", b));
    }
    s
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

fn main() {
    let rows = load_pinned("../../work/w109/G6-solvers/pmt_em_pinned.json");
    let mut n1: Vec<_> = rows.iter().filter(|(_, n, _)| *n == 1).cloned().collect();
    n1.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    // header
    println!("r,em_excel,t_f64,u_f64,lnu_f64,t_ext,u_ext,lnu_ext,logp_fyl2xp1_ext");
    for (r, _, em_x) in &n1 {
        let t = -rx::excel_log1p(*r); // f64 t
        let u = rx::excel_exp(t);
        let lnu = rx::excel_ln(u);
        // extended chain: t_ext = -fyl2xp1(ln2,r); u_ext = exp_ext(t_ext); lnu_ext = ln_ext(u_ext)
        let lp_ext = rx::ext_fyl2xp1(&rx::ext_ln2(), &rx::ext_from_f64(*r), CW);
        let t_ext = rx::ext_sub(&rx::ext_from_f64(0.0), &lp_ext, CW); // -log1p, extended
        // exp_ext
        let z = rx::ext_mul(&t_ext, &rx::ext_l2e(), CW);
        let k = rx::ext_rndint(&z, CW);
        let f = rx::ext_sub(&z, &k, CW);
        let neg = rx::ext_to_f64(&f, CW) < 0.0;
        let w = rx::ext_f2xm1(&rx::ext_abs(&f, CW), CW);
        let mut m = rx::ext_add(&w, &rx::ext_one(), CW);
        if neg {
            m = rx::ext_div(&rx::ext_one(), &m, CW);
        }
        let u_ext = rx::ext_scale(&m, &k, CW);
        let lnu_ext = rx::ext_fyl2x(&rx::ext_ln2(), &u_ext, CW);
        println!(
            "{},{},{},{},{},{},{},{},{}",
            hx(*r),
            hx(*em_x),
            hx(t),
            hx(u),
            hx(lnu),
            ext_hex(&t_ext),
            ext_hex(&u_ext),
            ext_hex(&lnu_ext),
            ext_hex(&lp_ext),
        );
    }
}
