//! two-mode of fused DERFC0 w×F on 2870 leftover DIRECT [0.5,4). Not an identity.
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
fn pqr(x: f64, r: &[f64; 9]) -> f64 {
    let u = horner(&P, x);
    let v = horner(&Q, x);
    let t = (x - 3.75) / (x + 3.75);
    let mut acc = u / v;
    for &c in r.iter().rev() {
        acc = acc * t + c;
    }
    acc
}
fn derfc0(x: f64, r: &[f64; 9]) -> f64 {
    if x <= 2.0 {
        pqr(x, r)
    } else {
        f::nswc_derfc0(x)
    }
}
fn qmul(w: f64, ff: f64) -> f64 {
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
            if ulp_distance(qmul(w, poke(ff, k)), t).unwrap_or(99) == 0 {
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
    let mut nlo = 0usize;
    let mut nhi = 0usize;
    let mut lo_tmu = 0usize;
    let mut lo_tmu_only = 0usize;
    let mut lo_up = 0usize;
    let mut lo_w1 = 0usize;
    let mut lo_f1 = 0usize;
    let mut hi_tmd = 0usize;
    let mut hi_tmd_only = 0usize;
    let mut hi_dn = 0usize;
    let mut hi_w1 = 0usize;
    let mut hi_f1 = 0usize;
    let mut lo_tmu2870 = 0usize;
    let mut hi_tmd2870 = 0usize;
    let mut nf = 0usize;
    let mut n1 = 0usize;
    let mut tmu_b = [0usize; 4];
    let mut tmd_b = [0usize; 4];
    let mut lo_tmu_k = [0usize; 9];
    let mut lo_not_k = [0usize; 9];
    let mut hi_tmd_k = [0usize; 9];
    let mut hi_not_k = [0usize; 9];
    println!("fused DERFC0 two-mode on 2870 leftover DIRECT [0.5,4):");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let w = f::w_rn53(z);
        let fu = derfc0(z, &R0);
        let gp = qmul(w, derfc0(z, &r15));
        let dp = ulp_distance(gp, t).unwrap_or(99);
        let tmu = hit(qmul(w.next_up(), fu.next_up()), t);
        let tmd = hit(qmul(w.next_down(), fu.next_down()), t);
        let up = hit(qmul(w, fu).next_up(), t);
        let dn = hit(qmul(w, fu).next_down(), t);
        let w1u = hit(qmul(w.next_up(), fu), t);
        let f1u = hit(qmul(w, fu.next_up()), t);
        let w1d = hit(qmul(w.next_down(), fu), t);
        let f1d = hit(qmul(w, fu.next_down()), t);
        let fp = derfc0(z, &r15);
        let tmu2870 = hit(qmul(w.next_up(), fp.next_up()), t);
        let tmd2870 = hit(qmul(w.next_down(), fp.next_down()), t);
        let i = if dp == 0 {
            nf += 1;
            0
        } else if dp == 1 {
            n1 += 1;
            1
        } else if gp < t {
            nlo += 1;
            let k = k_of(t, w, fu);
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
                _ => {}
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
            if tmu2870 {
                lo_tmu2870 += 1;
            }
            if tmu {
                println!(
                    "  LO tmu z={:.16} 555={} last-mul_up={up} w+1={w1u} F+1={f1u} d={dp}",
                    z,
                    is555(z)
                );
            }
            2
        } else {
            nhi += 1;
            let k = k_of(t, w, fu);
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
                _ => {}
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
            if tmd2870 {
                hi_tmd2870 += 1;
            }
            if tmd {
                println!(
                    "  HI tmd z={:.16} 555={} last-mul_dn={dn} w-1={w1d} F-1={f1d} d={dp}",
                    z,
                    is555(z)
                );
            }
            3
        };
        if tmu {
            tmu_b[i] += 1;
        }
        if tmd {
            tmd_b[i] += 1;
        }
    }
    println!("2870 DIRECT fused={nf} 1-ULP={n1} leftover-low n={nlo} leftover-high n={nhi}");
    println!(
        "leftover-low fused tmu={lo_tmu} tmu_only={lo_tmu_only} 2870-F tmu={lo_tmu2870} last-mul_up={lo_up} w+1={lo_w1} F+1={lo_f1}"
    );
    println!(
        "leftover-high fused tmd={hi_tmd} tmd_only={hi_tmd_only} 2870-F tmd={hi_tmd2870} last-mul_dn={hi_dn} w-1={hi_w1} F-1={hi_f1}"
    );
    print!("leftover-low tmu |k|");
    for (i, c) in lo_tmu_k.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    print!("\nleftover-low not-tmu |k|");
    for (i, c) in lo_not_k.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    print!("\nleftover-high tmd |k|");
    for (i, c) in hi_tmd_k.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    print!("\nleftover-high not-tmd |k|");
    for (i, c) in hi_not_k.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!();
    print!("tmu");
    for (lab, c) in ["fused", "1-ULP", "lo", "hi"].iter().zip(tmu_b) {
        print!(" {lab}:{c}");
    }
    print!(" tmd");
    for (lab, c) in ["fused", "1-ULP", "lo", "hi"].iter().zip(tmd_b) {
        print!(" {lab}:{c}");
    }
    println!();
}
