//! 0x210 leftover-low: (w±k)×F original vs F±k. Analog of lead+2. Not an identity.
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
fn cody(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, 0x210, 0);
    xden = maybe(xden, 0x210, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, 0x210, 2 + 2 * i as u32);
        xden = maybe(xden, 0x210, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, 0x210, 16);
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
    let mut lows: Vec<(f64, u64)> = Vec::new();
    for r in &dirs {
        let g = mul(f::w_rn53(r.z), cody(r.z));
        let t = f64::from_bits(r.qbits);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d >= 2 && g < t {
            lows.push((r.z, r.qbits));
        }
    }
    println!("0x210 leftover-low n={}", lows.len());
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 6] = [
        ("wF", Box::new(|z| mul(f::w_rn53(z), cody(z)))),
        ("w+1", Box::new(|z| mul(f::w_rn53(z).next_up(), cody(z)))),
        (
            "w+2",
            Box::new(|z| mul(f::w_rn53(z).next_up().next_up(), cody(z))),
        ),
        ("F+1", Box::new(|z| mul(f::w_rn53(z), cody(z).next_up()))),
        (
            "F+2",
            Box::new(|z| mul(f::w_rn53(z), cody(z).next_up().next_up())),
        ),
        (
            "w+1F+1",
            Box::new(|z| mul(f::w_rn53(z).next_up(), cody(z).next_up())),
        ),
    ];
    for (name, ev) in &graphs {
        let mut dmid = 0usize;
        let mut hit = 0usize;
        for r in &dirs {
            if ulp_distance(ev(r.z), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                dmid += 1;
            }
        }
        for &(lz, bits) in &lows {
            if ulp_distance(ev(lz), f64::from_bits(bits)).unwrap_or(99) == 0 {
                hit += 1;
            }
        }
        println!("{name:8} leftover-low hit={hit}/{} dmid={dmid}/{}", lows.len(), dirs.len());
    }
}
