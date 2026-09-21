//! PQR R[1]+1 t=(z-3.75)/(z+3.75) nudge vs 6 leftover-low. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use std::env;

const SIX: [f64; 6] = [
    4.0520833333333330,
    4.0625000000000000,
    5.2395833333333330,
    5.2708333333333330,
    5.3333333333333330,
    6.0000000000000000,
];
const P0: [f64; 8] = [
    0.16506148041280876191828601e-03,
    0.15471455377139313353998665e-03,
    0.44852548090298868465196794e-04,
    -0.49177280017226285450486205e-05,
    -0.69353602078656412367801676e-05,
    -0.20508667787746282746857743e-05,
    -0.28982842617824971177267380e-06,
    -0.17272433544836633301127174e-07,
];
const Q0: [f64; 8] = [
    1.0,
    0.16272656776533322859856317e+01,
    0.12040996037066026106794322e+01,
    0.52400246352158386907601472e+00,
    0.14497345252798672362384241e+00,
    0.25592517111042546492590736e-01,
    0.26869088293991371028123158e-02,
    0.13133767840925681614496481e-03,
];
const R0: [f64; 9] = [
    0.145589721275038539045668824025,
    -0.273421931495426482902320421863,
    0.226008066916621506788789064272,
    -0.163571895523923805648814425592,
    0.102604312032193978662297299832,
    -0.548023266949835519254211506880e-01,
    0.241432239725390106956523668160e-01,
    -0.822062115403915116036874169600e-02,
    0.180296241564687154310619200000e-02,
];

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
fn horner(cs: &[f64], x: f64) -> f64 {
    let mut a = 0.0;
    for &c in cs.iter().rev() {
        a = a * x + c;
    }
    a
}
fn r1() -> [f64; 9] {
    let mut r = R0;
    r[1] = poke(R0[1], 1);
    r
}
fn pqr_t(x: f64, t: f64) -> f64 {
    let r = r1();
    let u = horner(&P0, x);
    let v = horner(&Q0, x);
    let mut acc = u / v;
    for &c in r.iter().rev() {
        acc = acc * t + c;
    }
    acc
}
fn t0(z: f64) -> f64 {
    (z - 3.75) / (z + 3.75)
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
    println!("6 leftover-low t=(z-3.75)/(z+3.75):");
    for &lz in &SIX {
        println!("  z={lz:.16} t={:.16}", t0(lz));
    }
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 8] = [
        ("R1", Box::new(|z| qw(z, pqr_t(z, t0(z))))),
        ("t+1", Box::new(|z| qw(z, pqr_t(z, t0(z).next_up())))),
        (
            "t+2",
            Box::new(|z| qw(z, pqr_t(z, t0(z).next_up().next_up()))),
        ),
        ("t-1", Box::new(|z| qw(z, pqr_t(z, t0(z).next_down())))),
        (
            "t-2",
            Box::new(|z| qw(z, pqr_t(z, t0(z).next_down().next_down()))),
        ),
        (
            "num+1",
            Box::new(|z| qw(z, pqr_t(z, (z - 3.75).next_up() / (z + 3.75)))),
        ),
        (
            "den-1",
            Box::new(|z| qw(z, pqr_t(z, (z - 3.75) / (z + 3.75).next_down()))),
        ),
        (
            "num-1",
            Box::new(|z| qw(z, pqr_t(z, (z - 3.75).next_down() / (z + 3.75)))),
        ),
    ];
    println!("6 leftover-low R[1]+1 t-nudge:");
    for &lz in &SIX {
        let Some(r) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
            continue;
        };
        let tgt = f64::from_bits(r.qbits);
        print!("  z={lz:.16}");
        for (name, ev) in &graphs {
            let g = ev(lz);
            let d = ulp_distance(g, tgt).unwrap_or(99);
            let dir = if g < tgt {
                "L"
            } else if g > tgt {
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
        for &lz in &SIX {
            let Some(r) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
                continue;
            };
            if ulp_distance(ev(lz), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                hit += 1;
            }
        }
        println!("{name:8} hit={hit}/6 Q={qe} d={qd}");
    }
}
