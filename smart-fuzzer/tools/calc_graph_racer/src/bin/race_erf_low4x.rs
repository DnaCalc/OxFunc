//! 4 P-side LOW leftover: Cody A/B, 0.46875 cut, series, arg nudge. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

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
const RPINV: f64 = 0.56418958354775628695;
const TWOSQPI: f64 = 1.1283791670955125739;

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
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn cody_ab(z: f64, mask: u32) -> f64 {
    let ye = ef(z);
    let ysq = ext_mul(&ye, &ye, CW);
    let mut xnum = ext_mul(&ef(AA[4]), &ysq, CW);
    let mut xden = ysq;
    xnum = maybe(xnum, mask, 0);
    xden = maybe(xden, mask, 1);
    for i in 0..3 {
        xnum = ext_mul(&ext_add(&xnum, &ef(AA[i]), CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &ef(BB[i]), CW), &ysq, CW);
        xnum = maybe(xnum, mask, 2 + 2 * i as u32);
        xden = maybe(xden, mask, 3 + 2 * i as u32);
    }
    let mut q = ext_div(
        &ext_mul(&ye, &ext_add(&xnum, &ef(AA[3]), CW), CW),
        &ext_add(&xden, &ef(BB[3]), CW),
        CW,
    );
    q = maybe(q, mask, 10);
    ext_to_f64(&q, CW)
}
fn cody_ab_f64(z: f64) -> f64 {
    let ysq = z * z;
    let mut xnum = AA[4] * ysq;
    let mut xden = ysq;
    for i in 0..3 {
        xnum = (xnum + AA[i]) * ysq;
        xden = (xden + BB[i]) * ysq;
    }
    z * (xnum + AA[3]) / (xden + BB[3])
}
fn taylor_named(z: f64, n: usize) -> f64 {
    let z2 = z * z;
    let mut term = z;
    let mut s = z;
    for k in 1..n {
        term *= -z2 / (k as f64);
        s += term / (2.0 * k as f64 + 1.0);
    }
    TWOSQPI * s
}
fn kummer_exp(z: f64, n: usize) -> f64 {
    let z2 = z * z;
    let mut term = 1.0;
    let mut m = 1.0;
    for k in 1..n {
        term *= z2 / (k as f64 + 0.5);
        m += term;
        if term.abs() < 1e-20 {
            break;
        }
    }
    TWOSQPI * z * (-z2).exp() * m
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
            let Some(e) = parse_bits_hex(&w.expected_bits) else {
                continue;
            };
            if x > 0.0 && x < 0.5 {
                map.entry(x.to_bits()).or_insert(e.to_bits());
            }
        }
    }
    let rows: Vec<_> = map
        .into_iter()
        .map(|(b, e)| (f64::from_bits(b), e))
        .collect();
    let mut joint = AS0;
    joint[0] = poke(AS0[0], 4);
    joint[1] = poke(AS0[1], -2);
    joint[2] = poke(AS0[2], -5);
    joint[3] = poke(AS0[3], 1);
    let one_w_up = |z: f64| {
        let t = ext_mul(&ef(z), &ef(z), CW);
        let mut acc = ef(0.0);
        for &c in joint.iter().rev() {
            acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
        }
        let s = ext_to_f64(&ext_add(&ef(1.0), &acc, CW), CW).next_up();
        z * s
    };
    let mut lows: Vec<(f64, u64, u64)> = Vec::new();
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let g = erf_a(z, &joint);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d >= 2 && g < t && ulp_distance(one_w_up(z), t).unwrap_or(99) != 0 {
            lows.push((z, bits, d));
        }
    }
    println!("LOW 1+w-miss n={}", lows.len());
    for &(z, bits, dj) in &lows {
        println!(
            "  z={z:.20} zbits={:#x} excel={bits:#x} j_ulp={dj} vs_cut46875={}",
            z.to_bits(),
            if z < 0.46875 { "below" } else { "above" }
        );
    }

    let ab_then_j = |z: f64| {
        if z < 0.46875 {
            cody_ab(z, 0)
        } else {
            erf_a(z, &joint)
        }
    };
    let graphs: [(&str, Box<dyn Fn(f64) -> f64>); 16] = [
        ("joint", Box::new(|z| erf_a(z, &joint))),
        ("A21pub", Box::new(|z| erf_a(z, &AS0))),
        ("AB x87", Box::new(|z| cody_ab(z, 0))),
        ("AB f64", Box::new(|z| cody_ab_f64(z))),
        ("AB 0x1", Box::new(|z| cody_ab(z, 1))),
        ("AB 0x4", Box::new(|z| cody_ab(z, 4))),
        ("AB 0x400", Box::new(|z| cody_ab(z, 0x400))),
        ("cut.46875", Box::new(ab_then_j)),
        ("libm erf", Box::new(|z| libm::erf(z))),
        ("1-libm erfc", Box::new(|z| 1.0 - libm::erfc(z))),
        ("2/sqrtpi*z", Box::new(|z| TWOSQPI * z)),
        ("tay12", Box::new(|z| taylor_named(z, 12))),
        ("tay24", Box::new(|z| taylor_named(z, 24))),
        ("kummer12", Box::new(|z| kummer_exp(z, 12))),
        ("kummer24", Box::new(|z| kummer_exp(z, 24))),
        ("rpinv*2z", Box::new(|z| (RPINV * 2.0) * z)),
    ];
    for &(z, bits, _) in &lows {
        let t = f64::from_bits(bits);
        print!("  z={z:.16}");
        for (name, ev) in &graphs {
            let g = ev(z);
            let d = ulp_distance(g, t).unwrap_or(99);
            let dir = if g < t { "L" } else if g > t { "H" } else { "=" };
            print!(" {name}={d}{dir}");
        }
        println!();
    }
    for (name, ev) in &graphs {
        let mut hit = 0usize;
        let mut u1 = 0usize;
        let mut keep = 0usize;
        for &(z, bits) in &rows {
            if ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99) == 0 {
                keep += 1;
            }
        }
        for &(z, bits, _) in &lows {
            let d = ulp_distance(ev(z), f64::from_bits(bits)).unwrap_or(99);
            if d == 0 {
                hit += 1;
                println!("  HIT {name} z={z:.16}");
            } else if d == 1 {
                u1 += 1;
            }
        }
        println!("{name:12} hit={hit}/{} ulp1={u1} keep={keep}/{}", lows.len(), rows.len());
    }

    println!("arg nudge / last-add 1+w on LOW:");
    for &(z, bits, _) in &lows {
        let t = f64::from_bits(bits);
        let dj_up = ulp_distance(erf_a(z.next_up(), &joint), t).unwrap_or(99);
        let dj_dn = ulp_distance(erf_a(z.next_down(), &joint), t).unwrap_or(99);
        let ab_up = ulp_distance(cody_ab(z.next_up(), 0), t).unwrap_or(99);
        let ab_dn = ulp_distance(cody_ab(z.next_down(), 0), t).unwrap_or(99);
        let lm_up = ulp_distance(libm::erf(z.next_up()), t).unwrap_or(99);
        let lm_dn = ulp_distance(libm::erf(z.next_down()), t).unwrap_or(99);
        let xe = ef(z);
        let tt = ext_mul(&xe, &xe, CW);
        let mut acc = ef(0.0);
        for &c in joint.iter().rev() {
            acc = ext_add(&ext_mul(&acc, &tt, CW), &ef(c), CW);
        }
        let w = acc;
        let lead = ext_add(&ef(1.0), &w, CW);
        let a1 = ext_to_f64(&ext_mul(&xe, &lead, CW), CW);
        let a2 = ext_to_f64(&ext_add(&xe, &ext_mul(&xe, &w, CW), CW), CW);
        let a3 = ext_to_f64(&ext_add(&xe, &ext_mul(&xe, &ef(ext_to_f64(&w, CW)), CW), CW), CW);
        println!(
            "  z={z:.16} j(z+)= {dj_up} j(z-)= {dj_dn} AB(z+)= {ab_up} AB(z-)= {ab_dn} lm(z+)= {lm_up} lm(z-)= {lm_dn} z*(1+w)={} z+z*w={} z+z*st(w)={}",
            ulp_distance(a1, t).unwrap_or(99),
            ulp_distance(a2, t).unwrap_or(99),
            ulp_distance(a3, t).unwrap_or(99)
        );
    }

    println!("A/B HW=1 keep vs LOW hit (bar AB x87):");
    let base_keep = rows
        .iter()
        .filter(|&&(z, bits)| ulp_distance(cody_ab(z, 0), f64::from_bits(bits)).unwrap_or(99) == 0)
        .count();
    println!("  mask0 keep={base_keep}");
    for b in 0..11u32 {
        let m = 1u32 << b;
        let mut keep = 0usize;
        let mut hit = 0usize;
        for &(z, bits) in &rows {
            if ulp_distance(cody_ab(z, m), f64::from_bits(bits)).unwrap_or(99) == 0 {
                keep += 1;
            }
        }
        for &(z, bits, _) in &lows {
            if ulp_distance(cody_ab(z, m), f64::from_bits(bits)).unwrap_or(99) == 0 {
                hit += 1;
            }
        }
        if hit > 0 || keep > base_keep {
            println!("  bit={b} mask={m:#x} hit={hit}/4 keep={keep}");
        }
    }
}
