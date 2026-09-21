//! leftover-high KEEP vs WORSEN of joint; k=2 KEEP vs CONV last-store of 1+w.
//! Not an identity.
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
fn tz_of(z: f64) -> usize {
    ((z.to_bits() & ((1u64 << 52) - 1)).trailing_zeros() as usize).min(52)
}
fn signed(g: f64, t: f64) -> String {
    let d = ulp_distance(g, t).unwrap_or(99);
    let s = if g < t {
        "L"
    } else if g > t {
        "H"
    } else {
        "="
    };
    format!("{d}{s}")
}
fn prtz(name: &str, h: &[usize; 53]) {
    print!("{name} tz");
    for (i, c) in h.iter().enumerate() {
        if *c > 0 {
            print!(" {i}:{c}");
        }
    }
    println!();
}
fn prk(name: &str, h: &BTreeMap<i32, usize>, none: usize) {
    print!("{name} k");
    for (k, v) in h {
        print!(" {k}:{v}");
    }
    if none > 0 {
        print!(" none:{none}");
    }
    println!();
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

    let mut nkeep_hi = 0usize;
    let mut nworsen = 0usize;
    let mut keep_hi_555 = 0usize;
    let mut worsen_555 = 0usize;
    let mut keep_hi_tz = [0usize; 53];
    let mut worsen_tz = [0usize; 53];
    let mut keep_hi_k: BTreeMap<i32, usize> = BTreeMap::new();
    let mut worsen_k: BTreeMap<i32, usize> = BTreeMap::new();

    println!("leftover-high KEEP vs WORSEN of joint:");
    for (&zb, &eb) in &map {
        let z = f64::from_bits(zb);
        let t = f64::from_bits(eb);
        let gj = erf_a(z, &joint);
        let dj = ulp_distance(gj, t).unwrap_or(99);
        if !(dj >= 2 && gj > t) {
            continue;
        }
        let ff = erf_a(z, &AS0);
        let d0 = ulp_distance(ff, t).unwrap_or(99);
        let k = k_of(z, t, ff);
        let s555 = is555(z);
        let tz = tz_of(z);
        if d0 >= 2 && ff > t {
            nkeep_hi += 1;
            keep_hi_tz[tz] += 1;
            if s555 {
                keep_hi_555 += 1;
            }
            if let Some(kk) = k {
                *keep_hi_k.entry(kk).or_insert(0) += 1;
            }
            println!(
                "  KEEP-HI z={:.16} bits={:#x} 555={s555} tz={tz} j_d={dj} u_d={d0} k={k:?} last-mul_dn={} 3z={:.16} 3z_bits={:#x}",
                z,
                z.to_bits(),
                ulp_distance(ff.next_down(), t).unwrap_or(99) == 0,
                3.0 * z,
                (3.0 * z).to_bits()
            );
            let su = one_w(z, &AS0);
            let sj = one_w(z, &joint);
            let variants: [(&str, f64); 12] = [
                ("unpoked F", ff),
                ("joint F", gj),
                ("unpoked last-mul dn F", ff.next_down()),
                ("joint last-mul dn F", gj.next_down()),
                ("unpoked z*(1+w) RN", z * su),
                ("joint z*(1+w) RN", z * sj),
                ("unpoked z*dn(1+w)", z * su.next_down()),
                ("joint z*dn(1+w)", z * sj.next_down()),
                ("unpoked dn(z)*dn(1+w)", z.next_down() * su.next_down()),
                ("joint dn(z)*dn(1+w)", z.next_down() * sj.next_down()),
                ("unpoked z*dn2(1+w)", z * su.next_down().next_down()),
                ("joint z*dn2(1+w)", z * sj.next_down().next_down()),
            ];
            for (name, g) in variants {
                println!("    {name:24} {}", signed(g, t));
            }
            print!("    unpoked 1+w last-store k vs z*(poke s):");
            for kk in -4i32..=0 {
                let g = z * poke(su, kk);
                print!(" k={kk}:{}", signed(g, t));
            }
            println!();
            print!("    joint 1+w last-store k vs z*(poke s):");
            for kk in -4i32..=0 {
                let g = z * poke(sj, kk);
                print!(" k={kk}:{}", signed(g, t));
            }
            println!();
            print!("    unpoked F last-store k:");
            for kk in -4i32..=0 {
                print!(" k={kk}:{}", signed(poke(ff, kk), t));
            }
            println!();
        } else if d0 == 1 && ff > t {
            nworsen += 1;
            worsen_tz[tz] += 1;
            if s555 {
                worsen_555 += 1;
            }
            if let Some(kk) = k {
                *worsen_k.entry(kk).or_insert(0) += 1;
            }
        }
    }
    println!("KEEP-HI n={nkeep_hi} 555={keep_hi_555}");
    prtz("KEEP-HI", &keep_hi_tz);
    prk("KEEP-HI", &keep_hi_k, 0);
    println!("WORSEN-HI n={nworsen} 555={worsen_555}");
    prtz("WORSEN-HI", &worsen_tz);
    prk("WORSEN-HI", &worsen_k, 0);

    let mut keep_w_hit = 0usize;
    let mut conv_w_hit = 0usize;
    let mut keep_w_k: BTreeMap<i32, usize> = BTreeMap::new();
    let mut conv_w_k: BTreeMap<i32, usize> = BTreeMap::new();
    let mut keep_w_none = 0usize;
    let mut conv_w_none = 0usize;
    let mut keep_jw_hit = 0usize;
    let mut conv_jw_hit = 0usize;
    let mut nkeep = 0usize;
    let mut nconv = 0usize;
    println!("k=2 leftover-low KEEP vs CONV last-store of 1+w of unpoked:");
    for (&zb, &eb) in &map {
        let z = f64::from_bits(zb);
        let t = f64::from_bits(eb);
        let ff = erf_a(z, &AS0);
        let d0 = ulp_distance(ff, t).unwrap_or(99);
        if !(d0 >= 2 && ff < t) {
            continue;
        }
        if k_of(z, t, ff) != Some(2) {
            continue;
        }
        let gj = erf_a(z, &joint);
        let dj = ulp_distance(gj, t).unwrap_or(99);
        let su = one_w(z, &AS0);
        let sj = one_w(z, &joint);
        let kw = k_of(z, t, z * su);
        let jw_hit = ulp_distance(z * sj.next_up(), t).unwrap_or(99) == 0
            || ulp_distance(z * sj, t).unwrap_or(99) == 0;
        let uw_hit = ulp_distance(z * su.next_up(), t).unwrap_or(99) == 0
            || ulp_distance(z * su, t).unwrap_or(99) == 0;
        if dj >= 2 && gj < t {
            nkeep += 1;
            if uw_hit {
                keep_w_hit += 1;
            }
            if jw_hit {
                keep_jw_hit += 1;
            }
            match kw {
                Some(kk) => *keep_w_k.entry(kk).or_insert(0) += 1,
                None => keep_w_none += 1,
            }
            println!(
                "  KEEP k=2 z={:.16} bits={:#x} 555={} tz={} u_1+w_k={kw:?} u_z*up(1+w)={} j_z*up(1+w)={} j_d={dj}",
                z,
                z.to_bits(),
                is555(z),
                tz_of(z),
                ulp_distance(z * su.next_up(), t).unwrap_or(99) == 0,
                ulp_distance(z * sj.next_up(), t).unwrap_or(99) == 0
            );
        } else if dj == 1 {
            nconv += 1;
            if uw_hit {
                conv_w_hit += 1;
            }
            if jw_hit {
                conv_jw_hit += 1;
            }
            match kw {
                Some(kk) => *conv_w_k.entry(kk).or_insert(0) += 1,
                None => conv_w_none += 1,
            }
        }
    }
    println!(
        "KEEP k=2 n={nkeep} unpoked z*(1+w or up) hit={keep_w_hit} joint z*(1+w or up) hit={keep_jw_hit}"
    );
    prk("KEEP k=2 unpoked z*(1+w) last-store", &keep_w_k, keep_w_none);
    println!(
        "CONV k=2 n={nconv} unpoked z*(1+w or up) hit={conv_w_hit} joint z*(1+w or up) hit={conv_jw_hit}"
    );
    prk("CONV k=2 unpoked z*(1+w) last-store", &conv_w_k, conv_w_none);
}
