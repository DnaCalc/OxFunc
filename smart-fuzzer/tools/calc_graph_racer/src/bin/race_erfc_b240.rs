//! Combine best 1-bit stores: bit6 (i=2 xnum) + bit9 (i=3 xden). Not an identity.
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
    let masks = [
        ("unmask", 0u32),
        ("bit6 i2xnum", 0x40u32),
        ("bit9 i3xden", 0x200u32),
        ("bit6+9", 0x240u32),
        ("0x210", 0x210u32),
        ("0x210+bit6", 0x250u32),
    ];
    println!("2-bit combos DIRECT [0.5,4):");
    for &(name, m) in &masks {
        let mut fused = 0usize;
        let mut conv = 0usize;
        let mut lose = 0usize;
        let mut n = 0usize;
        for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
            n += 1;
            let t = f64::from_bits(r.qbits);
            let u = ulp_distance(mul(f::w_rn53(r.z), cody_mask(r.z, 0)), t).unwrap_or(99) == 0;
            let h = ulp_distance(mul(f::w_rn53(r.z), cody_mask(r.z, m)), t).unwrap_or(99) == 0;
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
        println!(
            "  {name} mask={m:#x} fused={fused}/{n} CONV={conv} LOSE={lose} net={:+}",
            conv as i32 - lose as i32
        );
    }
}
