//! Further Q-bit graphs: Cephes masks, Cody0x74/Cephes piecewise, flush.
//! Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
const C: [f64; 9] = [
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
const D: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];
const PP: [f64; 6] = [
    0.305326634961232344,
    0.360344899949804439,
    0.125781726111229246,
    0.0160837851487422766,
    6.58749161529837803e-4,
    0.0163153871373020978,
];
const QQ: [f64; 5] = [
    2.56852019228982242,
    1.87295284992346047,
    0.527905102951428412,
    0.0605183413124413191,
    0.00233520497626869185,
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
fn cephes_mask(x: f64, mask: u32) -> f64 {
    let ax = x.abs();
    if ax < 1.0 {
        return f::cephes_f(x);
    }
    let xe = ef(ax);
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
fn cody_mask(y: f64, mask: u32) -> f64 {
    let y = y.abs();
    let ye = ef(y);
    if y <= 4.0 {
        let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
        let mut xden = ye;
        xnum = maybe(xnum, mask, 0);
        xden = maybe(xden, mask, 1);
        for i in 0..7 {
            xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
            xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
            xnum = maybe(xnum, mask, 2 + 2 * i as u32);
            xden = maybe(xden, mask, 3 + 2 * i as u32);
        }
        let mut q = ext_div(
            &ext_add(&xnum, &ef(C[7]), CW),
            &ext_add(&xden, &ef(D[7]), CW),
            CW,
        );
        q = maybe(q, mask, 16);
        ext_to_f64(&q, CW)
    } else {
        let ysq = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
        let mut xnum = ext_mul(&ef(PP[5]), &ysq, CW);
        let mut xden = ysq;
        for i in 0..4 {
            xnum = ext_mul(&ext_add(&xnum, &ef(PP[i]), CW), &ysq, CW);
            xden = ext_mul(&ext_add(&xden, &ef(QQ[i]), CW), &ysq, CW);
        }
        let r = ext_div(
            &ext_mul(&ysq, &ext_add(&xnum, &ef(PP[4]), CW), CW),
            &ext_add(&xden, &ef(QQ[4]), CW),
            CW,
        );
        (f::RPINV - ext_to_f64(&r, CW)) / y
    }
}

fn flush(v: f64) -> f64 {
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

#[derive(Default, Clone, Copy)]
struct Acc {
    exact: usize,
    n: usize,
    max_ulp: u64,
    dmid: usize,
    dn: usize,
}
impl Acc {
    fn add(&mut self, d: u64, direct: bool) {
        self.n += 1;
        if direct {
            self.dn += 1;
        }
        if d == 0 {
            self.exact += 1;
            if direct {
                self.dmid += 1;
            }
        } else {
            self.max_ulp = self.max_ulp.max(d);
        }
    }
}
fn fmt(a: &Acc) -> String {
    format!("{}/{} max={} d={}/{}", a.exact, a.n, a.max_ulp, a.dmid, a.dn)
}

fn score(rows: &[f::QRow], eval: impl Fn(f64) -> f64) -> (Acc, Acc) {
    let mut mid = Acc::default();
    let mut tail = Acc::default();
    for r in rows {
        if r.z < 0.5 {
            continue;
        }
        let qg = eval(r.z);
        if !qg.is_finite() {
            continue;
        }
        let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
        if d > ULP_CAP {
            continue;
        }
        if r.z < 4.0 {
            mid.add(d, r.direct);
        } else {
            tail.add(d, r.direct);
        }
    }
    (mid, tail)
}

fn qw(z: f64, ff: f64) -> f64 {
    f::w_rn53(z) * ff
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);

    println!("## Cephes x87 masks Q=w*F64");
    for (name, mask) in [
        ("cephes mask0", 0u32),
        ("cephes 0x5005", 0x5005),
        ("cephes 0x24a5", 0x24a5),
        ("cephes 0x0509", 0x0509),
    ] {
        let (m, t) = score(&rows, |z| qw(z, cephes_mask(z, mask)));
        println!("{name:20} mid {} tail {}", fmt(&m), fmt(&t));
    }

    println!("\n## piecewise Cody 0x74 / Cephes 0x5005");
    for cut in [4.0, 4.9, 5.6, 8.0] {
        let (m, t) = score(&rows, |z| {
            let ff = if z < cut {
                cody_mask(z, 0x74)
            } else {
                cephes_mask(z, 0x5005)
            };
            qw(z, ff)
        });
        println!("cut={cut:.1} mid {} tail {}", fmt(&m), fmt(&t));
    }

    println!("\n## flush DBL_MIN on Cody 0x74 and mask0");
    for (name, mask) in [("cody0 flush", 0u32), ("cody74 flush", 0x74)] {
        let (m, t) = score(&rows, |z| flush(qw(z, cody_mask(z, mask))));
        println!("{name:16} mid {} tail {}", fmt(&m), fmt(&t));
    }

    println!("\n## Cody 0x210 Q (dmid leader) + Cephes 0x5005 tail");
    let (m, t) = score(&rows, |z| {
        let ff = if z < 4.9 {
            cody_mask(z, 0x210)
        } else {
            cephes_mask(z, 0x5005)
        };
        qw(z, ff)
    });
    println!("cody210/cephes5005 cut4.9 mid {} tail {}", fmt(&m), fmt(&t));

    println!("\n## joint C/D (C0+4 C1+1 D0-1 D7-1) as Q=w*F64");
    let (m, t) = score(&rows, |z| {
        let mut cc = C;
        let mut dd = D;
        for _ in 0..4 {
            cc[0] = cc[0].next_up();
        }
        cc[1] = cc[1].next_up();
        dd[0] = dd[0].next_down();
        dd[7] = dd[7].next_down();
        let ye = ef(z.abs());
        if z.abs() <= 4.0 {
            let mut xnum = ext_mul(&ef(cc[8]), &ye, CW);
            let mut xden = ye;
            for i in 0..7 {
                xnum = ext_mul(&ext_add(&xnum, &ef(cc[i]), CW), &ye, CW);
                xden = ext_mul(&ext_add(&xden, &ef(dd[i]), CW), &ye, CW);
            }
            let ff = ext_to_f64(
                &ext_div(
                    &ext_add(&xnum, &ef(cc[7]), CW),
                    &ext_add(&xden, &ef(dd[7]), CW),
                    CW,
                ),
                CW,
            );
            qw(z, ff)
        } else {
            qw(z, cody_mask(z, 0))
        }
    });
    println!("joint C/D Q mid {} tail {}", fmt(&m), fmt(&t));
}
