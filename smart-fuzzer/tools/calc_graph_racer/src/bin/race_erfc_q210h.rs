//! Named F on 0x210 Q-mid DIRECT hard; overlap vs CR/0x74/j3432. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
use calc_graph_racer::score::ulp_distance;
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeSet;
use std::env;

const CW: u16 = CW_PC64_RN;
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
fn qwf(z: f64, ff: f64) -> f64 {
    let v = f::w_rn53(z) * ff;
    if v.abs() < f64::MIN_POSITIVE {
        0.0
    } else {
        v
    }
}
fn hard_set(rows: &[f::QRow], ff: impl Fn(f64) -> f64) -> BTreeSet<u64> {
    let mut s = BTreeSet::new();
    for r in rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let d = ulp_distance(qwf(r.z, ff(r.z)), f64::from_bits(r.qbits)).unwrap_or(99);
        if d >= 2 {
            s.insert(r.z.to_bits());
        }
    }
    s
}

fn main() {
    let dir = env::args().nth(1).expect("dir");
    let rows = f::load_q_rows_tagged(&dir);
    let mut jc = C0;
    jc[0] = poke(C0[0], 4);
    jc[1] = poke(C0[1], 1);
    jc[4] = poke(C0[4], -1);
    let mut jd = D0;
    jd[0] = poke(D0[0], -1);
    jd[4] = poke(D0[4], -1);

    let h0 = hard_set(&rows, |z| cody(z, &C0, &D0, 0));
    let h74 = hard_set(&rows, |z| cody(z, &C0, &D0, 0x74));
    let h210 = hard_set(&rows, |z| cody(z, &C0, &D0, 0x210));
    let hj = hard_set(&rows, |z| cody(z, &jc, &jd, 0));
    let inter_all: BTreeSet<u64> = h0.intersection(&h74).cloned().collect::<BTreeSet<_>>()
        .intersection(&h210)
        .cloned()
        .collect::<BTreeSet<_>>()
        .intersection(&hj)
        .cloned()
        .collect();
    let only210: BTreeSet<u64> = h210
        .difference(&h0)
        .cloned()
        .collect::<BTreeSet<_>>()
        .difference(&h74)
        .cloned()
        .collect::<BTreeSet<_>>()
        .difference(&hj)
        .cloned()
        .collect();
    println!(
        "DIRECT hard ulp>=2 n: CR0={} 0x74={} 0x210={} j3432={}",
        h0.len(),
        h74.len(),
        h210.len(),
        hj.len()
    );
    println!(
        "0x210 ∩ CR0={} ∩74={} ∩j={} all4={} only210={}",
        h210.intersection(&h0).count(),
        h210.intersection(&h74).count(),
        h210.intersection(&hj).count(),
        inter_all.len(),
        only210.len()
    );
    println!("only_0x210 hard:");
    for &zb in &only210 {
        let z = f64::from_bits(zb);
        let r = rows.iter().find(|r| r.z.to_bits() == zb).unwrap();
        let t = f64::from_bits(r.qbits);
        let g = qwf(z, cody(z, &C0, &D0, 0x210));
        println!(
            "  z={z:.16} ulp={} {} band={}",
            ulp_distance(g, t).unwrap_or(99),
            if g < t { "low" } else { "high" },
            if z < 1.0 {
                "[0.5,1)"
            } else if z < 2.0 {
                "[1,2)"
            } else {
                "[2,4)"
            }
        );
    }

    let hard210: Vec<&f::QRow> = rows
        .iter()
        .filter(|r| h210.contains(&r.z.to_bits()))
        .collect();
    let named: [(&str, Box<dyn Fn(f64) -> f64>); 10] = [
        ("0x210", Box::new(|z| qwf(z, cody(z, &C0, &D0, 0x210)))),
        ("CR0", Box::new(|z| qwf(z, cody(z, &C0, &D0, 0)))),
        ("0x74", Box::new(|z| qwf(z, cody(z, &C0, &D0, 0x74)))),
        ("j3432", Box::new(|z| qwf(z, cody(z, &jc, &jd, 0)))),
        ("codyF", Box::new(|z| qwf(z, f::cody_erfcx_f(z)))),
        ("pqr", Box::new(|z| qwf(z, f::nswc_pqr_f(z)))),
        ("derfc0", Box::new(|z| qwf(z, f::nswc_derfc0(z)))),
        ("ccdd", Box::new(|z| qwf(z, f::nswc_ccdd_f(z)))),
        ("cephes_f", Box::new(|z| qwf(z, f::cephes_f(z)))),
        ("libm", Box::new(|z| libm::erfc(z))),
    ];
    println!("named F on 0x210 hard {} (keep is Q-mid z in [0.5,4)):", hard210.len());
    for (name, ev) in &named {
        let mut keep = 0usize;
        let mut hit = 0usize;
        let mut u1 = 0usize;
        let mut hit_only = 0usize;
        for r in &rows {
            if r.z < 0.5 || r.z >= 4.0 {
                continue;
            }
            if ulp_distance(ev(r.z), f64::from_bits(r.qbits)).unwrap_or(99) == 0 {
                keep += 1;
            }
        }
        for r in &hard210 {
            let d = ulp_distance(ev(r.z), f64::from_bits(r.qbits)).unwrap_or(99);
            if d == 0 {
                hit += 1;
                println!("  HIT {name} z={:.16}", r.z);
                if only210.contains(&r.z.to_bits()) {
                    hit_only += 1;
                }
            } else if d == 1 {
                u1 += 1;
            }
        }
        println!(
            "{name:10} hard_hit={hit}/{} ulp1={u1} only210_hit={hit_only} keep={keep}",
            hard210.len()
        );
    }

    let mut band_n = [0usize; 3];
    let mut band_h = [0usize; 3];
    for r in &rows {
        if !r.direct || r.z < 0.5 || r.z >= 4.0 {
            continue;
        }
        let b = if r.z < 1.0 {
            0
        } else if r.z < 2.0 {
            1
        } else {
            2
        };
        band_n[b] += 1;
        if h210.contains(&r.z.to_bits()) {
            band_h[b] += 1;
        }
    }
    println!(
        "0x210 hard by band [0.5,1) {}/{} [1,2) {}/{} [2,4) {}/{}",
        band_h[0], band_n[0], band_h[1], band_n[1], band_h[2], band_n[2]
    );
}
