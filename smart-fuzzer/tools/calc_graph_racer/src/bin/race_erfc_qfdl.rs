//! fdlibm RA/SA/RB as Q=w*F vs Cody C/D. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{excel_exp, ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn cody_cd(y: f64) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C[8]), &ye, CW);
    let mut xden = ye;
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D[i]), CW), &ye, CW);
    }
    ext_to_f64(
        &ext_div(&ext_add(&xnum, &ef(C[7]), CW), &ext_add(&xden, &ef(D[7]), CW), CW),
        CW,
    )
}
fn fdlibm_f(x: f64) -> f64 {
    const RA: [f64; 8] = [
        -9.86494403484714822705e-03,
        -6.93858572707181764372e-01,
        -1.05586262253232909814e+01,
        -6.23753324503260060396e+01,
        -1.62396669462573470355e+02,
        -1.84605092906711035994e+02,
        -8.12874355063065934246e+01,
        -9.81432934416914548592e+00,
    ];
    const SA: [f64; 8] = [
        1.96512716674392571292e+01,
        1.37657754143519042600e+02,
        4.34565877475229228821e+02,
        6.45387271733267880336e+02,
        4.29008140027567833386e+02,
        1.08635005541779435134e+02,
        6.57024977031928170135e+00,
        -6.04244152148580987438e-02,
    ];
    const RB: [f64; 7] = [
        -9.86494292470009928597e-03,
        -7.99283237680523006574e-01,
        -1.77579549177547519889e+01,
        -1.60636384855821916062e+02,
        -6.37566443368389627722e+02,
        -1.02509513161107724954e+03,
        -4.83519191608651397019e+02,
    ];
    const SB: [f64; 7] = [
        3.03380607434824582924e+01,
        3.25792512996573918826e+02,
        1.53672958608443695994e+03,
        3.19985821950859553908e+03,
        2.55305040643316442583e+03,
        4.74528541206955367215e+02,
        -2.24409524465858183362e+01,
    ];
    let ax = x.abs();
    let ix = ((ax.to_bits() >> 32) as u32) & 0x7fffffff;
    if ix < 0x3ff40000 {
        return f64::NAN;
    }
    let s = 1.0 / (ax * ax);
    let (rr, ss) = if ix < 0x4006db6d {
        let r = RA[0]
            + s * (RA[1]
                + s * (RA[2]
                    + s * (RA[3] + s * (RA[4] + s * (RA[5] + s * (RA[6] + s * RA[7]))))));
        let st = 1.0
            + s * (SA[0]
                + s * (SA[1]
                    + s * (SA[2]
                        + s * (SA[3] + s * (SA[4] + s * (SA[5] + s * (SA[6] + s * SA[7])))))));
        (r, st)
    } else {
        let r = RB[0]
            + s * (RB[1] + s * (RB[2] + s * (RB[3] + s * (RB[4] + s * (RB[5] + s * RB[6])))));
        let st = 1.0
            + s * (SB[0]
                + s * (SB[1] + s * (SB[2] + s * (SB[3] + s * (SB[4] + s * (SB[5] + s * SB[6]))))));
        (r, st)
    };
    let z = f64::from_bits(ax.to_bits() & 0xffff_ffff_0000_0000);
    let q = excel_exp(-z * z - 0.5625) * excel_exp((z - ax) * (z + ax) + rr / ss) / ax;
    let w = excel_exp(-(ax * ax));
    if w == 0.0 {
        f64::NAN
    } else {
        q / w
    }
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
    let graphs: [(&str, fn(f64) -> f64); 3] = [
        ("cody C/D all mid", |z| qw(z, cody_cd(z))),
        ("cody z<1.25 else fdlibm F", |z| {
            if z < 1.25 {
                qw(z, cody_cd(z))
            } else {
                qw(z, fdlibm_f(z))
            }
        }),
        ("fdlibm F z>=1.25 only (nan below)", |z| qw(z, fdlibm_f(z))),
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
