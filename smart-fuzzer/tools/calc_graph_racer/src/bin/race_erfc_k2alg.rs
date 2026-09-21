//! two-mode Q vs last-store k=±2 of unmask F on 0x210 leftover |k|=2. Not an identity.
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
fn signed_ulp(a: f64, b: f64) -> i32 {
    if a == b {
        return 0;
    }
    let mut n = 0i32;
    let mut v = b;
    if a > b {
        while v < a && n < 32 {
            v = v.next_up();
            n += 1;
        }
        n
    } else {
        while v > a && n < 32 {
            v = v.next_down();
            n -= 1;
        }
        n
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut n_k2 = 0usize;
    let mut n_eq = 0usize;
    let mut n_tmu = 0usize;
    let mut n_tmu_eq = 0usize;
    let mut n_miss = 0usize;
    let mut miss_eq = 0usize;
    let mut n_km2 = 0usize;
    let mut n_tmd = 0usize;
    let mut n_tmd_eq = 0usize;
    let mut n_hmiss = 0usize;
    let mut hmiss_eq = 0usize;
    println!("leftover-low |k|=2 last-store vs two-mode of unmask F:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g210 = mul(w, cody(z, &C0, &D0, MASK));
        let d210 = ulp_distance(g210, t).unwrap_or(99);
        let k = k_of(t, w, fu);
        let q_k2 = mul(w, poke(fu, 2));
        let q_km2 = mul(w, poke(fu, -2));
        let q_tmu = mul(w.next_up(), fu.next_up());
        let q_tmd = mul(w.next_down(), fu.next_down());
        let q_w1 = mul(w.next_up(), fu);
        let q_f1 = mul(w, fu.next_up());
        let q_wm1 = mul(w.next_down(), fu);
        let q_fm1 = mul(w, fu.next_down());
        let q_prod2 = poke(mul(w, fu), 2);
        let q_prodm2 = poke(mul(w, fu), -2);
        if d210 >= 2 && g210 < t && k == Some(2) {
            n_k2 += 1;
            let eq = q_tmu.to_bits() == q_k2.to_bits();
            if eq {
                n_eq += 1;
            }
            let tmu = hit(q_tmu, t);
            if tmu {
                n_tmu += 1;
                if eq {
                    n_tmu_eq += 1;
                }
            } else {
                n_miss += 1;
                if eq {
                    miss_eq += 1;
                }
                println!(
                    "  LO k=2 not-tmu z={:.16} bits={:#x} 555={} d210={d210} tmu_d={} tmu-k2={} k2_d={} eq_k3={} eq_prod3={} w+1={} F+1={} prod2_d={}",
                    z,
                    z.to_bits(),
                    is555(z),
                    ulp_distance(q_tmu, t).unwrap_or(99),
                    signed_ulp(q_tmu, q_k2),
                    ulp_distance(q_k2, t).unwrap_or(99),
                    q_tmu.to_bits() == mul(w, poke(fu, 3)).to_bits(),
                    q_tmu.to_bits() == poke(mul(w, fu), 3).to_bits(),
                    hit(q_w1, t),
                    hit(q_f1, t),
                    ulp_distance(q_prod2, t).unwrap_or(99)
                );
            }
        } else if d210 >= 2 && g210 > t && k == Some(-2) {
            n_km2 += 1;
            let eq = q_tmd.to_bits() == q_km2.to_bits();
            let tmd = hit(q_tmd, t);
            if tmd {
                n_tmd += 1;
                if eq {
                    n_tmd_eq += 1;
                }
            } else {
                n_hmiss += 1;
                if eq {
                    hmiss_eq += 1;
                }
                println!(
                    "  HI k=-2 not-tmd z={:.16} bits={:#x} 555={} d210={d210} tmd_d={} tmd-km2={} km2_d={} eq_km1={} eq_km3={} eq_prodm1={} eq_prodm3={} w-1={} F-1={} prodm2_d={}",
                    z,
                    z.to_bits(),
                    is555(z),
                    ulp_distance(q_tmd, t).unwrap_or(99),
                    signed_ulp(q_tmd, q_km2),
                    ulp_distance(q_km2, t).unwrap_or(99),
                    q_tmd.to_bits() == mul(w, poke(fu, -1)).to_bits(),
                    q_tmd.to_bits() == mul(w, poke(fu, -3)).to_bits(),
                    q_tmd.to_bits() == poke(mul(w, fu), -1).to_bits(),
                    q_tmd.to_bits() == poke(mul(w, fu), -3).to_bits(),
                    hit(q_wm1, t),
                    hit(q_fm1, t),
                    ulp_distance(q_prodm2, t).unwrap_or(99)
                );
            }
        }
    }
    println!(
        "leftover-low |k|=2 n={n_k2} tmu={n_tmu} Q_tmu==Q_k2 {n_eq}/{n_k2} tmu_eq={n_tmu_eq}/{n_tmu} miss_eq={miss_eq}/{n_miss}"
    );
    println!(
        "leftover-high |k|=-2 n={n_km2} tmd={n_tmd} Q_tmd==Q_km2 tmd_eq={n_tmd_eq}/{n_tmd} miss_eq={hmiss_eq}/{n_hmiss}"
    );

    let mut n114_k2 = 0usize;
    let mut n114_tmu = 0usize;
    let mut n114_eq = 0usize;
    let mut c114 = C0;
    c114[3] = poke(C0[3], -1);
    c114[4] = poke(C0[4], 1);
    c114[5] = poke(C0[5], -1);
    c114[6] = poke(C0[6], 1);
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g114 = mul(w, cody(z, &c114, &D0, MASK));
        let d114 = ulp_distance(g114, t).unwrap_or(99);
        if !(d114 >= 2 && g114 < t && k_of(t, w, fu) == Some(2)) {
            continue;
        }
        n114_k2 += 1;
        let q_k2 = mul(w, poke(fu, 2));
        let q_tmu = mul(w.next_up(), fu.next_up());
        if q_tmu.to_bits() == q_k2.to_bits() {
            n114_eq += 1;
        }
        if hit(q_tmu, t) {
            n114_tmu += 1;
        } else {
            println!(
                "  114 LO k=2 not-tmu z={:.16} 555={} tmu_d={} tmu-k2={} k2_d={}",
                z,
                is555(z),
                ulp_distance(q_tmu, t).unwrap_or(99),
                signed_ulp(q_tmu, q_k2),
                ulp_distance(q_k2, t).unwrap_or(99)
            );
        }
    }
    println!("114 leftover-low |k|=2 n={n114_k2} tmu={n114_tmu} Q_tmu==Q_k2 {n114_eq}/{n114_k2}");

    let mut nall = 0usize;
    let mut up_f = [0usize; 6];
    let mut up_p = [0usize; 6];
    let mut dn_f = [0usize; 6];
    let mut dn_p = [0usize; 6];
    let mut up_f_none = 0usize;
    let mut up_p_none = 0usize;
    let mut dn_f_none = 0usize;
    let mut dn_p_none = 0usize;
    let mut tmu_ex = 0usize;
    let mut tmd_ex = 0usize;
    println!("DIRECT mid unmask two-mode vs last-store of F / last-mul of product:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        nall += 1;
        let q_tmu = mul(w.next_up(), fu.next_up());
        let q_tmd = mul(w.next_down(), fu.next_down());
        let prod = mul(w, fu);
        if hit(q_tmu, t) {
            tmu_ex += 1;
        }
        if hit(q_tmd, t) {
            tmd_ex += 1;
        }
        let mut uf = false;
        let mut up = false;
        let mut df = false;
        let mut dp = false;
        for ak in 0i32..=5 {
            if q_tmu.to_bits() == mul(w, poke(fu, ak)).to_bits() {
                up_f[ak as usize] += 1;
                uf = true;
            }
            if q_tmu.to_bits() == poke(prod, ak).to_bits() {
                up_p[ak as usize] += 1;
                up = true;
            }
            if q_tmd.to_bits() == mul(w, poke(fu, -ak)).to_bits() {
                dn_f[ak as usize] += 1;
                df = true;
            }
            if q_tmd.to_bits() == poke(prod, -ak).to_bits() {
                dn_p[ak as usize] += 1;
                dp = true;
            }
        }
        if !uf {
            up_f_none += 1;
        }
        if !up {
            up_p_none += 1;
        }
        if !df {
            dn_f_none += 1;
        }
        if !dp {
            dn_p_none += 1;
        }
    }
    let pr = |name: &str, h: &[usize; 6], none: usize| {
        print!("{name}");
        for (i, c) in h.iter().enumerate() {
            if *c > 0 {
                print!(" {i}:{c}");
            }
        }
        if none > 0 {
            print!(" none:{none}");
        }
        println!();
    };
    println!("DIRECT mid n={nall} tmu_excel={tmu_ex} tmd_excel={tmd_ex}");
    pr("two-mode-up == w*F+k", &up_f, up_f_none);
    pr("two-mode-up == (wF)+k", &up_p, up_p_none);
    pr("two-mode-down == w*F-k", &dn_f, dn_f_none);
    pr("two-mode-down == (wF)-k", &dn_p, dn_p_none);
}
