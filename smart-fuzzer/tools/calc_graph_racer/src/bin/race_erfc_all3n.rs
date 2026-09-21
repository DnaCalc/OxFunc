//! named mid F last-store k on all3 leftover two-mode 7z. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use std::env;

const ZB: [u64; 7] = [
    0x3ff4800000000000,
    0x3ffd555555555555,
    0x4007c00000000000,
    0x4009eaaaaaaaaaab,
    0x3fe0000000000002,
    0x4000eaaaaaaaaaab,
    0x4002eaaaaaaaaaab,
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
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn k_of(t: f64, w: f64, ff: f64) -> Option<i32> {
    for ak in 0i32..=8 {
        for &s in &[-1i32, 1] {
            let k = if ak == 0 { 0 } else { s * ak };
            if ak == 0 && s < 0 {
                continue;
            }
            if ulp_distance(mul(w, poke(ff, k)), t).unwrap_or(99) == 0 {
                return Some(k);
            }
            if ak == 0 {
                break;
            }
        }
    }
    None
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut want = std::collections::BTreeMap::new();
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let zb = r.z.to_bits();
        if ZB.contains(&zb) {
            want.insert(zb, r.qbits);
        }
    }
    let named: [(&str, fn(f64) -> f64); 8] = [
        ("nswc_derfc0", f::nswc_derfc0),
        ("nswc_pqr", f::nswc_pqr_f),
        ("nswc_ccdd", f::nswc_ccdd_f),
        ("cody_erfcx", f::cody_erfcx_f),
        ("cephes", f::cephes_f),
        ("cdflib_erfc1", f::cdflib_erfc1_f),
        ("cf_as714", f::cf_as714_f),
        ("cf_gautschi", f::cf_gautschi_f),
    ];
    println!("named F last-store k on all3 leftover two-mode 7z:");
    for zb in ZB {
        let z = f64::from_bits(zb);
        let Some(&tb) = want.get(&zb) else {
            println!("  missing z={z:.16}");
            continue;
        };
        let t = f64::from_bits(tb);
        let w = f::w_rn53(z);
        print!("  z={z:.16} bits={zb:#x}");
        for (name, ev) in named {
            let ff = ev(z);
            let d = ulp_distance(mul(w, ff), t).unwrap_or(99);
            print!(" {name} d={d} k={:?}", k_of(t, w, ff));
        }
        println!();
    }
}
