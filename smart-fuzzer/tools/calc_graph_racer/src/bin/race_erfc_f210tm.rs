//! two-mode last-mul cover of 0x210/114/0x74 F vs unmask. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
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

fn cover(name: &str, rows: &[f::QRow], ff: impl Fn(f64) -> f64) {
    let mut n = 0usize;
    let mut up = [0usize; 6];
    let mut dn = [0usize; 6];
    let mut up_none = 0usize;
    let mut dn_none = 0usize;
    let mut tmu_ex = 0usize;
    let mut tmd_ex = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fval = ff(z);
        n += 1;
        let prod = mul(w, fval);
        let q_tmu = mul(w.next_up(), fval.next_up());
        let q_tmd = mul(w.next_down(), fval.next_down());
        if hit(q_tmu, t) {
            tmu_ex += 1;
        }
        if hit(q_tmd, t) {
            tmd_ex += 1;
        }
        let mut uok = false;
        let mut dok = false;
        for ak in 0i32..=5 {
            if q_tmu.to_bits() == poke(prod, ak).to_bits() {
                up[ak as usize] += 1;
                uok = true;
            }
            if q_tmd.to_bits() == poke(prod, -ak).to_bits() {
                dn[ak as usize] += 1;
                dok = true;
            }
        }
        if !uok {
            up_none += 1;
        }
        if !dok {
            dn_none += 1;
        }
    }
    print!("{name} n={n} tmu_excel={tmu_ex} tmd_excel={tmd_ex} up==(wF)+k");
    for (i, c) in up.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if up_none > 0 {
        print!(" none:{up_none}");
    }
    print!(" down");
    for (i, c) in dn.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if dn_none > 0 {
        print!(" none:{dn_none}");
    }
    println!();
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut c114 = C0;
    c114[3] = poke(C0[3], -1);
    c114[4] = poke(C0[4], 1);
    c114[5] = poke(C0[5], -1);
    c114[6] = poke(C0[6], 1);
    println!("DIRECT mid two-mode last-mul cover of each F:");
    cover("unmask", &rows, |z| cody(z, &C0, &D0, 0));
    cover("0x210", &rows, |z| cody(z, &C0, &D0, 0x210));
    cover("0x74", &rows, |z| cody(z, &C0, &D0, 0x74));
    cover("114", &rows, |z| cody(z, &c114, &D0, 0x210));

    let mut nlo = 0usize;
    let mut lo_up = [0usize; 6];
    let mut lo_none = 0usize;
    let mut lo_ex = 0usize;
    let mut lo_unmask_tmu = 0usize;
    let mut lo_both = 0usize;
    let mut lo_210_only = 0usize;
    let mut lo_store_mul_same = 0usize;
    let mut lo_swap = 0usize;
    println!("0x210 leftover-low two-mode of 0x210 F vs last-mul of w*F_210:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let f210 = cody(z, &C0, &D0, 0x210);
        let g = mul(w, f210);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d >= 2 && g < t) {
            continue;
        }
        nlo += 1;
        let fu = cody(z, &C0, &D0, 0);
        let tmu_u = hit(mul(w.next_up(), fu.next_up()), t);
        let q_tmu = mul(w.next_up(), f210.next_up());
        let tmu_210 = hit(q_tmu, t);
        if tmu_u {
            lo_unmask_tmu += 1;
        }
        if tmu_210 {
            lo_ex += 1;
            let ks = k_of(t, w, f210);
            let mut km = None;
            for ak in 0i32..=5 {
                if ulp_distance(poke(g, ak), t).unwrap_or(99) == 0 {
                    km = Some(ak);
                    break;
                }
            }
            if ks == km {
                lo_store_mul_same += 1;
            } else {
                lo_swap += 1;
                println!(
                    "  leftover-low tmu swap z={:.16} store_k={ks:?} mul_k={km:?} unmask_tmu={tmu_u}",
                    z
                );
            }
            if tmu_u {
                lo_both += 1;
            } else {
                lo_210_only += 1;
                println!("  0x210-F tmu not-unmask z={:.16} store_k={ks:?} mul_k={km:?}", z);
            }
        } else if tmu_u {
            println!("  unmask tmu not-0x210-F z={:.16}", z);
        }
        let mut ok = false;
        for ak in 0i32..=5 {
            if q_tmu.to_bits() == poke(g, ak).to_bits() {
                lo_up[ak as usize] += 1;
                ok = true;
            }
        }
        if !ok {
            lo_none += 1;
            println!("  none z={:.16} tmu_d={}", z, ulp_distance(q_tmu, t).unwrap_or(99));
        }
    }
    println!(
        "leftover-low tmu store==mul {lo_store_mul_same}/{lo_ex} swap={lo_swap}"
    );
    print!("leftover-low n={nlo} 0x210-F tmu={lo_ex} unmask tmu={lo_unmask_tmu} both={lo_both} 210-only={lo_210_only} up==(w F_210)+k");
    for (i, c) in lo_up.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if lo_none > 0 {
        print!(" none:{lo_none}");
    }
    println!();

    let mut nhi = 0usize;
    let mut hi_dn = [0usize; 6];
    let mut hi_none = 0usize;
    let mut hi_ex = 0usize;
    let mut hi_u = 0usize;
    let mut hi_both = 0usize;
    let mut hi_same = 0usize;
    let mut hi_swap = 0usize;
    println!("0x210 leftover-high two-mode of 0x210 F vs last-mul of w*F_210:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let f210 = cody(z, &C0, &D0, 0x210);
        let g = mul(w, f210);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d >= 2 && g > t) {
            continue;
        }
        nhi += 1;
        let fu = cody(z, &C0, &D0, 0);
        let tmd_u = hit(mul(w.next_down(), fu.next_down()), t);
        let q_tmd = mul(w.next_down(), f210.next_down());
        let tmd_210 = hit(q_tmd, t);
        if tmd_u {
            hi_u += 1;
        }
        if tmd_210 {
            hi_ex += 1;
            let ks = k_of(t, w, f210);
            let mut km = None;
            for ak in 0i32..=5 {
                if ulp_distance(poke(g, -ak), t).unwrap_or(99) == 0 {
                    km = Some(-ak);
                    break;
                }
            }
            if ks == km {
                hi_same += 1;
            } else {
                hi_swap += 1;
                println!(
                    "  leftover-high tmd swap z={:.16} store_k={ks:?} mul_k={km:?} unmask_tmd={tmd_u}",
                    z
                );
            }
            if tmd_u {
                hi_both += 1;
            } else {
                println!("  0x210-F tmd not-unmask z={:.16} store_k={ks:?} mul_k={km:?}", z);
            }
        } else if tmd_u {
            println!("  unmask tmd not-0x210-F z={:.16}", z);
        }
        let mut ok = false;
        for ak in 0i32..=5 {
            if q_tmd.to_bits() == poke(g, -ak).to_bits() {
                hi_dn[ak as usize] += 1;
                ok = true;
            }
        }
        if !ok {
            hi_none += 1;
        }
    }
    println!("leftover-high tmd store==mul {hi_same}/{hi_ex} swap={hi_swap}");
    print!("leftover-high n={nhi} 0x210-F tmd={hi_ex} unmask tmd={hi_u} both={hi_both} down==(w F_210)-k");
    for (i, c) in hi_dn.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if hi_none > 0 {
        print!(" none:{hi_none}");
    }
    println!();

    use std::collections::BTreeSet;
    let mut sets: [BTreeSet<u64>; 4] = Default::default();
    let names = ["unmask", "0x210", "0x74", "114"];
    let evals: [u32; 4] = [0, 0x210, 0x74, 0x210];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let zb = z.to_bits();
        for (i, &mask) in evals.iter().enumerate() {
            let c = if i == 3 { &c114 } else { &C0 };
            let fv = cody(z, c, &D0, mask);
            if hit(mul(w.next_up(), fv.next_up()), t) {
                sets[i].insert(zb);
            }
        }
    }
    println!("tmu_excel set sizes {} {} {} {}", sets[0].len(), sets[1].len(), sets[2].len(), sets[3].len());
    for i in 0..4 {
        for j in (i + 1)..4 {
            let n = sets[i].intersection(&sets[j]).count();
            println!("  {}∩{} tmu_excel={}", names[i], names[j], n);
        }
    }
    let all = sets[0]
        .intersection(&sets[1])
        .filter(|z| sets[2].contains(z) && sets[3].contains(z))
        .count();
    println!("  all4 tmu_excel={all}");
    println!("all4 tmu_excel z:");
    for zb in sets[0]
        .intersection(&sets[1])
        .filter(|z| sets[2].contains(z) && sets[3].contains(z))
    {
        let z = f64::from_bits(*zb);
        let t = {
            let mut bits = None;
            for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
                if r.z.to_bits() == *zb {
                    bits = Some(r.qbits);
                    break;
                }
            }
            f64::from_bits(bits.unwrap())
        };
        let w = f::w_rn53(z);
        let gu = mul(w, cody(z, &C0, &D0, 0));
        let g210 = mul(w, cody(z, &C0, &D0, 0x210));
        let du = ulp_distance(gu, t).unwrap_or(99);
        let d210 = ulp_distance(g210, t).unwrap_or(99);
        println!(
            "  z={:.16} bits={:#x} unmask_d={du}{} d210={d210}{}",
            z,
            zb,
            if gu < t {
                "L"
            } else if gu > t {
                "H"
            } else {
                "="
            },
            if g210 < t {
                "L"
            } else if g210 > t {
                "H"
            } else {
                "="
            }
        );
    }
    println!("  0x210-only tmu_excel:");
    for zb in sets[1].difference(&sets[0]) {
        let z = f64::from_bits(*zb);
        let mut tb = None;
        for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
            if r.z.to_bits() == *zb {
                tb = Some(r.qbits);
                break;
            }
        }
        let t = f64::from_bits(tb.unwrap());
        let w = f::w_rn53(z);
        let du = ulp_distance(mul(w, cody(z, &C0, &D0, 0)), t).unwrap_or(99);
        let d210 = ulp_distance(mul(w, cody(z, &C0, &D0, 0x210)), t).unwrap_or(99);
        println!("    z={:.16} unmask_d={du} d210={d210}", z);
    }
    let mut n114lo = 0usize;
    let mut n114_ex = 0usize;
    let mut n114_u = 0usize;
    let mut n114_both = 0usize;
    let mut k114 = [0usize; 6];
    let mut n114_same = 0usize;
    let mut n114_swap = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let f114 = cody(z, &c114, &D0, 0x210);
        let g = mul(w, f114);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d >= 2 && g < t) {
            continue;
        }
        n114lo += 1;
        let tmu114 = hit(mul(w.next_up(), f114.next_up()), t);
        let tmuu = hit(mul(w.next_up(), cody(z, &C0, &D0, 0).next_up()), t);
        if tmu114 {
            n114_ex += 1;
            let ks = k_of(t, w, f114);
            let mut km = None;
            for ak in 0i32..=5 {
                if ulp_distance(poke(g, ak), t).unwrap_or(99) == 0 {
                    km = Some(ak);
                    break;
                }
            }
            if ks == km {
                n114_same += 1;
            } else {
                n114_swap += 1;
                println!(
                    "  114 leftover-low tmu swap z={:.16} store_k={ks:?} mul_k={km:?} unmask_tmu={tmuu}",
                    z
                );
            }
        }
        if tmuu {
            n114_u += 1;
        }
        if tmu114 && tmuu {
            n114_both += 1;
        }
        for ak in 0i32..=5 {
            if mul(w.next_up(), f114.next_up()).to_bits() == poke(g, ak).to_bits() {
                k114[ak as usize] += 1;
            }
        }
    }
    println!("114 leftover-low tmu store==mul {n114_same}/{n114_ex} swap={n114_swap}");
    print!("114 leftover-low n={n114lo} 114-F tmu={n114_ex} unmask tmu={n114_u} both={n114_both} up==(w F_114)+k");
    for (i, c) in k114.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!();
    let mut n12 = 0usize;
    println!("Cody leftover-low 2L store k=1 mul k=2:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 2 && g < t) {
            continue;
        }
        if k_of(t, w, fu) != Some(1) {
            continue;
        }
        if ulp_distance(poke(g, 2), t).unwrap_or(99) != 0 {
            continue;
        }
        n12 += 1;
        let tmu = hit(mul(w.next_up(), fu.next_up()), t);
        println!(
            "  z={:.16} bits={:#x} tmu={tmu} w+1={} F+1={}",
            z,
            z.to_bits(),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
    }
    println!("Cody leftover-low 2L store1/mul2 n={n12}");
    let mut n_hm = 0usize;
    println!("Cody leftover-high 2H store k=-1 mul k=-2:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 2 && g > t) {
            continue;
        }
        if k_of(t, w, fu) != Some(-1) {
            continue;
        }
        if ulp_distance(poke(g, -2), t).unwrap_or(99) != 0 {
            continue;
        }
        n_hm += 1;
        let tmd = hit(mul(w.next_down(), fu.next_down()), t);
        println!(
            "  z={:.16} bits={:#x} tmd={tmd} w-1={} F-1={}",
            z,
            z.to_bits(),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("Cody leftover-high 2H store-1/mul-2 n={n_hm}");
    println!("  unmask-only tmu_excel:");
    for zb in sets[0].difference(&sets[1]) {
        let z = f64::from_bits(*zb);
        let mut tb = None;
        for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
            if r.z.to_bits() == *zb {
                tb = Some(r.qbits);
                break;
            }
        }
        let t = f64::from_bits(tb.unwrap());
        let w = f::w_rn53(z);
        let du = ulp_distance(mul(w, cody(z, &C0, &D0, 0)), t).unwrap_or(99);
        let d210 = ulp_distance(mul(w, cody(z, &C0, &D0, 0x210)), t).unwrap_or(99);
        println!("    z={:.16} unmask_d={du} d210={d210}", z);
    }
    println!("0x74 leftover tmu/tmd store vs last-mul:");
    let mut lo74 = 0usize;
    let mut lo74_tmu = 0usize;
    let mut lo74_same = 0usize;
    let mut lo74_swap = 0usize;
    let mut lo74_u = 0usize;
    let mut lo74_both = 0usize;
    let mut hi74 = 0usize;
    let mut hi74_tmd = 0usize;
    let mut hi74_same = 0usize;
    let mut hi74_swap = 0usize;
    let mut hi74_u = 0usize;
    let mut hi74_both = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let f74 = cody(z, &C0, &D0, 0x74);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, f74);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d >= 2 && g < t {
            lo74 += 1;
            let tmu74 = hit(mul(w.next_up(), f74.next_up()), t);
            let tmuu = hit(mul(w.next_up(), fu.next_up()), t);
            if tmuu {
                lo74_u += 1;
            }
            if tmuu && !tmu74 {
                println!("  0x74 lo unmask-tmu not-0x74-F z={:.16}", z);
            }
            if tmu74 {
                lo74_tmu += 1;
                if tmuu {
                    lo74_both += 1;
                }
                let ks = k_of(t, w, f74);
                let mut km = None;
                for ak in 0i32..=8 {
                    if ulp_distance(poke(g, ak), t).unwrap_or(99) == 0 {
                        km = Some(ak);
                        break;
                    }
                }
                if ks == km {
                    lo74_same += 1;
                } else {
                    lo74_swap += 1;
                    println!(
                        "  0x74 lo tmu swap z={:.16} store_k={ks:?} mul_k={km:?} unmask_tmu={tmuu}",
                        z
                    );
                }
            }
        } else if d >= 2 && g > t {
            hi74 += 1;
            let tmd74 = hit(mul(w.next_down(), f74.next_down()), t);
            let tmdu = hit(mul(w.next_down(), fu.next_down()), t);
            if tmdu {
                hi74_u += 1;
            }
            if tmd74 {
                hi74_tmd += 1;
                if tmdu {
                    hi74_both += 1;
                }
                let ks = k_of(t, w, f74);
                let mut km = None;
                for ak in 0i32..=8 {
                    if ulp_distance(poke(g, -ak), t).unwrap_or(99) == 0 {
                        km = Some(-ak);
                        break;
                    }
                }
                if ks == km {
                    hi74_same += 1;
                } else {
                    hi74_swap += 1;
                    println!(
                        "  0x74 hi tmd swap z={:.16} store_k={ks:?} mul_k={km:?} unmask_tmd={tmdu}",
                        z
                    );
                }
            }
        }
    }
    println!(
        "0x74 leftover-low n={lo74} tmu={lo74_tmu} unmask_tmu={lo74_u} both={lo74_both} store==mul {lo74_same}/{lo74_tmu} swap={lo74_swap}"
    );
    println!(
        "0x74 leftover-high n={hi74} tmd={hi74_tmd} unmask_tmd={hi74_u} both={hi74_both} store==mul {hi74_same}/{hi74_tmd} swap={hi74_swap}"
    );
    println!("Cody leftover-low 2L store1/mul2 vs 0x74:");
    for z in [
        1.125,
        1.625,
        3.1770833333333335,
        3.3020833333333335,
        3.5,
        3.65625,
        3.78125,
        2.3020833333333335,
        2.75,
    ] {
        for r in rows.iter().filter(|rr| rr.direct && (rr.z - z).abs() < 1e-14) {
            let t = f64::from_bits(r.qbits);
            let w = f::w_rn53(r.z);
            let f74 = cody(r.z, &C0, &D0, 0x74);
            let fu = cody(r.z, &C0, &D0, 0);
            let f210 = cody(r.z, &C0, &D0, 0x210);
            let g74 = mul(w, f74);
            let gu = mul(w, fu);
            let g210 = mul(w, f210);
            println!(
                "  z={:.16} unmask={} 0x210={} 0x74={} unmask_tmu={} 0x74_tmu={} 0x74_k={:?}",
                r.z,
                {
                    let d = ulp_distance(gu, t).unwrap_or(99);
                    format!("{d}{}", if gu < t { "L" } else if gu > t { "H" } else { "=" })
                },
                {
                    let d = ulp_distance(g210, t).unwrap_or(99);
                    format!("{d}{}", if g210 < t { "L" } else if g210 > t { "H" } else { "=" })
                },
                {
                    let d = ulp_distance(g74, t).unwrap_or(99);
                    format!("{d}{}", if g74 < t { "L" } else if g74 > t { "H" } else { "=" })
                },
                hit(mul(w.next_up(), fu.next_up()), t),
                hit(mul(w.next_up(), f74.next_up()), t),
                k_of(t, w, f74)
            );
        }
    }
    let mut hi_u = BTreeSet::new();
    let mut hi_74 = BTreeSet::new();
    let mut hi_210 = BTreeSet::new();
    let mut hi_114 = BTreeSet::new();
    let mut tmd_u = BTreeSet::new();
    let mut tmd_74 = BTreeSet::new();
    let mut tmd_210 = BTreeSet::new();
    let mut tmd_114 = BTreeSet::new();
    let mut tmd_u_all = BTreeSet::new();
    let mut tmd_74_all = BTreeSet::new();
    println!("leftover-high and leftover-high tmd set identity:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f114 = cody(z, &c114, &D0, 0x210);
        let g114 = mul(w, f114);
        let gu = mul(w, fu);
        let g74 = mul(w, f74);
        let g210 = mul(w, f210);
        let du = ulp_distance(gu, t).unwrap_or(99);
        let d74 = ulp_distance(g74, t).unwrap_or(99);
        let d210 = ulp_distance(g210, t).unwrap_or(99);
        let d114 = ulp_distance(g114, t).unwrap_or(99);
        let tmu = hit(mul(w.next_down(), fu.next_down()), t);
        let tm74 = hit(mul(w.next_down(), f74.next_down()), t);
        let tm210 = hit(mul(w.next_down(), f210.next_down()), t);
        let tm114 = hit(mul(w.next_down(), f114.next_down()), t);
        if tmu {
            tmd_u_all.insert(zb);
        }
        if tm74 {
            tmd_74_all.insert(zb);
        }
        if du >= 2 && gu > t {
            hi_u.insert(zb);
            if tmu {
                tmd_u.insert(zb);
            }
        }
        if d74 >= 2 && g74 > t {
            hi_74.insert(zb);
            if tm74 {
                tmd_74.insert(zb);
            }
        }
        if d210 >= 2 && g210 > t {
            hi_210.insert(zb);
            if tm210 {
                tmd_210.insert(zb);
            }
        }
        if d114 >= 2 && g114 > t {
            hi_114.insert(zb);
            if tm114 {
                tmd_114.insert(zb);
            }
        }
    }
    println!(
        "leftover-high n unmask={} 0x74={} 0x210={} 114={} ∩74={} ∩210={} ∩114={}",
        hi_u.len(),
        hi_74.len(),
        hi_210.len(),
        hi_114.len(),
        hi_u.intersection(&hi_74).count(),
        hi_u.intersection(&hi_210).count(),
        hi_u.intersection(&hi_114).count()
    );
    println!(
        "leftover-high tmd n unmask={} 0x74={} 0x210={} 114={} ∩74={} ∩210={} ∩114={} 210∩114={}",
        tmd_u.len(),
        tmd_74.len(),
        tmd_210.len(),
        tmd_114.len(),
        tmd_u.intersection(&tmd_74).count(),
        tmd_u.intersection(&tmd_210).count(),
        tmd_u.intersection(&tmd_114).count(),
        tmd_210.intersection(&tmd_114).count()
    );
    println!(
        "global tmd_excel unmask={} 0x74={} ∩={}",
        tmd_u_all.len(),
        tmd_74_all.len(),
        tmd_u_all.intersection(&tmd_74_all).count()
    );
    print!("  leftover-high tmd unmask-only");
    for zb in tmd_u.difference(&tmd_74) {
        print!(" {:.6}", f64::from_bits(*zb));
    }
    println!();
    print!("  leftover-high tmd 0x74-only");
    for zb in tmd_74.difference(&tmd_u) {
        print!(" {:.6}", f64::from_bits(*zb));
    }
    println!();
    print!("  leftover-high tmd 114-only vs unmask");
    for zb in tmd_114.difference(&tmd_u) {
        print!(" {:.6}", f64::from_bits(*zb));
    }
    println!();
    print!("  leftover-high tmd unmask-only vs 114");
    for zb in tmd_u.difference(&tmd_114) {
        print!(" {:.6}", f64::from_bits(*zb));
    }
    println!();
    print!("  leftover-high 114 not unmask");
    for zb in hi_114.difference(&hi_u) {
        print!(" {:.16} bits={:#x}", f64::from_bits(*zb), zb);
    }
    println!();
    print!("  leftover-high unmask not 114");
    for zb in hi_u.difference(&hi_114) {
        print!(" {:.16} bits={:#x}", f64::from_bits(*zb), zb);
    }
    println!();
    print!("  leftover-high unmask not 0x74");
    for zb in hi_u.difference(&hi_74) {
        print!(" {:.6}", f64::from_bits(*zb));
    }
    println!();
    print!("  leftover-high 0x74 not unmask");
    for zb in hi_74.difference(&hi_u) {
        print!(" {:.6}", f64::from_bits(*zb));
    }
    println!();
    let mut lo_u = BTreeSet::new();
    let mut lo_74 = BTreeSet::new();
    let mut lo_210 = BTreeSet::new();
    let mut tmu_u = BTreeSet::new();
    let mut tmu_74 = BTreeSet::new();
    let mut tmu_210 = BTreeSet::new();
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f210 = cody(z, &C0, &D0, 0x210);
        let gu = mul(w, fu);
        let g74 = mul(w, f74);
        let g210 = mul(w, f210);
        let du = ulp_distance(gu, t).unwrap_or(99);
        let d74 = ulp_distance(g74, t).unwrap_or(99);
        let d210 = ulp_distance(g210, t).unwrap_or(99);
        if du >= 2 && gu < t {
            lo_u.insert(zb);
            if hit(mul(w.next_up(), fu.next_up()), t) {
                tmu_u.insert(zb);
            }
        }
        if d74 >= 2 && g74 < t {
            lo_74.insert(zb);
            if hit(mul(w.next_up(), f74.next_up()), t) {
                tmu_74.insert(zb);
            }
        }
        if d210 >= 2 && g210 < t {
            lo_210.insert(zb);
            if hit(mul(w.next_up(), f210.next_up()), t) {
                tmu_210.insert(zb);
            }
        }
    }
    println!(
        "leftover-low n unmask={} 0x74={} 0x210={} ∩74={} ∩210={} 74∩210={}",
        lo_u.len(),
        lo_74.len(),
        lo_210.len(),
        lo_u.intersection(&lo_74).count(),
        lo_u.intersection(&lo_210).count(),
        lo_74.intersection(&lo_210).count()
    );
    println!(
        "leftover-low tmu n unmask={} 0x74={} 0x210={} ∩74={} ∩210={} 74∩210={}",
        tmu_u.len(),
        tmu_74.len(),
        tmu_210.len(),
        tmu_u.intersection(&tmu_74).count(),
        tmu_u.intersection(&tmu_210).count(),
        tmu_74.intersection(&tmu_210).count()
    );
    print!("  leftover-low unmask not 0x74");
    for zb in lo_u.difference(&lo_74) {
        print!(" {:.6}", f64::from_bits(*zb));
    }
    println!();
    print!("  leftover-low 0x74 not unmask");
    for zb in lo_74.difference(&lo_u) {
        print!(" {:.6}", f64::from_bits(*zb));
    }
    println!();
    print!("  leftover-low tmu unmask not 0x74");
    for zb in tmu_u.difference(&tmu_74) {
        print!(" {:.6}", f64::from_bits(*zb));
    }
    println!();
    print!("  leftover-low unmask not 0x210");
    for zb in lo_u.difference(&lo_210) {
        print!(" {:.6}", f64::from_bits(*zb));
    }
    println!();
    print!("  leftover-low 0x210 not unmask");
    for zb in lo_210.difference(&lo_u) {
        print!(" {:.6}", f64::from_bits(*zb));
    }
    println!();
    print!("  leftover-low tmu unmask not 0x210");
    for zb in tmu_u.difference(&tmu_210) {
        print!(" {:.6}", f64::from_bits(*zb));
    }
    println!();
    print!("  leftover-low tmu 0x210 not unmask");
    for zb in tmu_210.difference(&tmu_u) {
        print!(" {:.6}", f64::from_bits(*zb));
    }
    println!();
    print!("  leftover-low tmu 0x74 not unmask");
    for zb in tmu_74.difference(&tmu_u) {
        print!(" {:.6}", f64::from_bits(*zb));
    }
    println!();
    println!("leftover-high tmd shared z:");
    for zb in tmd_u.intersection(&tmd_74) {
        let z = f64::from_bits(*zb);
        println!("  z={:.16} bits={:#x} in210_tmd={}", z, zb, tmd_210.contains(zb));
    }
    println!("leftover-high tmd vs not-tmd last-store/mul k of unmask F:");
    let mut tmd_sk = [0usize; 9];
    let mut tmd_mk = [0usize; 9];
    let mut not_sk = [0usize; 9];
    let mut not_mk = [0usize; 9];
    let mut tmd_same = 0usize;
    let mut tmd_swap = 0usize;
    let mut not_same = 0usize;
    let mut not_swap = 0usize;
    let mut tmd_555 = 0usize;
    let mut not_555 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !hi_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let ks = k_of(t, w, fu);
        let mut km = None;
        for ak in 0i32..=8 {
            if ulp_distance(poke(g, -ak), t).unwrap_or(99) == 0 {
                km = Some(-ak);
                break;
            }
        }
        let is555 = {
            let h = format!("{:x}", zb);
            h.contains("55555555") || h.contains("aaaaaaaa")
        };
        let tmd = tmd_u.contains(&zb);
        let sk = ks.unwrap_or(99).unsigned_abs() as usize;
        let mk = km.unwrap_or(99).unsigned_abs() as usize;
        if tmd {
            if sk < 9 {
                tmd_sk[sk] += 1;
            }
            if mk < 9 {
                tmd_mk[mk] += 1;
            }
            if is555 {
                tmd_555 += 1;
            }
            if ks == km {
                tmd_same += 1;
            } else {
                tmd_swap += 1;
            }
            println!(
                "  TMD z={:.16} bits={:#x} 555={is555} d={} store_k={ks:?} mul_k={km:?} w-1={} F-1={}",
                z,
                zb,
                ulp_distance(g, t).unwrap_or(99),
                hit(mul(w.next_down(), fu), t),
                hit(mul(w, fu.next_down()), t)
            );
        } else {
            if sk < 9 {
                not_sk[sk] += 1;
            }
            if mk < 9 {
                not_mk[mk] += 1;
            }
            if is555 {
                not_555 += 1;
            }
            if ks == km {
                not_same += 1;
            } else {
                not_swap += 1;
            }
            if ks == Some(-2) {
                println!(
                    "  not-tmd store k=-2 z={:.16} bits={:#x} 555={is555} d={} mul_k={km:?} w-1={} F-1={}",
                    z,
                    zb,
                    ulp_distance(g, t).unwrap_or(99),
                    hit(mul(w.next_down(), fu), t),
                    hit(mul(w, fu.next_down()), t)
                );
            }
        }
    }
    let prk = |name: &str, h: &[usize; 9]| {
        print!("{name}");
        for (i, c) in h.iter().enumerate() {
            if *c > 0 {
                print!(" {i}:{c}");
            }
        }
        println!();
    };
    println!("tmd n={} 555={tmd_555} store==mul {tmd_same} swap={tmd_swap}", tmd_u.len());
    prk("  tmd store |k|", &tmd_sk);
    prk("  tmd mul |k|", &tmd_mk);
    println!(
        "not-tmd leftover-high n={} 555={not_555} store==mul {not_same} swap={not_swap}",
        hi_u.len() - tmd_u.len()
    );
    prk("  not-tmd store |k|", &not_sk);
    prk("  not-tmd mul |k|", &not_mk);
    println!("leftover-low tmu vs not-tmu last-store/mul k of unmask F:");
    let mut lo_tmu_sk = [0usize; 9];
    let mut lo_tmu_mk = [0usize; 9];
    let mut lo_not_sk = [0usize; 9];
    let mut lo_not_mk = [0usize; 9];
    let mut lo_tmu_n = 0usize;
    let mut lo_not_n = 0usize;
    let mut lo_tmu_555 = 0usize;
    let mut lo_not_555 = 0usize;
    let mut lo_tmu_same = 0usize;
    let mut lo_tmu_swap = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !lo_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let ks = k_of(t, w, fu);
        let mut km = None;
        for ak in 0i32..=8 {
            if ulp_distance(poke(g, ak), t).unwrap_or(99) == 0 {
                km = Some(ak);
                break;
            }
        }
        let is555 = {
            let h = format!("{:x}", zb);
            h.contains("55555555") || h.contains("aaaaaaaa")
        };
        let tmu = tmu_u.contains(&zb);
        let sk = ks.unwrap_or(99).unsigned_abs() as usize;
        let mk = km.unwrap_or(99).unsigned_abs() as usize;
        if tmu {
            lo_tmu_n += 1;
            if sk < 9 {
                lo_tmu_sk[sk] += 1;
            }
            if mk < 9 {
                lo_tmu_mk[mk] += 1;
            }
            if is555 {
                lo_tmu_555 += 1;
            }
            if ks == km {
                lo_tmu_same += 1;
            } else {
                lo_tmu_swap += 1;
            }
        } else {
            lo_not_n += 1;
            if sk < 9 {
                lo_not_sk[sk] += 1;
            }
            if mk < 9 {
                lo_not_mk[mk] += 1;
            }
            if is555 {
                lo_not_555 += 1;
            }
            if ks == Some(2) {
                println!(
                    "  not-tmu store k=2 z={:.16} bits={:#x} 555={is555} d={} mul_k={km:?} w+1={} F+1={}",
                    z,
                    zb,
                    ulp_distance(g, t).unwrap_or(99),
                    hit(mul(w.next_up(), fu), t),
                    hit(mul(w, fu.next_up()), t)
                );
            }
        }
    }
    println!("tmu n={lo_tmu_n} 555={lo_tmu_555} store==mul {lo_tmu_same} swap={lo_tmu_swap}");
    prk("  tmu store |k|", &lo_tmu_sk);
    prk("  tmu mul |k|", &lo_tmu_mk);
    println!("not-tmu leftover-low n={lo_not_n} 555={lo_not_555}");
    prk("  not-tmu store |k|", &lo_not_sk);
    prk("  not-tmu mul |k|", &lo_not_mk);
    println!("leftover-high tmd cluster vs leftover-low tmu:");
    let mut hi_band = [0usize; 3];
    let mut lo_band = [0usize; 3];
    let band = |z: f64| {
        if z < 2.05 {
            0
        } else if z <= 2.53 {
            1
        } else {
            2
        }
    };
    println!("  leftover-high tmd:");
    for zb in &tmd_u {
        let z = f64::from_bits(*zb);
        hi_band[band(z)] += 1;
        let z3 = 3.0 * z;
        let tz = ((*zb & ((1u64 << 52) - 1)).trailing_zeros() as usize).min(52);
        let tz3 = ((z3.to_bits() & ((1u64 << 52) - 1)).trailing_zeros() as usize).min(52);
        let is555 = {
            let h = format!("{:x}", zb);
            h.contains("55555555") || h.contains("aaaaaaaa")
        };
        println!(
            "    z={:.16} bits={:#x} 555={is555} tz={tz} 3z={:.16} 3z_bits={:#x} tz3={tz3} band={}",
            z,
            zb,
            z3,
            z3.to_bits(),
            band(z)
        );
    }
    println!("  leftover-low tmu:");
    for zb in &tmu_u {
        let z = f64::from_bits(*zb);
        lo_band[band(z)] += 1;
        let z3 = 3.0 * z;
        let tz = ((*zb & ((1u64 << 52) - 1)).trailing_zeros() as usize).min(52);
        let tz3 = ((z3.to_bits() & ((1u64 << 52) - 1)).trailing_zeros() as usize).min(52);
        let is555 = {
            let h = format!("{:x}", zb);
            h.contains("55555555") || h.contains("aaaaaaaa")
        };
        println!(
            "    z={:.16} bits={:#x} 555={is555} tz={tz} 3z={:.16} tz3={tz3} band={}",
            z,
            zb,
            z3,
            band(z)
        );
    }
    println!(
        "tmd bands <2.05:{} [2.05,2.53]:{} >2.53:{}",
        hi_band[0], hi_band[1], hi_band[2]
    );
    println!(
        "tmu bands <2.05:{} [2.05,2.53]:{} >2.53:{}",
        lo_band[0], lo_band[1], lo_band[2]
    );
    let dy3 = |z: f64| {
        let z3 = 3.0 * z;
        ((z3.to_bits() & ((1u64 << 52) - 1)).trailing_zeros() as usize) >= 40
    };
    let mut lo_d3 = 0usize;
    let mut lo_d3_tmu = 0usize;
    let mut hi_d3 = 0usize;
    let mut hi_d3_tmd = 0usize;
    let mut lo_k2_d3 = 0usize;
    let mut lo_k2 = 0usize;
    let mut hi_km2_d3 = 0usize;
    let mut hi_km2 = 0usize;
    println!("3z-dyadic leftover vs two-mode:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d3 = dy3(z);
        if lo_u.contains(&zb) {
            if d3 {
                lo_d3 += 1;
                if tmu_u.contains(&zb) {
                    lo_d3_tmu += 1;
                }
            }
            if k_of(t, w, fu) == Some(2) {
                lo_k2 += 1;
                if d3 {
                    lo_k2_d3 += 1;
                }
            }
        }
        if hi_u.contains(&zb) {
            if d3 {
                hi_d3 += 1;
                if tmd_u.contains(&zb) {
                    hi_d3_tmd += 1;
                }
            } else {
                println!(
                    "  leftover-high not 3z-dyadic z={:.16} bits={:#x} tmd={} k={:?}",
                    z,
                    zb,
                    tmd_u.contains(&zb),
                    k_of(t, w, fu)
                );
            }
            if k_of(t, w, fu) == Some(-2) {
                hi_km2 += 1;
                if d3 {
                    hi_km2_d3 += 1;
                }
            }
        }
        let _ = g;
    }
    println!(
        "leftover-low 3z-dyadic {lo_d3}/{} tmu among them {lo_d3_tmu}; leftover-low tmu {} 3z-dyadic {lo_d3_tmu}",
        lo_u.len(),
        tmu_u.len()
    );
    println!("leftover-low store k=+2 n={lo_k2} 3z-dyadic {lo_k2_d3}");
    println!(
        "leftover-high 3z-dyadic {hi_d3}/{} tmd among them {hi_d3_tmd}; leftover-high tmd {} 3z-dyadic {hi_d3_tmd}",
        hi_u.len(),
        tmd_u.len()
    );
    println!("leftover-high store k=-2 n={hi_km2} 3z-dyadic {hi_km2_d3}");
    let grids = [16.0, 32.0, 48.0, 96.0];
    let near = |a: f64, b: f64| (a - b).abs() < 1e-12;
    let spacing = |dz: f64| -> Option<u32> {
        for &g in &grids {
            if near(dz, 1.0 / g) {
                return Some(g as u32);
            }
        }
        None
    };
    let pairs = |name: &str, set: &std::collections::BTreeSet<u64>| {
        let zs: Vec<f64> = set.iter().copied().map(f64::from_bits).collect();
        println!("{name} grid pairs n={}", zs.len());
        for i in 0..zs.len() {
            for j in (i + 1)..zs.len() {
                if let Some(g) = spacing((zs[j] - zs[i]).abs()) {
                    println!(
                        "  1/{g} z={:.16} z={:.16}",
                        zs[i], zs[j]
                    );
                }
            }
        }
    };
    pairs("leftover-high tmd", &tmd_u);
    pairs("leftover-low tmu", &tmu_u);
    pairs("leftover-high", &hi_u);
    pairs("leftover-low", &lo_u);
    println!("mixed leftover-high tmd vs leftover-low tmu grid pairs:");
    for &hb in &tmd_u {
        let zh = f64::from_bits(hb);
        for &lb in &tmu_u {
            let zl = f64::from_bits(lb);
            if let Some(g) = spacing((zh - zl).abs()) {
                println!("  1/{g} tmd={:.16} tmu={:.16}", zh, zl);
            }
        }
    }
    let in_set = |z: f64, set: &std::collections::BTreeSet<u64>| {
        set.iter().any(|&b| near(f64::from_bits(b), z))
    };
    let classify = |z: f64| -> &'static str {
        if in_set(z, &tmd_u) {
            "hi-tmd"
        } else if in_set(z, &tmu_u) {
            "lo-tmu"
        } else if in_set(z, &hi_u) {
            "hi"
        } else if in_set(z, &lo_u) {
            "lo"
        } else {
            "other"
        }
    };
    println!("neighbors of leftover-high tmd and leftover-low tmu grid-pair ends:");
    let foci = [
        2.0520833333333335,
        2.1145833333333335,
        2.3645833333333335,
        2.375,
        2.5,
        2.5208333333333335,
        0.8645833333333333,
        0.8958333333333333,
        2.53125,
        2.5625,
        2.3020833333333335,
        2.3125,
        0.875,
        0.8437499999999999,
    ];
    let steps = [16.0, 32.0, 48.0, 96.0];
    for &z0 in &foci {
        print!("  z={z0:.16}");
        for &g in &steps {
            let up = z0 + 1.0 / g;
            let dn = z0 - 1.0 / g;
            print!(
                " +1/{g}={:.16}({}) -1/{g}={:.16}({})",
                up,
                classify(up),
                dn,
                classify(dn)
            );
        }
        println!();
    }
    println!("leftover-low tmu last-store k=+3 of unmask F:");
    let mut lo_k3_eq = 0usize;
    let mut lo_k3_1h = 0usize;
    let mut lo_k3_other = 0usize;
    for &zb in &tmu_u {
        let z = f64::from_bits(zb);
        let mut tbits = None;
        for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
            if r.z.to_bits() == zb {
                tbits = Some(r.qbits);
                break;
            }
        }
        let t = f64::from_bits(tbits.unwrap());
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g3 = mul(w, poke(fu, 3));
        let d = ulp_distance(g3, t).unwrap_or(99);
        let s = if g3 < t {
            "L"
        } else if g3 > t {
            "H"
        } else {
            "="
        };
        if d == 0 {
            lo_k3_eq += 1;
            println!("  k=+3 exact z={:.16} bits={:#x}", z, zb);
        } else if d == 1 && g3 > t {
            lo_k3_1h += 1;
        } else {
            lo_k3_other += 1;
            println!("  k=+3 {d}{s} z={:.16}", z);
        }
    }
    println!(
        "leftover-low tmu n={} k=+3 exact={lo_k3_eq} 1H={lo_k3_1h} other={lo_k3_other}",
        tmu_u.len()
    );
    println!("leftover-high tmd last-store k=-3 of unmask F:");
    let mut hi_k3_eq = 0usize;
    let mut hi_k3_1l = 0usize;
    let mut hi_k3_other = 0usize;
    for &zb in &tmd_u {
        let z = f64::from_bits(zb);
        let mut tbits = None;
        for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
            if r.z.to_bits() == zb {
                tbits = Some(r.qbits);
                break;
            }
        }
        let t = f64::from_bits(tbits.unwrap());
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g3 = mul(w, poke(fu, -3));
        let d = ulp_distance(g3, t).unwrap_or(99);
        let s = if g3 < t {
            "L"
        } else if g3 > t {
            "H"
        } else {
            "="
        };
        if d == 0 {
            hi_k3_eq += 1;
            println!("  k=-3 exact z={:.16} bits={:#x}", z, zb);
        } else if d == 1 && g3 < t {
            hi_k3_1l += 1;
        } else {
            hi_k3_other += 1;
            println!("  k=-3 {d}{s} z={:.16}", z);
        }
    }
    println!(
        "leftover-high tmd n={} k=-3 exact={hi_k3_eq} 1L={hi_k3_1l} other={hi_k3_other}",
        tmd_u.len()
    );
    let signed = |g: f64, t: f64| -> String {
        let d = ulp_distance(g, t).unwrap_or(99);
        let s = if g < t {
            "L"
        } else if g > t {
            "H"
        } else {
            "="
        };
        format!("{d}{s}")
    };
    println!("leftover-low tmu last-store k=+3 vs last-mul k=+3:");
    let mut same = 0usize;
    let mut store_eq_mul_1h = 0usize;
    let mut store_1h_mul_1h = 0usize;
    let mut other_cross = 0usize;
    for &zb in &tmu_u {
        let z = f64::from_bits(zb);
        let mut tbits = None;
        for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
            if r.z.to_bits() == zb {
                tbits = Some(r.qbits);
                break;
            }
        }
        let t = f64::from_bits(tbits.unwrap());
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let st = signed(mul(w, poke(fu, 3)), t);
        let mu = signed(poke(g, 3), t);
        if st == mu {
            same += 1;
        } else if st == "0=" && mu == "1H" {
            store_eq_mul_1h += 1;
        } else if st == "1H" && mu == "1H" {
            store_1h_mul_1h += 1;
        } else {
            other_cross += 1;
        }
        if st != mu {
            println!("  z={:.16} store_k3={st} mul_k3={mu}", z);
        }
    }
    println!(
        "leftover-low tmu store_k3 vs mul_k3 same={same} store= mul=1H={store_eq_mul_1h} both 1H={store_1h_mul_1h} other={other_cross}"
    );
    println!("leftover-high tmd last-store k=-3 vs last-mul k=-3:");
    let mut hsame = 0usize;
    for &zb in &tmd_u {
        let z = f64::from_bits(zb);
        let mut tbits = None;
        for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
            if r.z.to_bits() == zb {
                tbits = Some(r.qbits);
                break;
            }
        }
        let t = f64::from_bits(tbits.unwrap());
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let st = signed(mul(w, poke(fu, -3)), t);
        let mu = signed(poke(g, -3), t);
        if st == mu {
            hsame += 1;
        } else {
            println!("  z={:.16} store_k-3={st} mul_k-3={mu}", z);
        }
    }
    println!(
        "leftover-high tmd store_k-3 vs mul_k-3 same={hsame}/{}",
        tmd_u.len()
    );
    println!("leftover-high tmd last-store k=-2 vs last-mul k=-2:");
    let mut h2same = 0usize;
    for &zb in &tmd_u {
        let z = f64::from_bits(zb);
        let mut tbits = None;
        for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
            if r.z.to_bits() == zb {
                tbits = Some(r.qbits);
                break;
            }
        }
        let t = f64::from_bits(tbits.unwrap());
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let st = signed(mul(w, poke(fu, -2)), t);
        let mu = signed(poke(g, -2), t);
        if st == mu {
            h2same += 1;
        } else {
            println!("  z={:.16} store_k-2={st} mul_k-2={mu}", z);
        }
    }
    println!(
        "leftover-high tmd store_k-2 vs mul_k-2 same={h2same}/{}",
        tmd_u.len()
    );
    println!("leftover-low tmu last-store k=+2 vs last-mul k=+2:");
    let mut l2same = 0usize;
    for &zb in &tmu_u {
        let z = f64::from_bits(zb);
        let mut tbits = None;
        for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
            if r.z.to_bits() == zb {
                tbits = Some(r.qbits);
                break;
            }
        }
        let t = f64::from_bits(tbits.unwrap());
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let st = signed(mul(w, poke(fu, 2)), t);
        let mu = signed(poke(g, 2), t);
        if st == mu {
            l2same += 1;
        } else {
            println!("  z={:.16} store_k+2={st} mul_k+2={mu}", z);
        }
    }
    println!(
        "leftover-low tmu store_k+2 vs mul_k+2 same={l2same}/{}",
        tmu_u.len()
    );
    println!("leftover-low store k=+2 not-tmu last-mul k=+2 vs k=+3:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !lo_u.contains(&zb) || tmu_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if k_of(t, w, fu) != Some(2) {
            continue;
        }
        let g = mul(w, fu);
        println!(
            "  z={:.16} d={} store_k2={} store_k3={} mul_k2={} mul_k3={} tmu={} w+1={} F+1={}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            signed(mul(w, poke(fu, 2)), t),
            signed(mul(w, poke(fu, 3)), t),
            signed(poke(g, 2), t),
            signed(poke(g, 3), t),
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
    }
    println!("leftover-high store k=-2 not-tmd last-mul k=-2 vs k=-3:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !hi_u.contains(&zb) || tmd_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if k_of(t, w, fu) != Some(-2) {
            continue;
        }
        let g = mul(w, fu);
        println!(
            "  z={:.16} d={} store_k-2={} store_k-3={} mul_k-2={} mul_k-3={} tmd={} w-1={} F-1={}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            signed(mul(w, poke(fu, -2)), t),
            signed(mul(w, poke(fu, -3)), t),
            signed(poke(g, -2), t),
            signed(poke(g, -3), t),
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("leftover-high last-store k=-3 exact:");
    let mut n_hi_k3 = 0usize;
    let mut n_hi_k3_tmd = 0usize;
    let mut n_hi_k3_k2 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !hi_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if ulp_distance(mul(w, poke(fu, -3)), t).unwrap_or(99) != 0 {
            continue;
        }
        n_hi_k3 += 1;
        let tmd = tmd_u.contains(&zb);
        if tmd {
            n_hi_k3_tmd += 1;
        }
        let k2 = k_of(t, w, fu) == Some(-2);
        if k2 {
            n_hi_k3_k2 += 1;
        }
        println!(
            "  z={:.16} bits={:#x} tmd={tmd} store_k={:?} d={} mul_k-3={}",
            z,
            zb,
            k_of(t, w, fu),
            ulp_distance(mul(w, fu), t).unwrap_or(99),
            signed(poke(mul(w, fu), -3), t)
        );
    }
    println!(
        "leftover-high last-store k=-3 exact n={n_hi_k3} tmd={n_hi_k3_tmd} also store k=-2={n_hi_k3_k2}"
    );
    println!("leftover-low last-store k=+3 exact:");
    let mut n_lo_k3 = 0usize;
    let mut n_lo_k3_tmu = 0usize;
    let mut n_lo_k3_k2 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !lo_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if ulp_distance(mul(w, poke(fu, 3)), t).unwrap_or(99) != 0 {
            continue;
        }
        n_lo_k3 += 1;
        let tmu = tmu_u.contains(&zb);
        if tmu {
            n_lo_k3_tmu += 1;
        }
        let k2 = k_of(t, w, fu) == Some(2);
        if k2 {
            n_lo_k3_k2 += 1;
        }
        println!(
            "  z={:.16} bits={:#x} tmu={tmu} store_k={:?} d={} mul_k+3={}",
            z,
            zb,
            k_of(t, w, fu),
            ulp_distance(mul(w, fu), t).unwrap_or(99),
            signed(poke(mul(w, fu), 3), t)
        );
    }
    println!(
        "leftover-low last-store k=+3 exact n={n_lo_k3} tmu={n_lo_k3_tmu} also store k=+2={n_lo_k3_k2}"
    );
    println!("leftover-low primary k=+3 (k_of=3):");
    let mut n_p3 = 0usize;
    let mut n_p3_tmu = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !lo_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if k_of(t, w, fu) != Some(3) {
            continue;
        }
        n_p3 += 1;
        let g = mul(w, fu);
        let tmu = tmu_u.contains(&zb);
        if tmu {
            n_p3_tmu += 1;
        }
        println!(
            "  z={:.16} d={} tmu={tmu} store_k2={} store_k3={} mul_k2={} mul_k3={} tmu_hit={} w+1={} F+1={}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            signed(mul(w, poke(fu, 2)), t),
            signed(mul(w, poke(fu, 3)), t),
            signed(poke(g, 2), t),
            signed(poke(g, 3), t),
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
    }
    println!("leftover-low primary k=+3 n={n_p3} tmu={n_p3_tmu}");
    println!("leftover-high primary k=-3 (k_of=-3):");
    let mut n_pm3 = 0usize;
    let mut n_pm3_tmd = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !hi_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if k_of(t, w, fu) != Some(-3) {
            continue;
        }
        n_pm3 += 1;
        let g = mul(w, fu);
        let tmd = tmd_u.contains(&zb);
        if tmd {
            n_pm3_tmd += 1;
        }
        println!(
            "  z={:.16} d={} tmd={tmd} store_k-2={} store_k-3={} mul_k-2={} mul_k-3={} tmd_hit={} w-1={} F-1={}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            signed(mul(w, poke(fu, -2)), t),
            signed(mul(w, poke(fu, -3)), t),
            signed(poke(g, -2), t),
            signed(poke(g, -3), t),
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("leftover-high primary k=-3 n={n_pm3} tmd={n_pm3_tmd}");
    println!("leftover-low last-mul k=+2 exact:");
    let mut n_lm2 = 0usize;
    let mut n_lm2_st2 = 0usize;
    let mut n_lm2_st2_not = 0usize;
    let mut n_lm2_tmu = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !lo_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(poke(g, 2), t).unwrap_or(99) != 0 {
            continue;
        }
        n_lm2 += 1;
        let st2 = ulp_distance(mul(w, poke(fu, 2)), t).unwrap_or(99) == 0;
        if st2 {
            n_lm2_st2 += 1;
        } else {
            n_lm2_st2_not += 1;
        }
        let tmu = tmu_u.contains(&zb);
        if tmu {
            n_lm2_tmu += 1;
        }
        println!(
            "  z={:.16} bits={:#x} d={} tmu={tmu} store_k={:?} store_k2={} store_k3={} mul_k2={} mul_k3={} tmu_hit={} w+1={} F+1={}",
            z,
            zb,
            ulp_distance(g, t).unwrap_or(99),
            k_of(t, w, fu),
            signed(mul(w, poke(fu, 2)), t),
            signed(mul(w, poke(fu, 3)), t),
            signed(poke(g, 2), t),
            signed(poke(g, 3), t),
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
    }
    println!(
        "leftover-low last-mul k=+2 exact n={n_lm2} store_k2_exact={n_lm2_st2} store_k2_not={n_lm2_st2_not} tmu={n_lm2_tmu}"
    );
    println!("leftover-high last-mul k=-2 exact:");
    let mut n_hm2 = 0usize;
    let mut n_hm2_st2 = 0usize;
    let mut n_hm2_st2_not = 0usize;
    let mut n_hm2_tmd = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !hi_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(poke(g, -2), t).unwrap_or(99) != 0 {
            continue;
        }
        n_hm2 += 1;
        let st2 = ulp_distance(mul(w, poke(fu, -2)), t).unwrap_or(99) == 0;
        if st2 {
            n_hm2_st2 += 1;
        } else {
            n_hm2_st2_not += 1;
        }
        let tmd = tmd_u.contains(&zb);
        if tmd {
            n_hm2_tmd += 1;
        }
        println!(
            "  z={:.16} bits={:#x} d={} tmd={tmd} store_k={:?} store_k-2={} store_k-3={} mul_k-2={} mul_k-3={} tmd_hit={} w-1={} F-1={}",
            z,
            zb,
            ulp_distance(g, t).unwrap_or(99),
            k_of(t, w, fu),
            signed(mul(w, poke(fu, -2)), t),
            signed(mul(w, poke(fu, -3)), t),
            signed(poke(g, -2), t),
            signed(poke(g, -3), t),
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!(
        "leftover-high last-mul k=-2 exact n={n_hm2} store_k-2_exact={n_hm2_st2} store_k-2_not={n_hm2_st2_not} tmd={n_hm2_tmd}"
    );
    println!("leftover-low last-store k=+2 exact last-mul k=+2 not:");
    let mut n_st2_mul2_not = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !lo_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(mul(w, poke(fu, 2)), t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(poke(g, 2), t).unwrap_or(99) == 0 {
            continue;
        }
        n_st2_mul2_not += 1;
        println!(
            "  z={:.16} d={} tmu={} store_k={:?} store_k2={} mul_k2={} mul_k3={} mul_k4={}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            tmu_u.contains(&zb),
            k_of(t, w, fu),
            signed(mul(w, poke(fu, 2)), t),
            signed(poke(g, 2), t),
            signed(poke(g, 3), t),
            signed(poke(g, 4), t)
        );
    }
    println!("leftover-low last-store k=+2 exact last-mul k=+2 not n={n_st2_mul2_not}");
    println!("leftover-high last-store k=-2 exact last-mul k=-2 not:");
    let mut n_hst2_mul2_not = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !hi_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(mul(w, poke(fu, -2)), t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(poke(g, -2), t).unwrap_or(99) == 0 {
            continue;
        }
        n_hst2_mul2_not += 1;
        println!(
            "  z={:.16} d={} tmd={} store_k={:?} store_k-2={} mul_k-2={} mul_k-3={} mul_k-4={}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            tmd_u.contains(&zb),
            k_of(t, w, fu),
            signed(mul(w, poke(fu, -2)), t),
            signed(poke(g, -2), t),
            signed(poke(g, -3), t),
            signed(poke(g, -4), t)
        );
    }
    println!("leftover-high last-store k=-2 exact last-mul k=-2 not n={n_hst2_mul2_not}");
    println!("leftover last-store stall consecutive k same signed d:");
    let mut n_lo_stall = 0usize;
    let mut n_hi_stall = 0usize;
    let mut n_lo_stall_1l12 = 0usize;
    let mut n_hi_stall_eq23 = 0usize;
    let mut n_hi_stall_eq34 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        let lo = lo_u.contains(&zb);
        let hi = hi_u.contains(&zb);
        if !lo && !hi {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let mut stalls = Vec::new();
        for k in -4i32..4 {
            let a = signed(mul(w, poke(fu, k)), t);
            let b = signed(mul(w, poke(fu, k + 1)), t);
            if a == b {
                stalls.push((k, a.clone()));
            }
        }
        if stalls.is_empty() {
            continue;
        }
        if lo {
            n_lo_stall += 1;
            if stalls.iter().any(|(k, s)| *k == 1 && s == "1L") {
                n_lo_stall_1l12 += 1;
            }
        } else {
            n_hi_stall += 1;
            if stalls.iter().any(|(k, s)| *k == -3 && s == "0=") {
                n_hi_stall_eq23 += 1;
            }
            if stalls.iter().any(|(k, s)| *k == -4 && s == "0=") {
                n_hi_stall_eq34 += 1;
            }
        }
        let tag = if lo { "LO" } else { "HI" };
        print!(
            "  {tag} z={:.16} d={} store_k={:?} tmu={} tmd={} stall",
            z,
            ulp_distance(g, t).unwrap_or(99),
            k_of(t, w, fu),
            tmu_u.contains(&zb),
            tmd_u.contains(&zb)
        );
        for (k, s) in &stalls {
            print!(" {k}/{}={s}", k + 1);
        }
        println!(
            " mul_k-4..4 {} {} {} {} {} {} {} {} {}",
            signed(poke(g, -4), t),
            signed(poke(g, -3), t),
            signed(poke(g, -2), t),
            signed(poke(g, -1), t),
            signed(poke(g, 0), t),
            signed(poke(g, 1), t),
            signed(poke(g, 2), t),
            signed(poke(g, 3), t),
            signed(poke(g, 4), t)
        );
    }
    println!(
        "leftover last-store stall n_lo={n_lo_stall} n_hi={n_hi_stall} lo_stall_k+1/+2_1L={n_lo_stall_1l12} hi_stall_k-3/-2_eq={n_hi_stall_eq23} hi_stall_k-4/-3_eq={n_hi_stall_eq34}"
    );
    println!("leftover-low last-mul k=+4 exact:");
    let mut n_lm4 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !lo_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(poke(g, 4), t).unwrap_or(99) != 0 {
            continue;
        }
        n_lm4 += 1;
        println!(
            "  z={:.16} d={} tmu={} store_k={:?} store_k2={} store_k3={} store_k4={} mul_k2={} mul_k3={} mul_k4={}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            tmu_u.contains(&zb),
            k_of(t, w, fu),
            signed(mul(w, poke(fu, 2)), t),
            signed(mul(w, poke(fu, 3)), t),
            signed(mul(w, poke(fu, 4)), t),
            signed(poke(g, 2), t),
            signed(poke(g, 3), t),
            signed(poke(g, 4), t)
        );
    }
    println!("leftover-low last-mul k=+4 exact n={n_lm4}");
    println!("leftover-high last-mul k=-4 exact:");
    let mut n_hm4 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !hi_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(poke(g, -4), t).unwrap_or(99) != 0 {
            continue;
        }
        n_hm4 += 1;
        println!(
            "  z={:.16} d={} tmd={} store_k={:?} store_k-2={} store_k-3={} store_k-4={} mul_k-2={} mul_k-3={} mul_k-4={}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            tmd_u.contains(&zb),
            k_of(t, w, fu),
            signed(mul(w, poke(fu, -2)), t),
            signed(mul(w, poke(fu, -3)), t),
            signed(mul(w, poke(fu, -4)), t),
            signed(poke(g, -2), t),
            signed(poke(g, -3), t),
            signed(poke(g, -4), t)
        );
    }
    println!("leftover-high last-mul k=-4 exact n={n_hm4}");
    println!("pin 2.645833/2.65625/2.677083/2.6875/2.75 cluster:");
    let cluster = [
        2.6458333333333335,
        2.65625,
        2.6770833333333335,
        2.6875,
        2.75,
        2.3958333333333335,
        3.0625,
        2.125,
        1.875,
        3.40625,
        1.9895833333333333,
        3.75,
        3.5625,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !cluster.iter().any(|&z| (r.z - z).abs() < 1e-14) {
            continue;
        }
        let z = r.z;
        let zb = z.to_bits();
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        let g = mul(w, fu);
        let tz = ((zb & ((1u64 << 52) - 1)).trailing_zeros() as usize).min(52);
        let z3 = 3.0 * z;
        let tz3 = ((z3.to_bits() & ((1u64 << 52) - 1)).trailing_zeros() as usize).min(52);
        let hx = format!("{zb:x}");
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        let tag = if lo_u.contains(&zb) {
            "LO"
        } else if hi_u.contains(&zb) {
            "HI"
        } else {
            "OT"
        };
        println!(
            "  {tag} z={:.16} bits={:#x} 555={is555} tz={tz} tz3={tz3} d={} store_k={:?} tmu={} tmd={} 210={} 74={} 114={}",
            z,
            zb,
            ulp_distance(g, t).unwrap_or(99),
            k_of(t, w, fu),
            tmu_u.contains(&zb),
            tmd_u.contains(&zb),
            signed(mul(w, f210), t),
            signed(mul(w, f74), t),
            signed(mul(w, f114), t)
        );
        print!("    store");
        for k in -4i32..=4 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        println!();
        print!("    mul  ");
        for k in -4i32..=4 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmu_hit={} w+1={} F+1={} tmd_hit={} w-1={} F-1={}",
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    let mul_k_of = |g: f64, t: f64| -> Option<i32> {
        for ak in 0i32..=8 {
            if ulp_distance(poke(g, ak), t).unwrap_or(99) == 0 {
                return Some(ak);
            }
            if ak > 0 && ulp_distance(poke(g, -ak), t).unwrap_or(99) == 0 {
                return Some(-ak);
            }
        }
        None
    };
    println!("leftover-low last-store vs last-mul swap:");
    let mut n_lo_swap = 0usize;
    let mut n_lo_mul_first = 0usize;
    let mut n_lo_st_first = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !lo_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let ks = k_of(t, w, fu);
        let km = mul_k_of(g, t);
        if ks == km {
            continue;
        }
        n_lo_swap += 1;
        let dir = match (ks, km) {
            (Some(s), Some(m)) if m.abs() < s.abs() => {
                n_lo_mul_first += 1;
                "mul_first"
            }
            (Some(s), Some(m)) if s.abs() < m.abs() => {
                n_lo_st_first += 1;
                "store_first"
            }
            _ => "other",
        };
        println!(
            "  z={:.16} d={} tmu={} store_k={ks:?} mul_k={km:?} {dir} w+1={} F+1={}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            tmu_u.contains(&zb),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
    }
    println!(
        "leftover-low store/mul swap n={n_lo_swap} mul_first={n_lo_mul_first} store_first={n_lo_st_first}"
    );
    println!("leftover-high last-store vs last-mul swap:");
    let mut n_hi_swap = 0usize;
    let mut n_hi_mul_first = 0usize;
    let mut n_hi_st_first = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !hi_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let ks = k_of(t, w, fu);
        let km = mul_k_of(g, t);
        if ks == km {
            continue;
        }
        n_hi_swap += 1;
        let dir = match (ks, km) {
            (Some(s), Some(m)) if m.abs() < s.abs() => {
                n_hi_mul_first += 1;
                "mul_first"
            }
            (Some(s), Some(m)) if s.abs() < m.abs() => {
                n_hi_st_first += 1;
                "store_first"
            }
            _ => "other",
        };
        println!(
            "  z={:.16} d={} tmd={} store_k={ks:?} mul_k={km:?} {dir} w-1={} F-1={}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            tmd_u.contains(&zb),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!(
        "leftover-high store/mul swap n={n_hi_swap} mul_first={n_hi_mul_first} store_first={n_hi_st_first}"
    );
    println!("leftover-low primary k=+1:");
    let mut n_p1 = 0usize;
    let mut n_p1_lm2 = 0usize;
    let mut n_p1_wp = 0usize;
    let mut n_p1_fp = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !lo_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if k_of(t, w, fu) != Some(1) {
            continue;
        }
        n_p1 += 1;
        let g = mul(w, fu);
        let lm2 = ulp_distance(poke(g, 2), t).unwrap_or(99) == 0;
        if lm2 {
            n_p1_lm2 += 1;
        }
        let wp = hit(mul(w.next_up(), fu), t);
        let fp = hit(mul(w, fu.next_up()), t);
        if wp {
            n_p1_wp += 1;
        }
        if fp {
            n_p1_fp += 1;
        }
        println!(
            "  z={:.16} d={} tmu={} mul_k={:?} mul_k2={} w+1={wp} F+1={fp}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            tmu_u.contains(&zb),
            mul_k_of(g, t),
            signed(poke(g, 2), t)
        );
    }
    println!("leftover-low primary k=+1 n={n_p1} last-mul k=+2 exact={n_p1_lm2} w+1={n_p1_wp} F+1={n_p1_fp}");
    println!("leftover-high primary k=-1:");
    let mut n_m1 = 0usize;
    let mut n_m1_lm2 = 0usize;
    let mut n_m1_wm = 0usize;
    let mut n_m1_fm = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !hi_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if k_of(t, w, fu) != Some(-1) {
            continue;
        }
        n_m1 += 1;
        let g = mul(w, fu);
        let lm2 = ulp_distance(poke(g, -2), t).unwrap_or(99) == 0;
        if lm2 {
            n_m1_lm2 += 1;
        }
        let wm = hit(mul(w.next_down(), fu), t);
        let fm = hit(mul(w, fu.next_down()), t);
        if wm {
            n_m1_wm += 1;
        }
        if fm {
            n_m1_fm += 1;
        }
        println!(
            "  z={:.16} d={} tmd={} mul_k={:?} mul_k-2={} w-1={wm} F-1={fm}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            tmd_u.contains(&zb),
            mul_k_of(g, t),
            signed(poke(g, -2), t)
        );
    }
    println!("leftover-high primary k=-1 n={n_m1} last-mul k=-2 exact={n_m1_lm2} w-1={n_m1_wm} F-1={n_m1_fm}");
    println!("leftover-low primary k=+4:");
    let mut n_p4 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !lo_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if k_of(t, w, fu) != Some(4) {
            continue;
        }
        n_p4 += 1;
        let g = mul(w, fu);
        println!(
            "  z={:.16} bits={:#x} d={} tmu={} mul_k={:?} store_k2={} store_k3={} store_k4={} mul_k2={} mul_k3={} mul_k4={} w+1={} F+1={}",
            z,
            zb,
            ulp_distance(g, t).unwrap_or(99),
            tmu_u.contains(&zb),
            mul_k_of(g, t),
            signed(mul(w, poke(fu, 2)), t),
            signed(mul(w, poke(fu, 3)), t),
            signed(mul(w, poke(fu, 4)), t),
            signed(poke(g, 2), t),
            signed(poke(g, 3), t),
            signed(poke(g, 4), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
    }
    println!("leftover-low primary k=+4 n={n_p4}");
    println!("leftover-high primary k=-4:");
    let mut n_m4 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !hi_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if k_of(t, w, fu) != Some(-4) {
            continue;
        }
        n_m4 += 1;
        let g = mul(w, fu);
        println!(
            "  z={:.16} bits={:#x} d={} tmd={} mul_k={:?} store_k-2={} store_k-3={} store_k-4={} mul_k-2={} mul_k-3={} mul_k-4={} w-1={} F-1={}",
            z,
            zb,
            ulp_distance(g, t).unwrap_or(99),
            tmd_u.contains(&zb),
            mul_k_of(g, t),
            signed(mul(w, poke(fu, -2)), t),
            signed(mul(w, poke(fu, -3)), t),
            signed(mul(w, poke(fu, -4)), t),
            signed(poke(g, -2), t),
            signed(poke(g, -3), t),
            signed(poke(g, -4), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("leftover-high primary k=-4 n={n_m4}");
    println!("DIRECT mid last-store k=+1 vs w+1:");
    let mut n_s1 = 0usize;
    let mut n_s1_wp = 0usize;
    let mut n_s1_lo = 0usize;
    let mut n_s1_1ulp = 0usize;
    let mut n_s1_fuse = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if k_of(t, w, fu) != Some(1) {
            continue;
        }
        n_s1 += 1;
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        let wp = hit(mul(w.next_up(), fu), t);
        if wp {
            n_s1_wp += 1;
        }
        let kind = if d == 0 {
            n_s1_fuse += 1;
            "fused"
        } else if d == 1 && g < t {
            n_s1_1ulp += 1;
            "1L"
        } else if d >= 2 && g < t {
            n_s1_lo += 1;
            "LO"
        } else {
            "other"
        };
        if !wp || kind == "LO" {
            println!(
                "  z={:.16} d={} kind={kind} w+1={wp} F+1={} tmu={}",
                z,
                d,
                hit(mul(w, fu.next_up()), t),
                hit(mul(w.next_up(), fu.next_up()), t)
            );
        }
    }
    println!("DIRECT mid last-store k=+1 n={n_s1} w+1={n_s1_wp} leftover-low={n_s1_lo} 1L={n_s1_1ulp} fused={n_s1_fuse}");
    println!("DIRECT mid last-store k=-1 vs w-1:");
    let mut n_sm1 = 0usize;
    let mut n_sm1_wm = 0usize;
    let mut n_sm1_hi = 0usize;
    let mut n_sm1_1ulp = 0usize;
    let mut n_sm1_fuse = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if k_of(t, w, fu) != Some(-1) {
            continue;
        }
        n_sm1 += 1;
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        let wm = hit(mul(w.next_down(), fu), t);
        if wm {
            n_sm1_wm += 1;
        }
        let kind = if d == 0 {
            n_sm1_fuse += 1;
            "fused"
        } else if d == 1 && g > t {
            n_sm1_1ulp += 1;
            "1H"
        } else if d >= 2 && g > t {
            n_sm1_hi += 1;
            "HI"
        } else {
            "other"
        };
        if wm || kind == "HI" {
            println!(
                "  z={:.16} d={} kind={kind} w-1={wm} F-1={} tmd={}",
                z,
                d,
                hit(mul(w, fu.next_down()), t),
                hit(mul(w.next_down(), fu.next_down()), t)
            );
        }
    }
    println!("DIRECT mid last-store k=-1 n={n_sm1} w-1={n_sm1_wm} leftover-high={n_sm1_hi} 1H={n_sm1_1ulp} fused={n_sm1_fuse}");
    println!("1L last-store k=+1 miss w+1:");
    let mut n_1l_miss = 0usize;
    let mut n_1l_miss_lmup = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if k_of(t, w, fu) != Some(1) {
            continue;
        }
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g < t) {
            continue;
        }
        if hit(mul(w.next_up(), fu), t) {
            continue;
        }
        n_1l_miss += 1;
        let lmup = hit(g.next_up(), t);
        if lmup {
            n_1l_miss_lmup += 1;
        }
        let zb = z.to_bits();
        let hx = format!("{zb:x}");
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        println!(
            "  z={:.16} bits={:#x} 555={is555} mul_k={:?} last-mul_up={lmup} F+1={} tmu={} store_k2={}",
            z,
            zb,
            mul_k_of(g, t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_up(), fu.next_up()), t),
            signed(mul(w, poke(fu, 2)), t)
        );
    }
    println!("1L last-store k=+1 miss w+1 n={n_1l_miss} last-mul_up={n_1l_miss_lmup}");
    println!("1H last-store k=-1 miss w-1:");
    let mut n_1h_miss = 0usize;
    let mut n_1h_miss_lmdn = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if k_of(t, w, fu) != Some(-1) {
            continue;
        }
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if hit(mul(w.next_down(), fu), t) {
            continue;
        }
        n_1h_miss += 1;
        let lmdn = hit(g.next_down(), t);
        if lmdn {
            n_1h_miss_lmdn += 1;
        }
        let zb = z.to_bits();
        let hx = format!("{zb:x}");
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        println!(
            "  z={:.16} bits={:#x} 555={is555} mul_k={:?} last-mul_dn={lmdn} F-1={} tmd={} store_k-2={}",
            z,
            zb,
            mul_k_of(g, t),
            hit(mul(w, fu.next_down()), t),
            hit(mul(w.next_down(), fu.next_down()), t),
            signed(mul(w, poke(fu, -2)), t)
        );
    }
    println!("1H last-store k=-1 miss w-1 n={n_1h_miss} last-mul_dn={n_1h_miss_lmdn}");
    println!("DIRECT mid unmask tmd by d:");
    let mut tmd_d = [0usize; 6];
    let mut tmd_hi = 0usize;
    let mut tmd_1h = 0usize;
    let mut tmd_1h_k2 = 0usize;
    let mut tmd_1h_wm = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if !hit(mul(w.next_down(), fu.next_down()), t) {
            continue;
        }
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99).min(5) as usize;
        tmd_d[d] += 1;
        if hi_u.contains(&zb) {
            tmd_hi += 1;
        }
        if d == 1 && g > t {
            tmd_1h += 1;
            let k2 = ulp_distance(mul(w, poke(fu, -2)), t).unwrap_or(99) == 0;
            if k2 {
                tmd_1h_k2 += 1;
            }
            let wm = hit(mul(w.next_down(), fu), t);
            if wm {
                tmd_1h_wm += 1;
            }
            println!(
                "  1H-tmd z={:.16} bits={:#x} store_k={:?} store_k-2={} mul_k={:?} w-1={wm} F-1={} 210={}",
                z,
                zb,
                k_of(t, w, fu),
                signed(mul(w, poke(fu, -2)), t),
                mul_k_of(g, t),
                hit(mul(w, fu.next_down()), t),
                signed(mul(w, cody(z, &C0, &D0, 0x210)), t)
            );
        }
    }
    print!("DIRECT mid tmd by d");
    for (i, c) in tmd_d.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!(" leftover-high={tmd_hi} 1H={tmd_1h} 1H_store_k-2_exact={tmd_1h_k2} 1H_w-1={tmd_1h_wm}");
    println!("DIRECT mid unmask tmu by d:");
    let mut tmu_d = [0usize; 6];
    let mut tmu_lo = 0usize;
    let mut tmu_1l = 0usize;
    let mut tmu_1l_k2 = 0usize;
    let mut tmu_1l_wp = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if !hit(mul(w.next_up(), fu.next_up()), t) {
            continue;
        }
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99).min(5) as usize;
        tmu_d[d] += 1;
        if lo_u.contains(&zb) {
            tmu_lo += 1;
        }
        if d == 1 && g < t {
            tmu_1l += 1;
            let k2 = ulp_distance(mul(w, poke(fu, 2)), t).unwrap_or(99) == 0;
            if k2 {
                tmu_1l_k2 += 1;
            }
            let wp = hit(mul(w.next_up(), fu), t);
            if wp {
                tmu_1l_wp += 1;
            }
            println!(
                "  1L-tmu z={:.16} bits={:#x} store_k={:?} store_k2={} mul_k={:?} w+1={wp} F+1={} 210={}",
                z,
                zb,
                k_of(t, w, fu),
                signed(mul(w, poke(fu, 2)), t),
                mul_k_of(g, t),
                hit(mul(w, fu.next_up()), t),
                signed(mul(w, cody(z, &C0, &D0, 0x210)), t)
            );
        }
    }
    print!("DIRECT mid tmu by d");
    for (i, c) in tmu_d.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!(" leftover-low={tmu_lo} 1L={tmu_1l} 1L_store_k2_exact={tmu_1l_k2} 1L_w+1={tmu_1l_wp}");
    println!("DIRECT mid last-store stall k=-1/0=1H:");
    let mut n_stall_h = 0usize;
    let mut n_stall_h_tmd = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let s0 = signed(g, t);
        let s1 = signed(mul(w, poke(fu, -1)), t);
        if s0 != "1H" || s1 != "1H" {
            continue;
        }
        n_stall_h += 1;
        let tmd = hit(mul(w.next_down(), fu.next_down()), t);
        if tmd {
            n_stall_h_tmd += 1;
        }
        println!(
            "  z={:.16} d={} tmd={tmd} store_k={:?} store_k-2={} mul_k={:?} w-1={} F-1={}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            k_of(t, w, fu),
            signed(mul(w, poke(fu, -2)), t),
            mul_k_of(g, t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("DIRECT mid last-store stall k=-1/0=1H n={n_stall_h} tmd={n_stall_h_tmd}");
    println!("DIRECT mid last-store stall k=+1/0=1L:");
    let mut n_stall_l = 0usize;
    let mut n_stall_l_tmu = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let s0 = signed(g, t);
        let s1 = signed(mul(w, poke(fu, 1)), t);
        if s0 != "1L" || s1 != "1L" {
            continue;
        }
        n_stall_l += 1;
        let tmu = hit(mul(w.next_up(), fu.next_up()), t);
        if tmu {
            n_stall_l_tmu += 1;
        }
        println!(
            "  z={:.16} d={} tmu={tmu} store_k={:?} store_k2={} mul_k={:?} w+1={} F+1={}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            k_of(t, w, fu),
            signed(mul(w, poke(fu, 2)), t),
            mul_k_of(g, t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
    }
    println!("DIRECT mid last-store stall k=+1/0=1L n={n_stall_l} tmu={n_stall_l_tmu}");
    let is555 = |zb: u64| {
        let h = format!("{zb:x}");
        h.contains("55555555") || h.contains("aaaaaaaa")
    };
    let tz_of = |zb: u64| ((zb & ((1u64 << 52) - 1)).trailing_zeros() as usize).min(52);
    println!("leftover-low tmu double last-store k=+2/+3:");
    let mut n_lo_dbl = 0usize;
    let mut n_lo_dbl_555 = 0usize;
    let mut n_lo_dbl_wp = 0usize;
    let mut n_lo_dbl_fp = 0usize;
    let mut n_lo_dbl_m2 = 0usize;
    let mut n_lo_dbl_m3 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !tmu_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let st2 = ulp_distance(mul(w, poke(fu, 2)), t).unwrap_or(99) == 0;
        let st3 = ulp_distance(mul(w, poke(fu, 3)), t).unwrap_or(99) == 0;
        if !(st2 && st3) {
            continue;
        }
        n_lo_dbl += 1;
        if is555(zb) {
            n_lo_dbl_555 += 1;
        }
        let wp = hit(mul(w.next_up(), fu), t);
        let fp = hit(mul(w, fu.next_up()), t);
        if wp {
            n_lo_dbl_wp += 1;
        }
        if fp {
            n_lo_dbl_fp += 1;
        }
        let m2 = ulp_distance(poke(g, 2), t).unwrap_or(99) == 0;
        let m3 = ulp_distance(poke(g, 3), t).unwrap_or(99) == 0;
        if m2 {
            n_lo_dbl_m2 += 1;
        }
        if m3 {
            n_lo_dbl_m3 += 1;
        }
        let z3 = 3.0 * z;
        println!(
            "  z={:.16} bits={:#x} 555={} tz={} tz3={} d={} mul_k={:?} store_k2={} store_k3={} mul_k2={} mul_k3={} w+1={wp} F+1={fp} tmu_hit={}",
            z,
            zb,
            is555(zb),
            tz_of(zb),
            tz_of(z3.to_bits()),
            ulp_distance(g, t).unwrap_or(99),
            mul_k_of(g, t),
            signed(mul(w, poke(fu, 2)), t),
            signed(mul(w, poke(fu, 3)), t),
            signed(poke(g, 2), t),
            signed(poke(g, 3), t),
            hit(mul(w.next_up(), fu.next_up()), t)
        );
    }
    println!(
        "leftover-low tmu double k=+2/+3 n={n_lo_dbl} 555={n_lo_dbl_555} last-mul k=+2={n_lo_dbl_m2} last-mul k=+3={n_lo_dbl_m3} w+1={n_lo_dbl_wp} F+1={n_lo_dbl_fp}"
    );
    println!("1L tmu double last-store k=+1/+2:");
    let mut n_1l_dbl = 0usize;
    let mut n_1l_dbl_555 = 0usize;
    let mut n_1l_dbl_wp = 0usize;
    let mut n_1l_dbl_fp = 0usize;
    let mut n_1l_dbl_m1 = 0usize;
    let mut n_1l_dbl_m2 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g < t) {
            continue;
        }
        if !hit(mul(w.next_up(), fu.next_up()), t) {
            continue;
        }
        n_1l_dbl += 1;
        let zb = z.to_bits();
        if is555(zb) {
            n_1l_dbl_555 += 1;
        }
        let wp = hit(mul(w.next_up(), fu), t);
        let fp = hit(mul(w, fu.next_up()), t);
        if wp {
            n_1l_dbl_wp += 1;
        }
        if fp {
            n_1l_dbl_fp += 1;
        }
        let m1 = ulp_distance(poke(g, 1), t).unwrap_or(99) == 0;
        let m2 = ulp_distance(poke(g, 2), t).unwrap_or(99) == 0;
        if m1 {
            n_1l_dbl_m1 += 1;
        }
        if m2 {
            n_1l_dbl_m2 += 1;
        }
        let z3 = 3.0 * z;
        println!(
            "  z={:.16} bits={:#x} 555={} tz={} tz3={} store_k={:?} store_k1={} store_k2={} mul_k={:?} mul_k1={} mul_k2={} w+1={wp} F+1={fp} 210={}",
            z,
            zb,
            is555(zb),
            tz_of(zb),
            tz_of(z3.to_bits()),
            k_of(t, w, fu),
            signed(mul(w, poke(fu, 1)), t),
            signed(mul(w, poke(fu, 2)), t),
            mul_k_of(g, t),
            signed(poke(g, 1), t),
            signed(poke(g, 2), t),
            signed(mul(w, cody(z, &C0, &D0, 0x210)), t)
        );
    }
    println!(
        "1L tmu n={n_1l_dbl} 555={n_1l_dbl_555} last-mul k=+1={n_1l_dbl_m1} last-mul k=+2={n_1l_dbl_m2} w+1={n_1l_dbl_wp} F+1={n_1l_dbl_fp}"
    );
    println!("DIRECT mid double last-store k=+1/+2:");
    let mut n_d12 = 0usize;
    let mut n_d12_tmu = 0usize;
    let mut n_d12_1l = 0usize;
    let mut n_d12_lo = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if ulp_distance(mul(w, poke(fu, 1)), t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, 2)), t).unwrap_or(99) != 0 {
            continue;
        }
        n_d12 += 1;
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        let tmu = hit(mul(w.next_up(), fu.next_up()), t);
        if tmu {
            n_d12_tmu += 1;
        }
        if d == 1 && g < t {
            n_d12_1l += 1;
        }
        if lo_u.contains(&zb) {
            n_d12_lo += 1;
        }
        println!(
            "  z={:.16} d={} tmu={tmu} lo={} store_k={:?} mul_k={:?} w+1={} F+1={} 555={}",
            z,
            d,
            lo_u.contains(&zb),
            k_of(t, w, fu),
            mul_k_of(g, t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            is555(zb)
        );
    }
    println!("DIRECT mid double last-store k=+1/+2 n={n_d12} tmu={n_d12_tmu} 1L={n_d12_1l} leftover-low={n_d12_lo}");
    println!("DIRECT mid double last-store k=+2/+3:");
    let mut n_d23 = 0usize;
    let mut n_d23_tmu = 0usize;
    let mut n_d23_lo = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if ulp_distance(mul(w, poke(fu, 2)), t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, 3)), t).unwrap_or(99) != 0 {
            continue;
        }
        n_d23 += 1;
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        let tmu = hit(mul(w.next_up(), fu.next_up()), t);
        if tmu {
            n_d23_tmu += 1;
        }
        if lo_u.contains(&zb) {
            n_d23_lo += 1;
        }
        println!(
            "  z={:.16} d={} tmu={tmu} lo={} store_k={:?} mul_k={:?} w+1={} F+1={} 555={}",
            z,
            d,
            lo_u.contains(&zb),
            k_of(t, w, fu),
            mul_k_of(g, t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            is555(zb)
        );
    }
    println!("DIRECT mid double last-store k=+2/+3 n={n_d23} tmu={n_d23_tmu} leftover-low={n_d23_lo}");
    println!("1H tmd double last-store k=-1/-2:");
    let mut n_1h_dbl = 0usize;
    let mut n_1h_dbl_wm = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if !hit(mul(w.next_down(), fu.next_down()), t) {
            continue;
        }
        let st1 = ulp_distance(mul(w, poke(fu, -1)), t).unwrap_or(99) == 0;
        let st2 = ulp_distance(mul(w, poke(fu, -2)), t).unwrap_or(99) == 0;
        if !(st1 && st2) {
            continue;
        }
        n_1h_dbl += 1;
        let wm = hit(mul(w.next_down(), fu), t);
        if wm {
            n_1h_dbl_wm += 1;
        }
        let zb = z.to_bits();
        println!(
            "  z={:.16} bits={:#x} 555={} store_k={:?} mul_k={:?} store_k-1={} store_k-2={} mul_k-1={} mul_k-2={} w-1={wm} F-1={} 210={}",
            z,
            zb,
            is555(zb),
            k_of(t, w, fu),
            mul_k_of(g, t),
            signed(mul(w, poke(fu, -1)), t),
            signed(mul(w, poke(fu, -2)), t),
            signed(poke(g, -1), t),
            signed(poke(g, -2), t),
            hit(mul(w, fu.next_down()), t),
            signed(mul(w, cody(z, &C0, &D0, 0x210)), t)
        );
    }
    println!("1H tmd double last-store k=-1/-2 n={n_1h_dbl} w-1={n_1h_dbl_wm}");
    println!("leftover-high tmd double last-store k=-2/-3:");
    let mut n_hi_dbl = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !tmd_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let st2 = ulp_distance(mul(w, poke(fu, -2)), t).unwrap_or(99) == 0;
        let st3 = ulp_distance(mul(w, poke(fu, -3)), t).unwrap_or(99) == 0;
        if !(st2 && st3) {
            continue;
        }
        n_hi_dbl += 1;
        let g = mul(w, fu);
        println!(
            "  z={:.16} d={} store_k={:?} mul_k={:?} w-1={} F-1={}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            k_of(t, w, fu),
            mul_k_of(g, t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("leftover-high tmd double last-store k=-2/-3 n={n_hi_dbl}");
    println!("1L tmu / leftover-low tmu double grid neighbors:");
    let mut tmu1l: Vec<f64> = Vec::new();
    let mut tmu_lo_dbl: Vec<f64> = Vec::new();
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d == 1 && g < t && hit(mul(w.next_up(), fu.next_up()), t) {
            tmu1l.push(z);
        }
        if tmu_u.contains(&zb)
            && ulp_distance(mul(w, poke(fu, 2)), t).unwrap_or(99) == 0
            && ulp_distance(mul(w, poke(fu, 3)), t).unwrap_or(99) == 0
        {
            tmu_lo_dbl.push(z);
        }
    }
    let tag = |z: f64| -> &'static str {
        if tmu1l.iter().any(|&x| (x - z).abs() < 1e-14) {
            "1L-tmu"
        } else if tmu_lo_dbl.iter().any(|&x| (x - z).abs() < 1e-14) {
            "lo-tmu-dbl"
        } else if tmu_u.contains(&z.to_bits()) || tmu_u.iter().any(|&b| (f64::from_bits(b) - z).abs() < 1e-14) {
            "lo-tmu"
        } else if tmd_u.contains(&z.to_bits()) || tmd_u.iter().any(|&b| (f64::from_bits(b) - z).abs() < 1e-14) {
            "hi-tmd"
        } else if lo_u.contains(&z.to_bits()) || lo_u.iter().any(|&b| (f64::from_bits(b) - z).abs() < 1e-14) {
            "LO"
        } else if hi_u.contains(&z.to_bits()) || hi_u.iter().any(|&b| (f64::from_bits(b) - z).abs() < 1e-14) {
            "HI"
        } else {
            "other"
        }
    };
    let steps = [1.0 / 16.0, 1.0 / 32.0, 1.0 / 48.0, 1.0 / 96.0];
    let names = ["1/16", "1/32", "1/48", "1/96"];
    for &z in tmu1l.iter().chain(tmu_lo_dbl.iter()) {
        print!("  z={z:.16}");
        for (step, name) in steps.iter().zip(names.iter()) {
            print!(
                " {name}+{} {name}-{}",
                tag(z + step),
                tag(z - step)
            );
        }
        println!();
    }
    println!("leftover-low tmu not-double last-store k=+3:");
    let mut n_lo_nd = 0usize;
    let mut n_lo_nd_k3_1h = 0usize;
    let mut n_lo_nd_555 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !tmu_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let st3 = signed(mul(w, poke(fu, 3)), t);
        if st3 == "0=" {
            continue;
        }
        n_lo_nd += 1;
        if st3 == "1H" {
            n_lo_nd_k3_1h += 1;
        }
        if is555(zb) {
            n_lo_nd_555 += 1;
        }
        println!(
            "  z={:.16} bits={:#x} 555={} tz={} d={} store_k={:?} store_k2={} store_k3={} mul_k={:?} mul_k2={} mul_k3={} w+1={} F+1={}",
            z,
            zb,
            is555(zb),
            tz_of(zb),
            ulp_distance(g, t).unwrap_or(99),
            k_of(t, w, fu),
            signed(mul(w, poke(fu, 2)), t),
            st3,
            mul_k_of(g, t),
            signed(poke(g, 2), t),
            signed(poke(g, 3), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
    }
    println!("leftover-low tmu not-double n={n_lo_nd} store_k3_1H={n_lo_nd_k3_1h} 555={n_lo_nd_555}");
    println!("1H tmd not-double last-store k=-2:");
    let mut n_1h_nd = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if !hit(mul(w.next_down(), fu.next_down()), t) {
            continue;
        }
        let st1 = ulp_distance(mul(w, poke(fu, -1)), t).unwrap_or(99) == 0;
        let st2 = ulp_distance(mul(w, poke(fu, -2)), t).unwrap_or(99) == 0;
        if st1 && st2 {
            continue;
        }
        n_1h_nd += 1;
        let zb = z.to_bits();
        println!(
            "  z={:.16} bits={:#x} 555={} store_k={:?} mul_k={:?} store_k-1={} store_k-2={} mul_k-1={} mul_k-2={} w-1={} F-1={} 210={}",
            z,
            zb,
            is555(zb),
            k_of(t, w, fu),
            mul_k_of(g, t),
            signed(mul(w, poke(fu, -1)), t),
            signed(mul(w, poke(fu, -2)), t),
            signed(poke(g, -1), t),
            signed(poke(g, -2), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t),
            signed(mul(w, cody(z, &C0, &D0, 0x210)), t)
        );
    }
    println!("1H tmd not-double n={n_1h_nd}");
    println!("pin 0.8125 1L double last-store not-tmu:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 0.8125).abs() > 1e-14 {
            continue;
        }
        let z = r.z;
        let zb = z.to_bits();
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        let g = mul(w, fu);
        println!(
            "  z={:.16} bits={:#x} 555={} tz={} d={} store_k={:?} mul_k={:?} 210={} 74={} 114={}",
            z,
            zb,
            is555(zb),
            tz_of(zb),
            ulp_distance(g, t).unwrap_or(99),
            k_of(t, w, fu),
            mul_k_of(g, t),
            signed(mul(w, f210), t),
            signed(mul(w, f74), t),
            signed(mul(w, f114), t)
        );
        print!("    store");
        for k in -4i32..=4 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        println!();
        print!("    mul  ");
        for k in -4i32..=4 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmu={} w+1={} F+1={} tmd={} w-1={} F-1={}",
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
        for (step, name) in [
            (1.0 / 16.0, "1/16"),
            (1.0 / 32.0, "1/32"),
            (1.0 / 48.0, "1/48"),
            (1.0 / 96.0, "1/96"),
        ] {
            println!(
                "    {name}+{} {name}-{}",
                tag(z + step),
                tag(z - step)
            );
        }
    }
    println!("DIRECT mid last-store stall k=-1/0=1L:");
    let mut n_s10l = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if signed(g, t) != "1L" {
            continue;
        }
        if signed(mul(w, poke(fu, -1)), t) != "1L" {
            continue;
        }
        n_s10l += 1;
        println!(
            "  z={:.16} d={} tmu={} store_k={:?} mul_k={:?} store_k2={} w+1={} F+1={}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            hit(mul(w.next_up(), fu.next_up()), t),
            k_of(t, w, fu),
            mul_k_of(g, t),
            signed(mul(w, poke(fu, 2)), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
    }
    println!("DIRECT mid last-store stall k=-1/0=1L n={n_s10l}");
    println!("DIRECT mid last-store stall k=+3/+4=1H:");
    let mut n_s34h = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if signed(mul(w, poke(fu, 3)), t) != "1H" {
            continue;
        }
        if signed(mul(w, poke(fu, 4)), t) != "1H" {
            continue;
        }
        n_s34h += 1;
        let g = mul(w, fu);
        println!(
            "  z={:.16} d={} tmu={} store_k={:?} mul_k={:?} store_k1={} store_k2={}",
            z,
            ulp_distance(g, t).unwrap_or(99),
            hit(mul(w.next_up(), fu.next_up()), t),
            k_of(t, w, fu),
            mul_k_of(g, t),
            signed(mul(w, poke(fu, 1)), t),
            signed(mul(w, poke(fu, 2)), t)
        );
    }
    println!("DIRECT mid last-store stall k=+3/+4=1H n={n_s34h}");
    println!("DIRECT mid last-store stall k=+2/+3=1H:");
    let mut n_s23h = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if signed(mul(w, poke(fu, 2)), t) != "1H" {
            continue;
        }
        if signed(mul(w, poke(fu, 3)), t) != "1H" {
            continue;
        }
        n_s23h += 1;
        let g = mul(w, fu);
        let zb = z.to_bits();
        println!(
            "  z={:.16} bits={:#x} 555={} d={} tmu={} store_k={:?} mul_k={:?} store_k1={} w+1={} F+1={} 210={}",
            z,
            zb,
            is555(zb),
            ulp_distance(g, t).unwrap_or(99),
            hit(mul(w.next_up(), fu.next_up()), t),
            k_of(t, w, fu),
            mul_k_of(g, t),
            signed(mul(w, poke(fu, 1)), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            signed(mul(w, cody(z, &C0, &D0, 0x210)), t)
        );
    }
    println!("DIRECT mid last-store stall k=+2/+3=1H n={n_s23h}");
    println!("DIRECT mid double last-store k=0/+1:");
    let mut n_d01 = 0usize;
    let mut n_d01_fuse = 0usize;
    let mut n_d01_1l = 0usize;
    let mut n_d01_fp = 0usize;
    let mut n_d01_wp = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, 1)), t).unwrap_or(99) != 0 {
            continue;
        }
        n_d01 += 1;
        n_d01_fuse += 1;
        let fp = hit(mul(w, fu.next_up()), t);
        let wp = hit(mul(w.next_up(), fu), t);
        if fp {
            n_d01_fp += 1;
        }
        if wp {
            n_d01_wp += 1;
        }
        let zb = z.to_bits();
        println!(
            "  z={:.16} bits={:#x} 555={} mul_k={:?} store_k2={} mul_k1={} mul_k2={} w+1={wp} F+1={fp} tmu={} 210={}",
            z,
            zb,
            is555(zb),
            mul_k_of(g, t),
            signed(mul(w, poke(fu, 2)), t),
            signed(poke(g, 1), t),
            signed(poke(g, 2), t),
            hit(mul(w.next_up(), fu.next_up()), t),
            signed(mul(w, cody(z, &C0, &D0, 0x210)), t)
        );
        let _ = n_d01_1l;
    }
    println!("DIRECT mid double last-store k=0/+1 fused n={n_d01} w+1={n_d01_wp} F+1={n_d01_fp}");
    println!("DIRECT mid fused double last-store k=0/-1:");
    let mut n_d0m1 = 0usize;
    let mut n_d0m1_wm = 0usize;
    let mut n_d0m1_tmd = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, -1)), t).unwrap_or(99) != 0 {
            continue;
        }
        n_d0m1 += 1;
        let wm = hit(mul(w.next_down(), fu), t);
        let tmd = hit(mul(w.next_down(), fu.next_down()), t);
        if wm {
            n_d0m1_wm += 1;
        }
        if tmd {
            n_d0m1_tmd += 1;
        }
        let zb = z.to_bits();
        println!(
            "  z={:.16} bits={:#x} 555={} mul_k-1={} store_k-2={} w-1={wm} F-1={} tmd={tmd} 210={}",
            z,
            zb,
            is555(zb),
            signed(poke(g, -1), t),
            signed(mul(w, poke(fu, -2)), t),
            hit(mul(w, fu.next_down()), t),
            signed(mul(w, cody(z, &C0, &D0, 0x210)), t)
        );
    }
    println!("DIRECT mid fused double last-store k=0/-1 n={n_d0m1} w-1={n_d0m1_wm} tmd={n_d0m1_tmd}");
    println!("0x210 double last-store k=0/+1 and k=+1/+2:");
    let mut n_210_01 = 0usize;
    let mut n_210_12 = 0usize;
    let mut n_210_12_tmu = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let f210 = cody(z, &C0, &D0, 0x210);
        let g = mul(w, f210);
        let st0 = ulp_distance(g, t).unwrap_or(99) == 0;
        let st1 = ulp_distance(mul(w, poke(f210, 1)), t).unwrap_or(99) == 0;
        let st2 = ulp_distance(mul(w, poke(f210, 2)), t).unwrap_or(99) == 0;
        if st0 && st1 {
            n_210_01 += 1;
            println!(
                "  210 k=0/+1 z={:.16} d={} tmu={} unmask={} store_k2={} mul_k1={} w+1={}",
                z,
                ulp_distance(g, t).unwrap_or(99),
                hit(mul(w.next_up(), f210.next_up()), t),
                signed(mul(w, cody(z, &C0, &D0, 0)), t),
                signed(mul(w, poke(f210, 2)), t),
                signed(poke(g, 1), t),
                hit(mul(w.next_up(), f210), t)
            );
        }
        if st1 && st2 && !st0 {
            n_210_12 += 1;
            let tmu = hit(mul(w.next_up(), f210.next_up()), t);
            if tmu {
                n_210_12_tmu += 1;
            }
            println!(
                "  210 k=+1/+2 z={:.16} d={} tmu={tmu} unmask={} mul_k={:?} w+1={} F+1={}",
                z,
                ulp_distance(g, t).unwrap_or(99),
                signed(mul(w, cody(z, &C0, &D0, 0)), t),
                {
                    let mut km = None;
                    for ak in 0i32..=8 {
                        if ulp_distance(poke(g, ak), t).unwrap_or(99) == 0 {
                            km = Some(ak);
                            break;
                        }
                    }
                    km
                },
                hit(mul(w.next_up(), f210), t),
                hit(mul(w, f210.next_up()), t)
            );
        }
    }
    println!("0x210 double k=0/+1 n={n_210_01} double k=+1/+2 not-fused n={n_210_12} tmu={n_210_12_tmu}");
    println!("fused double k=0/+1 last-store/last-mul vs 0.8125 cluster:");
    let cluster = [
        0.75,
        0.7708333333333334,
        0.78125,
        0.8020833333333334,
        0.8125,
        0.8437499999999999,
        0.84375,
        0.875,
        0.9270833333333334,
        0.96875,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !cluster.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let zb = z.to_bits();
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        println!(
            "  z={:.16} bits={:#x} 555={} d={} store_k={:?} mul_k={:?} 210={} 74={} 114={}",
            z,
            zb,
            is555(zb),
            d,
            k_of(t, w, fu),
            mul_k_of(g, t),
            signed(mul(w, f210), t),
            signed(mul(w, cody(z, &C0, &D0, 0x74)), t),
            signed(mul(w, cody(z, &c114, &D0, 0x210)), t)
        );
        print!("    unmask store");
        for k in -2i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        for k in -2i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmu={} w+1={} F+1={}",
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
        print!("    210 store");
        for k in -2i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(f210, k)), t));
        }
        print!(" mul");
        let g210 = mul(w, f210);
        for k in -2i32..=3 {
            print!(" {k}:{}", signed(poke(g210, k), t));
        }
        println!(
            " tmu={}",
            hit(mul(w.next_up(), f210.next_up()), t)
        );
    }
    let class_of = |t: f64, w: f64, ff: f64| -> String {
        let g = mul(w, ff);
        let d = ulp_distance(g, t).unwrap_or(99);
        let stm1 = ulp_distance(mul(w, poke(ff, -1)), t).unwrap_or(99) == 0;
        let st0 = d == 0;
        let st1 = ulp_distance(mul(w, poke(ff, 1)), t).unwrap_or(99) == 0;
        let st2 = ulp_distance(mul(w, poke(ff, 2)), t).unwrap_or(99) == 0;
        let tmu = hit(mul(w.next_up(), ff.next_up()), t);
        if st0 && st1 && stm1 {
            "fuse01m1".into()
        } else if st0 && st1 {
            "fuse01".into()
        } else if st0 && stm1 {
            "fuse0m1".into()
        } else if st0 {
            "fuse".into()
        } else if d == 1 && g < t && st1 && st2 && tmu {
            "1L-tmu-12".into()
        } else if d == 1 && g < t && st1 && st2 {
            "1L-12".into()
        } else if d == 1 && g < t && st1 {
            "1L-k1".into()
        } else if d == 1 && g > t && stm1 {
            "1H-km1".into()
        } else {
            format!("{}{}", d, if g < t { "L" } else if g > t { "H" } else { "=" })
        }
    };
    println!("unmask fused double k=0/+1 → 210/74/114 class:");
    let mut n_210_tmu12 = 0usize;
    let mut n_210_0m1 = 0usize;
    let mut n_210_01 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, 1)), t).unwrap_or(99) != 0 {
            continue;
        }
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        let c210 = class_of(t, w, f210);
        let c74 = class_of(t, w, f74);
        let c114 = class_of(t, w, f114);
        if c210 == "1L-tmu-12" {
            n_210_tmu12 += 1;
        }
        if c210 == "fuse0m1" {
            n_210_0m1 += 1;
        }
        if c210 == "fuse01" || c210 == "fuse01m1" {
            n_210_01 += 1;
        }
        println!(
            "  z={:.16} 210={c210} 74={c74} 114={c114} w+1={} F+1={}",
            z,
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
    }
    println!(
        "unmask fuse01 n=11 → 210 fuse01={n_210_01} fuse0m1={n_210_0m1} 1L-tmu-12={n_210_tmu12}"
    );
    println!("unmask fused double k=0/-1 → 210/74/114 class:");
    let mut n_m_210_01 = 0usize;
    let mut n_m_210_0m1 = 0usize;
    let mut n_m_210_tmd = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, -1)), t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, 1)), t).unwrap_or(99) == 0 {
            continue;
        }
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        let c210 = class_of(t, w, f210);
        if c210 == "fuse01" || c210 == "fuse01m1" {
            n_m_210_01 += 1;
        }
        if c210 == "fuse0m1" {
            n_m_210_0m1 += 1;
        }
        if hit(mul(w.next_down(), f210.next_down()), t) {
            n_m_210_tmd += 1;
        }
        println!(
            "  z={:.16} 210={c210} 74={} 114={} w-1={} F-1={}",
            z,
            class_of(t, w, f74),
            class_of(t, w, f114),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!(
        "unmask fuse0m1 n=8 → 210 fuse0m1={n_m_210_0m1} fuse01={n_m_210_01} 210-tmd={n_m_210_tmd}"
    );
    println!("0x210 fused double k=0/-1:");
    let mut n_210_fuse0m1 = 0usize;
    let mut n_210_fuse0m1_un01 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let f210 = cody(z, &C0, &D0, 0x210);
        let g = mul(w, f210);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(f210, -1)), t).unwrap_or(99) != 0 {
            continue;
        }
        n_210_fuse0m1 += 1;
        let fu = cody(z, &C0, &D0, 0);
        let un01 = ulp_distance(mul(w, fu), t).unwrap_or(99) == 0
            && ulp_distance(mul(w, poke(fu, 1)), t).unwrap_or(99) == 0;
        if un01 {
            n_210_fuse0m1_un01 += 1;
        }
        println!(
            "  z={:.16} unmask={} 74={} 114={} unmask_fuse01={un01} mul_k-1={} tmd={}",
            z,
            class_of(t, w, fu),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            class_of(t, w, cody(z, &c114, &D0, 0x210)),
            signed(poke(g, -1), t),
            hit(mul(w.next_down(), f210.next_down()), t)
        );
    }
    println!("0x210 fuse0m1 n={n_210_fuse0m1} from unmask fuse01={n_210_fuse0m1_un01}");
    println!("0x210 1L-tmu-12:");
    let mut n_210_1ltmu = 0usize;
    let mut n_210_1ltmu_un01 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let f210 = cody(z, &C0, &D0, 0x210);
        let g = mul(w, f210);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g < t) {
            continue;
        }
        if ulp_distance(mul(w, poke(f210, 1)), t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(f210, 2)), t).unwrap_or(99) != 0 {
            continue;
        }
        if !hit(mul(w.next_up(), f210.next_up()), t) {
            continue;
        }
        n_210_1ltmu += 1;
        let fu = cody(z, &C0, &D0, 0);
        let un01 = ulp_distance(mul(w, fu), t).unwrap_or(99) == 0
            && ulp_distance(mul(w, poke(fu, 1)), t).unwrap_or(99) == 0;
        if un01 {
            n_210_1ltmu_un01 += 1;
        }
        println!(
            "  z={:.16} unmask={} 74={} 114={} unmask_fuse01={un01} mul_k={:?} w+1={}",
            z,
            class_of(t, w, fu),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            class_of(t, w, cody(z, &c114, &D0, 0x210)),
            {
                let mut km = None;
                for ak in 0i32..=8 {
                    if ulp_distance(poke(g, ak), t).unwrap_or(99) == 0 {
                        km = Some(ak);
                        break;
                    }
                }
                km
            },
            hit(mul(w.next_up(), f210), t)
        );
    }
    println!("0x210 1L-tmu-12 n={n_210_1ltmu} from unmask fuse01={n_210_1ltmu_un01}");
    println!("114 fuse0m1:");
    let mut n_114_0m1 = 0usize;
    let mut n_114_0m1_un01 = 0usize;
    let mut n_114_0m1_un0m1 = 0usize;
    let mut n_114_0m1_stall23 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let f114 = cody(z, &c114, &D0, 0x210);
        let g = mul(w, f114);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(f114, -1)), t).unwrap_or(99) != 0 {
            continue;
        }
        n_114_0m1 += 1;
        let fu = cody(z, &C0, &D0, 0);
        let un01 = ulp_distance(mul(w, fu), t).unwrap_or(99) == 0
            && ulp_distance(mul(w, poke(fu, 1)), t).unwrap_or(99) == 0;
        let un0m1 = ulp_distance(mul(w, fu), t).unwrap_or(99) == 0
            && ulp_distance(mul(w, poke(fu, -1)), t).unwrap_or(99) == 0
            && !un01;
        if un01 {
            n_114_0m1_un01 += 1;
        }
        if un0m1 {
            n_114_0m1_un0m1 += 1;
        }
        let stall23 = signed(mul(w, poke(fu, 2)), t) == "1H"
            && signed(mul(w, poke(fu, 3)), t) == "1H";
        if stall23 {
            n_114_0m1_stall23 += 1;
        }
        println!(
            "  z={:.16} unmask={} 210={} 74={} un01={un01} stall23={stall23} mul_k-1={} store_k+1={} store_k+2={} store_k+3={}",
            z,
            class_of(t, w, fu),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            signed(poke(g, -1), t),
            signed(mul(w, poke(f114, 1)), t),
            signed(mul(w, poke(f114, 2)), t),
            signed(mul(w, poke(f114, 3)), t)
        );
    }
    println!(
        "114 fuse0m1 n={n_114_0m1} from unmask fuse01={n_114_0m1_un01} from unmask fuse0m1={n_114_0m1_un0m1} stall23={n_114_0m1_stall23}"
    );
    println!("114 fuse01:");
    let mut n_114_01 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let f114 = cody(z, &c114, &D0, 0x210);
        let g = mul(w, f114);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(f114, 1)), t).unwrap_or(99) != 0 {
            continue;
        }
        n_114_01 += 1;
        let fu = cody(z, &C0, &D0, 0);
        println!(
            "  z={:.16} unmask={} 210={} 74={} mul_k1={} w+1={}",
            z,
            class_of(t, w, fu),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            signed(poke(g, 1), t),
            hit(mul(w.next_up(), f114), t)
        );
    }
    println!("114 fuse01 n={n_114_01}");
    println!("pin 114 vs unmask/210 stall cluster + 2.28125 + 0.8125 + 0.875:");
    let pinz = [
        0.7708333333333334,
        0.78125,
        0.8020833333333334,
        0.8125,
        0.875,
        2.28125,
        0.9895833333333334,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} unmask={} 210={} 114={}",
            z,
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f114)
        );
        print!("    114 store");
        for k in -2i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        print!(" mul");
        let g114 = mul(w, f114);
        for k in -2i32..=3 {
            print!(" {k}:{}", signed(poke(g114, k), t));
        }
        println!(
            " tmu={} tmd={} w-1={} F-1={}",
            hit(mul(w.next_up(), f114.next_up()), t),
            hit(mul(w.next_down(), f114.next_down()), t),
            hit(mul(w.next_down(), f114), t),
            hit(mul(w, f114.next_down()), t)
        );
        print!("    unm store");
        for k in -2i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" 210 store");
        for k in -2i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(f210, k)), t));
        }
        println!();
    }
    println!("114 1H-km1:");
    let mut n_114_1h = 0usize;
    let mut n_114_1h_un0m1 = 0usize;
    let mut n_114_1h_dbl = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let f114 = cody(z, &c114, &D0, 0x210);
        let g = mul(w, f114);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if ulp_distance(mul(w, poke(f114, -1)), t).unwrap_or(99) != 0 {
            continue;
        }
        n_114_1h += 1;
        let fu = cody(z, &C0, &D0, 0);
        let un0m1 = ulp_distance(mul(w, fu), t).unwrap_or(99) == 0
            && ulp_distance(mul(w, poke(fu, -1)), t).unwrap_or(99) == 0;
        if un0m1 {
            n_114_1h_un0m1 += 1;
        }
        let dbl = ulp_distance(mul(w, poke(f114, -2)), t).unwrap_or(99) == 0;
        if dbl {
            n_114_1h_dbl += 1;
        }
        println!(
            "  z={:.16} unmask={} 210={} 74={} un0m1={un0m1} store_k-2={} mul_k={:?} tmd={}",
            z,
            class_of(t, w, fu),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            signed(mul(w, poke(f114, -2)), t),
            {
                let mut km = None;
                for ak in 0i32..=8 {
                    if ulp_distance(poke(g, -ak), t).unwrap_or(99) == 0 {
                        km = Some(-ak);
                        break;
                    }
                    if ak > 0 && ulp_distance(poke(g, ak), t).unwrap_or(99) == 0 {
                        km = Some(ak);
                        break;
                    }
                }
                km
            },
            hit(mul(w.next_down(), f114.next_down()), t)
        );
    }
    println!("114 1H-km1 n={n_114_1h} from unmask fuse0m1={n_114_1h_un0m1} double k=-1/-2={n_114_1h_dbl}");
    println!("114 conversion inverse pair fuse0m1 ↔ 1H-km1:");
    let mut n_un_0m1_to_1h = 0usize;
    let mut n_un_1h_to_0m1 = 0usize;
    let mut n_un_1h_tmd_to_0m1 = 0usize;
    let mut n_un_fuse_to_1h = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let cu = class_of(t, w, fu);
        let c114 = class_of(t, w, f114);
        let tmd = hit(mul(w.next_down(), fu.next_down()), t);
        let tag = if cu == "fuse0m1" && c114 == "1H-km1" {
            n_un_0m1_to_1h += 1;
            "fuse0m1→1H"
        } else if (cu == "1H-km1" || cu.starts_with("1H")) && c114 == "fuse0m1" {
            n_un_1h_to_0m1 += 1;
            if tmd {
                n_un_1h_tmd_to_0m1 += 1;
            }
            "1H→fuse0m1"
        } else if cu == "fuse" && c114 == "1H-km1" {
            n_un_fuse_to_1h += 1;
            "fuse→1H"
        } else {
            continue;
        };
        let zb = z.to_bits();
        let hx = format!("{zb:x}");
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        println!(
            "  {tag} z={:.16} 555={is555} tmd={tmd} 210={} 74={} 114-store-2..1 {} {} {} {}",
            z,
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            signed(mul(w, poke(f114, -2)), t),
            signed(mul(w, poke(f114, -1)), t),
            signed(mul(w, poke(f114, 0)), t),
            signed(mul(w, poke(f114, 1)), t)
        );
    }
    println!(
        "114 inverse fuse0m1→1H n={n_un_0m1_to_1h} 1H→fuse0m1 n={n_un_1h_to_0m1} of which tmd={n_un_1h_tmd_to_0m1} fuse→1H n={n_un_fuse_to_1h}"
    );
    println!("pin 0.989583 vs 3.15625 114 last-store:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 0.9895833333333333).abs() > 1e-12 && (r.z - 3.15625).abs() > 1e-14 {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f114 = cody(z, &c114, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        println!(
            "  z={:.16} bits={:#x} unmask={} 210={} 74={} 114={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f74),
            class_of(t, w, f114)
        );
        for (name, ff) in [("unm", fu), ("210", f210), ("74", f74), ("114", f114)] {
            print!("    {name} store");
            for k in -3i32..=2 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let g = mul(w, ff);
            for k in -3i32..=2 {
                print!(" {k}:{}", signed(poke(g, k), t));
            }
            println!(
                " tmd={} tmu={}",
                hit(mul(w.next_down(), ff.next_down()), t),
                hit(mul(w.next_up(), ff.next_up()), t)
            );
        }
    }
    println!("1H tmd → 210/74/114 class:");
    let mut n_tmd = 0usize;
    let mut n_210_0m1 = 0usize;
    let mut n_74_0m1 = 0usize;
    let mut n_114_0m1 = 0usize;
    let mut n_all_0m1 = 0usize;
    let mut n_114_only = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if !hit(mul(w.next_down(), fu.next_down()), t) {
            continue;
        }
        n_tmd += 1;
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        let c210 = class_of(t, w, f210);
        let c74 = class_of(t, w, f74);
        let c114 = class_of(t, w, f114);
        if c210 == "fuse0m1" {
            n_210_0m1 += 1;
        }
        if c74 == "fuse0m1" {
            n_74_0m1 += 1;
        }
        if c114 == "fuse0m1" {
            n_114_0m1 += 1;
        }
        if c210 == "fuse0m1" && c74 == "fuse0m1" && c114 == "fuse0m1" {
            n_all_0m1 += 1;
        }
        if c114 == "fuse0m1" && c210 != "fuse0m1" && c74 != "fuse0m1" {
            n_114_only += 1;
        }
        println!(
            "  z={:.16} unmask={} 210={c210} 74={c74} 114={c114} dbl_k-2={}",
            z,
            class_of(t, w, fu),
            signed(mul(w, poke(fu, -2)), t)
        );
    }
    println!(
        "1H tmd n={n_tmd} → fuse0m1 210={n_210_0m1} 74={n_74_0m1} 114={n_114_0m1} all3={n_all_0m1} 114-only={n_114_only}"
    );
    println!("pin 2.250 vs 3.15625 last-store all graphs:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 2.25).abs() > 1e-14 && (r.z - 3.15625).abs() > 1e-14 {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!("  z={:.16} bits={:#x}", z, z.to_bits());
        for (name, ff) in [("unm", fu), ("210", f210), ("74", f74), ("114", f114)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in -3i32..=2 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in -3i32..=2 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " tmd={} w-1={} F-1={}",
                hit(mul(w.next_down(), ff.next_down()), t),
                hit(mul(w.next_down(), ff), t),
                hit(mul(w, ff.next_down()), t)
            );
        }
    }
    println!("1H tmd split last-store k=-2 and w-1:");
    let mut n_k2eq = 0usize;
    let mut n_k2_1l = 0usize;
    let mut n_wm = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if !hit(mul(w.next_down(), fu.next_down()), t) {
            continue;
        }
        let k2 = signed(mul(w, poke(fu, -2)), t);
        if k2 == "0=" {
            n_k2eq += 1;
        } else if k2 == "1L" {
            n_k2_1l += 1;
        }
        let wm = hit(mul(w.next_down(), fu), t);
        if wm {
            n_wm += 1;
        }
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} k-2={k2} w-1={wm} F-1={} 210={} 74={} 114={} 210-k-1={} 210-k0={}",
            z,
            hit(mul(w, fu.next_down()), t),
            class_of(t, w, f210),
            class_of(t, w, f74),
            class_of(t, w, f114),
            signed(mul(w, poke(f210, -1)), t),
            signed(mul(w, poke(f210, 0)), t)
        );
    }
    println!("1H tmd k-2 exact={n_k2eq} k-2 1L={n_k2_1l} w-1={n_wm}");
    println!("pin 3.4375/3.645833 last-store all graphs:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 3.4375).abs() > 1e-14 && (r.z - 3.6458333333333335).abs() > 1e-12 {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!("  z={:.16} bits={:#x}", z, z.to_bits());
        for (name, ff) in [("unm", fu), ("210", f210), ("74", f74), ("114", f114)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in -3i32..=2 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in -3i32..=2 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " tmd={} w-1={} F-1={}",
                hit(mul(w.next_down(), ff.next_down()), t),
                hit(mul(w.next_down(), ff), t),
                hit(mul(w, ff.next_down()), t)
            );
        }
    }
    println!("1L tmu last-store k=+2 not exact analog:");
    let mut n_1l_tmu = 0usize;
    let mut n_1l_tmu_k2eq = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g < t) {
            continue;
        }
        if !hit(mul(w.next_up(), fu.next_up()), t) {
            continue;
        }
        n_1l_tmu += 1;
        let k2eq = ulp_distance(mul(w, poke(fu, 2)), t).unwrap_or(99) == 0;
        if k2eq {
            n_1l_tmu_k2eq += 1;
        }
        println!(
            "  z={:.16} k+2={} w+1={} 210={} 74={} 114={}",
            z,
            signed(mul(w, poke(fu, 2)), t),
            hit(mul(w.next_up(), fu), t),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            class_of(t, w, cody(z, &c114, &D0, 0x210))
        );
    }
    println!("1L tmu n={n_1l_tmu} last-store k=+2 exact={n_1l_tmu_k2eq}");
    println!("114 leftover-high from unmask class:");
    let mut n_114_hi = 0usize;
    let mut n_114_hi_from_1h_tmd = 0usize;
    let mut n_114_hi_from_1h = 0usize;
    let mut n_114_hi_from_hi = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let f114 = cody(z, &c114, &D0, 0x210);
        let g114 = mul(w, f114);
        let d114 = ulp_distance(g114, t).unwrap_or(99);
        if !(d114 >= 2 && g114 > t) {
            continue;
        }
        n_114_hi += 1;
        let fu = cody(z, &C0, &D0, 0);
        let gu = mul(w, fu);
        let du = ulp_distance(gu, t).unwrap_or(99);
        let tmd_u = hit(mul(w.next_down(), fu.next_down()), t);
        let from = if du == 1 && gu > t && tmd_u {
            n_114_hi_from_1h_tmd += 1;
            "1H-tmd"
        } else if du == 1 && gu > t {
            n_114_hi_from_1h += 1;
            "1H"
        } else if du >= 2 && gu > t {
            n_114_hi_from_hi += 1;
            "HI"
        } else {
            "other"
        };
        println!(
            "  z={:.16} d114={} from={from} unmask={} 210={} 74={} tmd114={} store_k={:?} k-2={} k-3={}",
            z,
            d114,
            class_of(t, w, fu),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            hit(mul(w.next_down(), f114.next_down()), t),
            k_of(t, w, f114),
            signed(mul(w, poke(f114, -2)), t),
            signed(mul(w, poke(f114, -3)), t)
        );
    }
    println!(
        "114 leftover-high n={n_114_hi} from 1H-tmd={n_114_hi_from_1h_tmd} from 1H={n_114_hi_from_1h} from HI={n_114_hi_from_hi}"
    );
    println!("pin 0.84375-1ulp vs 3.15625 114 last-store:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let z = r.z;
        if (z - 0.8437499999999999).abs() > 1e-15 && (z - 3.15625).abs() > 1e-14 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!("  z={:.16} bits={:#x}", z, z.to_bits());
        for (name, ff) in [("unm", fu), ("210", f210), ("74", f74), ("114", f114)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in -3i32..=2 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in -3i32..=2 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " tmd={} w-1={} F-1={}",
                hit(mul(w.next_down(), ff.next_down()), t),
                hit(mul(w.next_down(), ff), t),
                hit(mul(w, ff.next_down()), t)
            );
        }
    }
    println!("1H tmd w-1 false 114 vs unmask last-store:");
    let mut n_wm_false = 0usize;
    let mut n_114_stay = 0usize;
    let mut n_114_2h = 0usize;
    let mut n_114_fuse = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if !hit(mul(w.next_down(), fu.next_down()), t) {
            continue;
        }
        if hit(mul(w.next_down(), fu), t) {
            continue;
        }
        n_wm_false += 1;
        let f114 = cody(z, &c114, &D0, 0x210);
        let c114 = class_of(t, w, f114);
        if c114 == "1H-km1" || c114 == "1H" {
            n_114_stay += 1;
        } else if c114.contains("2H") || {
            let g114 = mul(w, f114);
            ulp_distance(g114, t).unwrap_or(99) >= 2 && g114 > t
        } {
            n_114_2h += 1;
        } else if c114.starts_with("fuse") {
            n_114_fuse += 1;
        }
        println!(
            "  z={:.16} unmask={} 114={} 210={} k-2={} F-1={}",
            z,
            class_of(t, w, fu),
            c114,
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            signed(mul(w, poke(fu, -2)), t),
            hit(mul(w, fu.next_down()), t)
        );
        print!("    unm store");
        for k in -3i32..=1 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" 114 store");
        for k in -3i32..=1 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        print!(" 114 mul");
        let g114 = mul(w, f114);
        for k in -3i32..=1 {
            print!(" {k}:{}", signed(poke(g114, k), t));
        }
        println!(
            " tmd114={} F-1_114={}",
            hit(mul(w.next_down(), f114.next_down()), t),
            hit(mul(w, f114.next_down()), t)
        );
    }
    println!("1H tmd w-1 false n={n_wm_false} 114 stay={n_114_stay} 114 2H={n_114_2h} 114 fuse={n_114_fuse}");
    println!("pin 1.708333 vs 0.84375-1ulp vs 1.46875:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let z = r.z;
        if (z - 1.7083333333333333).abs() > 1e-12
            && (z - 0.8437499999999999).abs() > 1e-15
            && (z - 1.46875).abs() > 1e-14
        {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let f210 = cody(z, &C0, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} 555={}",
            z,
            z.to_bits(),
            {
                let h = format!("{:x}", z.to_bits());
                h.contains("55555555") || h.contains("aaaaaaaa")
            }
        );
        for (name, ff) in [("unm", fu), ("210", f210), ("114", f114)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in -4i32..=2 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in -4i32..=2 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " tmd={} w-1={} F-1={} F-2={}",
                hit(mul(w.next_down(), ff.next_down()), t),
                hit(mul(w.next_down(), ff), t),
                hit(mul(w, ff.next_down()), t),
                hit(mul(w, poke(ff, -2)), t)
            );
        }
    }
    println!("1H tmd store_k vs mul_k:");
    let mut n_tmd = 0usize;
    let mut n_km1 = 0usize;
    let mut n_km2 = 0usize;
    let mut n_mul_first = 0usize;
    let mut n_f1 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if !hit(mul(w.next_down(), fu.next_down()), t) {
            continue;
        }
        n_tmd += 1;
        let ks = k_of(t, w, fu);
        let km = mul_k_of(g, t);
        if ks == Some(-1) {
            n_km1 += 1;
        }
        if ks == Some(-2) {
            n_km2 += 1;
        }
        let mul_first = match (ks, km) {
            (Some(s), Some(m)) if m.abs() < s.abs() => true,
            _ => false,
        };
        if mul_first {
            n_mul_first += 1;
        }
        let f1 = hit(mul(w, fu.next_down()), t);
        if f1 {
            n_f1 += 1;
        }
        println!(
            "  z={:.16} store_k={ks:?} mul_k={km:?} mul_first={mul_first} F-1={f1} w-1={} stall-1/0={} k-2={}",
            z,
            hit(mul(w.next_down(), fu), t),
            signed(mul(w, poke(fu, -1)), t) == "1H" && signed(g, t) == "1H",
            signed(mul(w, poke(fu, -2)), t)
        );
    }
    println!(
        "1H tmd n={n_tmd} store k=-1={n_km1} store k=-2={n_km2} mul_first={n_mul_first} F-1={n_f1}"
    );
    println!("1L tmu store_k vs mul_k analog:");
    let mut n_tmu = 0usize;
    let mut n_kp1 = 0usize;
    let mut n_kp2 = 0usize;
    let mut n_mf = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g < t) {
            continue;
        }
        if !hit(mul(w.next_up(), fu.next_up()), t) {
            continue;
        }
        n_tmu += 1;
        let ks = k_of(t, w, fu);
        let km = mul_k_of(g, t);
        if ks == Some(1) {
            n_kp1 += 1;
        }
        if ks == Some(2) {
            n_kp2 += 1;
        }
        let mul_first = match (ks, km) {
            (Some(s), Some(m)) if m.abs() < s.abs() => true,
            _ => false,
        };
        if mul_first {
            n_mf += 1;
        }
        println!(
            "  z={:.16} store_k={ks:?} mul_k={km:?} mul_first={mul_first} F+1={} w+1={} k+2={}",
            z,
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_up(), fu), t),
            signed(mul(w, poke(fu, 2)), t)
        );
    }
    println!("1L tmu n={n_tmu} store k=+1={n_kp1} store k=+2={n_kp2} mul_first={n_mf}");
    println!("pin 1.46875 neighbors:");
    let steps = [
        (1.0 / 16.0, "1/16"),
        (1.0 / 32.0, "1/32"),
        (1.0 / 48.0, "1/48"),
        (1.0 / 96.0, "1/96"),
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 1.46875).abs() > 1e-14 {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} unmask={} 210={} 74={} 114={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            class_of(t, w, f114)
        );
        for (step, name) in steps {
            for (sgn, lab) in [(1.0, "+"), (-1.0, "-")] {
                let z2 = z + sgn * step;
                if let Some(rr) = rows.iter().find(|x| x.direct && (x.z - z2).abs() < 1e-12) {
                    let t2 = f64::from_bits(rr.qbits);
                    let w2 = f::w_rn53(rr.z);
                    let fu2 = cody(rr.z, &C0, &D0, 0);
                    println!(
                        "    {name}{lab} z={:.16} unmask={} tmd={}",
                        rr.z,
                        class_of(t2, w2, fu2),
                        hit(mul(w2.next_down(), fu2.next_down()), t2)
                    );
                } else {
                    println!("    {name}{lab} z={z2:.16} not-DIRECT");
                }
            }
        }
    }
    println!("fuse0m1 ±1/32 neighbors:");
    let mut n_0m1 = 0usize;
    let mut n_0m1_1h_tmd_m32 = 0usize;
    let mut n_0m1_1h_any = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, -1)), t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, 1)), t).unwrap_or(99) == 0 {
            continue;
        }
        n_0m1 += 1;
        print!(
            "  z={:.16} mul_k-1={} w-1={} tmd={}",
            z,
            signed(poke(g, -1), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w.next_down(), fu.next_down()), t)
        );
        for (step, name) in [(1.0 / 32.0, "1/32"), (1.0 / 16.0, "1/16")] {
            for (sgn, lab) in [(1.0, "+"), (-1.0, "-")] {
                let z2 = z + sgn * step;
                if let Some(rr) = rows.iter().find(|x| x.direct && (x.z - z2).abs() < 1e-12) {
                    let t2 = f64::from_bits(rr.qbits);
                    let w2 = f::w_rn53(rr.z);
                    let fu2 = cody(rr.z, &C0, &D0, 0);
                    let g2 = mul(w2, fu2);
                    let d2 = ulp_distance(g2, t2).unwrap_or(99);
                    let tmd2 = hit(mul(w2.next_down(), fu2.next_down()), t2);
                    let tag = class_of(t2, w2, fu2);
                    print!(" {name}{lab}={tag}");
                    if name == "1/32" && lab == "-" && d2 == 1 && g2 > t2 && tmd2 {
                        n_0m1_1h_tmd_m32 += 1;
                    }
                    if name == "1/32" && d2 == 1 && g2 > t2 {
                        n_0m1_1h_any += 1;
                    }
                }
            }
        }
        println!();
    }
    println!("fuse0m1 n={n_0m1} 1/32- is 1H tmd={n_0m1_1h_tmd_m32} 1/32 ± is 1H={n_0m1_1h_any}");
    println!("pin 1.5 vs 1.46875 last-store:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 1.5).abs() > 1e-14 && (r.z - 1.46875).abs() > 1e-14 {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!("  z={:.16} bits={:#x}", z, z.to_bits());
        for (name, ff) in [("unm", fu), ("210", f210), ("74", f74), ("114", f114)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in -3i32..=2 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in -3i32..=2 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " tmd={} w-1={} F-1={}",
                hit(mul(w.next_down(), ff.next_down()), t),
                hit(mul(w.next_down(), ff), t),
                hit(mul(w, ff.next_down()), t)
            );
        }
    }
    println!("pin 1.53125 vs 1.5 vs 1.46875 last-store:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 1.53125).abs() > 1e-14 && (r.z - 1.5).abs() > 1e-14 && (r.z - 1.46875).abs() > 1e-14 {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!("  z={:.16} bits={:#x}", z, z.to_bits());
        for (name, ff) in [("unm", fu), ("210", f210), ("74", f74), ("114", f114)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in -2i32..=3 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in -2i32..=3 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " tmu={} tmd={} w+1={} F+1={} w-1={} F-1={}",
                hit(mul(w.next_up(), ff.next_up()), t),
                hit(mul(w.next_down(), ff.next_down()), t),
                hit(mul(w.next_up(), ff), t),
                hit(mul(w, ff.next_up()), t),
                hit(mul(w.next_down(), ff), t),
                hit(mul(w, ff.next_down()), t)
            );
        }
    }
    println!("1L-k1 last-store stall k=0/+1=1L and mul_first:");
    let mut n_1lk1 = 0usize;
    let mut n_stall01 = 0usize;
    let mut n_mf = 0usize;
    let mut n_tmu = 0usize;
    let mut n_wp = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g < t) {
            continue;
        }
        if k_of(t, w, fu) != Some(1) {
            continue;
        }
        n_1lk1 += 1;
        let stall = signed(g, t) == "1L" && signed(mul(w, poke(fu, 1)), t) == "1L";
        if stall {
            n_stall01 += 1;
        }
        let km = mul_k_of(g, t);
        let mul_first = match km {
            Some(m) if m.abs() < 1 => true,
            _ => false,
        };
        if mul_first {
            n_mf += 1;
        }
        let tmu = hit(mul(w.next_up(), fu.next_up()), t);
        if tmu {
            n_tmu += 1;
        }
        let wp = hit(mul(w.next_up(), fu), t);
        if wp {
            n_wp += 1;
        }
        if stall || mul_first || (z - 1.53125).abs() < 1e-14 {
            println!(
                "  z={:.16} stall01={stall} mul_k={km:?} tmu={tmu} w+1={wp} F+1={} k+2={} 210={}",
                z,
                hit(mul(w, fu.next_up()), t),
                signed(mul(w, poke(fu, 2)), t),
                class_of(t, w, cody(z, &C0, &D0, 0x210))
            );
        }
    }
    println!("1L-k1 n={n_1lk1} stall k=0/+1=1L={n_stall01} mul_first={n_mf} tmu={n_tmu} w+1={n_wp}");
    println!("fuse0m1 1/32+ is 1L-k1:");
    let mut n_p32 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, -1)), t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, 1)), t).unwrap_or(99) == 0 {
            continue;
        }
        let z2 = z + 1.0 / 32.0;
        if let Some(rr) = rows.iter().find(|x| x.direct && (x.z - z2).abs() < 1e-12) {
            let t2 = f64::from_bits(rr.qbits);
            let w2 = f::w_rn53(rr.z);
            let fu2 = cody(rr.z, &C0, &D0, 0);
            let tag = class_of(t2, w2, fu2);
            if tag == "1L-k1" {
                n_p32 += 1;
                println!("  fuse0m1 z={:.16} +1/32 z={:.16} 1L-k1", z, rr.z);
            }
        }
    }
    println!("fuse0m1 1/32+ is 1L-k1 n={n_p32}");
    println!("pin 2.40625/2.4375/2.46875 vs 1.46875/1.5/1.53125:");
    let pinz = [2.40625, 2.4375, 2.46875, 2.4583333333333335, 1.46875, 1.5, 1.53125];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} unmask={} 210={} 74={} 114={} lo={} tmu_lo={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            class_of(t, w, f114),
            lo_u.contains(&z.to_bits()),
            tmu_u.contains(&z.to_bits())
        );
        print!("    unm store");
        for k in -2i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -2i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmu={} tmd={} w+1={} F+1={} w-1={} F-1={}",
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("114 leftover-low from unmask class:");
    let mut n_114_lo = 0usize;
    let mut n_from_1lk1 = 0usize;
    let mut n_from_lo = 0usize;
    let mut n_from_1l = 0usize;
    let mut n_from_1lk1_wp = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let f114 = cody(z, &c114, &D0, 0x210);
        let g114 = mul(w, f114);
        let d114 = ulp_distance(g114, t).unwrap_or(99);
        if !(d114 >= 2 && g114 < t) {
            continue;
        }
        n_114_lo += 1;
        let fu = cody(z, &C0, &D0, 0);
        let gu = mul(w, fu);
        let du = ulp_distance(gu, t).unwrap_or(99);
        let cu = class_of(t, w, fu);
        let from = if du == 1 && gu < t && k_of(t, w, fu) == Some(1) {
            n_from_1lk1 += 1;
            if hit(mul(w.next_up(), fu), t) {
                n_from_1lk1_wp += 1;
            }
            "1L-k1"
        } else if du == 1 && gu < t {
            n_from_1l += 1;
            "1L"
        } else if du >= 2 && gu < t {
            n_from_lo += 1;
            "LO"
        } else {
            "other"
        };
        if from != "LO" {
            println!(
                "  z={:.16} d114={} from={from} unmask={cu} 210={} 74={} w+1={} F+1={} 114-k={:?} tmu114={}",
                z,
                d114,
                class_of(t, w, cody(z, &C0, &D0, 0x210)),
                class_of(t, w, cody(z, &C0, &D0, 0x74)),
                hit(mul(w.next_up(), fu), t),
                hit(mul(w, fu.next_up()), t),
                k_of(t, w, f114),
                hit(mul(w.next_up(), f114.next_up()), t)
            );
        }
    }
    println!(
        "114 leftover-low n={n_114_lo} from 1L-k1={n_from_1lk1} w+1={n_from_1lk1_wp} from 1L-other={n_from_1l} from LO={n_from_lo}"
    );
    println!("pin 2.46875 vs 1.53125 114 last-store:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 2.46875).abs() > 1e-14 && (r.z - 1.53125).abs() > 1e-14 {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!("  z={:.16} bits={:#x}", z, z.to_bits());
        for (name, ff) in [("unm", fu), ("210", f210), ("74", f74), ("114", f114)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in -1i32..=4 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in -1i32..=4 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " tmu={} w+1={} F+1={}",
                hit(mul(w.next_up(), ff.next_up()), t),
                hit(mul(w.next_up(), ff), t),
                hit(mul(w, ff.next_up()), t)
            );
        }
    }
    println!("1L-k1 w+1 false 114 class:");
    let mut n_miss = 0usize;
    let mut n_miss_114_lo = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g < t) {
            continue;
        }
        if k_of(t, w, fu) != Some(1) {
            continue;
        }
        if hit(mul(w.next_up(), fu), t) {
            continue;
        }
        n_miss += 1;
        let f114 = cody(z, &c114, &D0, 0x210);
        let c114 = class_of(t, w, f114);
        let g114 = mul(w, f114);
        let d114 = ulp_distance(g114, t).unwrap_or(99);
        if d114 >= 2 && g114 < t {
            n_miss_114_lo += 1;
        }
        println!(
            "  z={:.16} 114={c114} 210={} 74={} F+1={} k+2={}",
            z,
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            hit(mul(w, fu.next_up()), t),
            signed(mul(w, poke(fu, 2)), t)
        );
    }
    println!("1L-k1 w+1 false n={n_miss} 114 leftover-low={n_miss_114_lo}");
    println!("pin 3.270833/3.46875/2.46875 114 vs 210 last-store:");
    let pinz = [3.2708333333333335, 3.46875, 2.46875];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!("  z={:.16} bits={:#x}", z, z.to_bits());
        for (name, ff) in [("unm", fu), ("210", f210), ("74", f74), ("114", f114)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in 0i32..=4 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in 0i32..=4 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " tmu={} w+1={} F+1={} store_k={:?} mul_k={:?}",
                hit(mul(w.next_up(), ff.next_up()), t),
                hit(mul(w.next_up(), ff), t),
                hit(mul(w, ff.next_up()), t),
                k_of(t, w, ff),
                mul_k_of(gg, t)
            );
        }
    }
    println!("1L-k1 w+1 true → 114 leftover-low:");
    let mut n_wp = 0usize;
    let mut n_wp_114_lo = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g < t) {
            continue;
        }
        if k_of(t, w, fu) != Some(1) {
            continue;
        }
        if !hit(mul(w.next_up(), fu), t) {
            continue;
        }
        n_wp += 1;
        let f114 = cody(z, &c114, &D0, 0x210);
        let g114 = mul(w, f114);
        let d114 = ulp_distance(g114, t).unwrap_or(99);
        if d114 >= 2 && g114 < t {
            n_wp_114_lo += 1;
            println!(
                "  z={:.16} d114={} 210={} 74={} tmu114={} 114-k={:?} 210-k={:?} unmask k+2={}",
                z,
                d114,
                class_of(t, w, cody(z, &C0, &D0, 0x210)),
                class_of(t, w, cody(z, &C0, &D0, 0x74)),
                hit(mul(w.next_up(), f114.next_up()), t),
                k_of(t, w, f114),
                k_of(t, w, cody(z, &C0, &D0, 0x210)),
                signed(mul(w, poke(fu, 2)), t)
            );
        }
    }
    println!("1L-k1 w+1 true n={n_wp} 114 leftover-low={n_wp_114_lo}");
    println!("1L-k1 → 114 fuse:");
    let mut n_1lk1 = 0usize;
    let mut n_114_fuse = 0usize;
    let mut n_114_fuse01 = 0usize;
    let mut n_114_fuse0m1 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g < t) {
            continue;
        }
        if k_of(t, w, fu) != Some(1) {
            continue;
        }
        n_1lk1 += 1;
        let f114 = cody(z, &c114, &D0, 0x210);
        let c114 = class_of(t, w, f114);
        if !c114.starts_with("fuse") {
            continue;
        }
        n_114_fuse += 1;
        if c114 == "fuse01" || c114 == "fuse01m1" {
            n_114_fuse01 += 1;
        }
        if c114 == "fuse0m1" {
            n_114_fuse0m1 += 1;
        }
        println!(
            "  z={:.16} 114={c114} 210={} 74={} w+1={} F+1={} k+2={} 114-k+1={} 114-k-1={}",
            z,
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            signed(mul(w, poke(fu, 2)), t),
            signed(mul(w, poke(f114, 1)), t),
            signed(mul(w, poke(f114, -1)), t)
        );
    }
    println!("1L-k1 n={n_1lk1} 114 fuse={n_114_fuse} fuse01={n_114_fuse01} fuse0m1={n_114_fuse0m1}");
    println!("pin 1.145833 vs 2.46875 vs 1.15625 114 last-store:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 1.1458333333333333).abs() > 1e-12
            && (r.z - 2.46875).abs() > 1e-14
            && (r.z - 1.15625).abs() > 1e-14
        {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!("  z={:.16} bits={:#x}", z, z.to_bits());
        for (name, ff) in [("unm", fu), ("210", f210), ("74", f74), ("114", f114)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in -2i32..=3 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in -2i32..=3 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " tmu={} w+1={} F+1={}",
                hit(mul(w.next_up(), ff.next_up()), t),
                hit(mul(w.next_up(), ff), t),
                hit(mul(w, ff.next_up()), t)
            );
        }
    }
    println!("1L-k1 → 210 fuse vs 114-only fuse last-store:");
    let pinz = [
        1.1458333333333333,
        1.40625,
        2.0208333333333335,
        3.6145833333333335,
        0.53125,
        1.1875,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} unmask={} 210={} 74={} 114={} w+1={} k+2={}",
            z,
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f74),
            class_of(t, w, f114),
            hit(mul(w.next_up(), fu), t),
            signed(mul(w, poke(fu, 2)), t)
        );
        print!("    210 store");
        for k in -2i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(f210, k)), t));
        }
        print!(" 114 store");
        for k in -2i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        println!();
    }
    println!("1L-k1 → 210 fuse:");
    let mut n_210_fuse = 0usize;
    let mut n_210_and_114 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g < t) {
            continue;
        }
        if k_of(t, w, fu) != Some(1) {
            continue;
        }
        let f210 = cody(z, &C0, &D0, 0x210);
        let c210 = class_of(t, w, f210);
        if !c210.starts_with("fuse") {
            continue;
        }
        n_210_fuse += 1;
        let f114 = cody(z, &c114, &D0, 0x210);
        let c114 = class_of(t, w, f114);
        let both = c114.starts_with("fuse");
        if both {
            n_210_and_114 += 1;
        }
        println!(
            "  z={:.16} 210={c210} 114={c114} 74={} w+1={} k+2={}",
            z,
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            hit(mul(w.next_up(), fu), t),
            signed(mul(w, poke(fu, 2)), t)
        );
    }
    println!("1L-k1 → 210 fuse n={n_210_fuse} also 114 fuse={n_210_and_114}");
    println!("pin 3.927083 vs 1.40625 last-store:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 3.9270833333333335).abs() > 1e-12 && (r.z - 1.40625).abs() > 1e-14 {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!("  z={:.16} bits={:#x}", z, z.to_bits());
        for (name, ff) in [("unm", fu), ("210", f210), ("74", f74), ("114", f114)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in -2i32..=3 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in -2i32..=3 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " tmu={} w+1={} F+1={}",
                hit(mul(w.next_up(), ff.next_up()), t),
                hit(mul(w.next_up(), ff), t),
                hit(mul(w, ff.next_up()), t)
            );
        }
    }
    println!("1L-k1 k=+2 2H skip → 210/114:");
    let mut n_skip = 0usize;
    let mut n_skip_210_fuse = 0usize;
    let mut n_skip_114_fuse = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g < t) {
            continue;
        }
        if k_of(t, w, fu) != Some(1) {
            continue;
        }
        if signed(mul(w, poke(fu, 2)), t) != "2H" {
            continue;
        }
        n_skip += 1;
        let c210 = class_of(t, w, cody(z, &C0, &D0, 0x210));
        let c114 = class_of(t, w, cody(z, &c114, &D0, 0x210));
        if c210.starts_with("fuse") {
            n_skip_210_fuse += 1;
        }
        if c114.starts_with("fuse") {
            n_skip_114_fuse += 1;
        }
        println!(
            "  z={:.16} 210={c210} 114={c114} 74={} w+1={}",
            z,
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            hit(mul(w.next_up(), fu), t)
        );
    }
    println!("1L-k1 k=+2 2H skip n={n_skip} 210 fuse={n_skip_210_fuse} 114 fuse={n_skip_114_fuse}");
    println!("pin 0.53125/0.5625/0.645833 vs 3.927083 last-store:");
    let pinz = [0.53125, 0.5625, 0.6458333333333333, 3.9270833333333335];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!("  z={:.16} bits={:#x}", z, z.to_bits());
        for (name, ff) in [("unm", fu), ("210", f210), ("74", f74), ("114", f114)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in -2i32..=3 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in -2i32..=3 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " tmu={} w+1={} F+1={}",
                hit(mul(w.next_up(), ff.next_up()), t),
                hit(mul(w.next_up(), ff), t),
                hit(mul(w, ff.next_up()), t)
            );
        }
    }
    println!("1H-km1 k=-2 2L skip → 210/114:");
    let mut n_skip = 0usize;
    let mut n_210_fuse = 0usize;
    let mut n_114_fuse = 0usize;
    let mut n_210_only = 0usize;
    let mut n_114_only = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if k_of(t, w, fu) != Some(-1) {
            continue;
        }
        if signed(mul(w, poke(fu, -2)), t) != "2L" {
            continue;
        }
        n_skip += 1;
        let f210 = cody(z, &C0, &D0, 0x210);
        let f114 = cody(z, &c114, &D0, 0x210);
        let c210 = class_of(t, w, f210);
        let c114 = class_of(t, w, f114);
        let f210b = c210.starts_with("fuse");
        let f114b = c114.starts_with("fuse");
        if f210b {
            n_210_fuse += 1;
        }
        if f114b {
            n_114_fuse += 1;
        }
        if f210b && !f114b {
            n_210_only += 1;
        }
        if f114b && !f210b {
            n_114_only += 1;
        }
        println!(
            "  z={:.16} 210={c210} 114={c114} 74={} tmd={} w-1={} F-1={}",
            z,
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!(
        "1H-km1 k=-2 2L skip n={n_skip} 210 fuse={n_210_fuse} 114 fuse={n_114_fuse} 210-only={n_210_only} 114-only={n_114_only}"
    );
    println!("1H-km1 k=-2 1L (no skip) → 210/114:");
    let mut n_1l = 0usize;
    let mut n_1l_210 = 0usize;
    let mut n_1l_114 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if k_of(t, w, fu) != Some(-1) {
            continue;
        }
        if signed(mul(w, poke(fu, -2)), t) != "1L" {
            continue;
        }
        n_1l += 1;
        let c210 = class_of(t, w, cody(z, &C0, &D0, 0x210));
        let c114 = class_of(t, w, cody(z, &c114, &D0, 0x210));
        if c210.starts_with("fuse") {
            n_1l_210 += 1;
        }
        if c114.starts_with("fuse") {
            n_1l_114 += 1;
        }
        if c210.starts_with("fuse") || c114.starts_with("fuse") || c210 != "1H-km1" || c114 != "1H-km1" {
            println!(
                "  z={:.16} 210={c210} 114={c114} 74={} tmd={}",
                z,
                class_of(t, w, cody(z, &C0, &D0, 0x74)),
                hit(mul(w.next_down(), fu.next_down()), t)
            );
        }
    }
    println!("1H-km1 k=-2 1L n={n_1l} 210 fuse={n_1l_210} 114 fuse={n_1l_114}");
    println!("pin 1H-km1 k=-2 1L 210=114 fuse n=8 last-store:");
    let pinz = [
        2.7395833333333335,
        2.8958333333333335,
        3.0,
        3.21875,
        3.25,
        3.4375,
        3.6458333333333335,
        3.6875,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        let tmd = hit(mul(w.next_down(), fu.next_down()), t);
        println!(
            "  z={:.16} tmd={tmd} unmask={} 210={} 114={} 74={}",
            z,
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f114),
            class_of(t, w, f74)
        );
        print!("    unm store");
        for k in -3i32..=1 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" 210 store");
        for k in -3i32..=1 {
            print!(" {k}:{}", signed(mul(w, poke(f210, k)), t));
        }
        print!(" 114 store");
        for k in -3i32..=1 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        println!(
            " w-1={} F-1={} 210-k-1={} 114-k-1={}",
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t),
            signed(mul(w, poke(f210, -1)), t),
            signed(mul(w, poke(f114, -1)), t)
        );
    }
    println!("1H-km1 k=-2 1L stay vs fuse split:");
    let mut n_stay = 0usize;
    let mut n_fuse = 0usize;
    let mut n_stay_wm = 0usize;
    let mut n_fuse_wm = 0usize;
    let mut n_stay_tmd = 0usize;
    let mut n_fuse_tmd = 0usize;
    let mut n_stay_k1_2h = 0usize;
    let mut n_fuse_k1_2h = 0usize;
    let mut n_stay_555 = 0usize;
    let mut n_fuse_555 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if k_of(t, w, fu) != Some(-1) {
            continue;
        }
        if signed(mul(w, poke(fu, -2)), t) != "1L" {
            continue;
        }
        let f210 = cody(z, &C0, &D0, 0x210);
        let fuse = class_of(t, w, f210).starts_with("fuse");
        let wm = hit(mul(w.next_down(), fu), t);
        let tmd = hit(mul(w.next_down(), fu.next_down()), t);
        let k1_2h = signed(mul(w, poke(fu, 1)), t) == "2H";
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        if fuse {
            n_fuse += 1;
            if wm {
                n_fuse_wm += 1;
            }
            if tmd {
                n_fuse_tmd += 1;
            }
            if k1_2h {
                n_fuse_k1_2h += 1;
            }
            if is555 {
                n_fuse_555 += 1;
            }
        } else {
            n_stay += 1;
            if wm {
                n_stay_wm += 1;
            }
            if tmd {
                n_stay_tmd += 1;
            }
            if k1_2h {
                n_stay_k1_2h += 1;
            }
            if is555 {
                n_stay_555 += 1;
            }
            println!(
                "  STAY z={:.16} tmd={tmd} w-1={wm} k+1={} k-3={} 555={is555} 114={}",
                z,
                signed(mul(w, poke(fu, 1)), t),
                signed(mul(w, poke(fu, -3)), t),
                class_of(t, w, cody(z, &c114, &D0, 0x210))
            );
        }
    }
    println!(
        "stay n={n_stay} w-1={n_stay_wm} tmd={n_stay_tmd} k+1 2H={n_stay_k1_2h} 555={n_stay_555}"
    );
    println!(
        "fuse n={n_fuse} w-1={n_fuse_wm} tmd={n_fuse_tmd} k+1 2H={n_fuse_k1_2h} 555={n_fuse_555}"
    );
    println!("pin 6 stay in fuse band vs 8 fuse last-store:");
    let pinz = [
        2.7395833333333335,
        2.8125,
        2.8645833333333335,
        2.8958333333333335,
        2.9895833333333335,
        3.0,
        3.125,
        3.21875,
        3.25,
        3.4375,
        3.4583333333333335,
        3.6458333333333335,
        3.6875,
        3.7395833333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let fuse = class_of(t, w, f210).starts_with("fuse");
        let tag = if fuse { "FUSE" } else { "STAY" };
        println!(
            "  {tag} z={:.16} unm k-3..1 {} {} {} {} {} 210 k-2..1 {} {} {} {} mul_k={:?} w-1={} tmd={}",
            z,
            signed(mul(w, poke(fu, -3)), t),
            signed(mul(w, poke(fu, -2)), t),
            signed(mul(w, poke(fu, -1)), t),
            signed(mul(w, poke(fu, 0)), t),
            signed(mul(w, poke(fu, 1)), t),
            signed(mul(w, poke(f210, -2)), t),
            signed(mul(w, poke(f210, -1)), t),
            signed(mul(w, poke(f210, 0)), t),
            signed(mul(w, poke(f210, 1)), t),
            mul_k_of(mul(w, fu), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w.next_down(), fu.next_down()), t)
        );
    }
    println!("pin 2.989583 vs 3.0 F bits and last-store:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 2.9895833333333335).abs() > 1e-12 && (r.z - 3.0).abs() > 1e-14 {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} 555={}",
            z,
            z.to_bits(),
            {
                let h = format!("{:x}", z.to_bits());
                h.contains("55555555") || h.contains("aaaaaaaa")
            }
        );
        println!(
            "    F unm={:#x} 210={:#x} d(unm,210)={} 74={:#x} d(unm,74)={} 114={:#x} d(unm,114)={}",
            fu.to_bits(),
            f210.to_bits(),
            ulp_distance(fu, f210).unwrap_or(99),
            f74.to_bits(),
            ulp_distance(fu, f74).unwrap_or(99),
            f114.to_bits(),
            ulp_distance(fu, f114).unwrap_or(99)
        );
        for (name, ff) in [("unm", fu), ("210", f210), ("74", f74), ("114", f114)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in -3i32..=2 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in -3i32..=2 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " tmd={} w-1={} F-1={}",
                hit(mul(w.next_down(), ff.next_down()), t),
                hit(mul(w.next_down(), ff), t),
                hit(mul(w, ff.next_down()), t)
            );
        }
    }
    println!("1H-km1 k=-2 1L d(unmask F, 210 F):");
    let mut n_d0_stay = 0usize;
    let mut n_d0_fuse = 0usize;
    let mut n_d1_stay = 0usize;
    let mut n_d1_fuse = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if k_of(t, w, fu) != Some(-1) {
            continue;
        }
        if signed(mul(w, poke(fu, -2)), t) != "1L" {
            continue;
        }
        let f210 = cody(z, &C0, &D0, 0x210);
        let df = ulp_distance(fu, f210).unwrap_or(99);
        let fuse = class_of(t, w, f210).starts_with("fuse");
        if df == 0 && !fuse {
            n_d0_stay += 1;
        }
        if df == 0 && fuse {
            n_d0_fuse += 1;
        }
        if df == 1 && !fuse {
            n_d1_stay += 1;
        }
        if df == 1 && fuse {
            n_d1_fuse += 1;
        }
        if df != 0 || fuse {
            println!(
                "  z={:.16} dF={df} fuse={fuse} 210<unm={} 114 dF={}",
                z,
                f210 < fu,
                ulp_distance(fu, cody(z, &c114, &D0, 0x210)).unwrap_or(99)
            );
        }
    }
    println!("dF=0 stay={n_d0_stay} fuse={n_d0_fuse} dF=1 stay={n_d1_stay} fuse={n_d1_fuse}");
    println!("1L-k1 d(unmask F, 210 F):");
    let mut n_d0_1l = 0usize;
    let mut n_d1up_fuse = 0usize;
    let mut n_d1up_other = 0usize;
    let mut n_d1dn = 0usize;
    let mut n_dgt1 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g < t) {
            continue;
        }
        if k_of(t, w, fu) != Some(1) {
            continue;
        }
        let f210 = cody(z, &C0, &D0, 0x210);
        let df = ulp_distance(fu, f210).unwrap_or(99);
        let c210 = class_of(t, w, f210);
        let c114 = class_of(t, w, cody(z, &c114, &D0, 0x210));
        if df == 0 {
            n_d0_1l += 1;
        } else if df == 1 && f210 > fu {
            if c210.starts_with("fuse") {
                n_d1up_fuse += 1;
            } else {
                n_d1up_other += 1;
            }
        } else if df == 1 && f210 < fu {
            n_d1dn += 1;
        } else {
            n_dgt1 += 1;
        }
        if df != 0 {
            println!(
                "  z={:.16} dF={df} 210>unm={} 210={c210} 114={c114} 74={}",
                z,
                f210 > fu,
                class_of(t, w, cody(z, &C0, &D0, 0x74))
            );
        }
    }
    println!(
        "1L-k1 dF=0 n={n_d0_1l} dF=1 up fuse={n_d1up_fuse} up other={n_d1up_other} dF=1 down={n_d1dn} dF>1={n_dgt1}"
    );
    println!("1L-k1 d(unmask F, 114 F):");
    let mut n114_d0 = 0usize;
    let mut n114_up_fuse = 0usize;
    let mut n114_up_other = 0usize;
    let mut n114_dn = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g < t) {
            continue;
        }
        if k_of(t, w, fu) != Some(1) {
            continue;
        }
        let f114 = cody(z, &c114, &D0, 0x210);
        let f210 = cody(z, &C0, &D0, 0x210);
        let df = ulp_distance(fu, f114).unwrap_or(99);
        let c114 = class_of(t, w, f114);
        let c210 = class_of(t, w, f210);
        if df == 0 {
            n114_d0 += 1;
        } else if df == 1 && f114 > fu {
            if c114.starts_with("fuse") {
                n114_up_fuse += 1;
            } else {
                n114_up_other += 1;
            }
        } else if df == 1 && f114 < fu {
            n114_dn += 1;
        }
        if df != 0 || c114.starts_with("fuse") || c210.starts_with("fuse") {
            println!(
                "  z={:.16} 114dF={df} 114>unm={} 114={c114} 210dF={} 210>unm={} 210={c210}",
                z,
                f114 > fu,
                ulp_distance(fu, f210).unwrap_or(99),
                f210 > fu
            );
        }
    }
    println!(
        "1L-k1 114 dF=0 n={n114_d0} dF=1 up fuse={n114_up_fuse} up other={n114_up_other} dF=1 down={n114_dn}"
    );
    println!("pin 3.927083 vs 1.40625 F bits:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 3.9270833333333335).abs() > 1e-12 && (r.z - 1.40625).abs() > 1e-14 {
            continue;
        }
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} unm={:#x} 210={:#x} d={} 74={:#x} d={} 114={:#x} d={}",
            z,
            fu.to_bits(),
            f210.to_bits(),
            ulp_distance(fu, f210).unwrap_or(99),
            f74.to_bits(),
            ulp_distance(fu, f74).unwrap_or(99),
            f114.to_bits(),
            ulp_distance(fu, f114).unwrap_or(99)
        );
    }
    println!("1H-km1 d(unmask F, 114 F):");
    let mut n_d0 = 0usize;
    let mut n_dn_fuse = 0usize;
    let mut n_dn_other = 0usize;
    let mut n_up = 0usize;
    let mut n_gt1 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if k_of(t, w, fu) != Some(-1) {
            continue;
        }
        let f114 = cody(z, &c114, &D0, 0x210);
        let f210 = cody(z, &C0, &D0, 0x210);
        let df = ulp_distance(fu, f114).unwrap_or(99);
        let c114 = class_of(t, w, f114);
        let c210 = class_of(t, w, f210);
        if df == 0 {
            n_d0 += 1;
        } else if df == 1 && f114 < fu {
            if c114.starts_with("fuse") {
                n_dn_fuse += 1;
            } else {
                n_dn_other += 1;
            }
        } else if df == 1 && f114 > fu {
            n_up += 1;
        } else {
            n_gt1 += 1;
        }
        if df != 0 || c114.starts_with("fuse") || c210.starts_with("fuse") {
            println!(
                "  z={:.16} 114dF={df} 114<unm={} 114={c114} 210dF={} 210<unm={} 210={c210}",
                z,
                f114 < fu,
                ulp_distance(fu, f210).unwrap_or(99),
                f210 < fu
            );
        }
    }
    println!(
        "1H-km1 114 dF=0 n={n_d0} dF=1 down fuse={n_dn_fuse} down other={n_dn_other} dF=1 up={n_up} dF>1={n_gt1}"
    );
    println!("1H tmd 114 dF including 0.84375-1ulp:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if !hit(mul(w.next_down(), fu.next_down()), t) {
            continue;
        }
        let f114 = cody(z, &c114, &D0, 0x210);
        let f210 = cody(z, &C0, &D0, 0x210);
        println!(
            "  z={:.16} unmask={} 114={} 114dF={} 114<unm={} 210={} 210dF={}",
            z,
            class_of(t, w, fu),
            class_of(t, w, f114),
            ulp_distance(fu, f114).unwrap_or(99),
            f114 < fu,
            class_of(t, w, f210),
            ulp_distance(fu, f210).unwrap_or(99)
        );
    }
    println!("74 dF vs unmask F:");
    let mut n_1lk1 = 0usize;
    let mut n_1lk1_nz = 0usize;
    let mut n_1hkm1 = 0usize;
    let mut n_1hkm1_nz = 0usize;
    let mut n_lo = 0usize;
    let mut n_lo_nz = 0usize;
    let mut n_hi = 0usize;
    let mut n_hi_nz = 0usize;
    let mut n_mid = 0usize;
    let mut n_mid_nz = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f74 = cody(z, &C0, &D0, 0x74);
        let df = ulp_distance(fu, f74).unwrap_or(99);
        n_mid += 1;
        if df != 0 {
            n_mid_nz += 1;
        }
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        let is_1lk1 = d == 1 && g < t && k_of(t, w, fu) == Some(1);
        let is_1hkm1 = d == 1 && g > t && k_of(t, w, fu) == Some(-1);
        if is_1lk1 {
            n_1lk1 += 1;
            if df != 0 {
                n_1lk1_nz += 1;
                println!("  1L-k1 dF={df} z={:.16} 74>unm={}", z, f74 > fu);
            }
        }
        if is_1hkm1 {
            n_1hkm1 += 1;
            if df != 0 {
                n_1hkm1_nz += 1;
                println!("  1H-km1 dF={df} z={:.16} 74<unm={}", z, f74 < fu);
            }
        }
        if lo_u.contains(&zb) {
            n_lo += 1;
            if df != 0 {
                n_lo_nz += 1;
                println!(
                    "  LO dF={df} z={:.16} 74>unm={} 74={}",
                    z,
                    f74 > fu,
                    class_of(t, w, f74)
                );
            }
        }
        if hi_u.contains(&zb) {
            n_hi += 1;
            if df != 0 {
                n_hi_nz += 1;
                println!(
                    "  HI dF={df} z={:.16} 74<unm={} 74={}",
                    z,
                    f74 < fu,
                    class_of(t, w, f74)
                );
            }
        }
    }
    println!("74 dF 1L-k1 n={n_1lk1} nz={n_1lk1_nz} 1H-km1 n={n_1hkm1} nz={n_1hkm1_nz}");
    println!("74 dF leftover-low n={n_lo} nz={n_lo_nz} leftover-high n={n_hi} nz={n_hi_nz}");
    println!("74 dF DIRECT mid n={n_mid} nz={n_mid_nz}");
    println!("74 dF DIRECT mid all nz:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f74 = cody(z, &C0, &D0, 0x74);
        let df = ulp_distance(fu, f74).unwrap_or(99);
        if df == 0 {
            continue;
        }
        let g = mul(w, fu);
        println!(
            "  z={:.16} dF={df} 74>unm={} unmask={} 74={} 210={} 114={}",
            z,
            f74 > fu,
            class_of(t, w, fu),
            class_of(t, w, f74),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &c114, &D0, 0x210))
        );
    }
    println!("pin 2.75 vs 1.625 last-store and F bits:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 2.75).abs() > 1e-14 && (r.z - 1.625).abs() > 1e-14 {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} F unm={:#x} 74={:#x} d={} 210={:#x} d={} 114={:#x} d={}",
            z,
            z.to_bits(),
            fu.to_bits(),
            f74.to_bits(),
            ulp_distance(fu, f74).unwrap_or(99),
            f210.to_bits(),
            ulp_distance(fu, f210).unwrap_or(99),
            f114.to_bits(),
            ulp_distance(fu, f114).unwrap_or(99)
        );
        for (name, ff) in [("unm", fu), ("74", f74), ("210", f210), ("114", f114)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in 0i32..=4 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in 0i32..=4 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " tmu={} w+1={} F+1={}",
                hit(mul(w.next_up(), ff.next_up()), t),
                hit(mul(w.next_up(), ff), t),
                hit(mul(w, ff.next_up()), t)
            );
        }
    }
    println!("leftover-low primary k=+1 dF 74/210/114:");
    let mut n = 0usize;
    let mut n_all_up = 0usize;
    let mut n_none = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !lo_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if k_of(t, w, fu) != Some(1) {
            continue;
        }
        n += 1;
        let f74 = cody(z, &C0, &D0, 0x74);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f114 = cody(z, &c114, &D0, 0x210);
        let d74 = ulp_distance(fu, f74).unwrap_or(99);
        let d210 = ulp_distance(fu, f210).unwrap_or(99);
        let d114 = ulp_distance(fu, f114).unwrap_or(99);
        if d74 == 1 && d210 == 1 && d114 == 1 && f74 > fu && f210 > fu && f114 > fu {
            n_all_up += 1;
        }
        if d74 == 0 && d210 == 0 && d114 == 0 {
            n_none += 1;
        }
        println!(
            "  z={:.16} d={} 74dF={d74} 74>={} 210dF={d210} 210>={} 114dF={d114} 114>={} 74={} 210={} 114={}",
            z,
            ulp_distance(mul(w, fu), t).unwrap_or(99),
            f74 > fu,
            f210 > fu,
            f114 > fu,
            class_of(t, w, f74),
            class_of(t, w, f210),
            class_of(t, w, f114)
        );
    }
    println!("leftover-low primary k=+1 n={n} all-mask next_up={n_all_up} all dF=0={n_none}");
    println!("leftover-high primary k=-1 dF 74/210/114:");
    let mut nh = 0usize;
    let mut n_all_dn = 0usize;
    let mut n_none_h = 0usize;
    let mut n_210_114_dn = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        if !hi_u.contains(&zb) {
            continue;
        }
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if k_of(t, w, fu) != Some(-1) {
            continue;
        }
        nh += 1;
        let f74 = cody(z, &C0, &D0, 0x74);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f114 = cody(z, &c114, &D0, 0x210);
        let d74 = ulp_distance(fu, f74).unwrap_or(99);
        let d210 = ulp_distance(fu, f210).unwrap_or(99);
        let d114 = ulp_distance(fu, f114).unwrap_or(99);
        if d74 == 1 && d210 == 1 && d114 == 1 && f74 < fu && f210 < fu && f114 < fu {
            n_all_dn += 1;
        }
        if d74 == 0 && d210 == 0 && d114 == 0 {
            n_none_h += 1;
        }
        if d74 == 0 && d210 == 1 && d114 == 1 && f210 < fu && f114 < fu {
            n_210_114_dn += 1;
        }
        println!(
            "  z={:.16} d={} 74dF={d74} 74<={} 210dF={d210} 210<={} 114dF={d114} 114<={} 74={} 210={} 114={} w-1={}",
            z,
            ulp_distance(mul(w, fu), t).unwrap_or(99),
            f74 < fu,
            f210 < fu,
            f114 < fu,
            class_of(t, w, f74),
            class_of(t, w, f210),
            class_of(t, w, f114),
            hit(mul(w.next_down(), fu), t)
        );
    }
    println!(
        "leftover-high primary k=-1 n={nh} all-mask next_down={n_all_dn} 210+114 next_down 74 stay={n_210_114_dn} all dF=0={n_none_h}"
    );
    println!("pin 3.395833 vs 0.75 last-store and F bits:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 3.3958333333333335).abs() > 1e-12 && (r.z - 0.75).abs() > 1e-14 {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} F unm={:#x} 114={:#x} d={} 114<unm={} 210dF={} 74dF={}",
            z,
            z.to_bits(),
            fu.to_bits(),
            f114.to_bits(),
            ulp_distance(fu, f114).unwrap_or(99),
            f114 < fu,
            ulp_distance(fu, f210).unwrap_or(99),
            ulp_distance(fu, f74).unwrap_or(99)
        );
        for (name, ff) in [("unm", fu), ("114", f114), ("210", f210), ("74", f74)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in -3i32..=1 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in -3i32..=1 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " tmd={} w-1={} F-1={}",
                hit(mul(w.next_down(), ff.next_down()), t),
                hit(mul(w.next_down(), ff), t),
                hit(mul(w, ff.next_down()), t)
            );
        }
    }
    println!("pin leftover-high k=-1 w-1 true n=3 last-store:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 1.3333333333333333).abs() > 1e-12
            && (r.z - 1.34375).abs() > 1e-14
            && (r.z - 3.3958333333333335).abs() > 1e-12
        {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        println!(
            "  z={:.16} bits={:#x} 555={is555} 114dF={} 114<unm={}",
            z,
            z.to_bits(),
            ulp_distance(fu, f114).unwrap_or(99),
            f114 < fu
        );
        print!("    unm store");
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmd={} F-1={}",
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w, fu.next_down()), t)
        );
        print!("    114 store");
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        print!(" mul");
        let g114 = mul(w, f114);
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(poke(g114, k), t));
        }
        println!(" {}", class_of(t, w, f114));
    }
    println!("pin 3.520833 vs 3.90625 fused 74 last-store:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if (r.z - 3.5208333333333335).abs() > 1e-12 && (r.z - 3.90625).abs() > 1e-14 {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} F unm={:#x} 74={:#x} d={} 74>unm={} 210dF={} 114dF={}",
            z,
            z.to_bits(),
            fu.to_bits(),
            f74.to_bits(),
            ulp_distance(fu, f74).unwrap_or(99),
            f74 > fu,
            ulp_distance(fu, f210).unwrap_or(99),
            ulp_distance(fu, f114).unwrap_or(99)
        );
        for (name, ff) in [("unm", fu), ("74", f74), ("210", f210), ("114", f114)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in -2i32..=2 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in -2i32..=2 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " w+1={} F+1={} w-1={} F-1={}",
                hit(mul(w.next_up(), ff), t),
                hit(mul(w, ff.next_up()), t),
                hit(mul(w.next_down(), ff), t),
                hit(mul(w, ff.next_down()), t)
            );
        }
    }
    println!("fused w+1 vs 74 dF:");
    let mut n_fuse = 0usize;
    let mut n_wp = 0usize;
    let mut n_wp_74up = 0usize;
    let mut n_wn = 0usize;
    let mut n_wn_74dn = 0usize;
    let mut n_wm = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        n_fuse += 1;
        let wp = hit(mul(w.next_up(), fu), t);
        let wm = hit(mul(w.next_down(), fu), t);
        let f74 = cody(z, &C0, &D0, 0x74);
        let df = ulp_distance(fu, f74).unwrap_or(99);
        if wp {
            n_wp += 1;
            if df == 1 && f74 > fu {
                n_wp_74up += 1;
                println!("  w+1 74-up z={:.16} 74={}", z, class_of(t, w, f74));
            }
        } else {
            n_wn += 1;
            if df == 1 && f74 < fu {
                n_wn_74dn += 1;
                println!("  no-w+1 74-dn z={:.16} 74={}", z, class_of(t, w, f74));
            }
        }
        if wm {
            n_wm += 1;
        }
        if df != 0 && !(wp && f74 > fu) && !(df == 1 && f74 < fu && !wp) {
            println!(
                "  other 74dF z={:.16} dF={df} 74>unm={} w+1={wp} w-1={wm} 74={}",
                z,
                f74 > fu,
                class_of(t, w, f74)
            );
        }
    }
    println!(
        "fused n={n_fuse} w+1={n_wp} of which 74 next_up={n_wp_74up} no-w+1={n_wn} of which 74 next_down={n_wn_74dn} w-1={n_wm}"
    );
    println!("fused w+1 vs fuse01 vs fuse0m1:");
    let mut n_wp = 0usize;
    let mut n_01 = 0usize;
    let mut n_0m1 = 0usize;
    let mut n_wp_01 = 0usize;
    let mut n_wm_0m1 = 0usize;
    let mut n_wp_only = 0usize;
    let mut n_01_only = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        let wp = hit(mul(w.next_up(), fu), t);
        let wm = hit(mul(w.next_down(), fu), t);
        let fp = hit(mul(w, fu.next_up()), t);
        let fm = hit(mul(w, fu.next_down()), t);
        let fuse01 = fp;
        let fuse0m1 = fm;
        if wp {
            n_wp += 1;
        }
        if fuse01 {
            n_01 += 1;
        }
        if fuse0m1 {
            n_0m1 += 1;
        }
        if wp && fuse01 {
            n_wp_01 += 1;
        }
        if wm && fuse0m1 {
            n_wm_0m1 += 1;
        }
        if wp && !fuse01 {
            n_wp_only += 1;
        }
        if fuse01 && !wp {
            n_01_only += 1;
        }
        if wp || fuse01 || fuse0m1 {
            println!(
                "  z={:.16} w+1={wp} F+1={fp} w-1={wm} F-1={fm} 74dF={}",
                z,
                ulp_distance(fu, cody(z, &C0, &D0, 0x74)).unwrap_or(99)
            );
        }
    }
    println!(
        "fused w+1 n={n_wp} fuse01 n={n_01} overlap={n_wp_01} w+1 only={n_wp_only} fuse01 only={n_01_only} fuse0m1 n={n_0m1} w-1∩F-1={n_wm_0m1}"
    );
    println!("fused w-1 vs fuse0m1:");
    let mut n_wm = 0usize;
    let mut n_fm = 0usize;
    let mut n_both = 0usize;
    let mut n_wm_only = 0usize;
    let mut n_fm_only = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        let wm = hit(mul(w.next_down(), fu), t);
        let fm = hit(mul(w, fu.next_down()), t);
        if wm {
            n_wm += 1;
        }
        if fm {
            n_fm += 1;
        }
        if wm && fm {
            n_both += 1;
        }
        if wm && !fm {
            n_wm_only += 1;
        }
        if fm && !wm {
            n_fm_only += 1;
        }
        if wm || fm {
            println!(
                "  z={:.16} w-1={wm} F-1={fm} tag={} 74dF={}",
                z,
                if wm && fm {
                    "both"
                } else if wm {
                    "w-1 only"
                } else {
                    "F-1 only"
                },
                ulp_distance(fu, cody(z, &C0, &D0, 0x74)).unwrap_or(99)
            );
        }
    }
    println!(
        "fused w-1 n={n_wm} fuse0m1 n={n_fm} overlap={n_both} w-1 only={n_wm_only} F-1 only={n_fm_only}"
    );
    println!("pin w-1 only vs w+1 only last-store:");
    let pinz = [
        1.4583333333333333,
        3.5520833333333335,
        3.90625,
        3.5208333333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f74 = cody(z, &C0, &D0, 0x74);
        println!(
            "  z={:.16} bits={:#x} 74dF={} 74>unm={} 74={}",
            z,
            z.to_bits(),
            ulp_distance(fu, f74).unwrap_or(99),
            f74 > fu,
            class_of(t, w, f74)
        );
        print!("    unm store");
        for k in -2i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -2i32..=2 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w+1={} F+1={} w-1={} F-1={}",
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
        print!("    74  store");
        for k in -2i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(f74, k)), t));
        }
        print!(" mul");
        let g74 = mul(w, f74);
        for k in -2i32..=2 {
            print!(" {k}:{}", signed(poke(g74, k), t));
        }
        println!();
    }
    println!("pin F-1 only vs fuse0m1 overlap last-store:");
    let pinz = [
        0.9895833333333334,
        2.2395833333333335,
        1.5,
        1.0833333333333333,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f114 = cody(z, &c114, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        println!(
            "  z={:.16} bits={:#x} unmask={} 210={} 114={} 74={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f114),
            class_of(t, w, f74)
        );
        print!("    unm store");
        for k in -3i32..=1 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=1 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w-1={} F-1={} 114dF={} 210dF={}",
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t),
            ulp_distance(fu, f114).unwrap_or(99),
            ulp_distance(fu, f210).unwrap_or(99)
        );
        print!("    114 store");
        for k in -3i32..=1 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        println!();
    }
    println!("fuse01 only n=5 210/114 dF:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        let wp = hit(mul(w.next_up(), fu), t);
        let fp = hit(mul(w, fu.next_up()), t);
        if !(fp && !wp) {
            continue;
        }
        let f210 = cody(z, &C0, &D0, 0x210);
        let f114 = cody(z, &c114, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        println!(
            "  z={:.16} 210dF={} 210>={} 210={} 114dF={} 114>={} 114={} 74dF={}",
            z,
            ulp_distance(fu, f210).unwrap_or(99),
            f210 > fu,
            class_of(t, w, f210),
            ulp_distance(fu, f114).unwrap_or(99),
            f114 > fu,
            class_of(t, w, f114),
            ulp_distance(fu, f74).unwrap_or(99)
        );
        print!("    unm store");
        for k in -2i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        println!(
            " mul k+1={}",
            signed(poke(g, 1), t)
        );
    }
    println!("pin 0.96875 vs stall cluster vs 2.28125:");
    let pinz = [0.8020833333333334, 0.96875, 2.28125];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let f210 = cody(z, &C0, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} F unm={:#x} 114={:#x} d={} 210dF={}",
            z,
            z.to_bits(),
            fu.to_bits(),
            f114.to_bits(),
            ulp_distance(fu, f114).unwrap_or(99),
            ulp_distance(fu, f210).unwrap_or(99)
        );
        print!("    unm store");
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!();
        print!("    114 store");
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        println!(" {}", class_of(t, w, f114));
        print!("    210 store");
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(f210, k)), t));
        }
        println!(" {}", class_of(t, w, f210));
    }
    println!("fuse01 overlap n=6 210/114 dF:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        let wp = hit(mul(w.next_up(), fu), t);
        let fp = hit(mul(w, fu.next_up()), t);
        if !(wp && fp) {
            continue;
        }
        let f210 = cody(z, &C0, &D0, 0x210);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} 210dF={} 210>={} 210={} 114dF={} 114>={} 114={} stall-2/-1={}",
            z,
            ulp_distance(fu, f210).unwrap_or(99),
            f210 > fu,
            class_of(t, w, f210),
            ulp_distance(fu, f114).unwrap_or(99),
            f114 > fu,
            class_of(t, w, f114),
            signed(mul(w, poke(fu, -2)), t) == signed(mul(w, poke(fu, -1)), t)
        );
    }
    println!("w+1 only n=5 210/114/74 dF:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        let wp = hit(mul(w.next_up(), fu), t);
        let fp = hit(mul(w, fu.next_up()), t);
        if !(wp && !fp) {
            continue;
        }
        let f210 = cody(z, &C0, &D0, 0x210);
        let f114 = cody(z, &c114, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        println!(
            "  z={:.16} 210dF={} 210>={} 210={} 114dF={} 114>={} 114={} 74dF={} 74>={} 74={}",
            z,
            ulp_distance(fu, f210).unwrap_or(99),
            f210 > fu,
            class_of(t, w, f210),
            ulp_distance(fu, f114).unwrap_or(99),
            f114 > fu,
            class_of(t, w, f114),
            ulp_distance(fu, f74).unwrap_or(99),
            f74 > fu,
            class_of(t, w, f74)
        );
        print!("    unm store");
        for k in -2i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        println!();
    }
    println!("fused last-store stall k=+1/+2=1H:");
    let mut n_stall = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        if signed(mul(w, poke(fu, 1)), t) != "1H" {
            continue;
        }
        if signed(mul(w, poke(fu, 2)), t) != "1H" {
            continue;
        }
        n_stall += 1;
        let wp = hit(mul(w.next_up(), fu), t);
        let fp = hit(mul(w, fu.next_up()), t);
        println!(
            "  z={:.16} w+1={wp} F+1={fp} 210={} 114={} 74={}",
            z,
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &c114, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x74))
        );
    }
    println!("fused stall k=+1/+2=1H n={n_stall}");
    println!("pin 1.25 vs 1.6875 vs 0.802083 vs 3.90625:");
    let pinz = [1.25, 1.6875, 0.8020833333333334, 3.90625];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        print!("  z={:.16} store", z);
        for k in -2i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -2i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w+1={} F+1={}",
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
    }
    println!("pin fused stall k=+1/+2=1H all 5 last-store:");
    let pinz = [1.0, 1.03125, 1.25, 1.6875, 2.1770833333333335];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        print!("  z={:.16} bits={:#x} 555={is555} store", z, z.to_bits());
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w+1={} F+1={} w-1={} F-1={}",
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("fused stall k=-2/-1=1L:");
    let mut n_f = 0usize;
    let mut n_f_wp = 0usize;
    let mut n_f_fp = 0usize;
    let mut n_f_hi = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        if signed(mul(w, poke(fu, -2)), t) != "1L" {
            continue;
        }
        if signed(mul(w, poke(fu, -1)), t) != "1L" {
            continue;
        }
        n_f += 1;
        let wp = hit(mul(w.next_up(), fu), t);
        let fp = hit(mul(w, fu.next_up()), t);
        if wp {
            n_f_wp += 1;
        }
        if fp {
            n_f_fp += 1;
        }
        let hi = signed(mul(w, poke(fu, 1)), t) == "1H"
            && signed(mul(w, poke(fu, 2)), t) == "1H";
        if hi {
            n_f_hi += 1;
        }
        println!(
            "  z={:.16} w+1={wp} F+1={fp} both-stall={hi} k+1={} k+2={}",
            z,
            signed(mul(w, poke(fu, 1)), t),
            signed(mul(w, poke(fu, 2)), t)
        );
    }
    println!("fused stall k=-2/-1=1L n={n_f} w+1={n_f_wp} F+1={n_f_fp} also k=+1/+2=1H={n_f_hi}");
    println!("fuse01 stall k=-2/-1=1L:");
    let mut n_01 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        if !hit(mul(w, fu.next_up()), t) {
            continue;
        }
        if signed(mul(w, poke(fu, -2)), t) != "1L" {
            continue;
        }
        if signed(mul(w, poke(fu, -1)), t) != "1L" {
            continue;
        }
        n_01 += 1;
        println!(
            "  z={:.16} w+1={} 114={}",
            z,
            hit(mul(w.next_up(), fu), t),
            class_of(t, w, cody(z, &c114, &D0, 0x210))
        );
    }
    println!("fuse01 stall k=-2/-1=1L n={n_01}");
    println!("pin fused stall k=-2/-1=1L k=+2 2H n=4:");
    let pinz = [
        1.3020833333333333,
        2.09375,
        2.5520833333333335,
        3.5520833333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        print!("  z={:.16} 555={is555} store", z);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w+1={} F+1={} w-1={} F-1={}",
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("fused stall k=-2/-1=1L +1/32 class:");
    let mut n_p32 = 0usize;
    let mut n_p32_0m1 = 0usize;
    let mut n_p32_hi = 0usize;
    let mut n_p32_miss = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        if signed(mul(w, poke(fu, -2)), t) != "1L" {
            continue;
        }
        if signed(mul(w, poke(fu, -1)), t) != "1L" {
            continue;
        }
        let z2 = z + 1.0 / 32.0;
        let stall23 = signed(mul(w, poke(fu, 2)), t) == "2H"
            && signed(mul(w, poke(fu, 3)), t) == "2H";
        if let Some(rr) = rows.iter().find(|x| x.direct && (x.z - z2).abs() < 1e-12) {
            n_p32 += 1;
            let t2 = f64::from_bits(rr.qbits);
            let w2 = f::w_rn53(rr.z);
            let fu2 = cody(rr.z, &C0, &D0, 0);
            let tag = class_of(t2, w2, fu2);
            if tag == "fuse0m1" {
                n_p32_0m1 += 1;
            }
            if tag.contains('H') {
                n_p32_hi += 1;
            }
            println!(
                "  z={:.16} +1/32 z={:.16} {} stall23={stall23} 210={} 114={} 74={}",
                z,
                rr.z,
                tag,
                class_of(t2, w2, cody(rr.z, &C0, &D0, 0x210)),
                class_of(t2, w2, cody(rr.z, &c114, &D0, 0x210)),
                class_of(t2, w2, cody(rr.z, &C0, &D0, 0x74))
            );
        } else {
            n_p32_miss += 1;
            println!("  z={:.16} +1/32 z={z2:.16} not-DIRECT stall23={stall23}", z);
        }
    }
    println!(
        "fused stall k=-2/-1=1L +1/32 DIRECT={n_p32} fuse0m1={n_p32_0m1} *H={n_p32_hi} miss={n_p32_miss}"
    );
    println!("fuse0m1 stall k=+2/+3=2H:");
    let mut n_0m1_s23 = 0usize;
    let mut n_0m1 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, -1)), t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, 1)), t).unwrap_or(99) == 0 {
            continue;
        }
        n_0m1 += 1;
        let s23 = signed(mul(w, poke(fu, 2)), t) == "2H"
            && signed(mul(w, poke(fu, 3)), t) == "2H";
        if s23 {
            n_0m1_s23 += 1;
            let z1 = z - 1.0 / 32.0;
            let z3 = z + 1.0 / 32.0;
            let m32 = rows
                .iter()
                .find(|x| x.direct && (x.z - z1).abs() < 1e-12)
                .map(|rr| {
                    class_of(
                        f64::from_bits(rr.qbits),
                        f::w_rn53(rr.z),
                        cody(rr.z, &C0, &D0, 0),
                    )
                })
                .unwrap_or_else(|| "not-DIRECT".into());
            let p32 = rows
                .iter()
                .find(|x| x.direct && (x.z - z3).abs() < 1e-12)
                .map(|rr| {
                    class_of(
                        f64::from_bits(rr.qbits),
                        f::w_rn53(rr.z),
                        cody(rr.z, &C0, &D0, 0),
                    )
                })
                .unwrap_or_else(|| "not-DIRECT".into());
            println!(
                "  z={:.16} 1/32-={m32} 1/32+={p32} w-1={} F-1={}",
                z,
                hit(mul(w.next_down(), fu), t),
                hit(mul(w, fu.next_down()), t)
            );
        }
    }
    println!("fuse0m1 n={n_0m1} stall k=+2/+3=2H n={n_0m1_s23}");
    println!("pin 2.552083 vs 2.583333 vs 2.614583 vs 2.520833:");
    let pinz = [
        2.5208333333333335,
        2.5520833333333335,
        2.5833333333333335,
        2.6145833333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        println!(
            "  z={:.16} bits={:#x} 555={is555} unmask={} 210={} 114={} 74={} 210dF={} 114dF={} 74dF={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f114),
            class_of(t, w, f74),
            ulp_distance(fu, f210).unwrap_or(99),
            ulp_distance(fu, f114).unwrap_or(99),
            ulp_distance(fu, f74).unwrap_or(99)
        );
        for (name, ff) in [("unm", fu), ("210", f210), ("74", f74), ("114", f114)] {
            print!("    {name} {} store", class_of(t, w, ff));
            for k in -3i32..=3 {
                print!(" {k}:{}", signed(mul(w, poke(ff, k)), t));
            }
            print!(" mul");
            let gg = mul(w, ff);
            for k in -3i32..=3 {
                print!(" {k}:{}", signed(poke(gg, k), t));
            }
            println!(
                " tmu={} tmd={} w+1={} F+1={} w-1={} F-1={}",
                hit(mul(w.next_up(), ff.next_up()), t),
                hit(mul(w.next_down(), ff.next_down()), t),
                hit(mul(w.next_up(), ff), t),
                hit(mul(w, ff.next_up()), t),
                hit(mul(w.next_down(), ff), t),
                hit(mul(w, ff.next_down()), t)
            );
        }
    }
    println!("pin stall k=+2 2H +1/32 analogs 1.302083/2.09375/3.552083:");
    let pinz = [
        1.3020833333333333,
        1.3333333333333333,
        1.34375,
        2.09375,
        2.125,
        3.5520833333333335,
        3.5833333333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        let f114 = cody(z, &c114, &D0, 0x210);
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        println!(
            "  z={:.16} bits={:#x} 555={is555} unmask={} 210={} 114={} 74={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f114),
            class_of(t, w, f74)
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w+1={} F+1={} w-1={} F-1={} tmu={} tmd={}",
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t),
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_down(), fu.next_down()), t)
        );
        print!("    114 store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        println!(
            " 114dF={} 210dF={} 74dF={}",
            ulp_distance(fu, f114).unwrap_or(99),
            ulp_distance(fu, f210).unwrap_or(99),
            ulp_distance(fu, f74).unwrap_or(99)
        );
    }
    println!("pin fuse0m1 stall k=+2/+3=2H n=3 last-store:");
    let pinz = [
        1.0833333333333333,
        2.4375,
        2.5833333333333335,
        1.0520833333333333,
        1.1145833333333333,
        2.40625,
        2.46875,
        2.6145833333333335,
        2.5520833333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        println!(
            "  z={:.16} bits={:#x} 555={is555} {}",
            z,
            z.to_bits(),
            class_of(t, w, fu)
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w+1={} F+1={} w-1={} F-1={}",
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("pin 1.270833 vs 1.302083 vs 1.333333 vs 1.34375:");
    let pinz = [
        1.2708333333333333,
        1.3020833333333333,
        1.3333333333333333,
        1.34375,
        1.3645833333333333,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        println!(
            "  z={:.16} bits={:#x} 555={is555} unmask={} 114={} 210={} 74={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f114),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x74))
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w-1={} F-1={} tmd={} 114dF={}",
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t),
            hit(mul(w.next_down(), fu.next_down()), t),
            ulp_distance(fu, f114).unwrap_or(99)
        );
    }
    println!("pin 1.0 vs 1.03125 vs 1.0625 analog:");
    let pinz = [1.0, 1.03125, 1.0625];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        println!("  z={:.16} bits={:#x} {}", z, z.to_bits(), class_of(t, w, fu));
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w+1={} F+1={} w-1={} F-1={}",
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("555 DIRECT mid +1/32 class pairs:");
    let mut n_555 = 0usize;
    let mut n_pair = 0usize;
    let mut n_same = 0usize;
    let mut n_fuse_0m1 = 0usize;
    let mut n_fuse_2h = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let hx = format!("{:x}", r.z.to_bits());
        if !(hx.contains("55555555") || hx.contains("aaaaaaaa")) {
            continue;
        }
        n_555 += 1;
        let z2 = r.z + 1.0 / 32.0;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let fu = cody(r.z, &C0, &D0, 0);
        let c0 = class_of(t, w, fu);
        if let Some(rr) = rows.iter().find(|x| x.direct && (x.z - z2).abs() < 1e-12) {
            n_pair += 1;
            let t2 = f64::from_bits(rr.qbits);
            let w2 = f::w_rn53(rr.z);
            let fu2 = cody(rr.z, &C0, &D0, 0);
            let c1 = class_of(t2, w2, fu2);
            if c0 == c1 {
                n_same += 1;
            }
            if c0 == "fuse" && c1 == "fuse0m1" {
                n_fuse_0m1 += 1;
            }
            if c0 == "fuse" && (c1 == "2H" || c1.ends_with("H")) {
                n_fuse_2h += 1;
            }
            let interesting = c0 != c1
                && (c0.starts_with("fuse")
                    || c1.starts_with("fuse")
                    || c0.contains('H') && c1.contains('L')
                    || c0.contains('L') && c1.contains('H'));
            if interesting {
                println!(
                    "  z={:.16} {} → +1/32 z={:.16} {}",
                    r.z, c0, rr.z, c1
                );
            }
        }
    }
    println!(
        "555 DIRECT mid n={n_555} +1/32 pair={n_pair} same-class={n_same} fuse→fuse0m1={n_fuse_0m1} fuse→*H={n_fuse_2h}"
    );
    println!("fused +1/32 class (all DIRECT mid):");
    let mut n_f = 0usize;
    let mut n_f_pair = 0usize;
    let mut n_f_0m1 = 0usize;
    let mut n_f_01 = 0usize;
    let mut n_f_fuse = 0usize;
    let mut n_f_h = 0usize;
    let mut n_f_l = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        n_f += 1;
        let z2 = z + 1.0 / 32.0;
        if let Some(rr) = rows.iter().find(|x| x.direct && (x.z - z2).abs() < 1e-12) {
            n_f_pair += 1;
            let t2 = f64::from_bits(rr.qbits);
            let w2 = f::w_rn53(rr.z);
            let fu2 = cody(rr.z, &C0, &D0, 0);
            let c1 = class_of(t2, w2, fu2);
            if c1 == "fuse0m1" {
                n_f_0m1 += 1;
                let s23 = signed(mul(w, poke(fu, 2)), t) == "2H"
                    && signed(mul(w, poke(fu, 3)), t) == "2H";
                let hx = format!("{:x}", z.to_bits());
                let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
                println!(
                    "  fuse→fuse0m1 z={:.16} +1/32 z={:.16} 555={is555} stall23={s23} w-1={} F-1={} k-2={} k-1={} k+1={} k+2={}",
                    z,
                    rr.z,
                    hit(mul(w.next_down(), fu), t),
                    hit(mul(w, fu.next_down()), t),
                    signed(mul(w, poke(fu, -2)), t),
                    signed(mul(w, poke(fu, -1)), t),
                    signed(mul(w, poke(fu, 1)), t),
                    signed(mul(w, poke(fu, 2)), t)
                );
            } else if c1 == "fuse01" || c1 == "fuse01m1" {
                n_f_01 += 1;
            } else if c1.starts_with("fuse") {
                n_f_fuse += 1;
            } else if c1.contains('H') {
                n_f_h += 1;
            } else {
                n_f_l += 1;
            }
        }
    }
    println!(
        "fused n={n_f} +1/32 pair={n_f_pair} fuse0m1={n_f_0m1} fuse01={n_f_01} stay-fuse={n_f_fuse} *H={n_f_h} *L={n_f_l}"
    );
    println!("pin 2.177083 vs 2.208333 vs 2.239583 vs 2.270833:");
    let pinz = [
        2.1770833333333335,
        2.2083333333333335,
        2.2395833333333335,
        2.2708333333333335,
        2.3020833333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        println!(
            "  z={:.16} bits={:#x} 555={is555} {}",
            z,
            z.to_bits(),
            class_of(t, w, fu)
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w+1={} F+1={} w-1={} F-1={} tmd={}",
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t),
            hit(mul(w.next_down(), fu.next_down()), t)
        );
    }
    println!("pin 1.0625 vs 1.270833 1H-km1 double k=-2/-1:");
    let pinz = [1.0625, 1.2708333333333333];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        println!(
            "  z={:.16} bits={:#x} {} tmd={} tmu={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_up(), fu.next_up()), t)
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w-1={} F-1={} 114={}",
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t),
            class_of(t, w, cody(z, &c114, &D0, 0x210))
        );
    }
    println!("1H-km1 last-store k=-2 exact:");
    let mut n_1h = 0usize;
    let mut n_k2 = 0usize;
    let mut n_k2_tmd = 0usize;
    let mut n_dbl = 0usize;
    let mut n_dbl_tmd = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, -1)), t).unwrap_or(99) != 0 {
            continue;
        }
        n_1h += 1;
        let k2 = ulp_distance(mul(w, poke(fu, -2)), t).unwrap_or(99) == 0;
        let tmd = hit(mul(w.next_down(), fu.next_down()), t);
        if k2 {
            n_k2 += 1;
            if tmd {
                n_k2_tmd += 1;
            }
            n_dbl += 1;
            if tmd {
                n_dbl_tmd += 1;
            }
            let hx = format!("{:x}", z.to_bits());
            let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
            println!(
                "  z={:.16} 555={is555} tmd={tmd} w-1={} F-1={} k+1={} k+2={} k+3={} mul_k-1={} mul_k-2={} 114={}",
                z,
                hit(mul(w.next_down(), fu), t),
                hit(mul(w, fu.next_down()), t),
                signed(mul(w, poke(fu, 1)), t),
                signed(mul(w, poke(fu, 2)), t),
                signed(mul(w, poke(fu, 3)), t),
                signed(poke(g, -1), t),
                signed(poke(g, -2), t),
                class_of(t, w, cody(z, &c114, &D0, 0x210))
            );
        }
    }
    println!(
        "1H-km1 n={n_1h} k=-2 exact={n_k2} tmd={n_k2_tmd} double k=-2/-1={n_dbl} tmd={n_dbl_tmd}"
    );
    println!("strict fuse (not fuse0m1/01) +1/32 fuse0m1:");
    let mut n_strict = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        if ulp_distance(g, t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, -1)), t).unwrap_or(99) == 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, 1)), t).unwrap_or(99) == 0 {
            continue;
        }
        let z2 = z + 1.0 / 32.0;
        if let Some(rr) = rows.iter().find(|x| x.direct && (x.z - z2).abs() < 1e-12) {
            let t2 = f64::from_bits(rr.qbits);
            let w2 = f::w_rn53(rr.z);
            let fu2 = cody(rr.z, &C0, &D0, 0);
            if class_of(t2, w2, fu2) == "fuse0m1" {
                n_strict += 1;
                println!(
                    "  z={:.16} store k-2={} k-1={} k+2={} k+3={} mul==next {}",
                    z,
                    signed(mul(w, poke(fu, -2)), t),
                    signed(mul(w, poke(fu, -1)), t),
                    signed(mul(w, poke(fu, 2)), t),
                    signed(mul(w, poke(fu, 3)), t),
                    class_of(t2, w2, fu2)
                );
            }
        }
    }
    println!("strict fuse +1/32 fuse0m1 n={n_strict}");
    println!("1H-km1 k=-2 exact ±1/32 class:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d == 1 && g > t) {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, -1)), t).unwrap_or(99) != 0 {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, -2)), t).unwrap_or(99) != 0 {
            continue;
        }
        let tmd = hit(mul(w.next_down(), fu.next_down()), t);
        print!(
            "  z={:.16} tmd={tmd} 114={}",
            z,
            class_of(t, w, cody(z, &c114, &D0, 0x210))
        );
        for (step, lab) in [(1.0 / 32.0, "+1/32"), (-1.0 / 32.0, "-1/32")] {
            let z2 = z + step;
            if let Some(rr) = rows.iter().find(|x| x.direct && (x.z - z2).abs() < 1e-12) {
                let t2 = f64::from_bits(rr.qbits);
                let w2 = f::w_rn53(rr.z);
                let fu2 = cody(rr.z, &C0, &D0, 0);
                print!(" {lab}={} z={:.16}", class_of(t2, w2, fu2), rr.z);
            } else {
                print!(" {lab}=not-DIRECT");
            }
        }
        println!();
    }
    println!("pin 2.145833 vs 2.177083 vs 2.208333 vs 2.239583 vs 2.270833:");
    let pinz = [
        2.1458333333333335,
        2.1770833333333335,
        2.2083333333333335,
        2.2395833333333335,
        2.2708333333333335,
        2.3020833333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} unmask={} 114={} 114dF={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f114),
            ulp_distance(fu, f114).unwrap_or(99)
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmd={} w-1={} F-1={}",
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
        print!("    114 store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        println!();
    }
    println!("pin 2.114583 vs 2.333333 vs 2.364583 chain ends:");
    let pinz = [
        2.1145833333333335,
        2.3333333333333335,
        2.3645833333333335,
        2.3958333333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let f210 = cody(z, &C0, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} unmask={} 210={} 114={} 74={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f114),
            class_of(t, w, cody(z, &C0, &D0, 0x74))
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmd={} tmu={} w-1={} F-1={} 114dF={} 210dF={}",
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t),
            ulp_distance(fu, f114).unwrap_or(99),
            ulp_distance(fu, f210).unwrap_or(99)
        );
    }
    println!("leftover-high tmd last-store +1/32:");
    let mut n_hi = 0usize;
    let mut n_tmd = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d >= 2 && g > t) {
            continue;
        }
        n_hi += 1;
        let tmd = hit(mul(w.next_down(), fu.next_down()), t);
        if !tmd {
            continue;
        }
        n_tmd += 1;
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        let z2 = z + 1.0 / 32.0;
        let p32 = rows
            .iter()
            .find(|x| x.direct && (x.z - z2).abs() < 1e-12)
            .map(|rr| {
                class_of(
                    f64::from_bits(rr.qbits),
                    f::w_rn53(rr.z),
                    cody(rr.z, &C0, &D0, 0),
                )
            })
            .unwrap_or_else(|| "not-DIRECT".into());
        print!(
            "  z={:.16} 555={is555} d={d} +1/32={p32} w-1={} F-1={} store",
            z,
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " 114={} 210={}",
            class_of(t, w, cody(z, &c114, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x210))
        );
    }
    println!("leftover-high n={n_hi} tmd={n_tmd}");
    println!("pin leftover-high tmd +1/32 neighbors 2.083333/2.53125/2.375:");
    let pinz = [
        2.0520833333333335,
        2.0833333333333335,
        2.375,
        2.40625,
        2.5,
        2.53125,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        println!(
            "  z={:.16} bits={:#x} {}",
            z,
            z.to_bits(),
            class_of(t, w, fu)
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmd={} tmu={} w-1={} F-1={} 114={}",
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t),
            class_of(t, w, cody(z, &c114, &D0, 0x210))
        );
    }
    println!("leftover-low tmu last-store +1/32:");
    let mut n_lo = 0usize;
    let mut n_tmu = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d >= 2 && g < t) {
            continue;
        }
        n_lo += 1;
        let tmu = hit(mul(w.next_up(), fu.next_up()), t);
        if !tmu {
            continue;
        }
        n_tmu += 1;
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        let z2 = z + 1.0 / 32.0;
        let p32 = rows
            .iter()
            .find(|x| x.direct && (x.z - z2).abs() < 1e-12)
            .map(|rr| {
                class_of(
                    f64::from_bits(rr.qbits),
                    f::w_rn53(rr.z),
                    cody(rr.z, &C0, &D0, 0),
                )
            })
            .unwrap_or_else(|| "not-DIRECT".into());
        print!(
            "  z={:.16} 555={is555} d={d} +1/32={p32} w+1={} F+1={} store",
            z,
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " 114={} 210={}",
            class_of(t, w, cody(z, &c114, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x210))
        );
    }
    println!("leftover-low n={n_lo} tmu={n_tmu}");
    println!("pin leftover-low tmu +1/32 fuse n=4:");
    let pinz = [
        1.8333333333333333,
        1.8645833333333333,
        2.3125,
        2.34375,
        2.4583333333333335,
        2.4895833333333335,
        3.875,
        3.90625,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f74 = cody(z, &C0, &D0, 0x74);
        println!(
            "  z={:.16} bits={:#x} unmask={} 74={} 114={} 210={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f74),
            class_of(t, w, cody(z, &c114, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x210))
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmu={} w+1={} F+1={} 74dF={}",
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            ulp_distance(fu, f74).unwrap_or(99)
        );
    }
    println!("pin leftover-high tmd pattern A 2.114583 vs 2.520833 w/F bits:");
    let pinz = [
        2.1145833333333335,
        2.1458333333333335,
        2.5208333333333335,
        2.5520833333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        println!(
            "  z={:.16} bits={:#x} w={:#x} F={:#x} Q={:#x} {} w+1={} F+1={} w-1={} F-1={}",
            z,
            z.to_bits(),
            w.to_bits(),
            fu.to_bits(),
            t.to_bits(),
            class_of(t, w, fu),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
        print!("    store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmd={} tz_z={} tz_w={} tz_F={}",
            hit(mul(w.next_down(), fu.next_down()), t),
            z.to_bits().trailing_zeros(),
            w.to_bits().trailing_zeros(),
            fu.to_bits().trailing_zeros()
        );
    }
    println!("leftover-high tmd tz_F tz_w +1/32:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d >= 2 && g > t) {
            continue;
        }
        if !hit(mul(w.next_down(), fu.next_down()), t) {
            continue;
        }
        let z2 = z + 1.0 / 32.0;
        let p32 = rows
            .iter()
            .find(|x| x.direct && (x.z - z2).abs() < 1e-12)
            .map(|rr| {
                class_of(
                    f64::from_bits(rr.qbits),
                    f::w_rn53(rr.z),
                    cody(rr.z, &C0, &D0, 0),
                )
            })
            .unwrap_or_else(|| "not-DIRECT".into());
        println!(
            "  z={:.16} d={d} +1/32={p32} tz_z={} tz_w={} tz_F={} tz_Q={} F={:#x}",
            z,
            z.to_bits().trailing_zeros(),
            w.to_bits().trailing_zeros(),
            fu.to_bits().trailing_zeros(),
            t.to_bits().trailing_zeros(),
            fu.to_bits()
        );
    }
    println!("leftover-low tmu tz_F tz_w +1/32:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d >= 2 && g < t) {
            continue;
        }
        if !hit(mul(w.next_up(), fu.next_up()), t) {
            continue;
        }
        let z2 = z + 1.0 / 32.0;
        let p32 = rows
            .iter()
            .find(|x| x.direct && (x.z - z2).abs() < 1e-12)
            .map(|rr| {
                class_of(
                    f64::from_bits(rr.qbits),
                    f::w_rn53(rr.z),
                    cody(rr.z, &C0, &D0, 0),
                )
            })
            .unwrap_or_else(|| "not-DIRECT".into());
        println!(
            "  z={:.16} d={d} +1/32={p32} tz_z={} tz_w={} tz_F={} tz_Q={}",
            z,
            z.to_bits().trailing_zeros(),
            w.to_bits().trailing_zeros(),
            fu.to_bits().trailing_zeros(),
            t.to_bits().trailing_zeros()
        );
    }
    println!("pin leftover-high tmd pattern B 2.375 vs 2.5 w/F bits:");
    let pinz = [2.375, 2.40625, 2.5, 2.53125];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        println!(
            "  z={:.16} bits={:#x} w={:#x} F={:#x} Q={:#x} {} tz_z={} tz_w={} tz_F={} tz_Q={}",
            z,
            z.to_bits(),
            w.to_bits(),
            fu.to_bits(),
            t.to_bits(),
            class_of(t, w, fu),
            z.to_bits().trailing_zeros(),
            w.to_bits().trailing_zeros(),
            fu.to_bits().trailing_zeros(),
            t.to_bits().trailing_zeros()
        );
        print!("    store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmd={} tmu={} w+1={} F+1={} w-1={} F-1={}",
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("pin leftover-high tmd pattern B 555 2.052083 vs 2.364583:");
    let pinz = [
        2.0520833333333335,
        2.0833333333333335,
        2.3645833333333335,
        2.3958333333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} w={:#x} F={:#x} {} 210={} 114={} tz_z={} tz_w={} tz_F={} tz_Q={} 210dF={} 114dF={}",
            z,
            z.to_bits(),
            w.to_bits(),
            fu.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f114),
            z.to_bits().trailing_zeros(),
            w.to_bits().trailing_zeros(),
            fu.to_bits().trailing_zeros(),
            t.to_bits().trailing_zeros(),
            ulp_distance(fu, f210).unwrap_or(99),
            ulp_distance(fu, f114).unwrap_or(99)
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmd={} tmu={} w-1={} F-1={} w+1={} F+1={}",
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
    }
    println!("leftover-low by d + last-store k=+3 exact:");
    let mut n_lo = 0usize;
    let mut by_d = [0usize; 8];
    let mut n_k3 = 0usize;
    let mut n_k3_114 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d >= 2 && g < t) {
            continue;
        }
        n_lo += 1;
        if d < 8 {
            by_d[d as usize] += 1;
        }
        let k3 = ulp_distance(mul(w, poke(fu, 3)), t).unwrap_or(99) == 0;
        if k3 {
            n_k3 += 1;
            let f114 = cody(z, &c114, &D0, 0x210);
            let c114s = class_of(t, w, f114);
            if c114s != class_of(t, w, fu) {
                n_k3_114 += 1;
            }
            let hx = format!("{:x}", z.to_bits());
            let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
            println!(
                "  z={:.16} 555={is555} d={d} 114={c114s} 210={} 114dF={} tmu={} store k+2={} k+3={} k+4={}",
                z,
                class_of(t, w, cody(z, &C0, &D0, 0x210)),
                ulp_distance(fu, f114).unwrap_or(99),
                hit(mul(w.next_up(), fu.next_up()), t),
                signed(mul(w, poke(fu, 2)), t),
                signed(mul(w, poke(fu, 3)), t),
                signed(mul(w, poke(fu, 4)), t)
            );
        }
    }
    print!("leftover-low n={n_lo} by d");
    for (i, c) in by_d.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!(" last-store k=+3 exact={n_k3} 114 class-change={n_k3_114}");
    println!("leftover-low d>=3:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d >= 3 && g < t) {
            continue;
        }
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        println!(
            "  z={:.16} 555={is555} d={d} {} 114={} 210={} tmu={} k+1={} k+2={} k+3={}",
            z,
            class_of(t, w, fu),
            class_of(t, w, cody(z, &c114, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            hit(mul(w.next_up(), fu.next_up()), t),
            signed(mul(w, poke(fu, 1)), t),
            signed(mul(w, poke(fu, 2)), t),
            signed(mul(w, poke(fu, 3)), t)
        );
    }
    println!("pin 2.364583 vs 2.395833 vs 2.427083 last-store:");
    let pinz = [
        2.3645833333333335,
        2.3958333333333335,
        2.4270833333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let f210 = cody(z, &C0, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} unmask={} 210={} 114={} 114dF={} 210dF={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f114),
            ulp_distance(fu, f114).unwrap_or(99),
            ulp_distance(fu, f210).unwrap_or(99)
        );
        print!("    unm store");
        for k in -3i32..=4 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=4 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!();
        print!("    114 store");
        for k in -3i32..=4 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        println!();
    }
    println!("pin leftover-low 3L n=3 2.65625 vs 2.395833 vs 2.427083:");
    let pinz = [
        2.3958333333333335,
        2.4270833333333335,
        2.65625,
        2.6875,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} unmask={} 114={} 210={} tz_z={} tmu={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f114),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            z.to_bits().trailing_zeros(),
            hit(mul(w.next_up(), fu.next_up()), t)
        );
        print!("    unm store");
        for k in -3i32..=4 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=4 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w+1={} F+1={}",
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
    }
    println!("pin leftover-low 4L n=2 3.0625 vs 3.75:");
    let pinz = [3.0625, 3.75, 3.09375, 3.78125];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} unmask={} 114={} 210={} tz_z={} tmu={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f114),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            z.to_bits().trailing_zeros(),
            hit(mul(w.next_up(), fu.next_up()), t)
        );
        print!("    unm store");
        for k in -3i32..=4 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=4 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w+1={} F+1={} 114dF={}",
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            ulp_distance(fu, f114).unwrap_or(99)
        );
    }
    println!("leftover-high by d + d>=3:");
    let mut n_hi = 0usize;
    let mut by_d = [0usize; 8];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d >= 2 && g > t) {
            continue;
        }
        n_hi += 1;
        if d < 8 {
            by_d[d as usize] += 1;
        }
        if d >= 3 {
            let hx = format!("{:x}", z.to_bits());
            let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
            println!(
                "  z={:.16} 555={is555} d={d} {} 114={} 210={} tmd={} k-1={} k-2={} k-3={}",
                z,
                class_of(t, w, fu),
                class_of(t, w, cody(z, &c114, &D0, 0x210)),
                class_of(t, w, cody(z, &C0, &D0, 0x210)),
                hit(mul(w.next_down(), fu.next_down()), t),
                signed(mul(w, poke(fu, -1)), t),
                signed(mul(w, poke(fu, -2)), t),
                signed(mul(w, poke(fu, -3)), t)
            );
        }
    }
    print!("leftover-high n={n_hi} by d");
    for (i, c) in by_d.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!();
    println!("pin 3.5625 leftover-high 4H 114=2H:");
    let pinz = [3.53125, 3.5520833333333335, 3.5625, 3.59375];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let f210 = cody(z, &C0, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} unmask={} 210={} 114={} 74={} 114dF={} 210dF={} tz_z={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f114),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            ulp_distance(fu, f114).unwrap_or(99),
            ulp_distance(fu, f210).unwrap_or(99),
            z.to_bits().trailing_zeros()
        );
        print!("    unm store");
        for k in -4i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -4i32..=2 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!();
        print!("    114 store");
        for k in -4i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        println!(
            " tmd={} w-1={} F-1={}",
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("leftover-high last-store k=-3 exact:");
    let mut n_k3 = 0usize;
    let mut n_k3_114 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d >= 2 && g > t) {
            continue;
        }
        if ulp_distance(mul(w, poke(fu, -3)), t).unwrap_or(99) != 0 {
            continue;
        }
        n_k3 += 1;
        let f114 = cody(z, &c114, &D0, 0x210);
        let c114s = class_of(t, w, f114);
        if c114s != class_of(t, w, fu) {
            n_k3_114 += 1;
        }
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        let z2 = z + 1.0 / 32.0;
        let p32 = rows
            .iter()
            .find(|x| x.direct && (x.z - z2).abs() < 1e-12)
            .map(|rr| {
                class_of(
                    f64::from_bits(rr.qbits),
                    f::w_rn53(rr.z),
                    cody(rr.z, &C0, &D0, 0),
                )
            })
            .unwrap_or_else(|| "not-DIRECT".into());
        println!(
            "  z={:.16} 555={is555} d={d} 114={c114s} 210={} tmd={} +1/32={p32} tz_z={} k-1={} k-2={} k-4={} mul_k-3={}",
            z,
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            hit(mul(w.next_down(), fu.next_down()), t),
            z.to_bits().trailing_zeros(),
            signed(mul(w, poke(fu, -1)), t),
            signed(mul(w, poke(fu, -2)), t),
            signed(mul(w, poke(fu, -4)), t),
            signed(poke(g, -3), t)
        );
    }
    println!("leftover-high last-store k=-3 exact n={n_k3} 114 class-change={n_k3_114}");
    println!("pin leftover-high 3H k=-3 exact n=4 last-store:");
    let pinz = [
        1.875,
        1.9895833333333333,
        2.125,
        3.40625,
        1.84375,
        1.90625,
        1.9583333333333333,
        2.0208333333333335,
        2.09375,
        2.15625,
        3.375,
        3.4375,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let f210 = cody(z, &C0, &D0, 0x210);
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        println!(
            "  z={:.16} bits={:#x} 555={is555} unmask={} 210={} 114={} 74={} 114dF={} 210dF={} tz_z={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f114),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            ulp_distance(fu, f114).unwrap_or(99),
            ulp_distance(fu, f210).unwrap_or(99),
            z.to_bits().trailing_zeros()
        );
        print!("    unm store");
        for k in -4i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -4i32..=2 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmd={} w-1={} F-1={} w+1={} F+1={}",
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
        print!("    114 store");
        for k in -4i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        println!();
    }
    println!("pin 2.645833 leftover-high 2H k=-3 vs 2.52 chain tail:");
    let pinz = [
        2.6145833333333335,
        2.6458333333333335,
        2.6770833333333335,
        2.7083333333333335,
        2.125,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        println!(
            "  z={:.16} bits={:#x} 555={is555} unmask={} 114={} 210={} 74={} 114dF={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f114),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            ulp_distance(fu, f114).unwrap_or(99)
        );
        print!("    unm store");
        for k in -4i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -4i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmd={} tmu={} w-1={} F-1={}",
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("pin 2.52 555 chain 2.708333..2.802083:");
    let pinz = [
        2.7083333333333335,
        2.7395833333333335,
        2.7708333333333335,
        2.8020833333333335,
        2.8333333333333335,
        2.8645833333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} unmask={} 114={} 210={} 114dF={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f114),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            ulp_distance(fu, f114).unwrap_or(99)
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmu={} tmd={} w+1={} F+1={} w-1={} F-1={}",
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("pin 2.739583 vs 2.770833 114 inverse 1H-km1 fuse:");
    let pinz = [2.7395833333333335, 2.7708333333333335];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let f210 = cody(z, &C0, &D0, 0x210);
        println!(
            "  z={:.16} unmask={} 210={} 114={} 114dF={} 210dF={} 114>unm={} 210>unm={}",
            z,
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f114),
            ulp_distance(fu, f114).unwrap_or(99),
            ulp_distance(fu, f210).unwrap_or(99),
            f114 > fu,
            f210 > fu
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!();
        print!("    114 store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        print!(" mul");
        let g114 = mul(w, f114);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g114, k), t));
        }
        println!();
        print!("    210 store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(f210, k)), t));
        }
        println!();
    }
    println!("1H-km1 +1/32 fuse 114 inverse pairs:");
    let mut n_pair = 0usize;
    let mut n_inv = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let cu = class_of(t, w, fu);
        if cu != "1H-km1" {
            continue;
        }
        let z2 = z + 1.0 / 32.0;
        let Some(rr) = rows.iter().find(|x| x.direct && (x.z - z2).abs() < 1e-12) else {
            continue;
        };
        let t2 = f64::from_bits(rr.qbits);
        let w2 = f::w_rn53(rr.z);
        let fu2 = cody(rr.z, &C0, &D0, 0);
        let c2 = class_of(t2, w2, fu2);
        if !c2.starts_with("fuse") {
            continue;
        }
        n_pair += 1;
        let f114 = cody(z, &c114, &D0, 0x210);
        let f114b = cody(rr.z, &c114, &D0, 0x210);
        let c114a = class_of(t, w, f114);
        let c114b = class_of(t2, w2, f114b);
        let inv = c114a.starts_with("fuse") && (c114b == "1H-km1" || c114b.starts_with("1H"));
        if inv {
            n_inv += 1;
        }
        println!(
            "  z={:.16} {} 114={} +1/32 z={:.16} {} 114={} inv={inv} 114dF={} {} 114>={}",
            z,
            cu,
            c114a,
            rr.z,
            c2,
            c114b,
            ulp_distance(fu, f114).unwrap_or(99),
            ulp_distance(fu2, f114b).unwrap_or(99),
            f114 > fu
        );
    }
    println!("1H-km1 +1/32 fuse n={n_pair} 114 inverse={n_inv}");
    println!("pin 3.0 vs 3.03125 114 fuse not-inverse:");
    let pinz = [3.0, 3.03125, 2.9895833333333335];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} unmask={} 114={} 210={} 114dF={} 114>unm={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f114),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            ulp_distance(fu, f114).unwrap_or(99),
            f114 > fu
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!();
        print!("    114 store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        println!();
    }
    println!("pin 1L-k1 210=114 fuse trio 1.40625/2.020833/3.614583:");
    let pinz = [
        1.40625,
        2.0208333333333335,
        3.6145833333333335,
        1.9895833333333333,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let f210 = cody(z, &C0, &D0, 0x210);
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        println!(
            "  z={:.16} bits={:#x} 555={is555} unmask={} 210={} 114={} 74={} 210dF={} 114dF={} tz_z={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f114),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            ulp_distance(fu, f210).unwrap_or(99),
            ulp_distance(fu, f114).unwrap_or(99),
            z.to_bits().trailing_zeros()
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w+1={} F+1={}",
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
        print!("    114 store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        println!(
            " 210>={} 114>={}",
            f210 > fu,
            f114 > fu
        );
    }
    println!("pin 3.552083..3.645833 and trio ±1/32:");
    let pinz = [
        3.5520833333333335,
        3.5833333333333335,
        3.6145833333333335,
        3.6458333333333335,
        1.375,
        1.40625,
        1.4375,
        1.9895833333333333,
        2.0208333333333335,
        2.0520833333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let f210 = cody(z, &C0, &D0, 0x210);
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        println!(
            "  z={:.16} bits={:#x} 555={is555} unmask={} 210={} 114={} 74={} 210dF={} 114dF={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f114),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            ulp_distance(fu, f210).unwrap_or(99),
            ulp_distance(fu, f114).unwrap_or(99)
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w+1={} F+1={} w-1={} F-1={} tmd={} tmu={}",
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t),
            hit(mul(w.next_down(), fu.next_down()), t),
            hit(mul(w.next_up(), fu.next_up()), t)
        );
        print!("    210 store");
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(f210, k)), t));
        }
        print!(" 114 store");
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        println!(
            " 210>={} 114>={}",
            f210 > fu,
            f114 > fu
        );
    }
    println!("1L-k1 210=fuse ±1/32:");
    let mut n = 0usize;
    let mut n_114 = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if class_of(t, w, fu) != "1L-k1" {
            continue;
        }
        let f210 = cody(z, &C0, &D0, 0x210);
        if class_of(t, w, f210) != "fuse" {
            continue;
        }
        n += 1;
        let f114 = cody(z, &c114, &D0, 0x210);
        let c114s = class_of(t, w, f114);
        if c114s == "fuse" {
            n_114 += 1;
        }
        let mut neigh = String::new();
        for (step, lab) in [(1.0 / 32.0, "+1/32"), (-1.0 / 32.0, "-1/32")] {
            let z2 = z + step;
            if let Some(rr) = rows.iter().find(|x| x.direct && (x.z - z2).abs() < 1e-12) {
                let t2 = f64::from_bits(rr.qbits);
                let w2 = f::w_rn53(rr.z);
                let fu2 = cody(rr.z, &C0, &D0, 0);
                neigh.push_str(&format!(
                    " {lab}={} z={:.16}",
                    class_of(t2, w2, fu2),
                    rr.z
                ));
            } else {
                neigh.push_str(&format!(" {lab}=not-DIRECT"));
            }
        }
        println!(
            "  z={:.16} 114={c114s} 74={} k-2={} k+2={} 210dF={}{neigh}",
            z,
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            signed(mul(w, poke(fu, -2)), t),
            signed(mul(w, poke(fu, 2)), t),
            ulp_distance(fu, f210).unwrap_or(99)
        );
    }
    println!("1L-k1 210=fuse n={n} also 114 fuse={n_114}");
    println!("pin 3.875..3.958333 3.927083 210-only chain:");
    let pinz = [
        3.875,
        3.8958333333333335,
        3.90625,
        3.9270833333333335,
        3.9583333333333335,
        3.9895833333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f74 = cody(z, &C0, &D0, 0x74);
        println!(
            "  z={:.16} bits={:#x} unmask={} 210={} 114={} 74={} 210dF={} 114dF={} 74dF={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f114),
            class_of(t, w, f74),
            ulp_distance(fu, f210).unwrap_or(99),
            ulp_distance(fu, f114).unwrap_or(99),
            ulp_distance(fu, f74).unwrap_or(99)
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        let g = mul(w, fu);
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " w+1={} F+1={}",
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
        print!("    210 store");
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(f210, k)), t));
        }
        print!(" 74 store");
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(f74, k)), t));
        }
        println!(
            " 210>={} 74>={}",
            f210 > fu,
            f74 > fu
        );
    }
    println!("1H-km1 210=fuse 210 last-store stall k=-2/-1=1L:");
    let mut n = 0usize;
    let mut n_stall = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if class_of(t, w, fu) != "1H-km1" {
            continue;
        }
        let f210 = cody(z, &C0, &D0, 0x210);
        if class_of(t, w, f210) != "fuse" {
            continue;
        }
        n += 1;
        let s2 = signed(mul(w, poke(f210, -2)), t);
        let s1 = signed(mul(w, poke(f210, -1)), t);
        let stall = s2 == "1L" && s1 == "1L";
        if stall {
            n_stall += 1;
        }
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} stall={stall} 114={} 74={} 210dF={} 210<={} k-2={} k-1={} k+1={} tmd={}",
            z,
            class_of(t, w, f114),
            class_of(t, w, cody(z, &C0, &D0, 0x74)),
            ulp_distance(fu, f210).unwrap_or(99),
            f210 < fu,
            s2,
            s1,
            signed(mul(w, poke(f210, 1)), t),
            hit(mul(w.next_down(), fu.next_down()), t)
        );
    }
    println!("1H-km1 210=fuse n={n} 210 stall k=-2/-1=1L n={n_stall}");
    println!("1H-km1 210=fuse vs z-3/32 fused last-store:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if class_of(t, w, fu) != "1H-km1" {
            continue;
        }
        let f210 = cody(z, &C0, &D0, 0x210);
        if class_of(t, w, f210) != "fuse" {
            continue;
        }
        let z2 = z - 3.0 / 32.0;
        let mut pat210 = String::new();
        for k in -2i32..=1 {
            pat210.push_str(&format!(" {k}:{}", signed(mul(w, poke(f210, k)), t)));
        }
        if let Some(rr) = rows.iter().find(|x| x.direct && (x.z - z2).abs() < 1e-12) {
            let t2 = f64::from_bits(rr.qbits);
            let w2 = f::w_rn53(rr.z);
            let fu2 = cody(rr.z, &C0, &D0, 0);
            let c2 = class_of(t2, w2, fu2);
            let mut patu = String::new();
            for k in -2i32..=1 {
                patu.push_str(&format!(" {k}:{}", signed(mul(w2, poke(fu2, k)), t2)));
            }
            println!(
                "  z={:.16} 210store{pat210} -3/32 z={:.16} {c2} unmstore{patu} same={}",
                z,
                rr.z,
                pat210 == patu
            );
        } else {
            println!("  z={:.16} 210store{pat210} -3/32 not-DIRECT", z);
        }
    }
    println!("pin 2.90625 vs 3.0 vs 3.34375 vs 3.4375 3/32 pairs:");
    let pinz = [2.90625, 3.0, 3.03125, 3.34375, 3.4375, 3.46875];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        println!(
            "  z={:.16} bits={:#x} unmask={} 210={} 114={} tz_z={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, cody(z, &c114, &D0, 0x210)),
            z.to_bits().trailing_zeros()
        );
        print!("    unm store");
        for k in -2i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" 210 store");
        for k in -2i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(f210, k)), t));
        }
        println!(
            " 210dF={} 210<={} w+1={} F+1={} tmd={}",
            ulp_distance(fu, f210).unwrap_or(99),
            f210 < fu,
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t),
            hit(mul(w.next_down(), fu.next_down()), t)
        );
    }
    println!("1H-km1 last-store k=-2 1L k=-1 exact k=0 1H:");
    let mut n = 0usize;
    let mut n_tmd = 0usize;
    let mut n_fm = 0usize;
    let mut n_wm = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if class_of(t, w, fu) != "1H-km1" {
            continue;
        }
        if signed(mul(w, poke(fu, -2)), t) != "1L" {
            continue;
        }
        if signed(mul(w, poke(fu, -1)), t) != "0=" {
            continue;
        }
        if signed(mul(w, poke(fu, 0)), t) != "1H" {
            continue;
        }
        n += 1;
        let g = mul(w, fu);
        let tmd = hit(mul(w.next_down(), fu.next_down()), t);
        let wm = hit(mul(w.next_down(), fu), t);
        let fm = hit(mul(w, fu.next_down()), t);
        if tmd {
            n_tmd += 1;
        }
        if fm {
            n_fm += 1;
        }
        if wm {
            n_wm += 1;
        }
        let hx = format!("{:x}", z.to_bits());
        let is555 = hx.contains("55555555") || hx.contains("aaaaaaaa");
        println!(
            "  z={:.16} 555={is555} tmd={tmd} w-1={wm} F-1={fm} mul_k-1={} mul_k-2={} tz_z={} tz_w={} tz_F={} tz_Q={} 210={} 114={}",
            z,
            signed(poke(g, -1), t),
            signed(poke(g, -2), t),
            z.to_bits().trailing_zeros(),
            w.to_bits().trailing_zeros(),
            fu.to_bits().trailing_zeros(),
            t.to_bits().trailing_zeros(),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &c114, &D0, 0x210))
        );
    }
    println!("1H-km1 store k=-2 1L k=-1 exact n={n} tmd={n_tmd} F-1={n_fm} w-1={n_wm}");
    println!("pin 3.0 vs 3.4375 tmd last-mul F±:");
    let pinz = [3.0, 3.4375];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let tmd = mul(w.next_down(), fu.next_down());
        println!(
            "  z={:.16} bits={:#x} w={:#x} F={:#x} Q={:#x}",
            z,
            z.to_bits(),
            w.to_bits(),
            fu.to_bits(),
            t.to_bits()
        );
        print!("    store");
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmd={} tmd_vs_mul-1={} tmd_vs_store-1={} w-1={} F-1={}",
            hit(tmd, t),
            ulp_distance(tmd, poke(g, -1)).unwrap_or(99),
            ulp_distance(tmd, mul(w, poke(fu, -1))).unwrap_or(99),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("1H-km1 tmd by k=-2:");
    let mut n_1h = 0usize;
    let mut n_tmd = 0usize;
    let mut n_tmd_k2eq = 0usize;
    let mut n_tmd_k2_1l = 0usize;
    let mut n_tmd_other = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        if class_of(t, w, fu) != "1H-km1" {
            continue;
        }
        n_1h += 1;
        if !hit(mul(w.next_down(), fu.next_down()), t) {
            continue;
        }
        n_tmd += 1;
        let k2 = signed(mul(w, poke(fu, -2)), t);
        if k2 == "0=" {
            n_tmd_k2eq += 1;
        } else if k2 == "1L" {
            n_tmd_k2_1l += 1;
        } else {
            n_tmd_other += 1;
        }
        println!(
            "  z={:.16} k-2={k2} w-1={} F-1={} 210={} 114={} tz_w={}",
            z,
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, cody(z, &c114, &D0, 0x210)),
            w.to_bits().trailing_zeros()
        );
    }
    println!(
        "1H-km1 n={n_1h} tmd={n_tmd} k-2 exact={n_tmd_k2eq} k-2 1L={n_tmd_k2_1l} other={n_tmd_other}"
    );
    println!("pin 3.4375 vs 3.645833 1H-km1 tmd k-2 1L:");
    let pinz = [3.4375, 3.6458333333333335, 3.0];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let g = mul(w, fu);
        let tmdv = mul(w.next_down(), fu.next_down());
        println!(
            "  z={:.16} bits={:#x} tz_w={} tz_F={} unmask={} 210={}",
            z,
            z.to_bits(),
            w.to_bits().trailing_zeros(),
            fu.to_bits().trailing_zeros(),
            class_of(t, w, fu),
            class_of(t, w, f210)
        );
        print!("    unm store");
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmd={} tmd==mul-1={} 210store -2:{} -1:{} 0:{}",
            hit(tmdv, t),
            hit(tmdv, poke(g, -1)),
            signed(mul(w, poke(f210, -2)), t),
            signed(mul(w, poke(f210, -1)), t),
            signed(mul(w, poke(f210, 0)), t)
        );
    }
    println!("pin 1H-km1 tmd k=-2 exact n=6:");
    let pinz = [
        0.8437499999999999,
        1.0625,
        1.2708333333333333,
        1.7083333333333333,
        2.25,
        3.15625,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let g = mul(w, fu);
        let tmdv = mul(w.next_down(), fu.next_down());
        println!(
            "  z={:.16} bits={:#x} unmask={} 210={} 114={} tz_w={} 114dF={}",
            z,
            z.to_bits(),
            class_of(t, w, fu),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, f114),
            w.to_bits().trailing_zeros(),
            ulp_distance(fu, f114).unwrap_or(99)
        );
        print!("    unm store");
        for k in -3i32..=1 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" mul");
        for k in -3i32..=1 {
            print!(" {k}:{}", signed(poke(g, k), t));
        }
        println!(
            " tmd==mul-1={} tmd==store-2={} w-1={} F-1={}",
            hit(tmdv, poke(g, -1)),
            hit(tmdv, mul(w, poke(fu, -2))),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("pin 2.250 vs 3.15625 210/114 fuse0m1:");
    let pinz = [2.25, 3.15625];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f210 = cody(z, &C0, &D0, 0x210);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} 210dF={} 114dF={} 210>={} 114>={}",
            z,
            ulp_distance(fu, f210).unwrap_or(99),
            ulp_distance(fu, f114).unwrap_or(99),
            f210 > fu,
            f114 > fu
        );
        print!("    210 store");
        for k in -3i32..=1 {
            print!(" {k}:{}", signed(mul(w, poke(f210, k)), t));
        }
        print!(" 114 store");
        for k in -3i32..=1 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        println!();
    }
    println!("114 fuse0m1 last-store patterns:");
    let mut n = 0usize;
    let mut n_pat = 0usize;
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        if class_of(t, w, f114) != "fuse0m1" {
            continue;
        }
        n += 1;
        let mut pat = String::new();
        for k in -3i32..=2 {
            pat.push_str(&format!(" {k}:{}", signed(mul(w, poke(f114, k)), t)));
        }
        let is_hit = signed(mul(w, poke(f114, -3)), t) == "2L"
            && signed(mul(w, poke(f114, -2)), t) == "1L"
            && signed(mul(w, poke(f114, -1)), t) == "0="
            && signed(mul(w, poke(f114, 0)), t) == "0="
            && signed(mul(w, poke(f114, 1)), t) == "1H";
        if is_hit {
            n_pat += 1;
        }
        let k2 = signed(mul(w, poke(f114, 2)), t);
        let k3 = signed(mul(w, poke(f114, 3)), t);
        println!(
            "  z={:.16} unmask={} 210={} 114dF={} 114>={} match250={is_hit} k+2={k2} k+3={k3} store{pat}",
            z,
            class_of(t, w, fu),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            ulp_distance(fu, f114).unwrap_or(99),
            f114 > fu
        );
    }
    println!("114 fuse0m1 n={n} last-store=-3:2L -2:1L -1/0 exact k+1 1H n={n_pat}");
    println!("pin 2.270833 vs 2.250 vs 1.5 114 fuse0m1 store:");
    let pinz = [
        1.5,
        2.2395833333333335,
        2.25,
        2.2708333333333335,
        3.15625,
        3.3645833333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        let f210 = cody(z, &C0, &D0, 0x210);
        println!(
            "  z={:.16} unmask={} 210={} 114={} 114dF={} tmd={}",
            z,
            class_of(t, w, fu),
            class_of(t, w, f210),
            class_of(t, w, f114),
            ulp_distance(fu, f114).unwrap_or(99),
            hit(mul(w.next_down(), fu.next_down()), t)
        );
        print!("    unm store");
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" 114 store");
        for k in -3i32..=2 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        println!(
            " w-1={} F-1={}",
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("pin 2.270833 vs 2.250 vs 3.15625 tmd vs mul:");
    let pinz = [2.25, 2.2708333333333335, 3.15625];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let g = mul(w, fu);
        let tmdv = mul(w.next_down(), fu.next_down());
        println!(
            "  z={:.16} tz_w={} tz_F={} tmd={} tmd_vs_mul-1={} tmd_vs_store-2={} w-1={} F-1={}",
            z,
            w.to_bits().trailing_zeros(),
            fu.to_bits().trailing_zeros(),
            hit(tmdv, t),
            ulp_distance(tmdv, poke(g, -1)).unwrap_or(99),
            ulp_distance(tmdv, mul(w, poke(fu, -2))).unwrap_or(99),
            hit(mul(w.next_down(), fu), t),
            hit(mul(w, fu.next_down()), t)
        );
    }
    println!("pin 114 fuse0m1 miss n=5 stall23 vs cluster:");
    let pinz = [
        0.7708333333333333,
        0.78125,
        0.8020833333333334,
        1.0833333333333333,
        2.5833333333333335,
        1.0520833333333333,
        1.1145833333333333,
        2.5520833333333335,
        2.6145833333333335,
    ];
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        if !pinz.iter().any(|&z| (r.z - z).abs() < 1e-12) {
            continue;
        }
        let z = r.z;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(z);
        let fu = cody(z, &C0, &D0, 0);
        let f114 = cody(z, &c114, &D0, 0x210);
        println!(
            "  z={:.16} unmask={} 210={} 114={} 114dF={} 114>={}",
            z,
            class_of(t, w, fu),
            class_of(t, w, cody(z, &C0, &D0, 0x210)),
            class_of(t, w, f114),
            ulp_distance(fu, f114).unwrap_or(99),
            f114 > fu
        );
        print!("    unm store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(fu, k)), t));
        }
        print!(" 114 store");
        for k in -3i32..=3 {
            print!(" {k}:{}", signed(mul(w, poke(f114, k)), t));
        }
        println!(
            " w+1={} F+1={}",
            hit(mul(w.next_up(), fu), t),
            hit(mul(w, fu.next_up()), t)
        );
    }
}
