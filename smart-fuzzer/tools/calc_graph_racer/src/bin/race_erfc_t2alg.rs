//! tail leftover |k|=±2 two-mode vs last-store of unmask Cephes F. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const P0: [f64; 9] = [
    2.46196981473530512524e-10,
    5.64189564831068821977e-1,
    7.46321056442269912687e0,
    4.86371970985681366614e1,
    1.96520832956077098242e2,
    5.26445194995477358631e2,
    9.34528527171957607540e2,
    1.02755188689515710272e3,
    5.57535335369399327526e2,
];
const Q0: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];
const R0: [f64; 6] = [
    5.64189583547755073984e-1,
    1.27536670759978104416e0,
    5.01905042251180477414e0,
    6.16021097993053585195e0,
    7.40974269950448939160e0,
    2.97886665372100240670e0,
];
const S0: [f64; 6] = [
    2.26052863220117276590e0,
    9.39603524938001434673e0,
    1.20489539808096656605e1,
    1.70814450747565897222e1,
    9.60896809063285878198e0,
    3.36907645100081516050e0,
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
fn polevl(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ef(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn p1evl(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ext_add(&x, &ef(coef[0]), CW);
    ans = maybe(ans, mask, bit);
    bit += 1;
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn cephes(x: f64, p: &[f64; 9], q: &[f64; 8], mask: u32) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (pp, b) = polevl(xe, p, mask, 0);
        let (qq, b) = p1evl(xe, q, mask, b);
        (pp, qq, b)
    } else {
        let (rr, b) = polevl(xe, &R0, mask, 0);
        let (ss, b) = p1evl(xe, &S0, mask, b);
        (rr, ss, b)
    };
    let mut v = ext_div(&num, &den, CW);
    v = maybe(v, mask, bit0);
    ext_to_f64(&v, CW)
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
    let mut p66 = P0;
    let mut q66 = Q0;
    p66[4] = poke(P0[4], -1);
    q66[0] = poke(Q0[0], 1);
    q66[1] = poke(Q0[1], -1);
    q66[3] = poke(Q0[3], 1);
    let graphs: [(&str, u32, [f64; 9], [f64; 8]); 2] = [
        ("0x5005", 0x5005, P0, Q0),
        ("66", 0x24a5, p66, q66),
    ];
    for (name, mask, p, q) in &graphs {
        let mut n_k2 = 0usize;
        let mut n_tmu = 0usize;
        let mut n_eq = 0usize;
        let mut n_km2 = 0usize;
        let mut n_tmd = 0usize;
        let mut n_heq = 0usize;
        println!("{name} leftover |k|=±2 two-mode vs unmask F, DIRECT 4<=z<20:");
        for r in rows.iter().filter(|rr| rr.direct && rr.z >= 4.0 && rr.z < 20.0) {
            let t = f64::from_bits(r.qbits);
            let z = r.z;
            let w = f::w_rn53(z);
            let fu = cephes(z, &P0, &Q0, 0);
            let g = mul(w, cephes(z, p, q, *mask));
            let d = ulp_distance(g, t).unwrap_or(99);
            let k = k_of(t, w, fu);
            if d >= 2 && g < t && k == Some(2) {
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
                        "  LO k=2 not-tmu z={:.16} 555={} tmu_d={} eq_k1={} eq_k3={} eq_k4={} eq_prod1={} eq_prod2={} eq_prod3={} eq_w1={} eq_f1={}",
                        z,
                        is555(z),
                        ulp_distance(q_tmu, t).unwrap_or(99),
                        q_tmu.to_bits() == mul(w, poke(fu, 1)).to_bits(),
                        q_tmu.to_bits() == mul(w, poke(fu, 3)).to_bits(),
                        q_tmu.to_bits() == mul(w, poke(fu, 4)).to_bits(),
                        q_tmu.to_bits() == poke(mul(w, fu), 1).to_bits(),
                        q_tmu.to_bits() == poke(mul(w, fu), 2).to_bits(),
                        q_tmu.to_bits() == poke(mul(w, fu), 3).to_bits(),
                        q_tmu.to_bits() == mul(w.next_up(), fu).to_bits(),
                        q_tmu.to_bits() == mul(w, fu.next_up()).to_bits()
                    );
                }
            } else if d >= 2 && g > t && k == Some(-2) {
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
                        "  HI k=-2 not-tmd z={:.16} 555={} tmd_d={} eq_km1={} eq_km3={} eq_prodm1={} eq_prodm2={} eq_prodm3={} eq_wm1={} eq_fm1={}",
                        z,
                        is555(z),
                        ulp_distance(q_tmd, t).unwrap_or(99),
                        q_tmd.to_bits() == mul(w, poke(fu, -1)).to_bits(),
                        q_tmd.to_bits() == mul(w, poke(fu, -3)).to_bits(),
                        q_tmd.to_bits() == poke(mul(w, fu), -1).to_bits(),
                        q_tmd.to_bits() == poke(mul(w, fu), -2).to_bits(),
                        q_tmd.to_bits() == poke(mul(w, fu), -3).to_bits(),
                        q_tmd.to_bits() == mul(w.next_down(), fu).to_bits(),
                        q_tmd.to_bits() == mul(w, fu.next_down()).to_bits()
                    );
                }
            }
        }
        println!("  leftover-low |k|=2 n={n_k2} tmu={n_tmu} Q_tmu==Q_k2 {n_eq}/{n_k2}");
        println!("  leftover-high |k|=-2 n={n_km2} tmd={n_tmd} Q_tmd==Q_km2 {n_heq}/{n_km2}");
    }
    let mut nall = 0usize;
    let mut up_p = [0usize; 6];
    let mut dn_p = [0usize; 6];
    let mut up_none = 0usize;
    let mut dn_none = 0usize;
    let mut tmu_ex = 0usize;
    let mut tmd_ex = 0usize;
    println!("DIRECT tail 4<=z<20 unmask two-mode vs last-mul of w*F:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 4.0 && rr.z < 20.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cephes(z, &P0, &Q0, 0);
        nall += 1;
        let prod = mul(w, fu);
        let q_tmu = mul(w.next_up(), fu.next_up());
        let q_tmd = mul(w.next_down(), fu.next_down());
        if hit(q_tmu, t) {
            tmu_ex += 1;
        }
        if hit(q_tmd, t) {
            tmd_ex += 1;
        }
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
    print!("n={nall} tmu_excel={tmu_ex} tmd_excel={tmd_ex} two-mode-up==(wF)+k");
    for (i, c) in up_p.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if up_none > 0 {
        print!(" none:{up_none}");
    }
    print!(" two-mode-down==(wF)-k");
    for (i, c) in dn_p.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if dn_none > 0 {
        print!(" none:{dn_none}");
    }
    println!();
    let mut nx = 0usize;
    let mut x_up = [0usize; 6];
    let mut x_dn = [0usize; 6];
    let mut x_up_none = 0usize;
    let mut x_dn_none = 0usize;
    println!("DIRECT XBIG z>=20 two-mode vs last-mul of w*F:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 20.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = cephes(z, &P0, &Q0, 0);
        nx += 1;
        let prod = mul(w, fu);
        let q_tmu = mul(w.next_up(), fu.next_up());
        let q_tmd = mul(w.next_down(), fu.next_down());
        let mut up = false;
        let mut dn = false;
        for ak in 0i32..=5 {
            if q_tmu.to_bits() == poke(prod, ak).to_bits() {
                x_up[ak as usize] += 1;
                up = true;
            }
            if q_tmd.to_bits() == poke(prod, -ak).to_bits() {
                x_dn[ak as usize] += 1;
                dn = true;
            }
        }
        if !up {
            x_up_none += 1;
        }
        if !dn {
            x_dn_none += 1;
        }
        if !up || !dn {
            println!(
                "  miss z={:.16} tmu_d={} tmd_d={} prod={:#x} tmu={:#x} tmd={:#x}",
                z,
                ulp_distance(q_tmu, t).unwrap_or(99),
                ulp_distance(q_tmd, t).unwrap_or(99),
                prod.to_bits(),
                q_tmu.to_bits(),
                q_tmd.to_bits()
            );
        }
    }
    print!("XBIG n={nx} two-mode-up==(wF)+k");
    for (i, c) in x_up.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if x_up_none > 0 {
        print!(" none:{x_up_none}");
    }
    print!(" down");
    for (i, c) in x_dn.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if x_dn_none > 0 {
        print!(" none:{x_dn_none}");
    }
    println!();
    let mut p66 = P0;
    let mut q66 = Q0;
    p66[4] = poke(P0[4], -1);
    q66[0] = poke(Q0[0], 1);
    q66[1] = poke(Q0[1], -1);
    q66[3] = poke(Q0[3], 1);
    println!("DIRECT tail 4<=z<20 two-mode last-mul cover of each F:");
    for (name, mask, p, q) in [
        ("unmask", 0u32, P0, Q0),
        ("0x5005", 0x5005, P0, Q0),
        ("0x4005", 0x4005, P0, Q0),
        ("66", 0x24a5, p66, q66),
    ] {
        let mut n = 0usize;
        let mut up = [0usize; 6];
        let mut none = 0usize;
        let mut tmu_ex = 0usize;
        for r in rows.iter().filter(|rr| rr.direct && rr.z >= 4.0 && rr.z < 20.0) {
            let t = f64::from_bits(r.qbits);
            let z = r.z;
            let w = f::w_rn53(z);
            let fv = cephes(z, &p, &q, mask);
            n += 1;
            let prod = mul(w, fv);
            let q_tmu = mul(w.next_up(), fv.next_up());
            if hit(q_tmu, t) {
                tmu_ex += 1;
            }
            let mut ok = false;
            for ak in 0i32..=5 {
                if q_tmu.to_bits() == poke(prod, ak).to_bits() {
                    up[ak as usize] += 1;
                    ok = true;
                }
            }
            if !ok {
                none += 1;
            }
        }
        print!("{name} n={n} tmu_excel={tmu_ex} up==(wF)+k");
        for (i, c) in up.iter().enumerate() {
            if *c > 0 {
                print!(" {i}:{c}");
            }
        }
        if none > 0 {
            print!(" none:{none}");
        }
        println!();
    }
}
