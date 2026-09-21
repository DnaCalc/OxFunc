//! R[1]+1 R[5]+1 vs fused DERFC0 leftover. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use std::env;

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
fn qmul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn pqr_r15(x: f64) -> f64 {
    let mut r = [
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
    r[1] = poke(r[1], 1);
    r[5] = poke(r[5], 1);
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
    let u = {
        let mut a = 0.0;
        for &c in P.iter().rev() {
            a = a * x + c;
        }
        a
    };
    let v = {
        let mut a = 0.0;
        for &c in Q.iter().rev() {
            a = a * x + c;
        }
        a
    };
    let t = (x - 3.75) / (x + 3.75);
    let mut acc = u / v;
    for &c in r.iter().rev() {
        acc = acc * t + c;
    }
    acc
}
fn derfc0_poked(x: f64) -> f64 {
    if x <= 2.0 {
        pqr_r15(x)
    } else {
        f::nswc_derfc0(x)
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut both = 0usize;
    let mut only_p = 0usize;
    let mut only_f = 0usize;
    let mut nf = 0usize;
    let mut np = 0usize;
    let mut nlo = 0usize;
    let mut hit_lo = 0usize;
    let mut ndf = 0usize;
    let mut ndp = 0usize;
    for r in rows.iter().filter(|rr| rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let gf = qmul(w, f::nswc_derfc0(r.z));
        let gp = qmul(w, derfc0_poked(r.z));
        let ef = ulp_distance(gf, t).unwrap_or(99) == 0;
        let ep = ulp_distance(gp, t).unwrap_or(99) == 0;
        if ef {
            nf += 1;
            if r.direct {
                ndf += 1;
            }
        }
        if ep {
            np += 1;
            if r.direct {
                ndp += 1;
            }
        }
        match (ep, ef) {
            (true, true) => both += 1,
            (true, false) => only_p += 1,
            (false, true) => only_f += 1,
            _ => {}
        }
        let d = ulp_distance(gf, t).unwrap_or(99);
        if d >= 2 && gf < t {
            nlo += 1;
            if ep {
                hit_lo += 1;
            }
        }
    }
    println!(
        "Q [0.5,4) fused={nf} poked={np} both={both} only_poke={only_p} only_fused={only_f} leftover-low {hit_lo}/{nlo} DIRECT fused={ndf} poked={ndp}"
    );
}
