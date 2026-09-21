//! Last-store of named F vs 2.687/3.0625 F+2 undershoot. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use std::env;

const U2: [f64; 2] = [2.6875000000000000, 3.0625000000000000];

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
        ("pqr", Box::new(|z| qw(z, f::nswc_pqr_f(z)))),
        ("upPQR", Box::new(|z| qw(z, f::nswc_pqr_f(z)).next_up())),
        ("cephes", Box::new(|z| qw(z, f::cephes_f(z)))),
        ("upCep", Box::new(|z| qw(z, f::cephes_f(z)).next_up())),
        ("cody", Box::new(|z| qw(z, f::cody_erfcx_f(z)))),
        ("upCody", Box::new(|z| qw(z, f::cody_erfcx_f(z)).next_up())),
        (
            "up2Cody",
            Box::new(|z| qw(z, f::cody_erfcx_f(z)).next_up().next_up()),
        ),
        ("upLibm", Box::new(|z| libm::erfc(z).next_up())),
    ];
    println!("2.687/3.0625 last-store:");
    for &lz in &U2 {
        let Some(r) = dirs.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
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
        let mut zs = Vec::new();
        for r in &dirs {
            if ulp_distance(ev(r.z), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                dmid += 1;
            }
        }
        for &lz in &U2 {
            let Some(r) = dirs.iter().find(|rr| (rr.z - lz).abs() < 1e-12) else {
                continue;
            };
            if ulp_distance(ev(lz), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                hit += 1;
                zs.push(lz);
            }
        }
        println!("{name:10} hit={hit}/2 {zs:?} dmid={dmid}/{}", dirs.len());
    }
}
