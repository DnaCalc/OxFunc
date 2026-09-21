//! leftover two-mode overlap 0x210 vs 114 vs 2870 DIRECT mid. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeSet;
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
const CP: [f64; 9] = [
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
const CQ: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];
const CR: [f64; 6] = [
    5.64189583547755073984e-1,
    1.27536670759978104416e0,
    5.01905042251180477414e0,
    6.16021097993053585195e0,
    7.40974269950448939160e0,
    2.97886665372100240670e0,
];
const CS: [f64; 6] = [
    2.26052863220117276590e0,
    9.39603524938001434673e0,
    1.20489539808096656605e1,
    1.70814450747565897222e1,
    9.60896809063285878198e0,
    3.36907645100081516050e0,
];
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
fn cephes(x: f64, mask: u32) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (pp, b) = polevl(xe, &CP, mask, 0);
        let (qq, b) = p1evl(xe, &CQ, mask, b);
        (pp, qq, b)
    } else {
        let (rr, b) = polevl(xe, &CR, mask, 0);
        let (ss, b) = p1evl(xe, &CS, mask, b);
        (rr, ss, b)
    };
    let mut v = ext_div(&num, &den, CW);
    v = maybe(v, mask, bit0);
    ext_to_f64(&v, CW)
}
fn is555(z: f64) -> bool {
    let h = format!("{:x}", z.to_bits());
    h.contains("55555555") || h.contains("aaaaaaaa")
}
fn inter(a: &BTreeSet<u64>, b: &BTreeSet<u64>) -> usize {
    a.intersection(b).count()
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
    let mut c114 = C0;
    c114[3] = poke(C0[3], -1);
    c114[4] = poke(C0[4], 1);
    c114[5] = poke(C0[5], -1);
    c114[6] = poke(C0[6], 1);
    let mut r15 = R0;
    r15[1] = poke(R0[1], 1);
    r15[5] = poke(R0[5], 1);
    r15[6] = poke(R0[6], -1);
    let mut lo210 = BTreeSet::new();
    let mut hi210 = BTreeSet::new();
    let mut lo114 = BTreeSet::new();
    let mut hi114 = BTreeSet::new();
    let mut lo2870 = BTreeSet::new();
    let mut hi2870 = BTreeSet::new();
    let mut tmu_c = BTreeSet::new();
    let mut tmd_c = BTreeSet::new();
    let mut tmu_d = BTreeSet::new();
    let mut tmd_d = BTreeSet::new();
    for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
        let t = f64::from_bits(r.qbits);
        let z = r.z;
        let zb = z.to_bits();
        let w = f::w_rn53(z);
        let g210 = mul(w, cody(z, &C0, &D0, MASK));
        let d210 = ulp_distance(g210, t).unwrap_or(99);
        let g114 = mul(w, cody(z, &c114, &D0, MASK));
        let d114 = ulp_distance(g114, t).unwrap_or(99);
        let g2870 = mul(w, derfc0(z, &r15));
        let d2870 = ulp_distance(g2870, t).unwrap_or(99);
        if d210 >= 2 && g210 < t {
            lo210.insert(zb);
        } else if d210 >= 2 && g210 > t {
            hi210.insert(zb);
        }
        if d114 >= 2 && g114 < t {
            lo114.insert(zb);
        } else if d114 >= 2 && g114 > t {
            hi114.insert(zb);
        }
        if d2870 >= 2 && g2870 < t {
            lo2870.insert(zb);
        } else if d2870 >= 2 && g2870 > t {
            hi2870.insert(zb);
        }
        let fc = cody(z, &C0, &D0, 0);
        if hit(mul(w.next_up(), fc.next_up()), t) {
            tmu_c.insert(zb);
        }
        if hit(mul(w.next_down(), fc.next_down()), t) {
            tmd_c.insert(zb);
        }
        let fd = derfc0(z, &R0);
        if hit(mul(w.next_up(), fd.next_up()), t) {
            tmu_d.insert(zb);
        }
        if hit(mul(w.next_down(), fd.next_down()), t) {
            tmd_d.insert(zb);
        }
    }
    let lo210_t = lo210.intersection(&tmu_c).cloned().collect::<BTreeSet<_>>();
    let hi210_t = hi210.intersection(&tmd_c).cloned().collect::<BTreeSet<_>>();
    let lo114_t = lo114.intersection(&tmu_c).cloned().collect::<BTreeSet<_>>();
    let hi114_t = hi114.intersection(&tmd_c).cloned().collect::<BTreeSet<_>>();
    let lo2870_t = lo2870.intersection(&tmu_d).cloned().collect::<BTreeSet<_>>();
    let hi2870_t = hi2870.intersection(&tmd_d).cloned().collect::<BTreeSet<_>>();
    println!(
        "leftover n 0x210 lo/hi={}/{} 114={}/{} 2870={}/{}",
        lo210.len(),
        hi210.len(),
        lo114.len(),
        hi114.len(),
        lo2870.len(),
        hi2870.len()
    );
    println!(
        "leftover∩tmu/tmd 0x210={}/{} 114={}/{} 2870={}/{}",
        lo210_t.len(),
        hi210_t.len(),
        lo114_t.len(),
        hi114_t.len(),
        lo2870_t.len(),
        hi2870_t.len()
    );
    println!(
        "leftover-low ∩ 0x210∩114={} 0x210∩2870={} 114∩2870={} all3={}",
        inter(&lo210, &lo114),
        inter(&lo210, &lo2870),
        inter(&lo114, &lo2870),
        lo210.intersection(&lo114).filter(|z| lo2870.contains(z)).count()
    );
    println!(
        "leftover-high ∩ 0x210∩114={} 0x210∩2870={} 114∩2870={} all3={}",
        inter(&hi210, &hi114),
        inter(&hi210, &hi2870),
        inter(&hi114, &hi2870),
        hi210.intersection(&hi114).filter(|z| hi2870.contains(z)).count()
    );
    println!(
        "tmu leftover-low ∩ 0x210∩114={} 0x210∩2870={} 114∩2870={} all3={}",
        inter(&lo210_t, &lo114_t),
        inter(&lo210_t, &lo2870_t),
        inter(&lo114_t, &lo2870_t),
        lo210_t.intersection(&lo114_t).filter(|z| lo2870_t.contains(z)).count()
    );
    println!(
        "tmd leftover-high ∩ 0x210∩114={} 0x210∩2870={} 114∩2870={} all3={}",
        inter(&hi210_t, &hi114_t),
        inter(&hi210_t, &hi2870_t),
        inter(&hi114_t, &hi2870_t),
        hi210_t.intersection(&hi114_t).filter(|z| hi2870_t.contains(z)).count()
    );
    println!(
        "cody tmu∩derfc tmu={} tmd∩tmd={}",
        inter(&tmu_c, &tmu_d),
        inter(&tmd_c, &tmd_d)
    );
    let pin = |label: &str, set: Vec<u64>| {
        println!("{label}:");
        for zb in set {
            let z = f64::from_bits(zb);
            let t = {
                let mut bits = None;
                for r in rows.iter().filter(|rr| rr.direct && rr.z >= 0.5 && rr.z < 4.0) {
                    if r.z.to_bits() == zb {
                        bits = Some(r.qbits);
                        break;
                    }
                }
                f64::from_bits(bits.unwrap())
            };
            let w = f::w_rn53(z);
            let fc = cody(z, &C0, &D0, 0);
            let fd = derfc0(z, &R0);
            let d74 = ulp_distance(mul(w, cody(z, &C0, &D0, 0x74)), t).unwrap_or(99);
            let d210 = ulp_distance(mul(w, cody(z, &C0, &D0, MASK)), t).unwrap_or(99);
            let ce0 = cephes(z, 0);
            let dce = ulp_distance(mul(w, ce0), t).unwrap_or(99);
            let dce5 = ulp_distance(mul(w, cephes(z, 0x5005)), t).unwrap_or(99);
            let gc = mul(w, fc);
            println!(
                "  z={:.16} bits={:#x} 555={} cody_k={:?} derfc_k={:?} cephes_k={:?} cody_d={} derfc_d={} d210={d210} d74={d74} dce0={dce} dce5005={dce5} last-mul_up={} last-mul_dn={} tmu={} tmd={}",
                z,
                zb,
                is555(z),
                k_of(t, w, fc),
                k_of(t, w, fd),
                k_of(t, w, ce0),
                ulp_distance(gc, t).unwrap_or(99),
                ulp_distance(mul(w, fd), t).unwrap_or(99),
                hit(gc.next_up(), t),
                hit(gc.next_down(), t),
                hit(mul(w.next_up(), fc.next_up()), t),
                hit(mul(w.next_down(), fc.next_down()), t)
            );
        }
    };
    pin(
        "all3 leftover-low tmu",
        lo210_t
            .intersection(&lo114_t)
            .filter(|z| lo2870_t.contains(z))
            .cloned()
            .collect(),
    );
    pin(
        "all3 leftover-high tmd",
        hi210_t
            .intersection(&hi114_t)
            .filter(|z| hi2870_t.contains(z))
            .cloned()
            .collect(),
    );
}
