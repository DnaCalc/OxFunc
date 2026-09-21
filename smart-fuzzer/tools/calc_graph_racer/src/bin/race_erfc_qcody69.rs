//! Cody 1969 Math. Comp. Table III n=7 and Table IV n=4/5 as Q=w*F.
//! Table III n=8 extra paper digits are f64-identical to SPECFUN C/D.
//! Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const SQRPI: f64 = 0.5641895835477562869480794515607726;

const C8: [f64; 9] = [
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
const D8: [f64; 8] = [
    15.7449261107098347,
    117.693950891312499,
    537.181101862009858,
    1621.38957456669019,
    3290.79923573345963,
    4362.61909014324716,
    3439.36767414372164,
    1230.33935480374942,
];

const P7: [f64; 8] = [
    3.004592610201616005e2,
    4.519189537118729422e2,
    3.393208167343436870e2,
    1.529892850469404039e2,
    4.316222722205673530e1,
    7.211758250883093659,
    5.641955174789739711e-1,
    -1.368648573827167067e-7,
];
const Q7: [f64; 8] = [
    3.004592609569832933e2,
    7.909509253278980272e2,
    9.313540948506096211e2,
    6.389802644656311665e2,
    2.775854447439876434e2,
    7.700015293522947295e1,
    1.278272731962942351e1,
    1.0,
];

const P4: [f64; 5] = [
    7.3738883116,
    6.8650184849,
    3.0317993362,
    5.6316961891e-1,
    4.318778745e-5,
];
const Q4: [f64; 5] = [
    7.3739608908,
    1.5184908190e1,
    1.2795529509e1,
    5.3542167949,
    1.0,
];

const TV_P5: [f64; 6] = [
    -6.58749161529837803157e-4,
    -1.60837851487422766278e-2,
    -1.25781726111229246e-1,
    -3.60344899949804439429e-1,
    -3.05326634961232344035e-1,
    -1.63153871373020978498e-2,
];
const TV_Q5: [f64; 6] = [
    2.33520497626869185443e-3,
    6.05183413124413191178e-2,
    5.27905102951428412248e-1,
    1.87295284992346047209,
    2.56852019228982242072,
    1.0,
];

const TV_P4: [f64; 5] = [
    -2.99610707703542174e-3,
    -4.94730910623250734e-2,
    -2.26956593539686930e-1,
    -2.78661308609647788e-1,
    -2.23192459734184686e-2,
];
const TV_Q4: [f64; 5] = [
    1.06209230528467918e-2,
    1.91308926107829841e-1,
    1.05167510706793207,
    1.98733201817135256,
    1.0,
];

const PQ_P: [f64; 6] = [
    3.05326634961232344e-1,
    3.60344899949804439e-1,
    1.25781726111229246e-1,
    1.60837851487422766e-2,
    6.58749161529837803e-4,
    1.63153871373020978e-2,
];
const PQ_Q: [f64; 5] = [
    2.56852019228982242,
    1.87295284992346047,
    5.27905102951428412e-1,
    6.05183413124413191e-2,
    2.33520497626869185e-3,
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn horner(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
    }
    acc
}
fn cody_cd(y: f64, c: &[f64], d: &[f64]) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(*c.last().unwrap()), &ye, CW);
    let mut xden = ye;
    let n = d.len();
    for i in 0..n.saturating_sub(1) {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_add(&xnum, &ef(c[n - 1]), CW),
            &ext_add(&xden, &ef(d[n - 1]), CW),
            CW,
        ),
        CW,
    )
}
fn rat_fwd(p: &[f64], q: &[f64], y: f64) -> f64 {
    let ye = ef(y.abs());
    ext_to_f64(&ext_div(&horner(p, ye), &horner(q, ye), CW), CW)
}
fn cody8(y: f64) -> f64 {
    cody_cd(y, &C8, &D8)
}
fn cody7(y: f64) -> f64 {
    rat_fwd(&P7, &Q7, y)
}
fn cody4mid(y: f64) -> f64 {
    rat_fwd(&P4, &Q4, y)
}
fn specfun_pq(x: f64) -> f64 {
    let y = x.abs();
    let ye = ef(y);
    let ysq = ext_mul(&ye, &ye, CW);
    let mut xnum = ext_mul(&ef(PQ_P[5]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..4 {
        xnum = ext_mul(&ext_add(&xnum, &ef(PQ_P[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(PQ_Q[i]), CW), &ysq, CW);
    }
    let r = ext_div(&ext_add(&xnum, &ef(PQ_P[4]), CW), &ext_add(&xden, &ef(PQ_Q[4]), CW), CW);
    ext_to_f64(&ext_div(&r, &ye, CW), CW)
}
fn table_iv(x: f64, p: &[f64], q: &[f64]) -> f64 {
    let y = x.abs();
    let ye = ef(y);
    let z = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
    let r = ext_div(&horner(p, z), &horner(q, z), CW);
    let inner = ext_add(&ef(SQRPI), &ext_mul(&z, &r, CW), CW);
    ext_to_f64(&ext_div(&inner, &ye, CW), CW)
}
fn qw(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}

#[derive(Default)]
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

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let graphs: [(&str, fn(f64) -> f64); 6] = [
        ("SPECFUN C/D n=8 x87", |z| qw(z, cody8(z))),
        ("Cody1969 TIII n=7 x87", |z| qw(z, cody7(z))),
        ("Cody1969 TIII n=4 x87", |z| qw(z, cody4mid(z))),
        ("n=7 z<4 else SPECFUN P/Q", |z| {
            if z < 4.0 {
                qw(z, cody7(z))
            } else {
                qw(z, specfun_pq(z))
            }
        }),
        ("C/D z<4 else TableIV n=5", |z| {
            if z < 4.0 {
                qw(z, cody8(z))
            } else {
                qw(z, table_iv(z, &TV_P5, &TV_Q5))
            }
        }),
        ("C/D z<4 else TableIV n=4", |z| {
            if z < 4.0 {
                qw(z, cody8(z))
            } else {
                qw(z, table_iv(z, &TV_P4, &TV_Q4))
            }
        }),
    ];
    for (name, ev) in graphs {
        let mut mid = Acc::default();
        let mut tail = Acc::default();
        for r in &rows {
            if r.z < 0.5 {
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
            if r.z < 4.0 {
                mid.add(d, r.direct);
            } else {
                tail.add(d, r.direct);
            }
        }
        println!("{name:40} mid {} tail {}", fmt(&mid), fmt(&tail));
    }
}
