//! Extra-digit 80-bit named P-side vs A21-joint hard leftovers (ulp≥2). Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
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
const T80: [Ext80; 5] = [
    Ext80([0xfb, 0x40, 0xec, 0xba, 0xf1, 0xf8, 0xad, 0x99, 0x02, 0x40]),
    Ext80([0xc0, 0xf7, 0x33, 0xf0, 0x74, 0x52, 0x0d, 0xb4, 0x05, 0x40]),
    Ext80([0xff, 0xb8, 0xad, 0xa1, 0xe5, 0x15, 0x80, 0x8b, 0x0a, 0x40]),
    Ext80([0x15, 0x80, 0x7c, 0x97, 0xe3, 0x99, 0xda, 0xda, 0x0b, 0x40]),
    Ext80([0x27, 0xe3, 0x9e, 0x10, 0x22, 0x4d, 0x28, 0xd9, 0x0e, 0x40]),
];
const U80: [Ext80; 5] = [
    Ext80([0x57, 0xcc, 0x35, 0x7d, 0xff, 0x31, 0x3f, 0x86, 0x04, 0x40]),
    Ext80([0xff, 0x6e, 0xd7, 0x31, 0xa6, 0xe8, 0x56, 0x82, 0x08, 0x40]),
    Ext80([0xae, 0x8e, 0xe8, 0x07, 0x34, 0x97, 0x92, 0x8f, 0x0b, 0x40]),
    Ext80([0x14, 0x4c, 0xe0, 0x0b, 0x08, 0x00, 0xca, 0xb0, 0x0d, 0x40]),
    Ext80([0x88, 0x38, 0xab, 0x47, 0xee, 0x64, 0x73, 0xc0, 0x0e, 0x40]),
];
const AA80: [Ext80; 5] = [
    Ext80([0xe6, 0x49, 0x1f, 0xa2, 0xf6, 0xd9, 0x4f, 0xca, 0x00, 0x40]),
    Ext80([0xbc, 0x16, 0xed, 0xb2, 0x69, 0x72, 0xba, 0xe3, 0x05, 0x40]),
    Ext80([0x7e, 0x57, 0x94, 0xba, 0x44, 0x1c, 0xbe, 0xbc, 0x07, 0x40]),
    Ext80([0xac, 0xd1, 0x90, 0xe8, 0x9a, 0x0a, 0x96, 0xc8, 0x0a, 0x40]),
    Ext80([0x00, 0x10, 0x19, 0x8e, 0xd1, 0x82, 0x3c, 0xbe, 0xfc, 0x3f]),
];
const BB80: [Ext80; 4] = [
    Ext80([0xc7, 0xec, 0x96, 0x7d, 0xa1, 0x71, 0xcf, 0xbc, 0x03, 0x40]),
    Ext80([0x44, 0xc0, 0xad, 0xf2, 0xab, 0x4e, 0x06, 0xf4, 0x06, 0x40]),
    Ext80([0x54, 0x3e, 0x6e, 0xe5, 0x94, 0xba, 0x53, 0xa0, 0x09, 0x40]),
    Ext80([0x6d, 0x64, 0x3d, 0xdc, 0x11, 0xca, 0xc3, 0xb1, 0x0a, 0x40]),
];
const PP80: [Ext80; 5] = [
    Ext80([0x00, 0x40, 0xdb, 0xa6, 0x10, 0xd4, 0x75, 0x83, 0xfc, 0x3f]),
    Ext80([0x00, 0x98, 0xc8, 0xe5, 0x48, 0xeb, 0x6b, 0xa6, 0xfd, 0xbf]),
    Ext80([0x00, 0x78, 0xca, 0xb8, 0xde, 0x8e, 0x52, 0xe9, 0xf9, 0xbf]),
    Ext80([0x00, 0x20, 0x47, 0x33, 0x1b, 0x89, 0x14, 0xbd, 0xf7, 0xbf]),
    Ext80([0x00, 0x60, 0xb5, 0x00, 0x90, 0xb0, 0x56, 0xc7, 0xef, 0xbf]),
];
const QQ80: [Ext80; 5] = [
    Ext80([0x00, 0x48, 0xe0, 0xd6, 0x6e, 0xce, 0xbb, 0xcb, 0xfd, 0x3f]),
    Ext80([0x00, 0xd0, 0x75, 0xb6, 0xa9, 0x62, 0x2a, 0x85, 0xfb, 0x3f]),
    Ext80([0x00, 0x78, 0x58, 0x9b, 0x26, 0x16, 0x81, 0xa6, 0xf7, 0x3f]),
    Ext80([0x00, 0x80, 0xd0, 0xe0, 0x10, 0x49, 0xee, 0x8a, 0xf2, 0x3f]),
    Ext80([0x00, 0x00, 0x09, 0x13, 0x15, 0x1a, 0xe2, 0x84, 0xed, 0xbf]),
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
fn horner_80(cs: &[Ext80], x: Ext80) -> Ext80 {
    let mut a = ef(0.0);
    for c in cs.iter().rev() {
        a = ext_add(&ext_mul(&a, &x, CW), c, CW);
    }
    a
}
fn polevl_80(cs: &[Ext80], x: Ext80) -> Ext80 {
    let mut a = cs[0];
    for c in &cs[1..] {
        a = ext_add(&ext_mul(&a, &x, CW), c, CW);
    }
    a
}
fn p1evl_80(cs: &[Ext80], x: Ext80) -> Ext80 {
    let mut a = ext_add(&x, &cs[0], CW);
    for c in &cs[1..] {
        a = ext_add(&ext_mul(&a, &x, CW), c, CW);
    }
    a
}
fn erf_joint(z: f64) -> f64 {
    let mut a = AS0;
    a[0] = poke(AS0[0], 4);
    a[1] = poke(AS0[1], -2);
    a[2] = poke(AS0[2], -5);
    a[3] = poke(AS0[3], 1);
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn cephes_80(z: f64) -> f64 {
    let ze = ef(z.abs());
    let zz = ext_mul(&ze, &ze, CW);
    ext_to_f64(
        &ext_div(
            &ext_mul(&ze, &polevl_80(&T80, zz), CW),
            &p1evl_80(&U80, zz),
            CW,
        ),
        CW,
    )
}
fn cody_80(z: f64) -> f64 {
    let ye = ef(z.abs());
    let ysq = ext_mul(&ye, &ye, CW);
    let mut xnum = ext_mul(&AA80[4], &ysq, CW);
    let mut xden = ysq;
    for i in 0..3 {
        xnum = ext_mul(&ext_add(&xnum, &AA80[i], CW), &ysq, CW);
        xden = ext_mul(&ext_add(&xden, &BB80[i], CW), &ysq, CW);
    }
    ext_to_f64(
        &ext_div(
            &ext_mul(&ye, &ext_add(&xnum, &AA80[3], CW), CW),
            &ext_add(&xden, &BB80[3], CW),
            CW,
        ),
        CW,
    )
}
fn fdlibm_80(z: f64) -> f64 {
    let ze = ef(z.abs());
    let zz = ext_mul(&ze, &ze, CW);
    let r = horner_80(&PP80, zz);
    let q = horner_80(&QQ80, zz);
    let s = ext_add(&ef(1.0), &ext_mul(&zz, &q, CW), CW);
    let y = ext_div(&r, &s, CW);
    ext_to_f64(&ext_add(&ze, &ext_mul(&ze, &y, CW), CW), CW)
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
    let graphs: [(&str, fn(f64) -> f64); 4] = [
        ("Cephes T/U 80-bit", cephes_80),
        ("Cody A/B 80-bit", cody_80),
        ("fdlibm PP/QQ 80-bit", fdlibm_80),
        ("libm::erf", libm::erf),
    ];
    let mut hard = Vec::new();
    for &(z, pbits) in &rows {
        let want = f64::from_bits(pbits);
        let d = ulp_distance(erf_joint(z), want).unwrap_or(99);
        if d >= 2 && d <= ULP_CAP {
            hard.push((z, pbits, d));
        }
    }
    println!("joint hard leftover n={}", hard.len());
    for (name, ev) in graphs {
        let mut hit = 0usize;
        let mut keep = 0usize;
        let mut hmax = 0u64;
        for &(z, pbits) in &rows {
            let want = f64::from_bits(pbits);
            let dj = ulp_distance(erf_joint(z), want).unwrap_or(99);
            let d = ulp_distance(ev(z), want).unwrap_or(u64::MAX);
            if dj == 0 && d == 0 {
                keep += 1;
            }
        }
        for &(z, pbits, _) in &hard {
            let d = ulp_distance(ev(z), f64::from_bits(pbits)).unwrap_or(u64::MAX);
            if d == 0 {
                hit += 1;
            } else if d <= ULP_CAP {
                hmax = hmax.max(d);
            }
        }
        println!("{name:22} keep_joint={keep}/866 hit_hard={hit}/{} hardmax={hmax}", hard.len());
    }
    println!("libm exacts on joint-hard:");
    for &(z, pbits, dj) in &hard {
        if ulp_distance(libm::erf(z), f64::from_bits(pbits)).unwrap_or(99) == 0 {
            println!("  HIT z={z:.16} joint_ulp={dj}");
        }
    }
}
