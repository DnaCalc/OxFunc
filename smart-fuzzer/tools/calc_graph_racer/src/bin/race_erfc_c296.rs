//! native cephes_f vs x87 unmask at all3 leftover-low 4z. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ZB: [u64; 4] = [
    0x3ff4800000000000,
    0x3ffd555555555555,
    0x4007c00000000000,
    0x4009eaaaaaaaaaab,
];
const P0: [f64; 9] = [
    2.46196981473530512524e-10,
    5.64189564831068821977e-1,
    7.46321056442269912687e0,
    4.86371970985681366614e1,
    1.96520832956077098242e2,
    5.26445194995477358631e2,
    9.34528527171957607540e2,
    1.02755188689515710272e3,
    5.57535335369399327526e2,
];
const Q0: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn polevl_x87(x: Ext80, coef: &[f64]) -> Ext80 {
    let mut ans = ef(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
    }
    ans
}
fn p1evl_x87(x: Ext80, coef: &[f64]) -> Ext80 {
    let mut ans = ext_add(&x, &ef(coef[0]), CW);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
    }
    ans
}
fn cephes_x87(x: f64) -> f64 {
    let xe = ef(x.abs());
    let num = polevl_x87(xe, &P0);
    let den = p1evl_x87(xe, &Q0);
    ext_to_f64(&ext_div(&num, &den, CW), CW)
}
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut want = std::collections::BTreeMap::new();
    for r in rows.iter().filter(|rr| rr.direct) {
        let zb = r.z.to_bits();
        if ZB.contains(&zb) {
            want.insert(zb, r.qbits);
        }
    }
    println!("native cephes_f vs x87 unmask P/Q at all3 leftover-low 4z:");
    for zb in ZB {
        let z = f64::from_bits(zb);
        let t = f64::from_bits(*want.get(&zb).unwrap());
        let w = f::w_rn53(z);
        let fnat = f::cephes_f(z);
        let fx87 = cephes_x87(z);
        let d_ff = ulp_distance(fnat, fx87).unwrap_or(99);
        let dnat = ulp_distance(mul(w, fnat), t).unwrap_or(99);
        let dx87 = ulp_distance(mul(w, fx87), t).unwrap_or(99);
        println!(
            "  z={:.16} F_native={:#x} F_x87={:#x} F_ulp={d_ff} Q_native={dnat}{} Q_x87={dx87}{}",
            z,
            fnat.to_bits(),
            fx87.to_bits(),
            if dnat == 0 { " FUSED" } else { "" },
            if dx87 == 0 { " FUSED" } else { "" }
        );
    }
}
