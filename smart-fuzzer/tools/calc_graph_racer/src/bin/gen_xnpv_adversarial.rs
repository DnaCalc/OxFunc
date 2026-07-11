//! W109 XNPV pilot — adversarial probe construction for the surviving axes.
//!
//! The 800-probe discovery pool cannot separate the 20 round-1 survivors, so
//! each remaining axis gets probes CONSTRUCTED to hit its double-rounding
//! window while pinning every other axis to an exact (window-free) value:
//!
//! * `years`  — integer day deltas where `RN53(RN64(delta/365)) != RN53(delta/365)`;
//! * `base`   — rates where `RN53(RN64(1+rate)) != RN53(1+rate)`;
//! * `term`   — values where `RN53(RN64(v/pow)) != RN53(v/pow)` for a fixed pow;
//! * `sum`    — all-same-date probes (years = 0, pow = 1, terms = values
//!   exactly) whose running sums hit add windows — pure summation probes;
//! * guard    — rate = 0 / -0 / tiny negative, to pin Excel's #NUM! domain.
//!
//! Output: `pool-adversarial.json` in the row work dir.

use calc_graph_racer::eval::format_bits_hex;
use calc_graph_racer::scheduler::ProbeCase;
use calc_graph_racer::score::WitnessArg;
use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::power_fn::power_kernel;
use std::path::PathBuf;

fn dr_div(a: f64, b: f64) -> f64 {
    let r = rx::ext_div(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN);
    rx::ext_to_f64(&r, rx::CW_PC64_RN)
}
fn dr_add(a: f64, b: f64) -> f64 {
    let r = rx::ext_add(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN);
    rx::ext_to_f64(&r, rx::CW_PC64_RN)
}

struct Rng(u64);
impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut v = self.0;
        v ^= v >> 12;
        v ^= v << 25;
        v ^= v >> 27;
        self.0 = v;
        v.wrapping_mul(0x2545F4914F6CDD1D)
    }
    fn uniform(&mut self, lo: f64, hi: f64) -> f64 {
        let u = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        lo + u * (hi - lo)
    }
}

fn probe(id: String, rate: f64, values: &[f64], dates: &[f64]) -> ProbeCase {
    ProbeCase {
        id,
        args: vec![
            WitnessArg::Scalar(format_bits_hex(rate)),
            WitnessArg::Array(values.iter().map(|v| format_bits_hex(*v)).collect()),
            WitnessArg::Array(dates.iter().map(|v| format_bits_hex(*v)).collect()),
        ],
    }
}

fn main() {
    let out_dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "../../work/w109/G6-11-xnpv".into()),
    );
    let anchor = 40000.0;
    let mut pool: Vec<ProbeCase> = Vec::new();

    // ---- years axis: delta/365 double-rounding windows ----
    let mut years_hits = 0u32;
    for delta in 1u64..2_900_000 {
        let d = delta as f64;
        if dr_div(d, 365.0) != d / 365.0 {
            // value0 = 0 so the sum is exactly the second term; modest rate.
            pool.push(probe(
                format!("adv-years-{delta}"),
                0.05,
                &[0.0, 1.0e6],
                &[anchor, anchor + d],
            ));
            years_hits += 1;
            if years_hits >= 24 {
                break;
            }
        }
    }
    println!("years-axis probes: {years_hits}");

    // ---- base axis: 1+rate double-rounding windows ----
    let mut rng = Rng(0x0109_ADE5_A100_0003);
    let mut base_hits = 0u32;
    for _ in 0..200_000_000u64 {
        if base_hits >= 24 {
            break;
        }
        // Rates whose low bits extend past binary64's 53-bit sum with 1.0:
        // sample magnitudes around 2^-1 .. 2^-40 with full-entropy mantissas.
        let exp_pow = (self_mod(&mut rng, 40) + 1) as i32;
        let rate = f64::from_bits(
            ((1023 - exp_pow as u64) << 52) | (rng.next_u64() & 0x000f_ffff_ffff_ffff),
        );
        if !(rate.is_finite() && rate > 0.0) {
            continue;
        }
        if dr_add(1.0, rate) != 1.0 + rate {
            pool.push(probe(
                format!("adv-base-{base_hits}"),
                rate,
                &[0.0, 1.0e6],
                &[anchor, anchor + 500.0],
            ));
            base_hits += 1;
        }
    }
    println!("base-axis probes: {base_hits}");

    // ---- term axis: v/pow double-rounding windows (fixed pow) ----
    let mut term_hits = 0u32;
    let rate_t = 0.07;
    let delta_t = 313.0; // fractional years, no div window (checked below)
    assert_eq!(dr_div(delta_t, 365.0), delta_t / 365.0);
    let years_t = delta_t / 365.0;
    let pow_t = power_kernel(1.0 + rate_t, years_t).unwrap();
    for _ in 0..200_000_000u64 {
        if term_hits >= 24 {
            break;
        }
        let v = rng.uniform(1.0, 2.0) * 1.0e6;
        if dr_div(v, pow_t) != v / pow_t {
            pool.push(probe(
                format!("adv-term-{term_hits}"),
                rate_t,
                &[0.0, v],
                &[anchor, anchor + delta_t],
            ));
            term_hits += 1;
        }
    }
    println!("term-axis probes: {term_hits}");

    // ---- sum axis: all-same-date probes, running sum hits an add window ----
    // years = 0 for every term -> pow = 1 -> terms are the raw values; the
    // only arithmetic left is the accumulation.
    let mut sum_hits = 0u32;
    for _ in 0..200_000_000u64 {
        if sum_hits >= 24 {
            break;
        }
        let a = rng.uniform(0.5, 2.0) * 1.0e6;
        let b = rng.uniform(0.5, 2.0);
        let c = rng.uniform(0.5, 2.0) * 1.0e-6;
        // forward strict: (a+b)+c ; forward stored-step differs when either
        // partial hits the window.
        let s1_strict = a + b;
        let s1_dr = dr_add(a, b);
        let s2_strict = s1_strict + c;
        let s2_dr = dr_add(s1_dr, c);
        if s2_strict != s2_dr {
            pool.push(probe(
                format!("adv-sum-{sum_hits}"),
                0.05,
                &[a, b, c],
                &[anchor, anchor, anchor],
            ));
            sum_hits += 1;
        }
    }
    println!("sum-axis probes: {sum_hits}");

    // ---- guard probes: Excel's negative/zero-rate domain ----
    for (i, rate) in [0.0, -0.0, -1.0e-9, -1.0e-300, 1.0e-300, -0.5, -1.0, -1.5]
        .iter()
        .enumerate()
    {
        pool.push(probe(
            format!("adv-guard-{i}"),
            *rate,
            &[-1000.0, 1100.0],
            &[anchor, anchor + 200.0],
        ));
    }
    println!("guard probes: 8");

    let path = out_dir.join("pool-adversarial.json");
    std::fs::create_dir_all(&out_dir).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&pool).unwrap()).unwrap();
    println!("wrote {} ({} probes)", path.display(), pool.len());
}

fn self_mod(rng: &mut Rng, m: u64) -> u64 {
    rng.next_u64() % m
}
