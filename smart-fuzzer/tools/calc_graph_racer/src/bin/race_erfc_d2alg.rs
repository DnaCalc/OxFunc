//! 2870 leftover |k|=±2 two-mode vs last-store of fused F. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use std::env;

const P: [f64; 8] = [
    0.16506148041280876191828601e-03,
    0.15471455377139313353998665e-03,
    0.44852548090298868465196794e-04,
    -0.49177280017226285450486205e-05,
    -0.69353602078656412367801676e-05,
    -0.20508667787746282746857743e-05,
    -0.28982842617824971177267380e-06,
    -0.17272433544836633301127174e-07,
];
const Q: [f64; 8] = [
    1.0,
    0.16272656776533322859856317e+01,
    0.12040996037066026106794322e+01,
    0.52400246352158386907601472e+00,
    0.14497345252798672362384241e+00,
    0.25592517111042546492590736e-01,
    0.26869088293991371028123158e-02,
    0.13133767840925681614496481e-03,
];
const R0: [f64; 9] = [
    0.145589721275038539045668824025,
    -0.273421931495426482902320421863,
    0.226008066916621506788789064272,
    -0.163571895523923805648814425592,
    0.102604312032193978662297299832,
    -0.548023266949835519254211506880e-01,
    0.241432239725390106956523668160e-01,
    -0.822062115403915116036874169600e-02,
    0.180296241564687154310619200000e-02,
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
fn horner(cs: &[f64], x: f64) -> f64 {
    let mut a = 0.0;
    for &c in cs.iter().rev() {
        a = a * x + c;
    }
    a
}
fn derfc0(x: f64, r: &[f64; 9]) -> f64 {
    if x <= 2.0 {
        let u = horner(&P, x);
        let v = horner(&Q, x);
        let t = (x - 3.75) / (x + 3.75);
        let mut acc = u / v;
        for &c in r.iter().rev() {
            acc = acc * t + c;
        }
        acc
    } else {
        f::nswc_derfc0(x)
    }
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut r15 = R0;
    r15[1] = poke(R0[1], 1);
    r15[5] = poke(R0[5], 1);
    r15[6] = poke(R0[6], -1);
    let mut n_k2 = 0usize;
    let mut n_tmu = 0usize;
    let mut n_eq = 0usize;
    let mut n_km2 = 0usize;
    let mut n_tmd = 0usize;
    let mut n_heq = 0usize;
    println!("2870 leftover |k|=±2 two-mode vs last-store of fused F:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = derfc0(z, &R0);
        let gp = mul(w, derfc0(z, &r15));
        let dp = ulp_distance(gp, t).unwrap_or(99);
        let k = k_of(t, w, fu);
        if dp >= 2 && gp < t && k == Some(2) {
            n_k2 += 1;
            let q_k2 = mul(w, poke(fu, 2));
            let q_tmu = mul(w.next_up(), fu.next_up());
            let eq = q_tmu.to_bits() == q_k2.to_bits();
            if eq {
                n_eq += 1;
            }
            if hit(q_tmu, t) {
                n_tmu += 1;
            } else {
                println!(
                    "  LO k=2 not-tmu z={:.16} 555={} tmu_d={} eq_k2={eq} eq_k3={} eq_prod3={}",
                    z,
                    is555(z),
                    ulp_distance(q_tmu, t).unwrap_or(99),
                    q_tmu.to_bits() == mul(w, poke(fu, 3)).to_bits(),
                    q_tmu.to_bits() == poke(mul(w, fu), 3).to_bits()
                );
            }
        } else if dp >= 2 && gp > t && k == Some(-2) {
            n_km2 += 1;
            let q_km2 = mul(w, poke(fu, -2));
            let q_tmd = mul(w.next_down(), fu.next_down());
            let eq = q_tmd.to_bits() == q_km2.to_bits();
            if eq {
                n_heq += 1;
            }
            if hit(q_tmd, t) {
                n_tmd += 1;
            } else {
                println!(
                    "  HI k=-2 not-tmd z={:.16} 555={} tmd_d={} eq_km2={eq} eq_km1={} eq_km3={} eq_prodm3={}",
                    z,
                    is555(z),
                    ulp_distance(q_tmd, t).unwrap_or(99),
                    q_tmd.to_bits() == mul(w, poke(fu, -1)).to_bits(),
                    q_tmd.to_bits() == mul(w, poke(fu, -3)).to_bits(),
                    q_tmd.to_bits() == poke(mul(w, fu), -3).to_bits()
                );
            }
        }
    }
    println!("leftover-low |k|=2 n={n_k2} tmu={n_tmu} Q_tmu==Q_k2 {n_eq}/{n_k2}");
    println!("leftover-high |k|=-2 n={n_km2} tmd={n_tmd} Q_tmd==Q_km2 {n_heq}/{n_km2}");
    let mut nall = 0usize;
    let mut up_p = [0usize; 6];
    let mut dn_p = [0usize; 6];
    let mut up_none = 0usize;
    let mut dn_none = 0usize;
    println!("DIRECT mid fused DERFC0 two-mode vs last-mul of w*F:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let _t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = derfc0(z, &R0);
        nall += 1;
        let prod = mul(w, fu);
        let q_tmu = mul(w.next_up(), fu.next_up());
        let q_tmd = mul(w.next_down(), fu.next_down());
        let mut up = false;
        let mut dn = false;
        for ak in 0i32..=5 {
            if q_tmu.to_bits() == poke(prod, ak).to_bits() {
                up_p[ak as usize] += 1;
                up = true;
            }
            if q_tmd.to_bits() == poke(prod, -ak).to_bits() {
                dn_p[ak as usize] += 1;
                dn = true;
            }
        }
        if !up {
            up_none += 1;
        }
        if !dn {
            dn_none += 1;
        }
    }
    print!("n={nall} two-mode-up==(wF)+k");
    for (i, c) in up_p.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if up_none > 0 {
        print!(" none:{up_none}");
    }
    print!(" down");
    for (i, c) in dn_p.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if dn_none > 0 {
        print!(" none:{dn_none}");
    }
    println!();
    let mut up2870 = [0usize; 6];
    let mut none2870 = 0usize;
    let mut tmu2870 = 0usize;
    let mut n2870 = 0usize;
    println!("DIRECT mid 2870 F two-mode last-mul cover:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fv = derfc0(z, &r15);
        n2870 += 1;
        let prod = mul(w, fv);
        let q_tmu = mul(w.next_up(), fv.next_up());
        if hit(q_tmu, t) {
            tmu2870 += 1;
        }
        let mut ok = false;
        for ak in 0i32..=5 {
            if q_tmu.to_bits() == poke(prod, ak).to_bits() {
                up2870[ak as usize] += 1;
                ok = true;
            }
        }
        if !ok {
            none2870 += 1;
        }
    }
    print!("2870 F n={n2870} tmu_excel={tmu2870} up==(wF)+k");
    for (i, c) in up2870.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if none2870 > 0 {
        print!(" none:{none2870}");
    }
    println!();
    let mut nlo = 0usize;
    let mut lo_k = [0usize; 6];
    let mut lo_ex = 0usize;
    let mut lo_none = 0usize;
    let mut lo_same = 0usize;
    let mut lo_swap = 0usize;
    println!("2870 leftover-low two-mode of 2870 F vs last-mul of w*F_2870:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fv = derfc0(z, &r15);
        let g = mul(w, fv);
        let d = ulp_distance(g, t).unwrap_or(99);
        if !(d >= 2 && g < t) {
            continue;
        }
        nlo += 1;
        let q_tmu = mul(w.next_up(), fv.next_up());
        if hit(q_tmu, t) {
            lo_ex += 1;
            let ks = k_of(t, w, fv);
            let mut km = None;
            for ak in 0i32..=5 {
                if ulp_distance(poke(g, ak), t).unwrap_or(99) == 0 {
                    km = Some(ak);
                    break;
                }
            }
            if ks == km {
                lo_same += 1;
            } else {
                lo_swap += 1;
                println!(
                    "  2870 leftover-low tmu swap z={:.16} store_k={ks:?} mul_k={km:?}",
                    z
                );
            }
        }
        let mut ok = false;
        for ak in 0i32..=5 {
            if q_tmu.to_bits() == poke(g, ak).to_bits() {
                lo_k[ak as usize] += 1;
                ok = true;
            }
        }
        if !ok {
            lo_none += 1;
        }
    }
    println!("2870 leftover-low tmu store==mul {lo_same}/{lo_ex} swap={lo_swap}");
    print!("leftover-low n={nlo} tmu_excel={lo_ex} up==(w F_2870)+k");
    for (i, c) in lo_k.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if lo_none > 0 {
        print!(" none:{lo_none}");
    }
    println!();
    println!("pin 3.270833 / 3.46875 fused vs 2870 last-store/mul/two-mode:");
    for r in rows.iter().filter(|rr| rr.direct) {
        let pins = [
            3.2708333333333335,
            3.46875,
            0.5000000000000002,
            1.71875,
            1.3333333333333333,
            1.125,
            3.5,
            0.53125,
        ];
        if !pins.iter().any(|&p| (r.z - p).abs() < 1e-14) {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = derfc0(z, &R0);
        let fp = derfc0(z, &r15);
        let gu = mul(w, fu);
        let gp = mul(w, fp);
        let ks_u = k_of(t, w, fu);
        let ks_p = k_of(t, w, fp);
        let mut km_u = None;
        let mut km_p = None;
        for ak in 0i32..=5 {
            if km_u.is_none() && ulp_distance(poke(gu, ak), t).unwrap_or(99) == 0 {
                km_u = Some(ak);
            }
            if km_p.is_none() && ulp_distance(poke(gp, ak), t).unwrap_or(99) == 0 {
                km_p = Some(ak);
            }
        }
        println!(
            "  z={:.16} fused {} store_k={ks_u:?} mul_k={km_u:?} tmu={} tmd={}  2870 {} store_k={ks_p:?} mul_k={km_p:?} tmu={} tmd={}",
            z,
            {
                let d = ulp_distance(gu, t).unwrap_or(99);
                format!("{d}{}", if gu < t { "L" } else if gu > t { "H" } else { "=" })
            },
            hit(mul(w.next_up(), fu.next_up()), t),
            hit(mul(w.next_down(), fu.next_down()), t),
            {
                let d = ulp_distance(gp, t).unwrap_or(99);
                format!("{d}{}", if gp < t { "L" } else if gp > t { "H" } else { "=" })
            },
            hit(mul(w.next_up(), fp.next_up()), t),
            hit(mul(w.next_down(), fp.next_down()), t)
        );
    }
    println!("NSWC leftover-high tmd store vs last-mul:");
    for (name, use2870) in [("fused", false), ("2870", true)] {
        let mut nhi = 0usize;
        let mut n_tmd = 0usize;
        let mut n_same = 0usize;
        let mut n_swap = 0usize;
        let mut n_mul_none = 0usize;
        let mut n_store_none = 0usize;
        for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
            let t = f64::from_bits(r.qbits);
            let z = r.z;
            let w = f::w_rn53(z);
            let fv = if use2870 {
                derfc0(z, &r15)
            } else {
                derfc0(z, &R0)
            };
            let g = mul(w, fv);
            let d = ulp_distance(g, t).unwrap_or(99);
            if !(d >= 2 && g > t) {
                continue;
            }
            nhi += 1;
            let q_tmd = mul(w.next_down(), fv.next_down());
            if !hit(q_tmd, t) {
                continue;
            }
            n_tmd += 1;
            let ks = k_of(t, w, fv);
            let mut km = None;
            for ak in 0i32..=8 {
                if ulp_distance(poke(g, -ak), t).unwrap_or(99) == 0 {
                    km = Some(-ak);
                    break;
                }
            }
            match (ks, km) {
                (Some(a), Some(b)) if a == b => n_same += 1,
                (Some(_), Some(_)) => {
                    n_swap += 1;
                    println!(
                        "  {name} tmd swap z={:.16} store_k={ks:?} mul_k={km:?} d={d}",
                        z
                    );
                }
                (Some(_), None) => {
                    n_mul_none += 1;
                    println!(
                        "  {name} tmd mul_none z={:.16} store_k={ks:?} d={d}",
                        z
                    );
                }
                (None, Some(_)) => {
                    n_store_none += 1;
                    println!(
                        "  {name} tmd store_none z={:.16} mul_k={km:?} d={d}",
                        z
                    );
                }
                (None, None) => {
                    n_mul_none += 1;
                    n_store_none += 1;
                    println!("  {name} tmd both_none z={:.16} d={d}", z);
                }
            }
        }
        println!(
            "{name} leftover-high n={nhi} tmd={n_tmd} store==mul {n_same}/{n_tmd} swap={n_swap} mul_none={n_mul_none} store_none={n_store_none}"
        );
    }
    println!("NSWC leftover-low tmu store vs last-mul:");
    for (name, use2870) in [("fused", false), ("2870", true)] {
        let mut nlo = 0usize;
        let mut n_tmu = 0usize;
        let mut n_same = 0usize;
        let mut n_swap = 0usize;
        let mut n_mul_none = 0usize;
        for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
            let t = f64::from_bits(r.qbits);
            let z = r.z;
            let w = f::w_rn53(z);
            let fv = if use2870 {
                derfc0(z, &r15)
            } else {
                derfc0(z, &R0)
            };
            let g = mul(w, fv);
            let d = ulp_distance(g, t).unwrap_or(99);
            if !(d >= 2 && g < t) {
                continue;
            }
            nlo += 1;
            let q_tmu = mul(w.next_up(), fv.next_up());
            if !hit(q_tmu, t) {
                continue;
            }
            n_tmu += 1;
            let ks = k_of(t, w, fv);
            let mut km = None;
            for ak in 0i32..=8 {
                if ulp_distance(poke(g, ak), t).unwrap_or(99) == 0 {
                    km = Some(ak);
                    break;
                }
            }
            match (ks, km) {
                (Some(a), Some(b)) if a == b => n_same += 1,
                (Some(_), Some(_)) => {
                    n_swap += 1;
                    println!(
                        "  {name} tmu swap z={:.16} store_k={ks:?} mul_k={km:?} d={d}",
                        z
                    );
                }
                (_, None) => {
                    n_mul_none += 1;
                    println!(
                        "  {name} tmu mul_none z={:.16} store_k={ks:?} d={d}",
                        z
                    );
                }
                _ => {}
            }
        }
        println!(
            "{name} leftover-low n={nlo} tmu={n_tmu} store==mul {n_same}/{n_tmu} swap={n_swap} mul_none={n_mul_none}"
        );
    }
}
