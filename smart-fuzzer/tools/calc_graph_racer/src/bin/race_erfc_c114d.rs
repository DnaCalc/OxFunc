//! 0x210-114 vs DERFC0-2870 DIRECT [0.5,4) overlap. Not an identity.
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
fn cody(y: f64, c: &[f64; 9]) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, MASK, 0);
    xden = maybe(xden, MASK, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, MASK, 2 + 2 * i as u32);
        xden = maybe(xden, MASK, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(c[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, MASK, 16);
    ext_to_f64(&q, CW)
}
fn horner(cs: &[f64], x: f64) -> f64 {
    let mut a = 0.0;
    for &c in cs.iter().rev() {
        a = a * x + c;
    }
    a
}
fn derfc0_poked(x: f64) -> f64 {
    if x <= 2.0 {
        let mut r = R0;
        r[1] = poke(R0[1], 1);
        r[5] = poke(R0[5], 1);
        r[6] = poke(R0[6], -1);
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut c114 = C0;
    c114[3] = poke(C0[3], -1);
    c114[4] = poke(C0[4], 1);
    c114[5] = poke(C0[5], -1);
    c114[6] = poke(C0[6], 1);
    let mut both = 0usize;
    let mut only_c = 0usize;
    let mut only_d = 0usize;
    let mut nc = 0usize;
    let mut nd = 0usize;
    let mut n = 0usize;
    println!("DIRECT [0.5,4) 0x210-114 vs DERFC0-2870:");
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        n += 1;
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(r.z);
        let ec = ulp_distance(mul(w, cody(r.z, &c114)), t).unwrap_or(99) == 0;
        let ed = ulp_distance(mul(w, derfc0_poked(r.z)), t).unwrap_or(99) == 0;
        if ec {
            nc += 1;
        }
        if ed {
            nd += 1;
        }
        match (ec, ed) {
            (true, true) => both += 1,
            (true, false) => only_c += 1,
            (false, true) => only_d += 1,
            _ => {}
        }
    }
    println!(
        "n={n} 114={nc} 2870={nd} both={both} only_114={only_c} only_2870={only_d} union={}",
        both + only_c + only_d
    );
}
