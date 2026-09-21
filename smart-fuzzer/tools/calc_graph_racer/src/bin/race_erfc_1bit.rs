//! Single-bit Cody Horner stores vs unmask DIRECT [0.5,4). Not an identity.
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
fn cody_mask(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        let bn = 2 + 2 * i;
        let bd = 3 + 2 * i;
        if mask & (1u32 << bn) != 0 {
            xnum = ef(ext_to_f64(&xnum, CW));
        }
        if mask & (1u32 << bd) != 0 {
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let dirs: Vec<&f::QRow> = rows
        .iter()
        .filter(|r| r.direct && r.z >= 0.5 && r.z < 4.0)
        .collect();
    let n = dirs.len();
    let score = |mask: u32| -> (usize, usize, usize) {
        let mut fused = 0usize;
        let mut conv = 0usize;
        let mut lose = 0usize;
        for r in &dirs {
            let t = f64::from_bits(r.qbits);
            let u = ulp_distance(mul(f::w_rn53(r.z), cody_mask(r.z, 0)), t).unwrap_or(99) == 0;
            let h = ulp_distance(mul(f::w_rn53(r.z), cody_mask(r.z, mask)), t).unwrap_or(99) == 0;
            if h {
                fused += 1;
            }
            if h && !u {
                conv += 1;
            }
            if u && !h {
                lose += 1;
            }
        }
        (fused, conv, lose)
    };
    let (b, _, _) = score(0);
    println!("unmask fused={b}/{n}");
    println!("1-bit stores (i=0..6 xnum bit=2+2i, xden bit=3+2i):");
    let mut best = b;
    let mut best_desc = String::from("unmask");
    for i in 0..7u32 {
        for &(side, off) in &[("xnum", 2u32), ("xden", 3u32)] {
            let bit = off + 2 * i;
            let mask = 1u32 << bit;
            let (fused, conv, lose) = score(mask);
            let net = conv as i32 - lose as i32;
            println!("  i={i} {side} bit={bit} mask={mask:#x} fused={fused} CONV={conv} LOSE={lose} net={net:+}");
            if fused > best {
                best = fused;
                best_desc = format!("i={i} {side}");
            }
        }
    }
    let (f210, c210, l210) = score(0x210);
    println!(
        "0x210 fused={f210} CONV={c210} LOSE={l210} net={:+}  best-1bit={best} {best_desc} (bar 105)",
        c210 as i32 - l210 as i32
    );
}
