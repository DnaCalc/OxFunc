//! Singleton P-side joint ulp=3; 0x555 3z; DIRECT 0x5005 ulp hist. Not an identity.
use calc_graph_racer::erfc_f_packets as f;
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
const CEPHES_P: [f64; 9] = [
    2.46196981473530512524e-10,
    5.64189564831068821977e-1,
    7.46321056442269912687e0,
    4.86371970985681366614e1,
    1.96520832956077098242e2,
    5.26445194995477358631e2,
    9.34528527171957607540e2,
    1.02755188689515710272e3,
    5.57535335369399327526e2,
];
const CEPHES_Q: [f64; 8] = [
    1.32281951154744992508e1,
    8.67072140885989742329e1,
    3.54937778887819891062e2,
    9.75708501743205489753e2,
    1.82390916687909736289e3,
    2.24633760818710981792e3,
    1.65666309194161350182e3,
    5.57535340817727675546e2,
];
const CEPHES_R: [f64; 6] = [
    5.64189583547755073984e-1,
    1.27536670759978104416e0,
    5.01905042251180477414e0,
    6.16021097993053585195e0,
    7.40974269950448939160e0,
    2.97886665372100240670e0,
];
const CEPHES_S: [f64; 6] = [
    2.26052863220117276590e0,
    9.39603524938001434673e0,
    1.20489539808096656605e1,
    1.70814450747565897222e1,
    9.60896809063285878198e0,
    3.36907645100081516050e0,
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
fn erf_a(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z);
    let t = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &t, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn cody_ab(z: f64) -> f64 {
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
fn maybe(x: Ext80, mask: u32, bit: u32) -> Ext80 {
    if bit < 32 && mask & (1u32 << bit) != 0 {
        ef(ext_to_f64(&x, CW))
    } else {
        x
    }
}
fn polevl_mask(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ef(coef[0]);
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn p1evl_mask(x: Ext80, coef: &[f64], mask: u32, mut bit: u32) -> (Ext80, u32) {
    let mut ans = ext_add(&x, &ef(coef[0]), CW);
    ans = maybe(ans, mask, bit);
    bit += 1;
    for &c in &coef[1..] {
        ans = ext_add(&ext_mul(&ans, &x, CW), &ef(c), CW);
        ans = maybe(ans, mask, bit);
        bit += 1;
    }
    (ans, bit)
}
fn cephes_mask(x: f64, mask: u32) -> f64 {
    let ax = x.abs();
    let xe = ef(ax);
    let (num, den, bit0) = if ax < 8.0 {
        let (p, b) = polevl_mask(xe, &CEPHES_P, mask, 0);
        let (q, b) = p1evl_mask(xe, &CEPHES_Q, mask, b);
        (p, q, b)
    } else {
        let (r, b) = polevl_mask(xe, &CEPHES_R, mask, 0);
        let (s, b) = p1evl_mask(xe, &CEPHES_S, mask, b);
        (r, s, b)
    };
    let mut q = ext_div(&num, &den, CW);
    q = maybe(q, mask, bit0);
    ext_to_f64(&q, CW)
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

    println!("joint ulp>=3 rows:");
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let g = erf_a(z, &joint);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d >= 3 {
            println!(
                "  z={z:.20} zbits={:#x} excel={bits:#x} j_ulp={d} j{}excel 3z={:.16} 3z_exact={}",
                z.to_bits(),
                if g < t { "<" } else { ">" },
                3.0 * z,
                (3.0 * z).fract() == 0.0 || (3.0 * z).to_bits() & 0xfffff == 0
            );
            for (name, ev) in [
                ("A21pub", Box::new(|z| erf_a(z, &AS0)) as Box<dyn Fn(f64) -> f64>),
                ("AB", Box::new(cody_ab)),
                ("libm erf", Box::new(|z| libm::erf(z))),
                ("1-libm erfc", Box::new(|z| 1.0 - libm::erfc(z))),
                ("j(z+)", Box::new(|z| erf_a(z.next_up(), &joint))),
                ("j(z++)", Box::new(|z| erf_a(z.next_up().next_up(), &joint))),
                ("up2(j)", Box::new(|z| erf_a(z, &joint).next_up().next_up())),
                ("up3(j)", Box::new(|z| erf_a(z, &joint).next_up().next_up().next_up())),
            ] {
                let gg = ev(z);
                println!(
                    "    {name} ulp={} {}",
                    ulp_distance(gg, t).unwrap_or(99),
                    if gg < t { "L" } else if gg > t { "H" } else { "=" }
                );
            }
        }
    }

    println!("0x555 +2 LOW vs 3z dyadic / erf(z) vs erf(round(3z)/3):");
    for &(z, bits) in &rows {
        let t = f64::from_bits(bits);
        let g = erf_a(z, &joint);
        let d = ulp_distance(g, t).unwrap_or(99);
        if d != 2 || g >= t {
            continue;
        }
        let m = z.to_bits() & 0xfffff_fffff;
        let is555 = (z.to_bits() & 0xfff) == 0x555;
        if !is555 {
            continue;
        }
        let z3 = 3.0 * z;
        let z3r = z3.round();
        let zb = z3r / 3.0;
        println!(
            "  z={z:.16} 3z={z3:.16} round3z={z3r} z'=round/3={zb:.16} j(z')={} libm(z')={} bits={m:#x}",
            ulp_distance(erf_a(zb, &joint), t).unwrap_or(99),
            ulp_distance(libm::erf(zb), t).unwrap_or(99)
        );
    }

    let qrows = f::load_q_rows_tagged(&dir);
    let mut h_all = [0usize; 6];
    let mut h_dir = [0usize; 6];
    let mut n_all = 0usize;
    let mut n_dir = 0usize;
    let mut d2_dir_low = 0usize;
    let mut d2_dir_high = 0usize;
    println!("0x5005 DIRECT ulp>=2 z>=20:");
    for r in &qrows {
        if r.z < 4.0 {
            continue;
        }
        let t = f64::from_bits(r.qbits);
        let g = qw(r.z, cephes_mask(r.z, 0x5005));
        let d = ulp_distance(g, t).unwrap_or(99).min(5) as usize;
        h_all[d] += 1;
        n_all += 1;
        if r.direct {
            h_dir[d] += 1;
            n_dir += 1;
            if d == 2 {
                if g < t {
                    d2_dir_low += 1;
                } else {
                    d2_dir_high += 1;
                }
            }
            if r.z >= 20.0 && d >= 2 {
                println!(
                    "  z={:.16} ulp={} {} excel={:#x}",
                    r.z,
                    ulp_distance(g, t).unwrap_or(99),
                    if g < t { "low" } else { "high" },
                    r.qbits
                );
            }
        }
    }
    println!(
        "0x5005 ALL  ulp0..5+ {} {} {} {} {} {} n={n_all}",
        h_all[0], h_all[1], h_all[2], h_all[3], h_all[4], h_all[5]
    );
    println!(
        "0x5005 DIR  ulp0..5+ {} {} {} {} {} {} n={n_dir} +2low={d2_dir_low} +2high={d2_dir_high}",
        h_dir[0], h_dir[1], h_dir[2], h_dir[3], h_dir[4], h_dir[5]
    );
}
