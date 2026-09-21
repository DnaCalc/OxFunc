//! j3432 Q-mid leftover 1-ULP fingerprint. Direct vs implied. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
fn cody(y: f64, c: &[f64; 9], d: &[f64; 8], mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(d[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
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
fn signed_ulp(got: f64, want: f64) -> i64 {
    let d = ulp_distance(got, want).unwrap_or(99) as i64;
    if d == 0 {
        0
    } else if got > want {
        d
    } else {
        -d
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut jc = C0;
    jc[4] = poke(C0[4], -1);
    jc[0] = poke(C0[0], 4);
    jc[1] = poke(C0[1], 1);
    let mut jd = D0;
    jd[4] = poke(D0[4], -1);
    jd[0] = poke(D0[0], -1);
    let mut hamc = C0;
    hamc[4] = poke(C0[4], -1);
    let mut hamd = D0;
    hamd[4] = poke(D0[4], -1);
    let ev_j = |z: f64| qw(z, cody(z, &jc, &jd, 0));
    let ev_c = |z: f64| qw(z, cody(z, &C0, &D0, 0));
    let ev_h = |z: f64| qw(z, cody(z, &hamc, &hamd, 0));
    let ev_74 = |z: f64| qw(z, cody(z, &C0, &D0, 0x74));
    let ev_210 = |z: f64| qw(z, cody(z, &C0, &D0, 0x210));
    let mut hist_all = BTreeMap::<i64, usize>::new();
    let mut hist_d = BTreeMap::<i64, usize>::new();
    let mut hist_i = BTreeMap::<i64, usize>::new();
    let mut band_n = [0usize; 7];
    let mut band_left = [0usize; 7];
    let mut band_left_d = [0usize; 7];
    let mut n = 0usize;
    let mut ex = 0usize;
    let mut left = 0usize;
    let mut left_d = 0usize;
    let mut left_i = 0usize;
    let mut m1 = 0usize;
    let mut m1d = 0usize;
    let mut m1i = 0usize;
    let mut hit_c = 0usize;
    let mut hit_h = 0usize;
    let mut hit_74 = 0usize;
    let mut hit_210 = 0usize;
    let mut maxu = 0u64;
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let gj = ev_j(r.z);
        if !gj.is_finite() {
            continue;
        }
        let d = ulp_distance(gj, want).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        n += 1;
        let b = ((r.z - 0.5) / 0.5).floor() as usize;
        let b = b.min(6);
        band_n[b] += 1;
        if d == 0 {
            ex += 1;
            continue;
        }
        left += 1;
        maxu = maxu.max(d);
        let s = signed_ulp(gj, want);
        *hist_all.entry(s).or_insert(0) += 1;
        band_left[b] += 1;
        if d == 1 {
            m1 += 1;
        }
        if r.direct {
            left_d += 1;
            band_left_d[b] += 1;
            *hist_d.entry(s).or_insert(0) += 1;
            if d == 1 {
                m1d += 1;
            }
        } else {
            left_i += 1;
            *hist_i.entry(s).or_insert(0) += 1;
            if d == 1 {
                m1i += 1;
            }
        }
        if ulp_distance(ev_c(r.z), want).unwrap_or(99) == 0 {
            hit_c += 1;
        }
        if ulp_distance(ev_h(r.z), want).unwrap_or(99) == 0 {
            hit_h += 1;
        }
        if ulp_distance(ev_74(r.z), want).unwrap_or(99) == 0 {
            hit_74 += 1;
        }
        if ulp_distance(ev_210(r.z), want).unwrap_or(99) == 0 {
            hit_210 += 1;
        }
    }
    println!(
        "j3432 {ex}/{n} leftover={left} max={maxu} m1={m1}  direct_left={left_d} m1d={m1d}  implied_left={left_i} m1i={m1i}"
    );
    println!(
        "leftover exact under CR={} ham2={} 0x74={} 0x210={}",
        hit_c, hit_h, hit_74, hit_210
    );
    for i in 0..7 {
        let lo = 0.5 + i as f64 * 0.5;
        println!(
            "[{:.1},{:.1}) n={} left={} leftd={}",
            lo,
            lo + 0.5,
            band_n[i],
            band_left[i],
            band_left_d[i]
        );
    }
    println!("signed ULP all leftover:");
    for (k, v) in &hist_all {
        println!("  {k:+} {v}");
    }
    println!("signed ULP DIRECT leftover:");
    for (k, v) in &hist_d {
        println!("  {k:+} {v}");
    }
    println!("signed ULP implied leftover:");
    for (k, v) in &hist_i {
        println!("  {k:+} {v}");
    }
    println!("HARD direct leftover ulp>=2 (z signed CR ham 0x74 0x210):");
    let mut hard_n = 0usize;
    let mut hard_also_cr = 0usize;
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 || !r.direct {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let gj = ev_j(r.z);
        let d = ulp_distance(gj, want).unwrap_or(99);
        if d < 2 || d > ULP_CAP {
            continue;
        }
        hard_n += 1;
        let s = signed_ulp(gj, want);
        let c0 = ulp_distance(ev_c(r.z), want).unwrap_or(99);
        let h0 = ulp_distance(ev_h(r.z), want).unwrap_or(99);
        let m74 = ulp_distance(ev_74(r.z), want).unwrap_or(99);
        let m210 = ulp_distance(ev_210(r.z), want).unwrap_or(99);
        if c0 == 0 {
            hard_also_cr += 1;
        }
        println!(
            "  z={:.16} s={s:+} CR={c0} ham={h0} 74={m74} 210={m210}",
            r.z
        );
    }
    println!("hard direct n={hard_n} of which CR-exact {hard_also_cr}");
    let named: [(&str, fn(f64) -> f64, bool); 4] = [
        ("nswc_derfc0", f::nswc_derfc0, true),
        ("nswc_pqr_f", f::nswc_pqr_f, true),
        ("cephes_f", f::cephes_f, true),
        ("libm::erfc as Q", libm::erfc, false),
    ];
    for (name, ev, as_f) in named {
        let mut hit_hard = 0usize;
        let mut keep = 0usize;
        let mut hard_max = 0u64;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let want = f64::from_bits(r.qbits);
            let gj = ev_j(r.z);
            let dj = ulp_distance(gj, want).unwrap_or(99);
            let qg = if as_f { qw(r.z, ev(r.z)) } else { ev(r.z) };
            let d = ulp_distance(qg, want).unwrap_or(u64::MAX);
            if dj == 0 && d == 0 {
                keep += 1;
            }
            if r.direct && dj >= 2 && dj <= ULP_CAP {
                if d == 0 {
                    hit_hard += 1;
                } else if d <= ULP_CAP {
                    hard_max = hard_max.max(d);
                }
            }
        }
        println!("{name:16} keep_j3432={keep}/3432 hit_hard={hit_hard}/{hard_n} hardmax={hard_max}");
    }
    let mut dboth = 0usize;
    let mut donly_j = 0usize;
    let mut donly_210 = 0usize;
    println!("DIRECT 0x210 vs j3432:");
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 || !r.direct {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let dj = ulp_distance(ev_j(r.z), want).unwrap_or(99);
        let d2 = ulp_distance(ev_210(r.z), want).unwrap_or(99);
        match (dj == 0, d2 == 0) {
            (true, true) => dboth += 1,
            (true, false) => {
                donly_j += 1;
                println!("  only_j3432 z={:.16} 210={}", r.z, d2);
            }
            (false, true) => {
                donly_210 += 1;
                println!("  only_0x210 z={:.16} j={}", r.z, dj);
            }
            _ => {}
        }
    }
    println!(
        "DIRECT both={dboth} only_j={donly_j} only_210={donly_210} union={}",
        dboth + donly_j + donly_210
    );
    println!("derfc0 hits on j3432-hard directs:");
    for r in &rows {
        if r.z < 0.5 || r.z >= 4.0 || !r.direct {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let dj = ulp_distance(ev_j(r.z), want).unwrap_or(99);
        if dj < 2 || dj > ULP_CAP {
            continue;
        }
        let d0 = ulp_distance(qw(r.z, f::nswc_derfc0(r.z)), want).unwrap_or(99);
        if d0 == 0 {
            println!("  HIT z={:.16} j3432_s={}", r.z, signed_ulp(ev_j(r.z), want));
        }
    }
}
