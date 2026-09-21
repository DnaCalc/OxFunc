//! Schonfelder 80-bit coeffs × t-map association on P-side. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const SCH: [f64; 18] = [
    1.4831105640848035818894480790578,
    -0.3010710733865949424707310463118,
    0.0689948306898315662466031807188,
    -0.0139162712647221876825465256678,
    0.0024207995224334636628916782398,
    -0.0003658639685848086446493825778,
    4.86209844323190482828875688e-5,
    -5.7492565580356848350542158e-6,
    6.113243578434764697067588e-7,
    -5.89910153129584343908468e-8,
    5.2070090920686482404558e-9,
    -4.232975879965543268108e-10,
    3.18811350664917497488e-11,
    -2.2361550188326842738e-12,
    1.467329847991084928e-13,
    -9.0440019853817478e-15,
    5.254813715470928e-16,
    -2.88742612228498e-17,
];
const SCH_80: [Ext80; 18] = [
    Ext80([0x51, 0xfc, 0x54, 0x8c, 0x24, 0x91, 0xd6, 0xbd, 0xff, 0x3f]),
    Ext80([0x9f, 0x07, 0x20, 0xef, 0xdb, 0xfc, 0x25, 0x9a, 0xfd, 0xbf]),
    Ext80([0x9d, 0xf3, 0x3f, 0x3f, 0x6b, 0x29, 0x4d, 0x8d, 0xfb, 0x3f]),
    Ext80([0x99, 0x57, 0x36, 0xb6, 0x7d, 0x12, 0x01, 0xe4, 0xf8, 0xbf]),
    Ext80([0x93, 0x58, 0x20, 0x6e, 0xc7, 0x46, 0xa6, 0x9e, 0xf6, 0x3f]),
    Ext80([0x78, 0xe3, 0x6a, 0x25, 0x3d, 0x6e, 0xd1, 0xbf, 0xf3, 0xbf]),
    Ext80([0x62, 0x71, 0x1d, 0x2f, 0x6f, 0x62, 0xee, 0xcb, 0xf0, 0x3f]),
    Ext80([0x8c, 0x16, 0xa9, 0x8d, 0xdf, 0xbc, 0xe9, 0xc0, 0xed, 0xbf]),
    Ext80([0x28, 0x26, 0xb7, 0x2f, 0xd6, 0xe3, 0x19, 0xa4, 0xea, 0x3f]),
    Ext80([0xaa, 0xe7, 0x6c, 0x4e, 0xa9, 0x4e, 0x5d, 0xfd, 0xe6, 0xbf]),
    Ext80([0x6f, 0xe5, 0xf3, 0x76, 0x1a, 0x56, 0xe9, 0xb2, 0xe3, 0x3f]),
    Ext80([0x27, 0x07, 0x3f, 0x65, 0xe0, 0xd6, 0xb5, 0xe8, 0xdf, 0xbf]),
    Ext80([0x5e, 0xef, 0x5c, 0x67, 0x8d, 0xf7, 0x36, 0x8c, 0xdc, 0x3f]),
    Ext80([0x04, 0xd7, 0xea, 0xdb, 0xd5, 0xfc, 0x5a, 0x9d, 0xd8, 0xbf]),
    Ext80([0x6a, 0x79, 0x0d, 0x63, 0x45, 0xe7, 0x34, 0xa5, 0xd4, 0x3f]),
    Ext80([0xf4, 0x9b, 0x88, 0x31, 0xf6, 0x18, 0xec, 0xa2, 0xd0, 0xbf]),
    Ext80([0x91, 0x5a, 0xbe, 0x28, 0x77, 0xae, 0x75, 0x97, 0xcc, 0x3f]),
    Ext80([0xc0, 0xf3, 0xd0, 0x88, 0xfa, 0xb5, 0x28, 0x85, 0xc8, 0xbf]),
];

