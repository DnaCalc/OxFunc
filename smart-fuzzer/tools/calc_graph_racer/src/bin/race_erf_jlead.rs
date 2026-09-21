//! A21-joint leftover vs x87 2/sqrt(pi) lead signed ULP. P-side. Not an identity.
use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{ulp_distance, WitnessArg, WitnessSet};
use oxfunc_core::excel_numeric::research as rx;
use rx::{ext_add, ext_div, ext_from_f64, ext_mul, ext_sqrt, ext_to_f64, Ext80, CW_PC64_RN};
use std::collections::BTreeMap;
use std::env;
use std::fs;

const CW: u16 = CW_PC64_RN;
const ULP_CAP: u64 = 1 << 20;
const PI: f64 = 3.1415926535897932384626433832795;
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
fn signed_ulp(got: f64, want: f64) -> i64 {
    let d = ulp_distance(got, want).unwrap_or(99) as i64;
    if d == 0 {
        0
    } else if got > want {
        d
    } else {
        -d
    }
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
fn erf_lead(z: f64) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in AS0[1..].iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    let tail = ext_mul(&u, &acc, CW);
    let lead = ext_div(&ef(2.0), &ext_sqrt(&ef(PI), CW), CW);
    ext_to_f64(&ext_mul(&xe, &ext_add(&lead, &tail, CW), CW), CW)
}
fn erf_a(z: f64, a: &[f64; 21]) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in a.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
}
fn erf_cr(z: f64) -> f64 {
    let xe = ef(z.abs());
    let u = ext_mul(&xe, &xe, CW);
    let mut acc = ef(0.0);
    for &c in AS0.iter().rev() {
        acc = ext_add(&ext_mul(&acc, &u, CW), &ef(c), CW);
    }
    ext_to_f64(&ext_mul(&xe, &ext_add(&ef(1.0), &acc, CW), CW), CW)
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
    let mut hist_j = BTreeMap::<i64, usize>::new();
    let mut hist_l = BTreeMap::<i64, usize>::new();
    let mut pair = BTreeMap::<(i64, i64), usize>::new();
    let mut band_n = [0usize; 5];
    let mut band_left = [0usize; 5];
    let mut band_lhit = [0usize; 5];
    let mut left = 0usize;
    let mut l_exact = 0usize;
    let mut cr_exact = 0usize;
    let mut same_sign = 0usize;
    let mut opp_sign = 0usize;
    let mut lead_closer = 0usize;
    let mut joint_closer = 0usize;
    let mut tie_abs = 0usize;
    let mut hard = 0usize;
    let mut shown = 0usize;
    println!("P-side n={}", rows.len());
    println!("hard leftover (joint |ulp|>=2) z sj sl scr:");
    for &(z, pbits) in &rows {
        let want = f64::from_bits(pbits);
        let gj = erf_joint(z);
        let gl = erf_lead(z);
        let gc = erf_cr(z);
        let dj = ulp_distance(gj, want).unwrap_or(u64::MAX);
        if dj > ULP_CAP {
            continue;
        }
        let b = ((z / 0.1) as usize).min(4);
        band_n[b] += 1;
        if dj == 0 {
            continue;
        }
        left += 1;
        band_left[b] += 1;
        let sj = signed_ulp(gj, want);
        let sl = signed_ulp(gl, want);
        let sc = signed_ulp(gc, want);
        *hist_j.entry(sj).or_insert(0) += 1;
        *hist_l.entry(sl).or_insert(0) += 1;
        *pair.entry((sj, sl)).or_insert(0) += 1;
        if sl == 0 {
            l_exact += 1;
            band_lhit[b] += 1;
        }
        if sc == 0 {
            cr_exact += 1;
        }
        let aj = sj.abs();
        let al = sl.abs();
        if al < aj {
            lead_closer += 1;
        } else if aj < al {
            joint_closer += 1;
        } else {
            tie_abs += 1;
        }
        if sj != 0 && sl != 0 {
            if sj.signum() == sl.signum() {
                same_sign += 1;
            } else {
                opp_sign += 1;
            }
        }
        if dj >= 2 {
            hard += 1;
            if shown < 12 {
                println!("  z={z:.16} {sj:+} {sl:+} {sc:+}");
                shown += 1;
            }
        }
    }
    println!(
        "joint leftover={left}/{} lead_exact_on_left={l_exact} CR_exact_on_left={cr_exact} hard(|ulp|>=2)={hard}",
        rows.len()
    );
    println!(
        "closer: lead={lead_closer} joint={joint_closer} tie={tie_abs}  same_sign={same_sign} opp_sign={opp_sign}"
    );
    for i in 0..5 {
        println!(
            "[{:.1},{:.1}) n={} left={} lead_hits={}",
            i as f64 * 0.1,
            i as f64 * 0.1 + 0.1,
            band_n[i],
            band_left[i],
            band_lhit[i]
        );
    }
    println!("signed ULP joint leftover:");
    for (k, v) in &hist_j {
        println!("  {k:+} {v}");
    }
    println!("signed ULP lead on joint leftover:");
    for (k, v) in &hist_l {
        println!("  {k:+} {v}");
    }
    println!("top (sj,sl) pairs:");
    let mut pv: Vec<_> = pair.into_iter().collect();
    pv.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    for ((sj, sl), n) in pv.into_iter().take(12) {
        println!("  joint{sj:+} lead{sl:+} {n}");
    }
    let cuts = [
        1e-8, 1e-6, 1e-4, 1e-3, 0.01, 0.05, 0.0625, 0.1, 0.125, 0.2, 0.25, 0.375, 0.46875,
    ];
    let mut at = AS0;
    at[1] = poke(AS0[1], -2);
    at[2] = poke(AS0[2], -5);
    at[3] = poke(AS0[3], 1);
    let mut a0only = AS0;
    a0only[0] = poke(AS0[0], 4);
    println!("A0-unpoke / A0-only vs joint/CR:");
    let named: [(&str, Box<dyn Fn(f64) -> f64>); 5] = [
        ("CR", Box::new(erf_cr)),
        ("joint A0+4 A1-2 A2-5 A3+1", Box::new(erf_joint)),
        (
            "A0 pub + joint A1-3",
            Box::new({
                let a = at;
                move |z| erf_a(z, &a)
            }),
        ),
        (
            "A0+4 only",
            Box::new({
                let a = a0only;
                move |z| erf_a(z, &a)
            }),
        ),
        ("x87 lead + CR tail", Box::new(erf_lead)),
    ];
    for (name, ev) in named {
        let mut ex = 0usize;
        let mut n = 0usize;
        let mut maxu = 0u64;
        let mut tiny_ex = 0usize;
        let mut tiny_n = 0usize;
        for &(z, pbits) in &rows {
            let pg = ev(z);
            let d = ulp_distance(pg, f64::from_bits(pbits)).unwrap_or(u64::MAX);
            if d > ULP_CAP {
                continue;
            }
            n += 1;
            if z < 1e-6 {
                tiny_n += 1;
                if d == 0 {
                    tiny_ex += 1;
                }
            }
            if d == 0 {
                ex += 1;
            } else {
                maxu = maxu.max(d);
            }
        }
        println!("{name:32} {ex}/{n} max={maxu} tiny {tiny_ex}/{tiny_n}");
    }
    println!("cut CR-then-joint  joint-then-CR  lead-then-joint");
    for &c in &cuts {
        let mut cj = 0usize;
        let mut jc = 0usize;
        let mut lj = 0usize;
        for &(z, pbits) in &rows {
            let want = f64::from_bits(pbits);
            let g_cj = if z < c { erf_cr(z) } else { erf_joint(z) };
            let g_jc = if z < c { erf_joint(z) } else { erf_cr(z) };
            let g_lj = if z < c { erf_lead(z) } else { erf_joint(z) };
            if ulp_distance(g_cj, want).unwrap_or(99) == 0 {
                cj += 1;
            }
            if ulp_distance(g_jc, want).unwrap_or(99) == 0 {
                jc += 1;
            }
            if ulp_distance(g_lj, want).unwrap_or(99) == 0 {
                lj += 1;
            }
        }
        println!("cut={c:.8} CR-then-j={cj} j-then-CR={jc} lead-then-j={lj}");
    }
}
