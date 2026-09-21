//! Named F vs 6 leftover-low in [4,8). Not an identity.
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
        ("erfc1", Box::new(|z| qw(z, f::cdflib_erfc1_f(z)))),
        (
            "up_e1",
            Box::new(|z| qw(z, f::cdflib_erfc1_f(z)).next_up()),
        ),
        ("lentz24", Box::new(|z| qw(z, f::cf_lentz_as714_n(z, 24)))),
        ("derfc0", Box::new(|z| qw(z, f::nswc_derfc0(z)))),
        ("ccdd", Box::new(|z| qw(z, f::nswc_ccdd_f(z)))),
        ("pqr", Box::new(|z| qw(z, f::nswc_pqr_f(z)))),
        ("cephes_f", Box::new(|z| qw(z, f::cephes_f(z)))),
        ("libm", Box::new(|z| libm::erfc(z))),
    ];
    println!("6 leftover-low named F:");
    for &lz in &SIX {
        let Some(r) = tail.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
            continue;
        };
        let t = f64::from_bits(r.qbits);
        print!("  z={lz:.16}");
        for (name, ev) in &graphs {
            let g = ev(lz);
            let d = ulp_distance(g, t).unwrap_or(99);
            let dir = if !g.is_finite() {
                "?"
            } else if g < t {
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
        let mut zs = Vec::new();
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
                zs.push(lz);
            }
        }
        println!("{name:10} hit={hit}/6 {zs:?} Q={qe} d={qd}");
    }
}
