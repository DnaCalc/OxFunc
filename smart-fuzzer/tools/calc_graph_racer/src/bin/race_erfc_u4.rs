//! 0x210 F+2 undershoot 4 vs named F. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use std::env;

const U4: [f64; 4] = [
    2.4270833333333335,
    2.6562500000000000,
    2.6875000000000000,
    3.0625000000000000,
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
    let dirs: Vec<&f::QRow> = rows
        .iter()
        .filter(|r| r.direct && r.z >= 0.5 && r.z < 4.0)
        .collect();
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 8] = [
        ("cody", Box::new(|z| qw(z, f::cody_erfcx_f(z)))),
        ("cephes_f", Box::new(|z| qw(z, f::cephes_f(z)))),
        ("pqr", Box::new(|z| qw(z, f::nswc_pqr_f(z)))),
        ("derfc0", Box::new(|z| qw(z, f::nswc_derfc0(z)))),
        ("erfc1", Box::new(|z| qw(z, f::cdflib_erfc1_f(z)))),
        ("lentz24", Box::new(|z| qw(z, f::cf_lentz_as714_n(z, 24)))),
        ("ccdd", Box::new(|z| qw(z, f::nswc_ccdd_f(z)))),
        ("libm", Box::new(|z| libm::erfc(z))),
    ];
    println!("4 F+2 undershoot named F:");
    for &lz in &U4 {
        let Some(r) = dirs.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
            println!("  z={lz:.16} MISSING");
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
        let mut dmid = 0usize;
        let mut hit = 0usize;
        let mut zs = Vec::new();
        for r in &dirs {
            if ulp_distance(ev(r.z), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                dmid += 1;
            }
        }
        for &lz in &U4 {
            let Some(r) = dirs.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
                continue;
            };
            if ulp_distance(ev(lz), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                hit += 1;
                zs.push(lz);
            }
        }
        println!("{name:10} hit={hit}/4 {zs:?} dmid={dmid}/{}", dirs.len());
    }
}
