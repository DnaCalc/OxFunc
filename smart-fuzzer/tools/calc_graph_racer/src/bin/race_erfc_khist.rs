//! Last-store |k| histogram on DIRECT mid and tail. Not an identity.
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
fn libm_f(z: f64) -> f64 {
    let w = f::w_rn53(z);
    if w == 0.0 {
        0.0
    } else {
        libm::erfc(z) / w
    }
}
fn mink(z: f64, t: f64, ff: f64, maxk: i32) -> Option<i32> {
    for ak in 0..=maxk {
        for &s in &[-1i32, 1] {
            let k = if ak == 0 { 0 } else { s * ak };
            if ak == 0 && s < 0 {
                continue;
            }
            if ulp_distance(mul(f::w_rn53(z), poke(ff, k)), t).unwrap_or(99) == 0 {
                return Some(ak);
            }
            if ak == 0 {
                break;
            }
        }
    }
    None
}
fn hist(name: &str, n: usize, h: &[usize], none: usize) {
    print!("{name} n={n}");
    for (i, c) in h.iter().enumerate() {
        if *c > 0 {
            print!(" |k|={i}:{c}");
        }
    }
    if none > 0 {
        print!(" none={none}");
    }
    let cov: usize = h.iter().sum();
    println!(" cover={cov}/{}", n);
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let names = ["unmask", "0x210", "derfc0", "cephes", "libm"];
    println!("DIRECT [0.5,4) |k| hist:");
    let mut h = vec![[0usize; 9]; 5];
    let mut none = [0usize; 5];
    let mut nmid = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        nmid += 1;
        let t = f64::from_bits(r.qbits);
        let fs = [
            cody0(r.z),
            cody210(r.z),
            f::nswc_derfc0(r.z),
            f::cephes_f(r.z),
            libm_f(r.z),
        ];
        for i in 0..5 {
            match mink(r.z, t, fs[i], 8) {
                Some(ak) => h[i][ak as usize] += 1,
                None => none[i] += 1,
            }
        }
    }
    for i in 0..5 {
        hist(names[i], nmid, &h[i], none[i]);
    }
    println!("DIRECT z>=4 |k| hist:");
    let mut ht = vec![[0usize; 9]; 5];
    let mut nonet = [0usize; 5];
    let mut ntail = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 4.0) {
        ntail += 1;
        let t = f64::from_bits(r.qbits);
        let fs = [
            cody0(r.z),
            cody210(r.z),
            f::nswc_derfc0(r.z),
            f::cephes_f(r.z),
            libm_f(r.z),
        ];
        for i in 0..5 {
            match mink(r.z, t, fs[i], 8) {
                Some(ak) => ht[i][ak as usize] += 1,
                None => nonet[i] += 1,
            }
        }
    }
    for i in 0..5 {
        hist(names[i], ntail, &ht[i], nonet[i]);
    }
}
