//! two-mode-down on leftover-high; two-mode-up on k=2 KEEP vs CONV. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
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
fn horner_w(z: f64, a: &[f64; 21]) -> Ext80 {
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    acc
}
fn erf_a(z: f64, a: &[f64; 21]) -> f64 {
    ext_to_f64(
        &ext_mul(&ef(z), &ext_add(&ef(1.0), &horner_w(z, a), CW), CW),
        CW,
    )
}
fn one_w(z: f64, a: &[f64; 21]) -> f64 {
    ext_to_f64(&ext_add(&ef(1.0), &horner_w(z, a), CW), CW)
}
fn k_of(_z: f64, t: f64, ff: f64) -> Option<i32> {
    for ak in 0i32..=8 {
        for &s in &[-1i32, 1] {
            let k = if ak == 0 { 0 } else { s * ak };
            if ak == 0 && s < 0 {
                continue;
            }
            if ulp_distance(poke(ff, k), t).unwrap_or(99) == 0 {
                return Some(k);
            }
            if ak == 0 {
                break;
            }
        }
    }
    None
}
fn is555(z: f64) -> bool {
    let h = format!("{:x}", z.to_bits());
    h.contains("55555555") || h.contains("aaaaaaaa")
}
fn hit(g: f64, t: f64) -> bool {
    ulp_distance(g, t).unwrap_or(99) == 0
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
    let mut joint = AS0;
    joint[0] = poke(AS0[0], 4);
    joint[1] = poke(AS0[1], -2);
    joint[2] = poke(AS0[2], -5);
    joint[3] = poke(AS0[3], 1);

    let mut n_hi = 0usize;
    let mut keep_hi = 0usize;
    let mut worsen_hi = 0usize;
    let mut tmd_u_keep = 0usize;
    let mut tmd_u_worsen = 0usize;
    let mut tmd_j_keep = 0usize;
    let mut tmd_j_worsen = 0usize;
    let mut tmd_u_all = 0usize;
    let mut tmd_j_all = 0usize;
    let mut zdn_u_all = 0usize;
    let mut sdn_u_all = 0usize;
    println!("two-mode-down on leftover-high of joint:");
    for (&zb, &eb) in &map {
        let z = f64::from_bits(zb);
        let t = f64::from_bits(eb);
        let gj = erf_a(z, &joint);
        let dj = ulp_distance(gj, t).unwrap_or(99);
        if !(dj >= 2 && gj > t) {
            continue;
        }
        n_hi += 1;
        let ff = erf_a(z, &AS0);
        let d0 = ulp_distance(ff, t).unwrap_or(99);
        let su = one_w(z, &AS0);
        let sj = one_w(z, &joint);
        let tmd_u = hit(z.next_down() * su.next_down(), t);
        let tmd_j = hit(z.next_down() * sj.next_down(), t);
        let zdn_u = hit(z.next_down() * su, t);
        let sdn_u = hit(z * su.next_down(), t);
        if tmd_u {
            tmd_u_all += 1;
        }
        if tmd_j {
            tmd_j_all += 1;
        }
        if zdn_u {
            zdn_u_all += 1;
        }
        if sdn_u {
            sdn_u_all += 1;
        }
        let is_keep = d0 >= 2 && ff > t;
        if is_keep {
            keep_hi += 1;
            if tmd_u {
                tmd_u_keep += 1;
            }
            if tmd_j {
                tmd_j_keep += 1;
            }
        } else if d0 == 1 && ff > t {
            worsen_hi += 1;
            if tmd_u {
                tmd_u_worsen += 1;
            }
            if tmd_j {
                tmd_j_worsen += 1;
            }
        }
        if tmd_u || tmd_j {
            println!(
                "  TMD z={:.16} bits={:#x} 555={} keep={} u_d={d0} j_d={dj} tmd_u={tmd_u} tmd_j={tmd_j} z*dn(1+w)_u={} dn(z)*(1+w)_u={zdn_u}",
                z,
                z.to_bits(),
                is555(z),
                is_keep,
                sdn_u
            );
        }
    }
    println!(
        "leftover-high n={n_hi} KEEP={keep_hi} WORSEN={worsen_hi} tmd_u={tmd_u_all} tmd_j={tmd_j_all} z*dn(1+w)_u={sdn_u_all} dn(z)*(1+w)_u={zdn_u_all}"
    );
    println!(
        "KEEP tmd_u={tmd_u_keep}/{keep_hi} tmd_j={tmd_j_keep}/{keep_hi} WORSEN tmd_u={tmd_u_worsen}/{worsen_hi} tmd_j={tmd_j_worsen}/{worsen_hi}"
    );

    let mut nkeep = 0usize;
    let mut nconv = 0usize;
    let mut nk3 = 0usize;
    let mut keep_tmu_u = 0usize;
    let mut conv_tmu_u = 0usize;
    let mut k3_tmu_u = 0usize;
    let mut keep_tmu_j = 0usize;
    let mut conv_tmu_j = 0usize;
    let mut k3_tmu_j = 0usize;
    let mut keep_zup = 0usize;
    let mut conv_zup = 0usize;
    let mut keep_sup = 0usize;
    let mut conv_sup = 0usize;
    let mut keep_tmu_only = 0usize;
    let mut conv_tmu_only = 0usize;
    let mut conv_tmu_and_sup = 0usize;
    let mut conv_tmu_and_zup = 0usize;
    let mut keep_tz = [0usize; 53];
    let mut conv_tmu_tz = [0usize; 53];
    let mut conv_not_tz = [0usize; 53];
    println!("two-mode-up on unpoked leftover-low k=2/k=3:");
    for (&zb, &eb) in &map {
        let z = f64::from_bits(zb);
        let t = f64::from_bits(eb);
        let ff = erf_a(z, &AS0);
        let d0 = ulp_distance(ff, t).unwrap_or(99);
        if !(d0 >= 2 && ff < t) {
            continue;
        }
        let k = k_of(z, t, ff);
        if k != Some(2) && k != Some(3) {
            continue;
        }
        let gj = erf_a(z, &joint);
        let dj = ulp_distance(gj, t).unwrap_or(99);
        let su = one_w(z, &AS0);
        let sj = one_w(z, &joint);
        let tmu_u = hit(z.next_up() * su.next_up(), t);
        let tmu_j = hit(z.next_up() * sj.next_up(), t);
        let zup_u = hit(z.next_up() * su, t);
        let sup_u = hit(z * su.next_up(), t);
        let keep = dj >= 2 && gj < t;
        let conv = dj == 1;
        let tz = ((z.to_bits() & ((1u64 << 52) - 1)).trailing_zeros() as usize).min(52);
        if k == Some(2) && keep {
            nkeep += 1;
            keep_tz[tz] += 1;
            if tmu_u {
                keep_tmu_u += 1;
            }
            if tmu_j {
                keep_tmu_j += 1;
            }
            if zup_u {
                keep_zup += 1;
            }
            if sup_u {
                keep_sup += 1;
            }
            if tmu_u && !sup_u && !zup_u {
                keep_tmu_only += 1;
            }
            println!(
                "  KEEP k=2 z={:.16} 555={} tmu_u={tmu_u} tmu_j={tmu_j} z*up(1+w)={sup_u} up(z)*(1+w)={zup_u}",
                z,
                is555(z)
            );
        } else if k == Some(2) && conv {
            nconv += 1;
            if tmu_u {
                conv_tmu_tz[tz] += 1;
            } else {
                conv_not_tz[tz] += 1;
            }
            if tmu_u {
                conv_tmu_u += 1;
            }
            if tmu_j {
                conv_tmu_j += 1;
            }
            if zup_u {
                conv_zup += 1;
            }
            if sup_u {
                conv_sup += 1;
            }
            if tmu_u && !sup_u && !zup_u {
                conv_tmu_only += 1;
            }
            if tmu_u && sup_u {
                conv_tmu_and_sup += 1;
            }
            if tmu_u && zup_u {
                conv_tmu_and_zup += 1;
            }
        } else if k == Some(3) && keep {
            nk3 += 1;
            if tmu_u {
                k3_tmu_u += 1;
            }
            if tmu_j {
                k3_tmu_j += 1;
            }
            println!(
                "  KEEP k=3 z={:.16} 555={} tmu_u={tmu_u} tmu_j={tmu_j} z*up(1+w)={} up(z)*(1+w)={}",
                z,
                is555(z),
                sup_u,
                zup_u
            );
        }
    }
    println!(
        "KEEP k=2 n={nkeep} tmu_u={keep_tmu_u} tmu_j={keep_tmu_j} z*up(1+w)_u={keep_sup} up(z)*(1+w)_u={keep_zup} tmu_only={keep_tmu_only}"
    );
    println!(
        "CONV k=2 n={nconv} tmu_u={conv_tmu_u} tmu_j={conv_tmu_j} z*up(1+w)_u={conv_sup} up(z)*(1+w)_u={conv_zup} tmu_only={conv_tmu_only} tmu&z*up={conv_tmu_and_sup} tmu&up(z)={conv_tmu_and_zup}"
    );
    println!("KEEP k=3 n={nk3} tmu_u={k3_tmu_u} tmu_j={k3_tmu_j}");
    let prtz = |name: &str, h: &[usize; 53]| {
        print!("{name} tz");
        for (i, c) in h.iter().enumerate() {
            if *c > 0 {
                print!(" {i}:{c}");
            }
        }
        println!();
    };
    prtz("KEEP k=2", &keep_tz);
    prtz("CONV k=2 tmu", &conv_tmu_tz);
    prtz("CONV k=2 not-tmu", &conv_not_tz);

    let names = ["fused", "1-ULP", "leftover-low", "leftover-high"];
    let idx = |s: &str| match s {
        "fused" => 0,
        "1-ULP" => 1,
        "leftover-low" => 2,
        _ => 3,
    };
    let buck = |g: f64, t: f64| {
        let d = ulp_distance(g, t).unwrap_or(99);
        if d == 0 {
            "fused"
        } else if d == 1 {
            "1-ULP"
        } else if g < t {
            "leftover-low"
        } else {
            "leftover-high"
        }
    };
    let mut nall = 0usize;
    let mut tmd_by = [0usize; 4];
    let mut tmu_by = [0usize; 4];
    let mut tmd_j_by = [0usize; 4];
    let mut tmu_j_by = [0usize; 4];
    let mut nbuck = [0usize; 4];
    println!("global unpoked two-mode by unpoked bucket:");
    for (&zb, &eb) in &map {
        let z = f64::from_bits(zb);
        let t = f64::from_bits(eb);
        nall += 1;
        let ff = erf_a(z, &AS0);
        let gj = erf_a(z, &joint);
        let bu = buck(ff, t);
        let i = idx(bu);
        nbuck[i] += 1;
        let su = one_w(z, &AS0);
        let sj = one_w(z, &joint);
        if hit(z.next_down() * su.next_down(), t) {
            tmd_by[i] += 1;
            println!(
                "  TMD-all z={:.16} bits={:#x} 555={} tz={} unpoked={bu} joint={} u_d={} high={} k={:?} last-mul_dn={} z*dn(1+w)={} dn(z)*(1+w)={}",
                z,
                z.to_bits(),
                is555(z),
                ((z.to_bits() & ((1u64 << 52) - 1)).trailing_zeros() as usize).min(52),
                buck(gj, t),
                ulp_distance(ff, t).unwrap_or(99),
                ff > t,
                k_of(z, t, ff),
                hit(ff.next_down(), t),
                hit(z * su.next_down(), t),
                hit(z.next_down() * su, t)
            );
        }
        if hit(z.next_up() * su.next_up(), t) {
            tmu_by[i] += 1;
        }
        if hit(z.next_down() * sj.next_down(), t) {
            tmd_j_by[i] += 1;
        }
        if hit(z.next_up() * sj.next_up(), t) {
            tmu_j_by[i] += 1;
        }
    }
    print!("n={nall} unpoked bucket");
    for (i, c) in nbuck.iter().enumerate() {
        print!(" {}:{}", names[i], c);
    }
    print!("\nunpoked tmd");
    for (i, c) in tmd_by.iter().enumerate() {
        print!(" {}:{}", names[i], c);
    }
    print!("\nunpoked tmu");
    for (i, c) in tmu_by.iter().enumerate() {
        print!(" {}:{}", names[i], c);
    }
    print!("\njoint tmd");
    for (i, c) in tmd_j_by.iter().enumerate() {
        print!(" {}:{}", names[i], c);
    }
    print!("\njoint tmu");
    for (i, c) in tmu_j_by.iter().enumerate() {
        print!(" {}:{}", names[i], c);
    }
    println!();
}