fn ef(x: f64) -> Ext80 {
    ext_from_f64(x)
}
fn clenshaw_prime(t: Ext80, a: &[Ext80]) -> Ext80 {
    let two = ef(2.0);
    let mut d1 = ef(0.0);
    let mut d2 = ef(0.0);
    for k in (1..a.len()).rev() {
        let dk = ext_add(
            &ext_sub(&ext_mul(&ext_mul(&two, &t, CW), &d1, CW), &d2, CW),
            &a[k],
            CW,
        );
        d2 = d1;
        d1 = dk;
    }
    ext_add(
        &ext_sub(&ext_mul(&t, &d1, CW), &d2, CW),
        &ext_mul(&ef(0.5), &a[0], CW),
        CW,
    )
}
fn t_half_mul(xe: Ext80) -> Ext80 {
    ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW)
}
fn t_mul_half(xe: Ext80) -> Ext80 {
    ext_sub(&ext_mul(&ext_mul(&xe, &xe, CW), &ef(0.5), CW), &ef(1.0), CW)
}
fn t_div2(xe: Ext80) -> Ext80 {
    ext_sub(
        &ext_div(&ext_mul(&xe, &xe, CW), &ef(2.0), CW),
        &ef(1.0),
        CW,
    )
}
fn t_x_xhalf(xe: Ext80) -> Ext80 {
    ext_sub(
        &ext_mul(&xe, &ext_div(&xe, &ef(2.0), CW), CW),
        &ef(1.0),
        CW,
    )
}
fn t_xxm2_over2(xe: Ext80) -> Ext80 {
    ext_div(
        &ext_sub(&ext_mul(&xe, &xe, CW), &ef(2.0), CW),
        &ef(2.0),
        CW,
    )
}
fn t_half_from_div(xe: Ext80) -> Ext80 {
    let h = ext_div(&ef(1.0), &ef(2.0), CW);
    ext_sub(&ext_mul(&h, &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW)
}
fn erf_with(z: f64, tmap: fn(Ext80) -> Ext80, a: &[Ext80]) -> f64 {
    let xe = ef(z.abs());
    let y = clenshaw_prime(tmap(xe), a);
    ext_to_f64(&ext_mul(&xe, &y, CW), CW)
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    const BANKS: [&str; 7] = [
        "answers-b9train.json",
        "answers-erfp.json",
        "answers-erfm.json",
        "answers-b8erf.json",
        "answers-b7erf.json",
        "answers-b11.json",
        "answers-b10.json",
    ];
    let mut map = BTreeMap::new();
    for name in BANKS {
        let path = format!("{dir}/{name}");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let bank: WitnessSet = serde_json::from_str(&text).unwrap();
        for w in &bank.witnesses {
            let x = match &w.args[0] {
                WitnessArg::Scalar(s) => parse_bits_hex(s).unwrap(),
                _ => continue,
            };
            if x < 0.0 || x >= 0.5 {
                continue;
            }
            let Some(p) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            map.insert(x.to_bits(), (x, p.to_bits()));
        }
    }
    let rows: Vec<_> = map.into_values().collect();
    let sch_f: [Ext80; 18] = SCH.map(ef);
    let maps: [(&str, fn(Ext80) -> Ext80); 6] = [
        ("0.5*(x*x)-1", t_half_mul),
        ("(x*x)*0.5-1", t_mul_half),
        ("(x*x)/2-1", t_div2),
        ("x*(x/2)-1", t_x_xhalf),
        ("(x*x-2)/2", t_xxm2_over2),
        ("(1/2)*(x*x)-1", t_half_from_div),
    ];
    println!("P-side rows={}", rows.len());
    for (mname, tmap) in maps {
        for (cname, cs) in [("f64", &sch_f[..]), ("80bit", &SCH_80[..])] {
            let mut ex = 0usize;
            let mut n = 0usize;
            let mut maxu = 0u64;
            for &(z, pbits) in &rows {
                let pg = erf_with(z, tmap, cs);
                if !pg.is_finite() {
                    continue;
                }
                let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
                if d > ULP_CAP {
                    continue;
                }
                n += 1;
                if d == 0 {
                    ex += 1;
                } else {
                    maxu = maxu.max(d);
                }
            }
            println!("{mname:16} {cname:5} {ex}/{n} max={maxu}");
        }
    }
    let xe_t = |z: f64| {
        let xe = ef(z.abs());
        (xe, t_half_mul(xe))
    };
    let clenshaw_last = |z: f64, last: fn(Ext80) -> Ext80, a: &[Ext80]| {
        let (xe, t) = xe_t(z);
        let two = ef(2.0);
        let mut d1 = ef(0.0);
        let mut d2 = ef(0.0);
        for k in (1..a.len()).rev() {
            let dk = ext_add(
                &ext_sub(&ext_mul(&ext_mul(&two, &t, CW), &d1, CW), &d2, CW),
                &a[k],
                CW,
            );
            d2 = d1;
            d1 = dk;
        }
        let y = ext_add(&ext_sub(&ext_mul(&t, &d1, CW), &d2, CW), &last(a[0]), CW);
        ext_to_f64(&ext_mul(&xe, &y, CW), CW)
    };
    let last_half_mul = |a0: Ext80| ext_mul(&ef(0.5), &a0, CW);
    let last_div2 = |a0: Ext80| ext_div(&a0, &ef(2.0), CW);
    let last_one_over_two = |a0: Ext80| {
        ext_mul(&ext_div(&ef(1.0), &ef(2.0), CW), &a0, CW)
    };
    let mut hybrid = sch_f;
    hybrid[0] = SCH_80[0];
    let lasts: [(&str, fn(Ext80) -> Ext80); 3] = [
        ("0.5*a0", last_half_mul),
        ("a0/2", last_div2),
        ("(1/2)*a0", last_one_over_two),
    ];
    for (lname, last) in lasts {
        for (cname, cs) in [("f64", &sch_f[..]), ("80bit", &SCH_80[..]), ("hyb a0", &hybrid[..])] {
            let mut ex = 0usize;
            let mut n = 0usize;
            let mut maxu = 0u64;
            for &(z, pbits) in &rows {
                let pg = clenshaw_last(z, last, cs);
                if !pg.is_finite() {
                    continue;
                }
                let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
                if d > ULP_CAP {
                    continue;
                }
                n += 1;
                if d == 0 {
                    ex += 1;
                } else {
                    maxu = maxu.max(d);
                }
            }
            println!("last {lname:10} {cname:7} {ex}/{n} max={maxu}");
        }
    }
    fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
        if bit < 32 && mask & (1u32 << bit) != 0 {
            ef(ext_to_f64(&x, CW))
        } else {
            x
        }
    }
    let sch_hw = |z: f64, mask: u32| {
        let xe = ef(z.abs());
        let mut t = t_half_mul(xe);
        t = maybe(t, mask, 0);
        let two = ef(2.0);
        let mut d1 = ef(0.0);
        let mut d2 = ef(0.0);
        let mut bit = 1u32;
        for k in (1..sch_f.len()).rev() {
            let dk = ext_add(
                &ext_sub(&ext_mul(&ext_mul(&two, &t, CW), &d1, CW), &d2, CW),
                &sch_f[k],
                CW,
            );
            d2 = d1;
            d1 = maybe(dk, mask, bit);
            bit += 1;
        }
        let mut y = ext_add(
            &ext_sub(&ext_mul(&t, &d1, CW), &d2, CW),
            &ext_mul(&ef(0.5), &sch_f[0], CW),
            CW,
        );
        y = maybe(y, mask, bit);
        ext_to_f64(&ext_mul(&xe, &y, CW), CW)
    };
    let score_m = |mask: u32| {
        let mut ex = 0usize;
        let mut n = 0usize;
        for &(z, pbits) in &rows {
            let pg = sch_hw(z, mask);
            if !pg.is_finite() {
                continue;
            }
            let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if d == 0 {
                ex += 1;
            }
        }
        (ex, n)
    };
    let base = score_m(0);
    println!("SCH HW0 {}/{}", base.0, base.1);
    let mut best = base.0;
    let mut best_b = None;
    for b in 0..20u32 {
        let sc = score_m(1u32 << b);
        if sc.0 != base.0 {
            println!("  bit{b} {}/{} dlt={}", sc.0, sc.1, sc.0 as i32 - base.0 as i32);
        }
        if sc.0 > best {
            best = sc.0;
            best_b = Some(b);
        }
    }
    println!("SCH HW1 best {best} bit={best_b:?} (bars 751/866)");
    let m0 = 1u32;
    let mut best2 = best;
    let mut best2b = None;
    for b in 1..20u32 {
        let sc = score_m(m0 | (1u32 << b));
        if sc.0 != best {
            println!(
                "  bit0+{b} {}/{} dlt={}",
                sc.0,
                sc.1,
                sc.0 as i32 - best as i32
            );
        }
        if sc.0 > best2 {
            best2 = sc.0;
            best2b = Some(b);
        }
    }
    println!("SCH HW2 from bit0 best {best2} +bit={best2b:?} (bars 751/866)");
}
