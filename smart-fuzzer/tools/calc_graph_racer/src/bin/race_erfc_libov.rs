//! 0x210 vs libm / cephes / derfc0 DIRECT hard overlap on Q-mid. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeSet;
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
fn cody_cd(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
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
fn hard_of(rows: &[f::QRow], ev: impl Fn(f64) -> f64) -> BTreeSet<u64> {
    let mut s = BTreeSet::new();
    for r in rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        if ulp_distance(ev(r.z), f64::from_bits(r.qbits)).unwrap_or(99) >= 2 {
            s.insert(r.z.to_bits());
        }
    }
    s
}
fn exact_of(rows: &[f::QRow], ev: impl Fn(f64) -> f64) -> usize {
    rows.iter()
        .filter(|r| {
            r.direct
                && r.z >= 0.5
                && r.z < 4.0
                && ulp_distance(ev(r.z), f64::from_bits(r.qbits)).unwrap_or(99) == 0
        })
        .count()
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let h210 = hard_of(&rows, |z| qwf(z, cody_cd(z, 0x210)));
    let hlib = hard_of(&rows, |z| libm::erfc(z));
    let hceph = hard_of(&rows, |z| qwf(z, f::cephes_f(z)));
    let hder = hard_of(&rows, |z| qwf(z, f::nswc_derfc0(z)));
    let n_dir = rows
        .iter()
        .filter(|r| r.direct && r.z >= 0.5 && r.z < 4.0)
        .count();
    println!(
        "DIRECT n={n_dir} exacts: 0x210={} libm={} cephes={} derfc0={}",
        exact_of(&rows, |z| qwf(z, cody_cd(z, 0x210))),
        exact_of(&rows, |z| libm::erfc(z)),
        exact_of(&rows, |z| qwf(z, f::cephes_f(z))),
        exact_of(&rows, |z| qwf(z, f::nswc_derfc0(z))),
    );
    println!(
        "hard ulp>=2: 0x210={} libm={} cephes={} derfc0={}",
        h210.len(),
        hlib.len(),
        hceph.len(),
        hder.len()
    );
    println!(
        "0x210 ∩ libm={}  only210={} only_libm={}",
        h210.intersection(&hlib).count(),
        h210.difference(&hlib).count(),
        hlib.difference(&h210).count()
    );
    println!(
        "0x210 ∩ cephes={} ∩ derfc0={}  all4={}",
        h210.intersection(&hceph).count(),
        h210.intersection(&hder).count(),
        h210
            .intersection(&hlib)
            .cloned()
            .collect::<BTreeSet<_>>()
            .intersection(&hceph)
            .cloned()
            .collect::<BTreeSet<_>>()
            .intersection(&hder)
            .count()
    );
    println!("only_0x210 (libm exact or ulp1):");
    for &zb in h210.difference(&hlib) {
        let z = f64::from_bits(zb);
        let r = rows.iter().find(|r| r.z.to_bits() == zb).unwrap();
        let t = f64::from_bits(r.qbits);
        let d210 = ulp_distance(qwf(z, cody_cd(z, 0x210)), t).unwrap_or(99);
        let dl = ulp_distance(libm::erfc(z), t).unwrap_or(99);
        println!(
            "  z={z:.16} 210={d210}{} libm={dl}{}",
            if qwf(z, cody_cd(z, 0x210)) < t { "L" } else { "H" },
            if libm::erfc(z) < t { "L" } else if libm::erfc(z) > t { "H" } else { "=" }
        );
    }
    println!("only_libm (0x210 exact or ulp1):");
    let mut shown = 0usize;
    for &zb in hlib.difference(&h210) {
        let z = f64::from_bits(zb);
        let r = rows.iter().find(|r| r.z.to_bits() == zb).unwrap();
        let t = f64::from_bits(r.qbits);
        let d210 = ulp_distance(qwf(z, cody_cd(z, 0x210)), t).unwrap_or(99);
        let dl = ulp_distance(libm::erfc(z), t).unwrap_or(99);
        if shown < 12 {
            println!(
                "  z={z:.16} 210={d210}{} libm={dl}{}",
                if qwf(z, cody_cd(z, 0x210)) < t || d210 == 0 { if d210 == 0 { "=" } else { "L" } } else { "H" },
                if libm::erfc(z) < t { "L" } else { "H" }
            );
            shown += 1;
        }
    }
    println!("only_libm n={} (first 12)", hlib.difference(&h210).count());

    let mut u1_both = 0usize;
    let mut ex_both = 0usize;
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let d0 = ulp_distance(qwf(r.z, cody_cd(r.z, 0x210)), t).unwrap_or(99);
        let dl = ulp_distance(libm::erfc(r.z), t).unwrap_or(99);
        if d0 == 0 && dl == 0 {
            ex_both += 1;
        }
        if d0 == 1 && dl == 1 {
            u1_both += 1;
        }
    }
    println!("both exact={ex_both} both ulp1={u1_both}");
}
