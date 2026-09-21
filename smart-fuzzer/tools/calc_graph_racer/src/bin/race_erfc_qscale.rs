//! Monic / scaled Cody C/D tables as Q=w*F. Conditioning forms, not ULP cubes.
//! Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn cody_f(y: f64, c: &[f64; 9], d: &[f64; 8]) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(c[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(c[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(d[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(c[7]), CW), &ext_add(&xden, &ef(d[7]), CW), CW),
        CW,
    )
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
fn score(rows: &[f::QRow], c: &[f64; 9], d: &[f64; 8]) -> Acc {
    let mut mid = Acc::default();
    for r in rows {
        if r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let qg = qw(r.z, cody_f(r.z, c, d));
        if !qg.is_finite() {
            continue;
        }
        let dist = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(u64::MAX);
        if dist > ULP_CAP {
            continue;
        }
        mid.add(dist, r.direct);
    }
    mid
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    println!("CR          {}", fmt(&score(&rows, &C0, &D0)));
    println!(
        "C7==D7 bits C7={:016x} D7={:016x} eq={}",
        C0[7].to_bits(),
        D0[7].to_bits(),
        C0[7].to_bits() == D0[7].to_bits()
    );

    let mut c = C0;
    let mut d = D0;
    c[7] = d[7];
    println!("C7 set to D7 {}", fmt(&score(&rows, &c, &d)));
    c = C0;
    d = D0;
    d[7] = c[7];
    println!("D7 set to C7 {}", fmt(&score(&rows, &c, &d)));

    let s = D0[7];
    let mut cm = C0;
    let mut dm = D0;
    for i in 0..9 {
        cm[i] /= s;
    }
    for i in 0..8 {
        dm[i] /= s;
    }
    println!("monic /D7    {}", fmt(&score(&rows, &cm, &dm)));

    let s = C0[7];
    cm = C0;
    dm = D0;
    for i in 0..9 {
        cm[i] /= s;
    }
    for i in 0..8 {
        dm[i] /= s;
    }
    println!("monic /C7    {}", fmt(&score(&rows, &cm, &dm)));

    cm = C0;
    cm[8] = 0.0;
    println!("C8=0 drop    {}", fmt(&score(&rows, &cm, &D0)));

    for k in -6i32..=6 {
        let sc = 2f64.powi(k);
        let mut cs = C0;
        let mut ds = D0;
        for i in 0..9 {
            cs[i] *= sc;
        }
        for i in 0..8 {
            ds[i] *= sc;
        }
        println!("scale 2^{k:+}   {}", fmt(&score(&rows, &cs, &ds)));
    }

    // RN53 of each C,D through x87 / D7 then loop
    let den = ef(D0[7]);
    let mut cx = C0;
    let mut dx = D0;
    for i in 0..9 {
        cx[i] = ext_to_f64(&ext_div(&ef(C0[i]), &den, CW), CW);
    }
    for i in 0..8 {
        dx[i] = ext_to_f64(&ext_div(&ef(D0[i]), &den, CW), CW);
    }
    println!("x87-monic /D7 {}", fmt(&score(&rows, &cx, &dx)));
}
