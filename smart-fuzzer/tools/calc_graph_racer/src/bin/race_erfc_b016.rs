//! Cody mask bits 0,1,16 and unions with 0x210. Not an identity.
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody_mask(y: f64, mask: u32) -> f64 {
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
    let masks: [(&str, u32); 12] = [
        ("unmask", 0),
        ("bit0 C8*y", 1),
        ("bit1 xden=y", 2),
        ("bit16 quot", 1 << 16),
        ("0x210", 0x210),
        ("0x210|bit0", 0x210 | 1),
        ("0x210|bit1", 0x210 | 2),
        ("0x210|bit16", 0x210 | (1 << 16)),
        ("bit0+bit9", 1 | 0x200),
        ("bit1+bit9", 2 | 0x200),
        ("bit16+bit9", (1 << 16) | 0x200),
        ("bit0+bit1", 3),
    ];
    println!("bits 0/1/16 and 0x210 unions DIRECT [0.5,4) n={n}:");
    for &(name, m) in &masks {
        let (fused, conv, lose) = score(m);
        println!(
            "  {name} mask={m:#x} fused={fused} CONV={conv} LOSE={lose} net={:+}",
            conv as i32 - lose as i32
        );
    }
}
