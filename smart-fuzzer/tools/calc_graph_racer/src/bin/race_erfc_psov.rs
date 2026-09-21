//! 0x5005 vs SPECFUN P/Q x87-SQRPI Q-tail overlap. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    ext_add, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_sub, ext_to_f64, Ext80, CW_PC64_RN,
};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const P: [f64; 6] = [
    0.305326634961232344,
    0.360344899949804439,
    0.125781726111229246,
    0.0160837851487422766,
    6.58749161529837803e-4,
    0.0163153871373020978,
];
const Q: [f64; 5] = [
    2.56852019228982242,
    1.87295284992346047,
    0.527905102951428412,
    0.0605183413124413191,
    0.00233520497626869185,
];
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

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn polevl_mask(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ef(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn p1evl_mask(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
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
fn cephes_5005(x: f64) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let mask = 0x5005u32;
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl_mask(xe, &CEPHES_P, mask, 0);
        let (q, b) = p1evl_mask(xe, &CEPHES_Q, mask, b);
        (p, q, b)
    } else {
        let (r, b) = polevl_mask(xe, &CEPHES_R, mask, 0);
        let (s, b) = p1evl_mask(xe, &CEPHES_S, mask, b);
        (r, s, b)
    };
    let mut q = ext_div(&num, &den, CW);
    q = maybe(q, mask, bit0);
    ext_to_f64(&q, CW)
}
fn pq_x87_sqrpi(y: f64) -> f64 {
    let ye = ef(y.abs());
    let ysq = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
    let mut xnum = ext_mul(&ef(P[5]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..4 {
        xnum = ext_mul(&ext_add(&xnum, &ef(P[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(Q[i]), CW), &ysq, CW);
    }
    let r = ext_div(
        &ext_mul(&ysq, &ext_add(&xnum, &ef(P[4]), CW), CW),
        &ext_add(&xden, &ef(Q[4]), CW),
        CW,
    );
    let s = ext_div(&ef(1.0), &ext_sqrt(&ef(std::f64::consts::PI), CW), CW);
    ext_to_f64(&ext_div(&ext_sub(&s, &r, CW), &ye, CW), CW)
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let ev5 = |z: f64| qw(z, cephes_5005(z));
    let evp = |z: f64| qw(z, pq_x87_sqrpi(z));
    let score = |ev: &dyn Fn(f64) -> f64| {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut dhit = 0usize;
        let mut dn = 0usize;
        let mut maxu = 0u64;
        for r in &rows {
            if r.z < 4.0 {
                continue;
            }
            let qg = ev(r.z);
            if !qg.is_finite() {
                continue;
            }
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if r.direct {
                dn += 1;
            }
            if d == 0 {
                ex += 1;
                if r.direct {
                    dhit += 1;
                }
            } else {
                maxu = maxu.max(d);
            }
        }
        (ex, n, maxu, dhit, dn)
    };
    for (name, ev) in [("0x5005", &ev5 as &dyn Fn(f64) -> f64), ("P/Q x87 SQRPI", &evp)] {
        let (ex, n, maxu, dhit, dn) = score(ev);
        println!("{name:16} {ex}/{n} max={maxu} d={dhit}/{dn}");
    }
    let mut both = 0usize;
    let mut only5 = 0usize;
    let mut onlyp = 0usize;
    let mut both_d = 0usize;
    let mut only5d = 0usize;
    let mut onlypd = 0usize;
    let mut band_n = [0usize; 6];
    let mut band_5 = [0usize; 6];
    let mut band_p = [0usize; 6];
    for r in &rows {
        if r.z < 4.0 {
            continue;
        }
        let want = f64::from_bits(r.qbits);
        let d5 = ulp_distance(ev5(r.z), want).unwrap_or(99);
        let dp = ulp_distance(evp(r.z), want).unwrap_or(99);
        let b = ((r.z - 4.0) / 2.0).floor() as usize;
        let b = b.min(5);
        band_n[b] += 1;
        if d5 == 0 {
            band_5[b] += 1;
        }
        if dp == 0 {
            band_p[b] += 1;
        }
        match (d5 == 0, dp == 0) {
            (true, true) => {
                both += 1;
                if r.direct {
                    both_d += 1;
                }
            }
            (true, false) => {
                only5 += 1;
                if r.direct {
                    only5d += 1;
                }
            }
            (false, true) => {
                onlyp += 1;
                if r.direct {
                    onlypd += 1;
                }
            }
            _ => {}
        }
    }
    println!(
        "overlap both={} only_5005={} only_pq={} union={}",
        both,
        only5,
        onlyp,
        both + only5 + onlyp
    );
    println!("DIRECT both={both_d} only_5005={only5d} only_pq={onlypd} union={}", both_d + only5d + onlypd);
    for i in 0..6 {
        let lo = 4.0 + i as f64 * 2.0;
        println!(
            "[{:.0},{:.0}) n={} 5005={} pq={}",
            lo,
            lo + 2.0,
            band_n[i],
            band_5[i],
            band_p[i]
        );
    }
    let cuts = [
        4.5, 5.0, 5.5, 5.6, 5.8, 6.0, 6.2, 6.5, 7.0, 8.0, 8.5, 10.0, 12.0, 16.0,
    ];
    for &c in &cuts {
        let mut p5 = 0usize;
        let mut five_p = 0usize;
        for r in &rows {
            if r.z < 4.0 {
                continue;
            }
            let want = f64::from_bits(r.qbits);
            let g = if r.z < c { evp(r.z) } else { ev5(r.z) };
            let h = if r.z < c { ev5(r.z) } else { evp(r.z) };
            if ulp_distance(g, want).unwrap_or(99) == 0 {
                p5 += 1;
            }
            if ulp_distance(h, want).unwrap_or(99) == 0 {
                five_p += 1;
            }
        }
        println!("cut={c:.1} pq-then-5005={p5} 5005-then-pq={five_p}");
    }
}
