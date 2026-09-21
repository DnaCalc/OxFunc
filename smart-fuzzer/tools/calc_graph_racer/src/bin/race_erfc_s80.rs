//! Sun/BSD McIlroy erf.c 80-bit printed decimals as Q=w*F. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{
    excel_exp, ext_add, ext_chs, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN,
};
use std::env;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const LSQRTPI_HI: f64 = 0.5723649429247000819387380943226;
const LSQRTPI80: Ext80 = Ext80([0x00, 0xe8, 0x0d, 0x3d, 0x47, 0x82, 0x86, 0x92, 0xfe, 0x3f]);
const RB: [f64; 11] = [
    -1.5306508387410807582e-10,
    2.15592846101742183841910806188e-8,
    6.24998557732436510470108714799e-1,
    8.24849222231141787631258921465,
    2.63974967372233173534823436057e1,
    9.86383092541570505318304640241,
    -7.28024154841991322228977878694,
    5.96303287280680116566600190708,
    -4.40070358507372993983608466806,
    2.39923700182518073731330332521,
    -6.89257464785841156285073338950e-1,
];
const SB: [f64; 4] = [
    1.0,
    1.56641558965626774835300238919e1,
    7.20522741000949622502957936376e1,
    9.60121069770492994166488642804e1,
];
const RC: [f64; 11] = [
    -2.47925334685189288817e-7,
    1.28735722546372485255126993930e-5,
    6.24664954087883916855616917019e-1,
    4.69798884785807402408863708843,
    7.61618295853929705430118701770,
    9.15640208659364240872946538730e-1,
    -3.59753040425048631334448145935e-1,
    1.42862267989304403403849619281e-1,
    -4.74392758811439801958087514322e-2,
    1.09964787987580810135757047874e-2,
    -1.28856240494889325194638463046e-3,
];
const SC: [f64; 4] = [
    1.0,
    9.97395106984001955652274773456,
    2.80952153365721279953959310660e1,
    2.19826478142545234106819407316e1,
];
const RB80: [Ext80; 11] = [
    Ext80([0x8c, 0x10, 0xb8, 0xd8, 0xac, 0xfd, 0x4b, 0xa8, 0xde, 0xbf]),
    Ext80([0x00, 0x30, 0x4c, 0x60, 0x44, 0x5e, 0x31, 0xb9, 0xe5, 0x3f]),
    Ext80([0x00, 0x28, 0x0b, 0x82, 0xcd, 0xe7, 0xff, 0x9f, 0xfe, 0x3f]),
    Ext80([0x00, 0xd8, 0x34, 0x02, 0xfb, 0xd2, 0xf9, 0x83, 0x02, 0x40]),
    Ext80([0x00, 0x78, 0x20, 0xf5, 0xc4, 0x12, 0x2e, 0xd3, 0x03, 0x40]),
    Ext80([0x00, 0x20, 0xf1, 0x5e, 0x60, 0x40, 0xd2, 0x9d, 0x02, 0x40]),
    Ext80([0x00, 0xa8, 0x34, 0xae, 0x1f, 0xbd, 0xf7, 0xe8, 0x01, 0xc0]),
    Ext80([0x00, 0x50, 0xb3, 0xb5, 0x50, 0x2a, 0xd1, 0xbe, 0x01, 0x40]),
    Ext80([0x00, 0x10, 0x03, 0x29, 0x53, 0x90, 0xd2, 0x8c, 0x01, 0xc0]),
    Ext80([0x00, 0xb8, 0x4d, 0x8c, 0x5a, 0x19, 0x8d, 0x99, 0x00, 0x40]),
    Ext80([0x00, 0x70, 0x70, 0xc7, 0x5d, 0x2d, 0x73, 0xb0, 0xfe, 0xbf]),
];
const SB80: [Ext80; 4] = [
    Ext80([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xff, 0x3f]),
    Ext80([0x00, 0x88, 0xea, 0xf2, 0xee, 0x61, 0xa0, 0xfa, 0x02, 0x40]),
    Ext80([0x00, 0xe0, 0xab, 0xbc, 0xab, 0xc3, 0x1a, 0x90, 0x05, 0x40]),
    Ext80([0x00, 0xd8, 0xf5, 0xbc, 0xe2, 0x32, 0x06, 0xc0, 0x05, 0x40]),
];
const RC80: [Ext80; 11] = [
    Ext80([0x00, 0x20, 0xcf, 0xce, 0x39, 0x99, 0x1a, 0x85, 0xe9, 0xbf]),
    Ext80([0x00, 0x60, 0x95, 0x8e, 0x62, 0x92, 0xfb, 0xd7, 0xee, 0x3f]),
    Ext80([0x00, 0x20, 0xca, 0xc3, 0xdc, 0x0a, 0xea, 0x9f, 0xfe, 0x3f]),
    Ext80([0x00, 0xa0, 0xbd, 0x50, 0xb5, 0xec, 0x55, 0x96, 0x01, 0x40]),
    Ext80([0x00, 0xf8, 0xeb, 0xe8, 0x52, 0xc5, 0xb7, 0xf3, 0x01, 0x40]),
    Ext80([0x00, 0xc0, 0x36, 0x18, 0x8f, 0x65, 0x67, 0xea, 0xfe, 0x3f]),
    Ext80([0x00, 0x80, 0x38, 0x86, 0xee, 0x8c, 0x31, 0xb8, 0xfd, 0xbf]),
    Ext80([0x00, 0xd0, 0xc3, 0x62, 0x83, 0x7c, 0x4a, 0x92, 0xfc, 0x3f]),
    Ext80([0x00, 0x30, 0x76, 0x49, 0xa7, 0xaf, 0x4f, 0xc2, 0xfa, 0xbf]),
    Ext80([0x00, 0xd0, 0xe9, 0xf4, 0x33, 0x93, 0x2a, 0xb4, 0xf8, 0x3f]),
    Ext80([0x00, 0xe8, 0x6e, 0xb6, 0xc6, 0xfa, 0xe4, 0xa8, 0xf5, 0xbf]),
];
const SC80: [Ext80; 4] = [
    Ext80([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xff, 0x3f]),
    Ext80([0x00, 0xa0, 0xdf, 0x8d, 0xb7, 0x4d, 0x95, 0x9f, 0x02, 0x40]),
    Ext80([0x00, 0x48, 0x3d, 0x25, 0x42, 0x00, 0xc3, 0xe0, 0x03, 0x40]),
    Ext80([0x00, 0x28, 0xac, 0x0d, 0x75, 0x76, 0xdc, 0xaf, 0x03, 0x40]),
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn horner_f(cs: &[f64], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for &c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), &ef(c), CW);
    }
    acc
}
fn horner_80(cs: &[Ext80], x: Ext80) -> Ext80 {
    let mut acc = ef(0.0);
    for c in cs.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &x, CW), c, CW);
    }
    acc
}
fn sun_corr_f64(x: f64) -> f64 {
    let y = x.abs();
    let ye = ef(y);
    let s = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
    let (r, den) = if y < 2.0 {
        (horner_f(&RC, s), horner_f(&SC, s))
    } else {
        (horner_f(&RB, s), horner_f(&SB, s))
    };
    let extra = ext_mul(&ef(-0.5), &s, CW);
    let ycorr = ext_add(
        &ext_add(&ext_div(&r, &den, CW), &extra, CW),
        &ef(-LSQRTPI_HI),
        CW,
    );
    ext_to_f64(&ycorr, CW)
}
fn sun_corr_80(x: f64) -> f64 {
    let y = x.abs();
    let ye = ef(y);
    let s = ext_div(&ef(1.0), &ext_mul(&ye, &ye, CW), CW);
    let (r, den) = if y < 2.0 {
        (horner_80(&RC80, s), horner_80(&SC80, s))
    } else {
        (horner_80(&RB80, s), horner_80(&SB80, s))
    };
    let extra = ext_mul(&ef(-0.5), &s, CW);
    let ycorr = ext_add(
        &ext_add(&ext_div(&r, &den, CW), &extra, CW),
        &ext_chs(&LSQRTPI80, CW),
        CW,
    );
    ext_to_f64(&ycorr, CW)
}
fn sun_f64(x: f64) -> f64 {
    excel_exp(sun_corr_f64(x)) / x.abs()
}
fn sun_80(x: f64) -> f64 {
    excel_exp(sun_corr_80(x)) / x.abs()
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
    for (name, ev) in [
        ("Sun f64-wide", sun_f64 as fn(f64) -> f64),
        ("Sun 80-bit decimals", sun_80),
    ] {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        let mut dmid = 0usize;
        let mut dn = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            let qg = qw(r.z, ev(r.z));
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
                    dmid += 1;
                }
            } else {
                maxu = maxu.max(d);
            }
        }
        println!("{name:22} {ex}/{n} max={maxu} dmid={dmid}/{dn}");
    }
}
