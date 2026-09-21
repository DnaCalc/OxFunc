//! Cody unmask vs DERFC0 vs 0x210 fused/last-store overlap DIRECT [0.5,4). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const C0: [f64; 9] = [
    0.564188496988670089,
    8.88314979438837594,
    66.1191906371416295,
    298.635138197400131,
    881.95222124176909,
    1712.04761263407058,
    2051.07837782607147,
    1230.33935479799725,
    2.15311535474403846e-8,
];
const D0: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn poke(x: f64, k: i32) -> f64 {
    let mut v = x;
    if k > 0 {
        for _ in 0..k {
            v = v.next_up();
        }
    } else {
        for _ in 0..(-k) {
            v = v.next_down();
        }
    }
    v
}
fn cody0(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ef(C0[7]), CW),
            &ext_add(&xden, &ef(D0[7]), CW),
            CW,
        ),
        CW,
    )
}
fn cody210(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        let bn = 2 + 2 * i;
        let bd = 3 + 2 * i;
        if 0x210u32 & (1u32 << bn) != 0 {
            xnum = ef(ext_to_f64(&xnum, CW));
        }
        if 0x210u32 & (1u32 << bd) != 0 {
            xden = ef(ext_to_f64(&xden, CW));
        }
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ef(C0[7]), CW),
            &ext_add(&xden, &ef(D0[7]), CW),
            CW,
        ),
        CW,
    )
}
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn hit_ls(z: f64, t: f64, ff: f64) -> bool {
    (-4i32..=4)
        .filter(|&k| k != 0)
        .any(|k| ulp_distance(mul(f::w_rn53(z), poke(ff, k)), t).unwrap_or(99) == 0)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut n0 = 0usize;
    let mut n210 = 0usize;
    let mut nd = 0usize;
    let mut nce = 0usize;
    let mut b_0d = 0usize;
    let mut o_0 = 0usize;
    let mut o_d = 0usize;
    let mut b_210d = 0usize;
    let mut o_210 = 0usize;
    let mut o_d2 = 0usize;
    let mut b_0210 = 0usize;
    let mut u_ls = 0usize;
    let mut b_ls = 0usize;
    let mut n = 0usize;
    println!("DIRECT [0.5,4) fused overlap:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        n += 1;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let f0 = cody0(r.z);
        let f210 = cody210(r.z);
        let fd = f::nswc_derfc0(r.z);
        let fce = f::cephes_f(r.z);
        let e0 = ulp_distance(mul(w, f0), t).unwrap_or(99) == 0;
        let e210 = ulp_distance(mul(w, f210), t).unwrap_or(99) == 0;
        let ed = ulp_distance(mul(w, fd), t).unwrap_or(99) == 0;
        let ece = ulp_distance(mul(w, fce), t).unwrap_or(99) == 0;
        if e0 {
            n0 += 1;
        }
        if e210 {
            n210 += 1;
        }
        if ed {
            nd += 1;
        }
        if ece {
            nce += 1;
        }
        match (e0, ed) {
            (true, true) => b_0d += 1,
            (true, false) => o_0 += 1,
            (false, true) => o_d += 1,
            _ => {}
        }
        match (e210, ed) {
            (true, true) => b_210d += 1,
            (true, false) => o_210 += 1,
            (false, true) => o_d2 += 1,
            _ => {}
        }
        if e0 && e210 {
            b_0210 += 1;
        }
        let ls0 = e0 || hit_ls(r.z, t, f0);
        let lsd = ed || hit_ls(r.z, t, fd);
        if ls0 || lsd {
            u_ls += 1;
        }
        if ls0 && lsd {
            b_ls += 1;
        }
    }
    println!("fused unmask={n0} 0x210={n210} derfc0={nd} cephes_f={nce} n={n}");
    println!("unmask vs derfc0 both={b_0d} only_unmask={o_0} only_derfc0={o_d}");
    println!("0x210 vs derfc0 both={b_210d} only_210={o_210} only_derfc0={o_d2}");
    println!("unmask ∩ 0x210 fused={b_0210}");
    println!("last-store unmask∪derfc0={u_ls} both_ls={b_ls} (each union is 226/226)");
}
