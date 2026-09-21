//! 2870 leftover-low misses of fused F±8. Not an identity.
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
fn qmul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn signed_k_win(z: f64, t: f64, r: &[f64; 9], maxk: i32) -> Option<i32> {
    let ff = derfc0(z, r);
    for ak in 0i32..=maxk {
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
    let mut r15 = R0;
    r15[1] = poke(R0[1], 1);
    r15[5] = poke(R0[5], 1);
    r15[6] = poke(R0[6], -1);
    let mut nmiss = 0usize;
    let mut ndir = 0usize;
    let mut nle2 = 0usize;
    let mut ngt2 = 0usize;
    let mut k16_hit = 0usize;
    let mut k32_hit = 0usize;
    let mut u1_miss = 0usize;
    println!("2870 leftover-low fused-F±8 misses:");
    for r in rows.iter().filter(|rr| rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let gp = qmul(f::w_rn53(r.z), derfc0(r.z, &r15));
        let dp = ulp_distance(gp, t).unwrap_or(99);
        if dp == 1 {
            if signed_k_win(r.z, t, &R0, 8).is_none() {
                u1_miss += 1;
                println!(
                    "  1-ULP MISS z={:.16} bits={:#x} qbits={:#x} direct={} k8={:?} k16={:?} k32={:?} nswc_d={:?}",
                    r.z,
                    r.z.to_bits(),
                    r.qbits,
                    r.direct,
                    signed_k_win(r.z, t, &R0, 8),
                    signed_k_win(r.z, t, &R0, 16),
                    signed_k_win(r.z, t, &R0, 32),
                    ulp_distance(qmul(f::w_rn53(r.z), f::nswc_derfc0(r.z)), t)
                );
            }
            continue;
        }
        if !(dp >= 2 && gp < t) {
            continue;
        }
        if signed_k_win(r.z, t, &R0, 8).is_some() {
            continue;
        }
        nmiss += 1;
        if r.direct {
            ndir += 1;
        }
        if r.z <= 2.0 {
            nle2 += 1;
        } else {
            ngt2 += 1;
        }
        let k16 = signed_k_win(r.z, t, &R0, 16);
        let k32 = signed_k_win(r.z, t, &R0, 32);
        if k16.is_some() {
            k16_hit += 1;
        }
        if k32.is_some() {
            k32_hit += 1;
        }
        if nmiss <= 20 {
            println!(
                "  z={:.16} d={dp} direct={} z2={} k16={:?} k32={:?}",
                r.z,
                r.direct,
                r.z <= 2.0,
                k16,
                k32
            );
        }
    }
    println!(
        "leftover-low miss n={nmiss} DIRECT={ndir} z<=2={nle2} z>2={ngt2} k16={k16_hit} k32={k32_hit} 1-ULP miss={u1_miss}"
    );
}
