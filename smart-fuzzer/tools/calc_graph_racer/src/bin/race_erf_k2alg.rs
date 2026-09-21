//! P-side leftover-low k=2 two-mode vs last-store of unpoked F. Not an identity.
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
    let mut n_k2 = 0usize;
    let mut n_tmu = 0usize;
    let mut n_eq = 0usize;
    let mut tmu_d_h = [0usize; 8];
    let mut miss_d_h = [0usize; 8];
    let mut nkeep_tmu = 0usize;
    let mut nconv_tmu = 0usize;
    let mut nkeep_miss = 0usize;
    let mut nconv_miss = 0usize;
    let mut n_eq_tmu = 0usize;
    let mut n_eq_miss = 0usize;
    let mut miss_eq_k1 = 0usize;
    let mut miss_eq_k3 = 0usize;
    let mut miss_eq_k4 = 0usize;
    let mut miss_eq_up = 0usize;
    let mut miss_eq_zup = 0usize;
    let mut miss_eq_sup = 0usize;
    let mut tz_f2 = [0usize; 53];
    let mut tz_f3 = [0usize; 53];
    let mut tz_f4 = [0usize; 53];
    let mut f2_555 = 0usize;
    let mut f3_555 = 0usize;
    let mut f4_555 = 0usize;
    println!("P leftover-low k=2 two-mode vs last-store of unpoked F:");
    for (&zb, &eb) in &map {
        let z = f64::from_bits(zb);
        let t = f64::from_bits(eb);
        let ff = erf_a(z, &AS0);
        if k_of(z, t, ff) != Some(2) {
            continue;
        }
        n_k2 += 1;
        let tz = ((z.to_bits() & ((1u64 << 52) - 1)).trailing_zeros() as usize).min(52);
        let s555 = is555(z);
        let s = one_w(z, &AS0);
        let q_k2 = poke(ff, 2);
        let q_tmu = z.next_up() * s.next_up();
        let eq = q_tmu.to_bits() == q_k2.to_bits();
        if eq {
            n_eq += 1;
        }
        let dt = ulp_distance(q_tmu, t).unwrap_or(99) as usize;
        let tmu = dt == 0;
        let gj = erf_a(z, &joint);
        let dj = ulp_distance(gj, t).unwrap_or(99);
        let keep = dj >= 2 && gj < t;
        let conv = dj == 1;
        if tmu {
            n_tmu += 1;
            if dt < 8 {
                tmu_d_h[dt] += 1;
            }
            if eq {
                n_eq_tmu += 1;
            }
            if keep {
                nkeep_tmu += 1;
            }
            if conv {
                nconv_tmu += 1;
            }
            tz_f2[tz] += 1;
            if s555 {
                f2_555 += 1;
            }
        } else {
            if dt < 8 {
                miss_d_h[dt] += 1;
            }
            if eq {
                n_eq_miss += 1;
            }
            if q_tmu.to_bits() == poke(ff, 1).to_bits() {
                miss_eq_k1 += 1;
            }
            if q_tmu.to_bits() == poke(ff, 3).to_bits() {
                miss_eq_k3 += 1;
                tz_f3[tz] += 1;
                if s555 {
                    f3_555 += 1;
                }
            }
            if q_tmu.to_bits() == poke(ff, 4).to_bits() {
                miss_eq_k4 += 1;
                tz_f4[tz] += 1;
                if s555 {
                    f4_555 += 1;
                }
            }
            if q_tmu.to_bits() == ff.next_up().to_bits() {
                miss_eq_up += 1;
            }
            if q_tmu.to_bits() == (z.next_up() * s).to_bits() {
                miss_eq_zup += 1;
            }
            if q_tmu.to_bits() == (z * s.next_up()).to_bits() {
                miss_eq_sup += 1;
            }
            if keep {
                nkeep_miss += 1;
            }
            if conv {
                nconv_miss += 1;
            }
            if keep || dt >= 2 {
                println!(
                    "  not-tmu z={:.16} 555={} keep={keep} conv={conv} tmu_d={dt} k2_d={} eq={eq} eq_k1={} eq_k3={} eq_up={}",
                    z,
                    is555(z),
                    ulp_distance(q_k2, t).unwrap_or(99),
                    q_tmu.to_bits() == poke(ff, 1).to_bits(),
                    q_tmu.to_bits() == poke(ff, 3).to_bits(),
                    q_tmu.to_bits() == ff.next_up().to_bits()
                );
            }
        }
    }
    println!("leftover-low k=2 n={n_k2} tmu={n_tmu} Q_tmu==Q_k2 {n_eq}/{n_k2} tmu_eq={n_eq_tmu}/{n_tmu} miss_eq={n_eq_miss}");
    print!("tmu d");
    for (i, c) in tmu_d_h.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    print!("\nnot-tmu d");
    for (i, c) in miss_d_h.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!();
    println!("KEEP tmu={nkeep_tmu} miss={nkeep_miss} CONV tmu={nconv_tmu} miss={nconv_miss}");
    println!(
        "not-tmu Q_tmu==F+1 {miss_eq_k1} ==F+3 {miss_eq_k3} ==F+4 {miss_eq_k4} ==last-mul_up {miss_eq_up} ==z+*(1+w) {miss_eq_zup} ==z*(1+w)+ {miss_eq_sup} of n={}",
        n_k2 - n_tmu
    );
    let prtz = |name: &str, h: &[usize; 53], n555: usize| {
        print!("{name} 555={n555} tz");
        for (i, c) in h.iter().enumerate() {
            if *c > 0 {
                print!(" {i}:{c}");
            }
        }
        println!();
    };
    prtz("F+2 tmu", &tz_f2, f2_555);
    prtz("F+3", &tz_f3, f3_555);
    prtz("F+4", &tz_f4, f4_555);

    let mut n_k3 = 0usize;
    let mut k3_tmu = 0usize;
    let mut k3_eq = [0usize; 6];
    println!("leftover-low k=3 two-mode vs last-store of F:");
    for (&zb, &eb) in &map {
        let z = f64::from_bits(zb);
        let t = f64::from_bits(eb);
        let ff = erf_a(z, &AS0);
        if k_of(z, t, ff) != Some(3) {
            continue;
        }
        n_k3 += 1;
        let s = one_w(z, &AS0);
        let q_tmu = z.next_up() * s.next_up();
        let dt = ulp_distance(q_tmu, t).unwrap_or(99);
        let gj = erf_a(z, &joint);
        let keep = ulp_distance(gj, t).unwrap_or(99) >= 2 && gj < t;
        if dt == 0 {
            k3_tmu += 1;
        }
        for kk in 1i32..=5 {
            if q_tmu.to_bits() == poke(ff, kk).to_bits() {
                k3_eq[kk as usize] += 1;
            }
        }
        println!(
            "  k=3 z={:.16} 555={} keep={keep} tmu_d={dt} eq_k={}",
            z,
            is555(z),
            (1i32..=5)
                .find(|&kk| q_tmu.to_bits() == poke(ff, kk).to_bits())
                .map(|kk| format!("{kk}"))
                .unwrap_or_else(|| "none".into())
        );
    }
    print!("leftover-low k=3 n={n_k3} tmu={k3_tmu} Q_tmu==F+k");
    for (i, c) in k3_eq.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!();

    let mut n_km1 = 0usize;
    let mut km1_tmd = 0usize;
    let mut km1_eq = [0usize; 6];
    let mut n_worsen_tmd = 0usize;
    let mut worsen_eq = [0usize; 6];
    let mut stay_eq = [0usize; 6];
    println!("unpoked 1-ULP HIGH k=-1 two-mode-down vs last-store of F:");
    for (&zb, &eb) in &map {
        let z = f64::from_bits(zb);
        let t = f64::from_bits(eb);
        let ff = erf_a(z, &AS0);
        if k_of(z, t, ff) != Some(-1) || !(ff > t) {
            continue;
        }
        n_km1 += 1;
        let s = one_w(z, &AS0);
        let q_tmd = z.next_down() * s.next_down();
        let dt = ulp_distance(q_tmd, t).unwrap_or(99);
        let gj = erf_a(z, &joint);
        let dj = ulp_distance(gj, t).unwrap_or(99);
        let worsen = dj >= 2 && gj > t;
        if dt == 0 {
            km1_tmd += 1;
            if worsen {
                n_worsen_tmd += 1;
            }
            println!(
                "  F-1 tmd z={:.16} bits={:#x} 555={} tz={} worsen={worsen} j_d={dj}",
                z,
                z.to_bits(),
                is555(z),
                ((z.to_bits() & ((1u64 << 52) - 1)).trailing_zeros() as usize).min(52)
            );
        }
        for ak in 1i32..=5 {
            if q_tmd.to_bits() == poke(ff, -ak).to_bits() {
                km1_eq[ak as usize] += 1;
                if worsen {
                    worsen_eq[ak as usize] += 1;
                } else {
                    stay_eq[ak as usize] += 1;
                }
            }
        }
    }
    print!("1-ULP HIGH k=-1 n={n_km1} tmd={km1_tmd} worsen_tmd={n_worsen_tmd} Q_tmd==F-k");
    for (i, c) in km1_eq.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!();
    print!("WORSEN two-mode F-k");
    for (i, c) in worsen_eq.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    print!("\nstay 1-ULP two-mode F-k");
    for (i, c) in stay_eq.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!();

    let mut n_k1 = 0usize;
    let mut k1_tmu = 0usize;
    let mut k1_eq = [0usize; 6];
    let mut k1_conv = 0usize;
    println!("unpoked 1-ULP LOW k=+1 two-mode-up vs last-store of F:");
    for (&zb, &eb) in &map {
        let z = f64::from_bits(zb);
        let t = f64::from_bits(eb);
        let ff = erf_a(z, &AS0);
        if k_of(z, t, ff) != Some(1) || !(ff < t) {
            continue;
        }
        n_k1 += 1;
        let s = one_w(z, &AS0);
        let q_tmu = z.next_up() * s.next_up();
        let dt = ulp_distance(q_tmu, t).unwrap_or(99);
        let gj = erf_a(z, &joint);
        let dj = ulp_distance(gj, t).unwrap_or(99);
        if dt == 0 {
            k1_tmu += 1;
        }
        if dj == 0 {
            k1_conv += 1;
        }
        for ak in 1i32..=5 {
            if q_tmu.to_bits() == poke(ff, ak).to_bits() {
                k1_eq[ak as usize] += 1;
            }
        }
    }
    print!("1-ULP LOW k=+1 n={n_k1} tmu={k1_tmu} joint_fused={k1_conv} Q_tmu==F+k");
    for (i, c) in k1_eq.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!();

    let mut nf = 0usize;
    let mut f_tmu = 0usize;
    let mut f_tmd = 0usize;
    let mut f_up = [0usize; 6];
    let mut f_dn = [0usize; 6];
    let mut f_up_none = 0usize;
    let mut f_dn_none = 0usize;
    println!("unpoked fused two-mode vs last-store of F:");
    for (&zb, &eb) in &map {
        let z = f64::from_bits(zb);
        let t = f64::from_bits(eb);
        let ff = erf_a(z, &AS0);
        if ulp_distance(ff, t).unwrap_or(99) != 0 {
            continue;
        }
        nf += 1;
        let s = one_w(z, &AS0);
        let q_tmu = z.next_up() * s.next_up();
        let q_tmd = z.next_down() * s.next_down();
        if ulp_distance(q_tmu, t).unwrap_or(99) == 0 {
            f_tmu += 1;
        }
        if ulp_distance(q_tmd, t).unwrap_or(99) == 0 {
            f_tmd += 1;
        }
        let mut up_hit = false;
        let mut dn_hit = false;
        for ak in 0i32..=5 {
            if q_tmu.to_bits() == poke(ff, ak).to_bits() {
                f_up[ak as usize] += 1;
                up_hit = true;
            }
            if q_tmd.to_bits() == poke(ff, -ak).to_bits() {
                f_dn[ak as usize] += 1;
                dn_hit = true;
            }
        }
        if !up_hit {
            f_up_none += 1;
        }
        if !dn_hit {
            f_dn_none += 1;
        }
    }
    print!("fused n={nf} tmu_excel={f_tmu} tmd_excel={f_tmd} two-mode-up==F+k");
    for (i, c) in f_up.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if f_up_none > 0 {
        print!(" none:{f_up_none}");
    }
    print!(" two-mode-down==F-k");
    for (i, c) in f_dn.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    if f_dn_none > 0 {
        print!(" none:{f_dn_none}");
    }
    println!();
}
