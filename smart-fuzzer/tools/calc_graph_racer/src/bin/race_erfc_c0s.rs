//! 0x210 C[0] / C[7] steps vs dmid / leftover-low / pin 0.5. Not an identity.
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn cody(y: f64, c: &[f64; 9], d: &[f64; 8]) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, 0x210, 0);
    xden = maybe(xden, 0x210, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
        xnum = maybe(xnum, 0x210, 2 + 2 * i as u32);
        xden = maybe(xden, 0x210, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(d[7]), CW),
        CW,
    );
    q = maybe(q, 0x210, 16);
    ext_to_f64(&q, CW)
}
fn qwf(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
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
    let mut lows: Vec<f64> = Vec::new();
    for r in &dirs {
        let g = qwf(r.z, cody(r.z, &C0, &D0));
        let t = f64::from_bits(r.qbits);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d >= 2 && g < t {
            lows.push(r.z);
        }
    }
    println!(
        "0x210 C[0] steps (bar dmid 105 / leftover-low {} / n={}):",
        lows.len(),
        dirs.len()
    );
    for k in 0i32..=8 {
        let mut c = C0;
        c[0] = poke(C0[0], k);
        let mut dmid = 0usize;
        let mut low_hit = 0usize;
        let mut pin = 99u64;
        for r in &dirs {
            let g = qwf(r.z, cody(r.z, &c, &D0));
            let t = f64::from_bits(r.qbits);
            let d = ulp_distance(g, t).unwrap_or(99);
            if d == 0 {
                dmid += 1;
            }
            if (r.z - 0.5).abs() < 1e-15 {
                pin = d;
            }
        }
        for &lz in &lows {
            let Some(r) = dirs.iter().find(|r| (r.z - lz).abs() < 1e-15) else {
                continue;
            };
            if ulp_distance(qwf(lz, cody(lz, &c, &D0)), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                low_hit += 1;
            }
        }
        println!("  C[0] {k:+} dmid={dmid} leftover-low hit={low_hit}/{} pin0.5={pin}", lows.len());
    }
    println!("C[7] steps:");
    for k in 0i32..=8 {
        let mut c = C0;
        c[7] = poke(C0[7], k);
        let mut dmid = 0usize;
        let mut low_hit = 0usize;
        let mut pin = 99u64;
        for r in &dirs {
            let g = qwf(r.z, cody(r.z, &c, &D0));
            let t = f64::from_bits(r.qbits);
            let d = ulp_distance(g, t).unwrap_or(99);
            if d == 0 {
                dmid += 1;
            }
            if (r.z - 0.5).abs() < 1e-15 {
                pin = d;
            }
        }
        for &lz in &lows {
            let Some(r) = dirs.iter().find(|r| (r.z - lz).abs() < 1e-15) else {
                continue;
            };
            if ulp_distance(qwf(lz, cody(lz, &c, &D0)), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                low_hit += 1;
            }
        }
        println!("  C[7] {k:+} dmid={dmid} leftover-low hit={low_hit}/{} pin0.5={pin}", lows.len());
    }
    println!("D[7] steps:");
    for k in -8i32..=2 {
        let mut d7 = D0;
        d7[7] = poke(D0[7], k);
        let mut dmid = 0usize;
        let mut low_hit = 0usize;
        let mut pin = 99u64;
        for r in &dirs {
            let g = qwf(r.z, cody(r.z, &C0, &d7));
            let t = f64::from_bits(r.qbits);
            let d = ulp_distance(g, t).unwrap_or(99);
            if d == 0 {
                dmid += 1;
            }
            if (r.z - 0.5).abs() < 1e-15 {
                pin = d;
            }
        }
        for &lz in &lows {
            let Some(r) = dirs.iter().find(|rr| (rr.z - lz).abs() < 1e-15) else {
                continue;
            };
            if ulp_distance(qwf(lz, cody(lz, &C0, &d7)), f64::from_bits(r.qbits)).unwrap_or(99) == 0
            {
                low_hit += 1;
            }
        }
        println!("  D[7] {k:+} dmid={dmid} leftover-low hit={low_hit}/{} pin0.5={pin}", lows.len());
    }
}
