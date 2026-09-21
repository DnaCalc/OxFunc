//! DERFC0-2870 leftover and 1-ULP vs fused F last-store. Not an identity.
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
fn signed_k(z: f64, t: f64, r: &[f64; 9]) -> Option<i32> {
    let ff = derfc0(z, r);
    for ak in 0i32..=8 {
        for &s in &[-1i32, 1] {
            let k = if ak == 0 { 0 } else { s * ak };
            if ak == 0 && s < 0 {
                continue;
            }
            if ulp_distance(qmul(f::w_rn53(z), poke(ff, k)), t).unwrap_or(99) == 0 {
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
    let mid: Vec<&f::QRow> = rows.iter().filter(|r| r.z >= 0.5 && r.z < 4.0).collect();
    let n = mid.len();
    let mut r15 = R0;
    r15[1] = poke(R0[1], 1);
    r15[5] = poke(R0[5], 1);
    r15[6] = poke(R0[6], -1);
    let mut fused = 0usize;
    let mut nlo = 0usize;
    let mut lo_cov = 0usize;
    let mut nhi = 0usize;
    let mut hi_cov = 0usize;
    let mut n1 = 0usize;
    let mut u1_cov = 0usize;
    let mut nd = 0usize;
    let mut ndf = 0usize;
    let mut lo_h = [0usize; 9];
    let mut hi_h = [0usize; 9];
    let mut u1_h = [0usize; 9];
    let mut flo = 0usize;
    let mut fhi = 0usize;
    let mut fhit_lo = 0usize;
    let mut fhit_hi = 0usize;
    println!("DERFC0-2870 leftover/1-ULP vs fused F last-store Q [0.5,4) n={n}:");
    for r in &mid {
        let t = f64::from_bits(r.qbits);
        let gf = qmul(f::w_rn53(r.z), derfc0(r.z, &R0));
        let gp = qmul(f::w_rn53(r.z), derfc0(r.z, &r15));
        let df = ulp_distance(gf, t).unwrap_or(99);
        let dp = ulp_distance(gp, t).unwrap_or(99);
        if r.direct {
            nd += 1;
            if dp == 0 {
                ndf += 1;
            }
        }
        if df >= 2 && gf < t {
            flo += 1;
            if dp == 0 {
                fhit_lo += 1;
            }
        }
        if df >= 2 && gf > t {
            fhi += 1;
            if dp == 0 {
                fhit_hi += 1;
            }
        }
        if dp == 0 {
            fused += 1;
            continue;
        }
        let k0 = signed_k(r.z, t, &R0);
        if dp == 1 {
            n1 += 1;
            match k0 {
                Some(kk) => {
                    let ak = kk.unsigned_abs() as usize;
                    if ak < 9 {
                        u1_h[ak] += 1;
                    }
                    u1_cov += 1;
                }
                None => {}
            }
            continue;
        }
        if gp < t {
            nlo += 1;
            match k0 {
                Some(kk) if kk != 0 => {
                    let ak = kk.unsigned_abs() as usize;
                    if ak < 9 {
                        lo_h[ak] += 1;
                    }
                    lo_cov += 1;
                }
                _ => {}
            }
        } else {
            nhi += 1;
            match k0 {
                Some(kk) if kk != 0 => {
                    let ak = kk.unsigned_abs() as usize;
                    if ak < 9 {
                        hi_h[ak] += 1;
                    }
                    hi_cov += 1;
                }
                _ => {}
            }
        }
    }
    println!("fused leftover-low vs 2870 {fhit_lo}/{flo} leftover-high {fhit_hi}/{fhi}");
    print!("2870 fused={fused} leftover-low {lo_cov}/{nlo} F± |k|");
    for (i, c) in lo_h.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    print!(" leftover-high {hi_cov}/{nhi} |k|");
    for (i, c) in hi_h.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    print!(" 1-ULP {u1_cov}/{n1} |k|");
    for (i, c) in u1_h.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!();
    println!(
        "2870 envelope vs fused last-store cover={} /{n} DIRECT {ndf}/{nd}",
        fused + lo_cov + hi_cov + u1_cov
    );
}
