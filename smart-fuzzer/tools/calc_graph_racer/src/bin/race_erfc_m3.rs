//! 0x210 leftover ulp≥3 last-div / two-factor. Not an identity.
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
const HARD: [f64; 6] = [
    2.4270833333333335,
    2.6562500000000000,
    3.0625000000000000,
    3.2708333333333335,
    3.4687500000000000,
    3.7500000000000000,
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
fn st(x: &Ext80) -> Ext80 {
    ef(ext_to_f64(x, CW))
}
fn parts(y: f64) -> (Ext80, Ext80) {
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
    (
        ext_add(&xnum, &ef(C0[7]), CW),
        ext_add(&xden, &ef(D0[7]), CW),
    )
}
fn specfun(z: f64) -> f64 {
    let (n, d) = parts(z);
    let mut q = ext_div(&n, &d, CW);
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
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 8] = [
        ("specfun", Box::new(|z| qwf(z, specfun(z)))),
        (
            "f64div",
            Box::new(|z| {
                let (n, d) = parts(z);
                qwf(z, ext_to_f64(&n, CW) / ext_to_f64(&d, CW))
            }),
        ),
        (
            "st_n_d",
            Box::new(|z| {
                let (n, d) = parts(z);
                qwf(z, ext_to_f64(&ext_div(&st(&n), &st(&d), CW), CW))
            }),
        ),
        (
            "dn(den)",
            Box::new(|z| {
                let (n, d) = parts(z);
                let dd = ef(ext_to_f64(&d, CW).next_down());
                qwf(z, ext_to_f64(&ext_div(&n, &dd, CW), CW))
            }),
        ),
        (
            "up(num)",
            Box::new(|z| {
                let (n, d) = parts(z);
                let nn = ef(ext_to_f64(&n, CW).next_up());
                qwf(z, ext_to_f64(&ext_div(&nn, &d, CW), CW))
            }),
        ),
        ("upF", Box::new(|z| qwf(z, specfun(z).next_up()))),
        (
            "up2F",
            Box::new(|z| qwf(z, specfun(z).next_up().next_up())),
        ),
        (
            "upw*upF",
            Box::new(|z| {
                let v = f::w_rn53(z).next_up() * specfun(z).next_up();
                if v.abs() < f64::MIN_POSITIVE {
                    0.0
                } else {
                    v
                }
            }),
        ),
    ];
    println!("0x210 leftover ulp>=3 last-div:");
    for &lz in &HARD {
        let Some(r) = dirs.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
            println!("  z={lz:.16} MISSING");
            continue;
        };
        let t = f64::from_bits(r.qbits);
        print!("  z={lz:.16}");
        for (name, ev) in &graphs {
            let g = ev(lz);
            let d = ulp_distance(g, t).unwrap_or(99);
            let dir = if g < t {
                "L"
            } else if g > t {
                "H"
            } else {
                "="
            };
            print!(" {name}={d}{dir}");
        }
        println!();
    }
    for (name, ev) in &graphs {
        let mut dmid = 0usize;
        let mut hit = 0usize;
        for r in &dirs {
            if ulp_distance(ev(r.z), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                dmid += 1;
            }
        }
        for &lz in &HARD {
            let Some(r) = dirs.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
                continue;
            };
            if ulp_distance(ev(lz), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                hit += 1;
            }
        }
        println!("{name:10} hit={hit}/6 dmid={dmid}/{}", dirs.len());
    }
}
