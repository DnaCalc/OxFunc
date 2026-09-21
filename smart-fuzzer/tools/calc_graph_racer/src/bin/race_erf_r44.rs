//! remaining 44 1-ULP of joint and unpoked A21. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sub, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const CW_RU: u16 = CW_PC64_RN | 0x0800;
const CW_RD: u16 = CW_PC64_RN | 0x0400;
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
fn joint_a() -> [f64; 21] {
    let mut a = AS0;
    a[0] = poke(AS0[0], 4);
    a[1] = poke(AS0[1], -2);
    a[2] = poke(AS0[2], -5);
    a[3] = poke(AS0[3], 1);
    a
}
fn a21(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn a21_tpoke(z: f64, a: &[f64; 21], tk: i32) -> f64 {
    let xe = ef(z);
    let t = ef(poke(ext_to_f64(&ext_mul(&xe, &xe, CW), CW), tk));
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn one_w(z: f64) -> Ext80 {
    let a = joint_a();
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_add(&ef(1.0), &acc, CW)
}
fn spec_ab(z: f64) -> f64 {
    let ye = ef(z);
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
fn erf_sch(z: f64) -> f64 {
    let xe = ef(z.abs());
    let t = ext_sub(&ext_mul(&ef(0.5), &ext_mul(&xe, &xe, CW), CW), &ef(1.0), CW);
    let two = ef(2.0);
    let mut d1 = ef(0.0);
    let mut d2 = ef(0.0);
    for k in (1..SCH.len()).rev() {
        let dk = ext_add(
            &ext_sub(&ext_mul(&ext_mul(&two, &t, CW), &d1, CW), &d2, CW),
            &ef(SCH[k]),
            CW,
        );
        d2 = d1;
        d1 = dk;
    }
    let y = ext_add(
        &ext_sub(&ext_mul(&t, &d1, CW), &d2, CW),
        &ext_mul(&ef(0.5), &ef(SCH[0]), CW),
        CW,
    );
    ext_to_f64(&ext_mul(&xe, &y, CW), CW)
}
fn kind(z: f64) -> &'static str {
    if z < 1e-4 {
        "tiny"
    } else {
        let m = z.to_bits() & ((1u64 << 52) - 1);
        if m.trailing_zeros() >= 20 {
            "dyad"
        } else if (z.to_bits() & 0xfff) == 0x555 {
            "555"
        } else {
            "other"
        }
    }
}
fn hit_ls(z: f64, t: f64) -> bool {
    let ow = one_w(z);
    let owf = ext_to_f64(&ow, CW);
    [-2i32, -1, 1, 2].iter().any(|&k| {
        ulp_distance(ext_to_f64(&ext_mul(&ef(poke(z, k)), &ow, CW), CW), t).unwrap_or(99) == 0
            || ulp_distance(ext_to_f64(&ext_mul(&ef(z), &ef(poke(owf, k)), CW), CW), t)
                .unwrap_or(99)
                == 0
    })
}
fn hit_jtm(z: f64, t: f64) -> bool {
    let oy = one_w(z);
    let ru = ext_to_f64(&ext_mul(&ef(z.next_up()), &oy, CW_RU), CW_RU);
    let ow = ext_to_f64(&oy, CW).next_up();
    let tf = ext_to_f64(&ext_mul(&ef(z.next_up()), &ef(ow), CW), CW);
    let rd = ext_to_f64(&ext_mul(&ef(z.next_down()), &oy, CW_RD), CW_RD);
    let ow2 = ext_to_f64(&oy, CW).next_down();
    let tfd = ext_to_f64(&ext_mul(&ef(z.next_down()), &ef(ow2), CW), CW);
    ulp_distance(ru, t).unwrap_or(99) == 0
        || ulp_distance(tf, t).unwrap_or(99) == 0
        || ulp_distance(rd, t).unwrap_or(99) == 0
        || ulp_distance(tfd, t).unwrap_or(99) == 0
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
    let ja = joint_a();
    let jn = |z: f64| a21(z, &ja);
    let cr = |z: f64| a21(z, &AS0);
    let mut left: Vec<(f64, u64)> = Vec::new();
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        if ulp_distance(jn(z), t).unwrap_or(99) == 0 || hit_ls(z, t) || hit_jtm(z, t) {
            continue;
        }
        if ulp_distance(cr(z), t).unwrap_or(99) == 0 {
            continue;
        }
        left.push((z, bits));
    }
    let mut n_t = 0usize;
    let mut n_d = 0usize;
    let mut n_5 = 0usize;
    let mut n_o = 0usize;
    let mut n_l = 0usize;
    let mut hit_ab = 0usize;
    let mut hit_sch = 0usize;
    let mut hit_lib = 0usize;
    let mut hit_pi = 0usize;
    let mut hit_tp = 0usize;
    let mut hit_tm = 0usize;
    println!("remaining 44 n={}", left.len());
    for &(z, bits) in &left {
        let t = f64::from_bits(bits);
        let k = kind(z);
        match k {
            "tiny" => n_t += 1,
            "dyad" => n_d += 1,
            "555" => n_5 += 1,
            _ => n_o += 1,
        }
        let gj = jn(z);
        let sj = if gj < t {
            "L"
        } else {
            "H"
        };
        if gj < t {
            n_l += 1;
        }
        let ab = ulp_distance(spec_ab(z), t).unwrap_or(99) == 0;
        let sc = ulp_distance(erf_sch(z), t).unwrap_or(99) == 0;
        let lb = ulp_distance(libm::erf(z), t).unwrap_or(99) == 0;
        let c2 = 1.1283791670955125739 * z;
        let pi = ulp_distance(c2, t).unwrap_or(99) == 0;
        let tp = ulp_distance(a21_tpoke(z, &AS0, 1), t).unwrap_or(99) == 0;
        let tm = ulp_distance(a21_tpoke(z, &AS0, -1), t).unwrap_or(99) == 0;
        if ab {
            hit_ab += 1;
        }
        if sc {
            hit_sch += 1;
        }
        if lb {
            hit_lib += 1;
        }
        if pi {
            hit_pi += 1;
        }
        if tp {
            hit_tp += 1;
        }
        if tm {
            hit_tm += 1;
        }
        println!(
            "  z={z:.16} kind={k} j={sj} AB={ab} sch={sc} libm={lb} 2rpi={pi} t+1={tp} t-1={tm}"
        );
    }
    println!(
        "kind tiny={n_t} dyad={n_d} 555={n_5} other={n_o} joint-low={n_l}/{} AB={hit_ab} sch={hit_sch} libm={hit_lib} 2rpi={hit_pi} t+1={hit_tp} t-1={hit_tm}",
        left.len()
    );
    println!("unpoked A[i]±1 on remaining:");
    for i in 0..8 {
        for k in [-1i32, 1] {
            let mut a = AS0;
            a[i] = poke(AS0[i], k);
            let mut h = 0usize;
            let mut keep = 0usize;
            for &(z, bits) in &rows {
                let t = f64::from_bits(bits);
                if ulp_distance(a21(z, &a), t).unwrap_or(99) == 0 {
                    keep += 1;
                }
            }
            for &(z, bits) in &left {
                let t = f64::from_bits(bits);
                if ulp_distance(a21(z, &a), t).unwrap_or(99) == 0 {
                    h += 1;
                }
            }
            println!("  A[{i}]{k:+} remain={h}/{} keep={keep}/{}", left.len(), rows.len());
        }
    }
}
