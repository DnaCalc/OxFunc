//! two-mode of unmask w×F on 0x210 leftover DIRECT [0.5,4). Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const MASK: u32 = 0x210;
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
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn hit(g: f64, t: f64) -> bool {
    ulp_distance(g, t).unwrap_or(99) == 0
}
fn is555(z: f64) -> bool {
    let h = format!("{:x}", z.to_bits());
    h.contains("55555555") || h.contains("aaaaaaaa")
}
fn k_of(_z: f64, t: f64, w: f64, ff: f64) -> Option<i32> {
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
    let mut c114 = C0;
    c114[3] = poke(C0[3], -1);
    c114[4] = poke(C0[4], 1);
    c114[5] = poke(C0[5], -1);
    c114[6] = poke(C0[6], 1);
    let mut nlo = 0usize;
    let mut nhi = 0usize;
    let mut lo_tmu = 0usize;
    let mut lo_up = 0usize;
    let mut lo_w1 = 0usize;
    let mut lo_f1 = 0usize;
    let mut lo_tmu_only = 0usize;
    let mut hi_tmd = 0usize;
    let mut hi_dn = 0usize;
    let mut hi_w1 = 0usize;
    let mut hi_f1 = 0usize;
    let mut hi_tmd_only = 0usize;
    let mut lo114 = 0usize;
    let mut lo114_tmu = 0usize;
    let mut hi114 = 0usize;
    let mut hi114_tmd = 0usize;
    let mut lo_tmu_k = [0usize; 9];
    let mut lo_not_k = [0usize; 9];
    let mut hi_tmd_k = [0usize; 9];
    let mut hi_not_k = [0usize; 9];
    let mut lo_tmu_none = 0usize;
    let mut lo_not_none = 0usize;
    let mut hi_tmd_none = 0usize;
    let mut hi_not_none = 0usize;
    let mut lo_tmu210 = 0usize;
    let mut hi_tmd210 = 0usize;
    println!("unmask two-mode on 0x210 leftover DIRECT [0.5,4):");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g210 = mul(w, cody(z, &C0, &D0, MASK));
        let d210 = ulp_distance(g210, t).unwrap_or(99);
        let g114 = mul(w, cody(z, &c114, &D0, MASK));
        let d114 = ulp_distance(g114, t).unwrap_or(99);
        let tmu = hit(mul(w.next_up(), fu.next_up()), t);
        let tmd = hit(mul(w.next_down(), fu.next_down()), t);
        let f210 = cody(z, &C0, &D0, MASK);
        let tmu210 = hit(mul(w.next_up(), f210.next_up()), t);
        let tmd210 = hit(mul(w.next_down(), f210.next_down()), t);
        let up = hit(mul(w, fu).next_up(), t);
        let dn = hit(mul(w, fu).next_down(), t);
        let w1u = hit(mul(w.next_up(), fu), t);
        let f1u = hit(mul(w, fu.next_up()), t);
        let w1d = hit(mul(w.next_down(), fu), t);
        let f1d = hit(mul(w, fu.next_down()), t);
        if d210 >= 2 && g210 < t {
            nlo += 1;
            let k = k_of(z, t, w, fu);
            match k {
                Some(kk) if tmu => {
                    let ak = kk.unsigned_abs() as usize;
                    if ak < 9 {
                        lo_tmu_k[ak] += 1;
                    }
                }
                Some(kk) => {
                    let ak = kk.unsigned_abs() as usize;
                    if ak < 9 {
                        lo_not_k[ak] += 1;
                    }
                }
                None if tmu => lo_tmu_none += 1,
                None => lo_not_none += 1,
            }
            if tmu {
                lo_tmu += 1;
            }
            if up {
                lo_up += 1;
            }
            if w1u {
                lo_w1 += 1;
            }
            if f1u {
                lo_f1 += 1;
            }
            if tmu && !up && !w1u && !f1u {
                lo_tmu_only += 1;
            }
            if tmu210 {
                lo_tmu210 += 1;
                if !tmu {
                    println!(
                        "  LO 0x210-F tmu not-unmask z={:.16} 555={} k={k:?} d210={d210}",
                        z,
                        is555(z)
                    );
                }
            }
            if tmu {
                println!(
                    "  LO tmu z={:.16} 555={} k={k:?} last-mul_up={up} w+1={w1u} F+1={f1u} d210={d210}",
                    z,
                    is555(z)
                );
            } else if k == Some(2) {
                println!(
                    "  LO k=2 not-tmu z={:.16} 555={} last-mul_up={up} w+1={w1u} F+1={f1u} d210={d210} d114={}",
                    z,
                    is555(z),
                    d114
                );
            }
        } else if d210 >= 2 && g210 > t {
            nhi += 1;
            let k = k_of(z, t, w, fu);
            match k {
                Some(kk) if tmd => {
                    let ak = kk.unsigned_abs() as usize;
                    if ak < 9 {
                        hi_tmd_k[ak] += 1;
                    }
                }
                Some(kk) => {
                    let ak = kk.unsigned_abs() as usize;
                    if ak < 9 {
                        hi_not_k[ak] += 1;
                    }
                }
                None if tmd => hi_tmd_none += 1,
                None => hi_not_none += 1,
            }
            if tmd {
                hi_tmd += 1;
            }
            if dn {
                hi_dn += 1;
            }
            if w1d {
                hi_w1 += 1;
            }
            if f1d {
                hi_f1 += 1;
            }
            if tmd && !dn && !w1d && !f1d {
                hi_tmd_only += 1;
            }
            if tmd210 {
                hi_tmd210 += 1;
                if !tmd {
                    println!(
                        "  HI 0x210-F tmd not-unmask z={:.16} 555={} k={k:?} d210={d210}",
                        z,
                        is555(z)
                    );
                }
            }
            if tmd {
                println!(
                    "  HI tmd z={:.16} 555={} k={k:?} last-mul_dn={dn} w-1={w1d} F-1={f1d} d210={d210}",
                    z,
                    is555(z)
                );
            } else if k == Some(-2) {
                println!(
                    "  HI k=-2 not-tmd z={:.16} 555={} last-mul_dn={dn} w-1={w1d} F-1={f1d} d210={d210} d114={}",
                    z,
                    is555(z),
                    d114
                );
            }
        }
        if d114 >= 2 && g114 < t {
            lo114 += 1;
            if tmu {
                lo114_tmu += 1;
                if !(d210 >= 2 && g210 < t) {
                    println!(
                        "  114-only LO tmu z={:.16} 555={} d210={d210} d114={d114}",
                        z,
                        is555(z)
                    );
                }
            }
        } else if d114 >= 2 && g114 > t {
            hi114 += 1;
            if tmd {
                hi114_tmd += 1;
                if !(d210 >= 2 && g210 > t) {
                    println!(
                        "  114-only HI tmd z={:.16} 555={} d210={d210} d114={d114}",
                        z,
                        is555(z)
                    );
                }
            }
        }
        if d210 >= 2 && g210 < t && tmu && !(d114 >= 2 && g114 < t) {
            println!(
                "  0x210-only LO tmu z={:.16} 555={} d210={d210} d114={d114}",
                z,
                is555(z)
            );
        }
        if d210 >= 2 && g210 > t && tmd && !(d114 >= 2 && g114 > t) {
            println!(
                "  0x210-only HI tmd z={:.16} 555={} d210={d210} d114={d114}",
                z,
                is555(z)
            );
        }
    }
    println!(
        "0x210 leftover-low n={nlo} unmask tmu={lo_tmu} tmu_only={lo_tmu_only} 0x210-F tmu={lo_tmu210} last-mul_up={lo_up} w+1={lo_w1} F+1={lo_f1}"
    );
    print!("  leftover-low tmu |k|");
    for (i, c) in lo_tmu_k.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if lo_tmu_none > 0 {
        print!(" none:{lo_tmu_none}");
    }
    print!("\n  leftover-low not-tmu |k|");
    for (i, c) in lo_not_k.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if lo_not_none > 0 {
        print!(" none:{lo_not_none}");
    }
    println!();
    println!(
        "0x210 leftover-high n={nhi} unmask tmd={hi_tmd} tmd_only={hi_tmd_only} 0x210-F tmd={hi_tmd210} last-mul_dn={hi_dn} w-1={hi_w1} F-1={hi_f1}"
    );
    print!("  leftover-high tmd |k|");
    for (i, c) in hi_tmd_k.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if hi_tmd_none > 0 {
        print!(" none:{hi_tmd_none}");
    }
    print!("\n  leftover-high not-tmd |k|");
    for (i, c) in hi_not_k.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if hi_not_none > 0 {
        print!(" none:{hi_not_none}");
    }
    println!();
    println!("114 leftover-low n={lo114} unmask tmu={lo114_tmu}");
    println!("114 leftover-high n={hi114} unmask tmd={hi114_tmd}");

    let names = ["fused", "1-ULP", "leftover-low", "leftover-high"];
    let idx = |s: &str| match s {
        "fused" => 0,
        "1-ULP" => 1,
        "leftover-low" => 2,
        _ => 3,
    };
    let buck = |g: f64, t: f64| {
        let d = ulp_distance(g, t).unwrap_or(99);
        if d == 0 {
            "fused"
        } else if d == 1 {
            "1-ULP"
        } else if g < t {
            "leftover-low"
        } else {
            "leftover-high"
        }
    };
    let mut nall = 0usize;
    let mut nb = [0usize; 4];
    let mut tmu_b = [0usize; 4];
    let mut tmd_b = [0usize; 4];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g210 = mul(w, cody(z, &C0, &D0, MASK));
        nall += 1;
        let i = idx(buck(g210, t));
        nb[i] += 1;
        if hit(mul(w.next_up(), fu.next_up()), t) {
            tmu_b[i] += 1;
        }
        if hit(mul(w.next_down(), fu.next_down()), t) {
            tmd_b[i] += 1;
        }
    }
    print!("DIRECT mid n={nall} 0x210 bucket");
    for (i, c) in nb.iter().enumerate() {
        print!(" {}:{}", names[i], c);
    }
    print!("\nunmask tmu");
    for (i, c) in tmu_b.iter().enumerate() {
        print!(" {}:{}", names[i], c);
    }
    print!("\nunmask tmd");
    for (i, c) in tmd_b.iter().enumerate() {
        print!(" {}:{}", names[i], c);
    }
    println!();
}
