//! PQR80 vs ham2 / j3432 Q-mid overlap. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const PP80: [Ext80; 8] = [
    Ext80([0x33, 0x14, 0x59, 0x30, 0x90, 0x5a, 0x14, 0xad, 0xf2, 0x3f]),
    Ext80([0xb7, 0x80, 0xa3, 0xb9, 0x2d, 0xdf, 0x3a, 0xa2, 0xf2, 0x3f]),
    Ext80([0x65, 0x54, 0x59, 0x48, 0x8a, 0x0e, 0x20, 0xbc, 0xf0, 0x3f]),
    Ext80([0xd8, 0xd3, 0x28, 0x82, 0x3d, 0xf6, 0x02, 0xa5, 0xed, 0xbf]),
    Ext80([0x7e, 0x1f, 0x5d, 0xfa, 0x61, 0x4a, 0xb6, 0xe8, 0xed, 0xbf]),
    Ext80([0x4c, 0xb0, 0x42, 0x2b, 0x7b, 0x9f, 0xa1, 0x89, 0xec, 0xbf]),
    Ext80([0x66, 0x36, 0x86, 0x46, 0x30, 0xb7, 0x99, 0x9b, 0xe9, 0xbf]),
    Ext80([0x6b, 0xb3, 0x83, 0xd7, 0xa8, 0x7b, 0x5e, 0x94, 0xe5, 0xbf]),
];
const QQ80: [Ext80; 8] = [
    Ext80([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xff, 0x3f]),
    Ext80([0x8b, 0xc7, 0x50, 0xb6, 0xe1, 0x3d, 0x4a, 0xd0, 0xff, 0x3f]),
    Ext80([0x60, 0x0b, 0xf1, 0x85, 0x91, 0xef, 0x1f, 0x9a, 0xff, 0x3f]),
    Ext80([0x93, 0x30, 0x40, 0xd9, 0x83, 0x06, 0x25, 0x86, 0xfe, 0x3f]),
    Ext80([0x31, 0xd3, 0x69, 0x95, 0xb5, 0xeb, 0x73, 0x94, 0xfc, 0x3f]),
    Ext80([0xa8, 0x82, 0xb8, 0x74, 0x00, 0x66, 0xa7, 0xd1, 0xf9, 0x3f]),
    Ext80([0x01, 0x31, 0xc7, 0xb2, 0x8c, 0xd9, 0x16, 0xb0, 0xf6, 0x3f]),
    Ext80([0xa8, 0x91, 0xa2, 0x37, 0x89, 0xb0, 0xb7, 0x89, 0xf2, 0x3f]),
];
const RR80: [Ext80; 9] = [
    Ext80([0x0c, 0x7c, 0x49, 0x0a, 0xce, 0x78, 0x15, 0x95, 0xfc, 0x3f]),
    Ext80([0xc3, 0x7f, 0x63, 0x90, 0x9b, 0xf5, 0xfd, 0x8b, 0xfd, 0xbf]),
    Ext80([0x8b, 0xaf, 0x01, 0x28, 0xa0, 0xa8, 0x6e, 0xe7, 0xfc, 0x3f]),
    Ext80([0x09, 0x3d, 0xa9, 0x47, 0x17, 0x64, 0x7f, 0xa7, 0xfc, 0xbf]),
    Ext80([0x64, 0x73, 0xd5, 0xda, 0xa4, 0x35, 0x22, 0xd2, 0xfb, 0x3f]),
    Ext80([0x2c, 0x1c, 0x04, 0x65, 0x8e, 0x67, 0x78, 0xe0, 0xfa, 0xbf]),
    Ext80([0xbd, 0x3a, 0xd2, 0x39, 0xac, 0x02, 0xc8, 0xc5, 0xf9, 0x3f]),
    Ext80([0xad, 0xf3, 0x12, 0x9a, 0xc0, 0xc8, 0xaf, 0x86, 0xf8, 0xbf]),
    Ext80([0xfd, 0x51, 0xc8, 0xe4, 0x38, 0x61, 0x51, 0xec, 0xf5, 0x3f]),
];
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
fn horner_80(cs: &[Ext80], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), c, CW);
    }
    acc
}
fn pqr_80(x: f64) -> f64 {
    let xe = ef(x.abs());
    let u = horner_80(&PP80, xe);
    let v = horner_80(&QQ80, xe);
    let t = ext_div(
        &ext_sub(&xe, &ef(3.75), CW),
        &ext_add(&xe, &ef(3.75), CW),
        CW,
    );
    let mut acc = ext_div(&u, &v, CW);
    for r in RR80.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), r, CW);
    }
    ext_to_f64(&acc, CW)
}
fn cody(y: f64, c: &[f64; 9], d: &[f64; 8]) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(c[7]), CW), &ext_add(&xden, &ef(d[7]), CW), CW),
        CW,
    )
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
    let mut hamc = C0;
    hamc[4] = poke(C0[4], -1);
    let mut hamd = D0;
    hamd[4] = poke(D0[4], -1);
    let mut jc = hamc;
    jc[0] = poke(C0[0], 4);
    jc[1] = poke(C0[1], 1);
    let mut jd = hamd;
    jd[0] = poke(D0[0], -1);
    let ev_p = |z: f64| qw(z, pqr_80(z));
    let ev_h = |z: f64| qw(z, cody(z, &hamc, &hamd));
    let ev_j = |z: f64| qw(z, cody(z, &jc, &jd));
    let ev_c = |z: f64| qw(z, cody(z, &C0, &D0));
    let score = |ev: &dyn Fn(f64) -> f64| {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        let mut dmid = 0usize;
        let mut dn = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let qg = ev(r.z);
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if r.direct {
                dn += 1;
            }
            if d == 0 {
                ex += 1;
                if r.direct {
                    dmid += 1;
                }
            } else {
                maxu = maxu.max(d);
            }
        }
        (ex, n, maxu, dmid, dn)
    };
    for (name, ev) in [
        ("C/D CR", &ev_c as &dyn Fn(f64) -> f64),
        ("ham2 C4-1 D4-1", &ev_h),
        ("j3432", &ev_j),
        ("PQR80", &ev_p),
    ] {
        let (ex, n, maxu, dmid, dn) = score(ev);
        println!("{name:18} {ex}/{n} max={maxu} dmid={dmid}/{dn}");
    }
    let ov = |a: &dyn Fn(f64) -> f64, b: &dyn Fn(f64) -> f64, lab: &str| {
        let mut both = 0usize;
        let mut only_a = 0usize;
        let mut only_b = 0usize;
        let mut band_a = [0usize; 7];
        let mut band_b = [0usize; 7];
        let mut band_n = [0usize; 7];
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let want = f64::from_bits(r.qbits);
            let da = ulp_distance(a(r.z), want).unwrap_or(99);
            let db = ulp_distance(b(r.z), want).unwrap_or(99);
            let bi = ((r.z - 0.5) / 0.5).floor() as usize;
            let bi = bi.min(6);
            band_n[bi] += 1;
            if da == 0 {
                band_a[bi] += 1;
            }
            if db == 0 {
                band_b[bi] += 1;
            }
            match (da == 0, db == 0) {
                (true, true) => both += 1,
                (true, false) => only_a += 1,
                (false, true) => only_b += 1,
                _ => {}
            }
        }
        println!(
            "{lab} both={} only_a={} only_b={} union={}",
            both,
            only_a,
            only_b,
            both + only_a + only_b
        );
        for i in 0..7 {
            let lo = 0.5 + i as f64 * 0.5;
            println!(
                "  [{:.1},{:.1}) n={} a={} b={}",
                lo,
                lo + 0.5,
                band_n[i],
                band_a[i],
                band_b[i]
            );
        }
        (both, only_a, only_b)
    };
    ov(&ev_p, &ev_j, "PQR80 vs j3432");
    ov(&ev_p, &ev_h, "PQR80 vs ham2");
    ov(&ev_j, &ev_c, "j3432 vs CR");
    let cuts = [
        0.75, 1.0, 1.25, 1.4, 1.45, 1.5, 1.55, 1.6, 1.75, 2.0, 2.5, 3.0, 3.5,
    ];
    for &c in &cuts {
        let mut pj = 0usize;
        let mut jp = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let want = f64::from_bits(r.qbits);
            let g = if r.z < c { ev_p(r.z) } else { ev_j(r.z) };
            let h = if r.z < c { ev_j(r.z) } else { ev_p(r.z) };
            if ulp_distance(g, want).unwrap_or(99) == 0 {
                pj += 1;
            }
            if ulp_distance(h, want).unwrap_or(99) == 0 {
                jp += 1;
            }
        }
        println!("cut={c:.2} PQR80-then-j3432={pj} j3432-then-PQR80={jp}");
    }
    let mut dboth = 0usize;
    let mut donly_p = 0usize;
    let mut donly_j = 0usize;
    let mut hist = std::collections::BTreeMap::<i64, usize>::new();
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 || !r.direct {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let gp = ev_p(r.z);
        let gj = ev_j(r.z);
        let dp = ulp_distance(gp, want).unwrap_or(99);
        let dj = ulp_distance(gj, want).unwrap_or(99);
        match (dp == 0, dj == 0) {
            (true, true) => dboth += 1,
            (true, false) => donly_p += 1,
            (false, true) => donly_j += 1,
            _ => {}
        }
        if dj != 0 {
            let s = if dp == 0 {
                0
            } else if gp > want {
                dp as i64
            } else {
                -(dp as i64)
            };
            *hist.entry(s).or_insert(0) += 1;
        }
    }
    println!(
        "DIRECT PQR80 vs j3432 both={} only_pqr={} only_j={} union={}",
        dboth,
        donly_p,
        donly_j,
        dboth + donly_p + donly_j
    );
    println!("signed ULP of PQR80 on j3432 direct misses:");
    for (k, v) in &hist {
        println!("  {k:+} {v}");
    }
}
