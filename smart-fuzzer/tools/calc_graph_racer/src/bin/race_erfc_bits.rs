//! Ablate 0x210 bits 4 and 9 vs unmask DIRECT [0.5,4). Not an identity.
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
fn hit0(z: f64, t: f64, mask: u32) -> bool {
    ulp_distance(mul(f::w_rn53(z), cody_mask(z, mask)), t).unwrap_or(99) == 0
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let masks = [
        ("unmask", 0u32),
        ("bit4", 0x10u32),
        ("bit9", 0x200u32),
        ("0x210", 0x210u32),
    ];
    let mut fused = [0usize; 4];
    let mut n = 0usize;
    let mut conv = [0usize; 4];
    let mut lose = [0usize; 4];
    println!("mask fused / CONV / LOSE vs unmask:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        n += 1;
        let t = f64::from_bits(r.qbits);
        let u = hit0(r.z, t, 0);
        for (i, &(_, m)) in masks.iter().enumerate() {
            let h = hit0(r.z, t, m);
            if h {
                fused[i] += 1;
            }
            if i > 0 {
                if h && !u {
                    conv[i] += 1;
                }
                if u && !h {
                    lose[i] += 1;
                }
            }
        }
    }
    for (i, &(name, m)) in masks.iter().enumerate() {
        println!(
            "  {name} mask={m:#x} fused={}/{} CONV={} LOSE={} net={:+}",
            fused[i],
            n,
            conv[i],
            lose[i],
            conv[i] as i32 - lose[i] as i32
        );
    }
    let mut both = 0usize;
    let mut only4 = 0usize;
    let mut only9 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let u = hit0(r.z, t, 0);
        let b4 = hit0(r.z, t, 0x10);
        let b9 = hit0(r.z, t, 0x200);
        let g4 = b4 && !u;
        let g9 = b9 && !u;
        match (g4, g9) {
            (true, true) => both += 1,
            (true, false) => only4 += 1,
            (false, true) => only9 += 1,
            _ => {}
        }
    }
    println!("CONV vs unmask: both_bits={both} only_bit4={only4} only_bit9={only9}");
}
