//! PC53 vs PC64 Cody F ulp vs last-store k on 0x210 leftover-low tmu. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC53_RN, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const MASK: u32 = 0x210;
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody_x87(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn cody_cw(y: f64, mask: u32, cw: u16) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, cw);
    let mut xden = ye;
    if mask & 1 != 0 {
        xnum = ef(ext_to_f64(&xnum, cw));
    }
    if mask & 2 != 0 {
        xden = ef(ext_to_f64(&xden, cw));
    }
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), cw), &ye, cw);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), cw), &ye, cw);
        if mask & (1u32 << (2 + 2 * i as u32)) != 0 {
            xnum = ef(ext_to_f64(&xnum, cw));
        }
        if mask & (1u32 << (3 + 2 * i as u32)) != 0 {
            xden = ef(ext_to_f64(&xden, cw));
        }
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), cw),
        &ext_add(&xden, &ef(D0[7]), cw),
        cw,
    );
    if mask & (1u32 << 16) != 0 {
        q = ef(ext_to_f64(&q, cw));
    }
    ext_to_f64(&q, cw)
}
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn hit(g: f64, t: f64) -> bool {
    ulp_distance(g, t).unwrap_or(99) == 0
}
fn signed_ulp(a: f64, b: f64) -> i32 {
    if a == b {
        return 0;
    }
    let mut n = 0i32;
    let mut v = b;
    if a > b {
        while v < a && n < 16 {
            v = v.next_up();
            n += 1;
        }
        n
    } else {
        while v > a && n < 16 {
            v = v.next_down();
            n -= 1;
        }
        n
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut nlo = 0usize;
    let mut ntmu = 0usize;
    let mut nmatch = 0usize;
    println!("PC53 vs PC64 Cody F ulp vs last-store k on 0x210 leftover-low tmu:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let g210 = mul(w, cody_x87(z, MASK));
        if !(ulp_distance(g210, t).unwrap_or(99) >= 2 && g210 < t) {
            continue;
        }
        nlo += 1;
        let fx = cody_x87(z, 0);
        let fnat = cody_cw(z, 0, CW_PC53_RN);
        let tmu = hit(mul(w.next_up(), fx.next_up()), t);
        if !tmu {
            continue;
        }
        ntmu += 1;
        let du = signed_ulp(fnat, fx);
        let mut k = None;
        for kk in 0i32..=8 {
            if ulp_distance(mul(w, {
                let mut v = fx;
                for _ in 0..kk {
                    v = v.next_up();
                }
                v
            }), t)
            .unwrap_or(99)
                == 0
            {
                k = Some(kk);
                break;
            }
        }
        let m = k == Some(du);
        if m {
            nmatch += 1;
        }
        println!(
            "  z={:.16} k={k:?} PC53-PC64_F={du} match={m} Q_pc53={} Q_pc64={}",
            z,
            ulp_distance(mul(w, fnat), t).unwrap_or(99),
            ulp_distance(mul(w, fx), t).unwrap_or(99)
        );
    }
    println!("0x210 leftover-low tmu n={ntmu}/{nlo} PC53-PC64 F ulp matches k={nmatch}/{ntmu}");
}
