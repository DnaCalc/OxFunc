//! D[7]−1 (dmid 108, pin kept) plus one ±1. Not an identity.
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
    let mut dbase = D0;
    dbase[7] = poke(D0[7], -1);
    let hit = |c: &[f64; 9], d: &[f64; 8], z: f64, bits: u64| {
        ulp_distance(qwf(z, cody(z, c, d)), f64::from_bits(bits)).unwrap_or(99) == 0
    };
    let sc = |c: &[f64; 9], d: &[f64; 8]| {
        let mut n = 0usize;
        let mut pin = false;
        let mut z240 = false;
        for r in &dirs {
            if hit(c, d, r.z, r.qbits) {
                n += 1;
                if (r.z - 0.5).abs() < 1e-15 {
                    pin = true;
                }
                if (r.z - 2.40625).abs() < 1e-12 {
                    z240 = true;
                }
            }
        }
        (n, pin, z240)
    };
    let (b, bp, b240) = sc(&C0, &dbase);
    println!("D[7]-1 dmid={b} pin={bp} z2.40625={b240} (bar 105 pin=true)");
    println!("extra ±1 (print pin and dmid>=108 or z240):");
    let mut best = b;
    let mut best_lab = "D[7]-1".to_string();
    for i in 0..9 {
        for k in [-1i32, 1] {
            let mut c = C0;
            c[i] = poke(C0[i], k);
            let (n, pin, z240) = sc(&c, &dbase);
            if pin && (n >= 108 || z240) {
                println!("  C[{i}] {k:+} dmid={n} pin={pin} z2.406={z240}");
            }
            if pin && n > best {
                best = n;
                best_lab = format!("D[7]-1 C[{i}] {k:+}");
            }
        }
    }
    for i in 0..7 {
        for k in [-1i32, 1] {
            let mut d = dbase;
            d[i] = poke(D0[i], k);
            let (n, pin, z240) = sc(&C0, &d);
            if pin && (n >= 108 || z240) {
                println!("  D[{i}] {k:+} dmid={n} pin={pin} z2.406={z240}");
            }
            if pin && n > best {
                best = n;
                best_lab = format!("D[7]-1 D[{i}] {k:+}");
            }
        }
    }
    println!("best pin-kept {best_lab} dmid={best}");
}
