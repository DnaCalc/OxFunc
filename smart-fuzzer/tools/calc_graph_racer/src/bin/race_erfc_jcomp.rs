//! 1−A21-joint as Q on [0.5,1) vs 0x210; compose by dmid. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::env;

const CW: u16 = CW_PC64_RN;
const AS0: [f64; 21] = [
    0.1283791670955125738961589031215,
    -0.3761263890318375246320529677070,
    0.1128379167095512573896158902931,
    -0.2686617064513125175943235372542e-01,
    0.5223977625442187842111812447877e-02,
    -0.8548327023450852832540164081187e-03,
    0.1205533298178966425020717182498e-03,
    -0.1492565035840625090430728526820e-04,
    0.1646211436588924261080723578109e-05,
    -0.1636584469123468757408968429674e-06,
    0.1480719281587021715400818627811e-07,
    -0.1229055530145120140800510155331e-08,
    0.9422759058437197017313055084212e-10,
    -0.6711366740969385085896257227159e-11,
    0.4463222608295664017461758843550e-12,
    -0.2783497395542995487275065856998e-13,
    0.1634095572365337143933023780777e-14,
    -0.9052845786901123985710019387938e-16,
    0.4708274559689744439341671426731e-17,
    -0.2187159356685015949749948252160e-18,
    0.7043407712019701609635599701333e-20,
];
const AA: [f64; 5] = [
    3.16112374387056560,
    113.864154151050156,
    377.485237685302021,
    3209.37758913846947,
    0.185777706184603153,
];
const BB: [f64; 4] = [
    23.6012909523441209,
    244.024637934444173,
    1282.61652607737228,
    2844.23683343917062,
];
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
fn erf_a(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z.abs());
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn cody_ab(z: f64) -> f64 {
    let ye = ef(z.abs());
    let ysq = ext_mul(&ye, &ye, CW);
    let mut xnum = ext_mul(&ef(AA[4]), &ysq, CW);
    let mut xden = ysq;
    for i in 0..3 {
        xnum = ext_mul(&ext_add(&xnum, &ef(AA[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(BB[i]), CW), &ysq, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_mul(&ye, &ext_add(&xnum, &ef(AA[3]), CW), CW),
            &ext_add(&xden, &ef(BB[3]), CW),
            CW,
        ),
        CW,
    )
}
fn cody_cd(y: f64, mask: u32) -> f64 {
    let ye = ef(y.abs());
    let mut xnum = ext_mul(&ef(C0[8]), &ye, CW);
    let mut xden = ye;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..7 {
        xnum = ext_mul(&ext_add(&xnum, &ef(C0[i]), CW), &ye, CW);
        xden = ext_mul(&ext_add(&xden, &ef(D0[i]), CW), &ye, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_add(&xnum, &ef(C0[7]), CW),
        &ext_add(&xden, &ef(D0[7]), CW),
        CW,
    );
    q = maybe(q, mask, 16);
    ext_to_f64(&q, CW)
}
fn q1p(p: f64) -> f64 {
    ext_to_f64(&ext_sub(&ef(1.0), &ef(p), CW), CW)
}
fn qwf(z: f64, ff: f64) -> f64 {
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
    let mut joint = AS0;
    joint[0] = poke(AS0[0], 4);
    joint[1] = poke(AS0[1], -2);
    joint[2] = poke(AS0[2], -5);
    joint[3] = poke(AS0[3], 1);

    println!("0x210 DIRECT hard [0.5,2):");
    let mut hard01: Vec<&f::QRow> = Vec::new();
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 2.0 {
            continue;
        }
        let g = qwf(r.z, cody_cd(r.z, 0x210));
        let t = f64::from_bits(r.qbits);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d >= 2 {
            hard01.push(r);
            println!(
                "  z={:.16} ulp={d} {} band={}",
                r.z,
                if g < t { "low" } else { "high" },
                if r.z < 1.0 { "[0.5,1)" } else { "[1,2)" }
            );
        }
    }

    let qgraphs: [(&str, Box<dyn Fn(f64) -> f64>); 6] = [
        ("0x210", Box::new(|z| qwf(z, cody_cd(z, 0x210)))),
        ("1-joint", Box::new(|z| q1p(erf_a(z, &joint)))),
        ("1-A21", Box::new(|z| q1p(erf_a(z, &AS0)))),
        ("1-AB", Box::new(|z| q1p(cody_ab(z)))),
        ("1-libm erf", Box::new(|z| 1.0 - libm::erf(z))),
        ("libm erfc", Box::new(|z| libm::erfc(z))),
    ];
    println!("named on [0.5,1) DIRECT / hard:");
    for (name, ev) in &qgraphs {
        let mut ex01 = 0usize;
        let mut n01 = 0usize;
        let mut hit = 0usize;
        for r in &rows {
            if !r.direct || r.z < 0.5 || r.z >= 1.0 {
                continue;
            }
            n01 += 1;
            let d = ulp_distance(ev(r.z), f64::from_bits(r.qbits)).unwrap_or(99);
            if d == 0 {
                ex01 += 1;
            }
        }
        for r in &hard01 {
            if r.z >= 1.0 {
                continue;
            }
            let d = ulp_distance(ev(r.z), f64::from_bits(r.qbits)).unwrap_or(99);
            if d == 0 {
                hit += 1;
                println!("  HIT {name} z={:.16}", r.z);
            }
        }
        println!("{name:12} [0.5,1) {ex01}/{n01} hard_hit={hit}");
    }

    println!("1-joint below cut else 0x210, DIRECT dmid (bar 105):");
    let mut best = 0usize;
    let mut best_c = 0.0;
    let mut best_mx = 99u64;
    for k in 0..=40 {
        let c = 0.5 + k as f64 * 0.02;
        let mut dmid = 0usize;
        let mut n = 0usize;
        let mut mx = 0u64;
        let mut h01 = 0usize;
        for r in &rows {
            if !r.direct || r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            n += 1;
            let qg = if r.z < c {
                q1p(erf_a(r.z, &joint))
            } else {
                qwf(r.z, cody_cd(r.z, 0x210))
            };
            let d = ulp_distance(qg, f64::from_bits(r.qbits)).unwrap_or(99);
            if d == 0 {
                dmid += 1;
                if r.z < 1.0 {
                    h01 += 1;
                }
            } else {
                mx = mx.max(d);
            }
        }
        let show = (c - 0.5).abs() < 1e-12
            || (c - 0.6174).abs() < 1e-3
            || (c - 0.75).abs() < 1e-12
            || (c - 0.84375).abs() < 1e-12
            || (c - 1.0).abs() < 1e-12
            || dmid >= 105;
        if show {
            println!("  cut={c:.4} dmid={dmid}/{n} maxd={mx} [0.5,1)ex={h01}");
        }
        if dmid > best || (dmid == best && mx < best_mx) {
            best = dmid;
            best_c = c;
            best_mx = mx;
        }
    }
    println!("  BEST dmid {best} maxd={best_mx} @ {best_c:.4}");
}
