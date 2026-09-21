//! 3 leftover-high of 0x5005 that miss F−1..4. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const CW_RD: u16 = CW_PC64_RN | 0x0400;
const MASK: u32 = 0x5005;
const CEPHES_P: [f64; 9] = [
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
const CEPHES_Q: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];
const CEPHES_R: [f64; 6] = [
    5.64189583547755073984e-1,
    1.27536670759978104416e0,
    5.01905042251180477414e0,
    6.16021097993053585195e0,
    7.40974269950448939160e0,
    2.97886665372100240670e0,
];
const CEPHES_S: [f64; 6] = [
    2.26052863220117276590e0,
    9.39603524938001434673e0,
    1.20489539808096656605e1,
    1.70814450747565897222e1,
    9.60896809063285878198e0,
    3.36907645100081516050e0,
];
const CC0: [f64; 9] = [
    -0.7040906288250128001000086e-04,
    -0.3858822461760510359506941e-02,
    -0.7708202127512212359395078e-01,
    -0.6713655014557429480440263e+00,
    -0.2081992124162995545731882e+01,
    0.2898831421475282558867888e+01,
    0.2199509380600429331650192e+02,
    0.2907064664404115316722996e+01,
    -0.4766208741588182425380950e+02,
];
const DD0: [f64; 10] = [
    1.0,
    0.5238852785508439144747174e+02,
    0.9646843357714742409535148e+03,
    0.7007152775135939601804416e+04,
    0.8515386792259821780601162e+04,
    -0.1002360095177164564992134e+06,
    -0.2065250031331232815791912e+06,
    0.5695324805290370358175984e+06,
    0.6589752493461331195697873e+06,
    -0.1192930193156561957631462e+07,
];
const E0: f64 = 0.540464821348814822409610122136;
const E1: f64 = -0.261515522487415653487049835220e-01;
const E2: f64 = -0.288573438386338758794591212600e-02;
const E3: f64 = -0.529353396945788057720258856000e-03;
const P0: [f64; 8] = [
    0.16506148041280876191828601e-03,
    0.15471455377139313353998665e-03,
    0.44852548090298868465196794e-04,
    -0.49177280017226285450486205e-05,
    -0.69353602078656412367801676e-05,
    -0.20508667787746282746857743e-05,
    -0.28982842617824971177267380e-06,
    -0.17272433544836633301127174e-07,
];
const Q0: [f64; 8] = [
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
const THREE: [f64; 3] = [4.458333333333333, 4.5, 5.0];

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
fn cephes(x: f64) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl(xe, &CEPHES_P, MASK, 0);
        let (q, b) = p1evl(xe, &CEPHES_Q, MASK, b);
        (p, q, b)
    } else {
        let (r, b) = polevl(xe, &CEPHES_R, MASK, 0);
        let (s, b) = p1evl(xe, &CEPHES_S, MASK, b);
        (r, s, b)
    };
    let mut v = ext_div(&num, &den, CW);
    v = maybe(v, MASK, bit0);
    ext_to_f64(&v, CW)
}
fn horner(cs: &[f64], x: f64) -> f64 {
    let mut a = 0.0;
    for &c in cs.iter().rev() {
        a = a * x + c;
    }
    a
}
fn pqr_r1(x: f64) -> f64 {
    let mut r = R0;
    r[1] = poke(R0[1], 1);
    let u = horner(&P0, x);
    let v = horner(&Q0, x);
    let t = (x - 3.75) / (x + 3.75);
    let mut acc = u / v;
    for &c in r.iter().rev() {
        acc = acc * t + c;
    }
    acc
}
fn e0p(x: f64) -> f64 {
    let z = 1.0 / (2.5 + x * x);
    let t = 13.0 * z - 1.0;
    let acc = (((horner(&CC0, z) / horner(&DD0, z) * t + E3) * t + E2) * t + E1) * t + poke(E0, 1);
    acc / x
}
fn mul(w: f64, ff: f64) -> f64 {
    let v = w * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn rd_mul(w: f64, ff: f64) -> f64 {
    let v = ext_to_f64(&ext_mul(&ef(w), &ef(ff), CW_RD), CW_RD);
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn signed(g: f64, t: f64) -> String {
    let d = ulp_distance(g, t).unwrap_or(99);
    let s = if g < t {
        "L"
    } else if g > t {
        "H"
    } else {
        "="
    };
    format!("{d}{s}")
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    println!("3 leftover-high miss F-1..4:");
    for &lz in &THREE {
        let Some(r) = rows
            .iter()
            .find(|rr| rr.direct && (rr.z - lz).abs() < 1e-12)
        else {
            println!("  z={lz:.16} MISSING");
            continue;
        };
        let t = f64::from_bits(r.qbits);
        let w = f::w_rn53(lz);
        let ff = cephes(lz);
        print!("  z={lz:.16} CR={}", signed(mul(w, ff), t));
        for k in 5..=8 {
            print!(" F-{k}={}", signed(mul(w, poke(ff, -k)), t));
        }
        let ge = mul(w, e0p(lz));
        let gr = mul(w, pqr_r1(lz));
        println!(
            " RD={} dnW*F={} W*dnF={} dnW*dnF={} E0+1={} upE0={} dnE0={} R1={} cephes_f={}",
            signed(rd_mul(w.next_down(), ff), t),
            signed(mul(w.next_down(), ff), t),
            signed(mul(w, ff.next_down()), t),
            signed(mul(w.next_down(), ff.next_down()), t),
            signed(ge, t),
            signed(ge.next_up(), t),
            signed(ge.next_down(), t),
            signed(gr, t),
            signed(mul(w, f::cephes_f(lz)), t)
        );
    }
}
