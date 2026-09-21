//! Cephes P/Q last-div / assoc vs 3 stubborn leftover-low. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const MASK: u32 = 0x5005;
const CEPHES_P: [f64; 9] = [
    2.46196981473530512524e-10,
    5.64189564831068821977e-1,
    7.46321056442269912687e0,
    4.86371970985681366614e1,
    1.96520832956077098242e2,
    5.26445194995477358631e2,
    9.34528527171957607540e2,
    1.02755188689515710272e3,
    5.57535335369399327526e2,
];
const CEPHES_Q: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];
const STUB: [f64; 3] = [4.0520833333333330, 5.3333333333333330, 6.0000000000000000];

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
fn parts(x: f64) -> (Ext80, Ext80) {
    let xe = ef(x.abs());
    let (num, b) = {
        let mut ans = ef(CEPHES_P[0]);
        let mut bit = 0u32;
        for &c in &CEPHES_P[1..] {
            ans = ext_add(&ext_mul(&ans, &xe, CW), &ef(c), CW);
            ans = maybe(ans, MASK, bit);
            bit += 1;
        }
        (ans, bit)
    };
    let mut den = ext_add(&xe, &ef(CEPHES_Q[0]), CW);
    den = maybe(den, MASK, b);
    let mut bit = b + 1;
    for &c in &CEPHES_Q[1..] {
        den = ext_add(&ext_mul(&den, &xe, CW), &ef(c), CW);
        den = maybe(den, MASK, bit);
        bit += 1;
    }
    (num, den)
}
fn specfun(z: f64) -> f64 {
    let (num, den) = parts(z);
    let mut q = ext_div(&num, &den, CW);
    q = maybe(q, MASK, 16);
    ext_to_f64(&q, CW)
}
fn qw(z: f64, ff: f64) -> f64 {
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
    let tail: Vec<&f::QRow> = rows.iter().filter(|r| r.direct && r.z >= 4.0).collect();
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 8] = [
        ("specfun", Box::new(|z| qw(z, specfun(z)))),
        (
            "f64div",
            Box::new(|z| {
                let (n, d) = parts(z);
                qw(z, ext_to_f64(&n, CW) / ext_to_f64(&d, CW))
            }),
        ),
        (
            "st_n_d",
            Box::new(|z| {
                let (n, d) = parts(z);
                qw(
                    z,
                    ext_to_f64(&ext_div(&st(&n), &st(&d), CW), CW),
                )
            }),
        ),
        (
            "dn(den)",
            Box::new(|z| {
                let (n, d) = parts(z);
                let dd = ef(ext_to_f64(&d, CW).next_down());
                qw(z, ext_to_f64(&ext_div(&n, &dd, CW), CW))
            }),
        ),
        (
            "up(num)",
            Box::new(|z| {
                let (n, d) = parts(z);
                let nn = ef(ext_to_f64(&n, CW).next_up());
                qw(z, ext_to_f64(&ext_div(&nn, &d, CW), CW))
            }),
        ),
        (
            "up(den)",
            Box::new(|z| {
                let (n, d) = parts(z);
                let dd = ef(ext_to_f64(&d, CW).next_up());
                qw(z, ext_to_f64(&ext_div(&n, &dd, CW), CW))
            }),
        ),
        (
            "upF",
            Box::new(|z| qw(z, specfun(z).next_up())),
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
    println!("3 stubborn P/Q last-div:");
    for &lz in &STUB {
        let Some(r) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
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
        let mut qd = 0usize;
        let mut qe = 0usize;
        let mut hit = 0usize;
        for row in rows.iter().filter(|rr| rr.z >= 4.0) {
            let g = ev(row.z);
            if !g.is_finite() {
                continue;
            }
            if ulp_distance(g, f64::from_bits(row.qbits)).unwrap_or(99) == 0 {
                qe += 1;
                if row.direct {
                    qd += 1;
                }
            }
        }
        for &lz in &STUB {
            let Some(r) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
                continue;
            };
            if ulp_distance(ev(lz), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                hit += 1;
            }
        }
        println!("{name:10} hit={hit}/3 Q={qe} d={qd}");
    }
}
