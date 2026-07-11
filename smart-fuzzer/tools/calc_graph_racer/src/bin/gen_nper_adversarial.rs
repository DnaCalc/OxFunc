//! W109 NPER — adversarial probes for the two surviving axes:
//! * base: rates where `RN53(RN64(1+rate)) != 1+rate` (separates x87-ln of a
//!   strict base from x87-ln of a double-rounded base);
//! * fdiv: probes where `ln_num / ln_den` hits a division double-rounding
//!   window (separates the strict from the double-rounded final divide).

use calc_graph_racer::eval::format_bits_hex;
use calc_graph_racer::scheduler::ProbeCase;
use calc_graph_racer::score::WitnessArg;
use oxfunc_core::excel_numeric::research as rx;
use std::path::PathBuf;

fn dr_add(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_add(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}
fn dr_mul(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_mul(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
}
fn dr_div(a: f64, b: f64) -> f64 {
    rx::ext_to_f64(
        &rx::ext_div(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN),
        rx::CW_PC64_RN,
    )
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

fn probe(id: String, rate: f64, pmt: f64, pv: f64) -> ProbeCase {
    ProbeCase {
        id,
        args: [rate, pmt, pv, 0.0, 0.0]
            .iter()
            .map(|v| WitnessArg::Scalar(format_bits_hex(*v)))
            .collect(),
    }
}

/// The confirmed spill-loop NPER front end (fv=0, type=0): returns
/// (ln_num, ln_den_strict_base, ln_den_dr_base).
fn lns(rate: f64, pmt: f64, pv: f64) -> (f64, f64, f64) {
    let tf_pmt = dr_mul(1.0, pmt); // tf = 1 exactly for type=0
    let pv_r = dr_mul(pv, rate);
    let num = tf_pmt; // fv=0 -> num = tf*pmt
    let den = dr_add(tf_pmt, pv_r);
    let ratio = dr_div(num, den);
    let ln_num = rx::excel_ln(ratio);
    let ln_den_s = rx::excel_ln(1.0 + rate);
    let ln_den_dr = rx::excel_ln(dr_add(1.0, rate));
    (ln_num, ln_den_s, ln_den_dr)
}

fn main() {
    let out_dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "../../work/w109/G6-08-nper".into()),
    );
    let mut rng = Rng(0x0109_6E08_AD5A);
    let mut pool = Vec::new();

    // base axis: 1+rate double-rounding windows in a realistic rate band.
    let mut base_hits = 0u32;
    for _ in 0..400_000_000u64 {
        if base_hits >= 20 {
            break;
        }
        let exp_pow = 1 + (rng.next_u64() % 40) as u64;
        let rate = f64::from_bits(((1023 - exp_pow) << 52) | (rng.next_u64() & 0x000f_ffff_ffff_ffff));
        if !(rate.is_finite() && rate > 1.0e-13 && rate < 0.6) {
            continue;
        }
        if dr_add(1.0, rate) != 1.0 + rate {
            let pv = 10000.0;
            let pmt = -(pv * (rate + 0.05));
            pool.push(probe(format!("adv-base-{base_hits}"), rate, pmt, pv));
            base_hits += 1;
        }
    }
    println!("base-axis probes: {base_hits}");

    // fdiv axis: ln_num/ln_den division double-rounding windows, restricted
    // to probes where the two base stagings AGREE (so only fdiv separates).
    let mut fdiv_hits = 0u32;
    for _ in 0..400_000_000u64 {
        if fdiv_hits >= 20 {
            break;
        }
        let rate = rng.uniform(1.0e-4, 0.4);
        let pv = rng.uniform(100.0, 1.0e6);
        let pmt = -(pv * (rate + rng.uniform(0.001, 0.3)));
        let (ln_num, ln_den_s, ln_den_dr) = lns(rate, pmt, pv);
        if ln_den_s != ln_den_dr || !ln_num.is_finite() || ln_den_s == 0.0 {
            continue;
        }
        if dr_div(ln_num, ln_den_s) != ln_num / ln_den_s {
            pool.push(probe(format!("adv-fdiv-{fdiv_hits}"), rate, pmt, pv));
            fdiv_hits += 1;
        }
    }
    println!("fdiv-axis probes: {fdiv_hits}");

    let path = out_dir.join("pool-adversarial.json");
    std::fs::write(&path, serde_json::to_string_pretty(&pool).unwrap()).unwrap();
    println!("wrote {} ({} probes)", path.display(), pool.len());

    // Linear-branch staging probes (rate = 0): -(fv+pv)/pmt with the fv+pv
    // add and the final divide each pushed into a double-rounding window.
    let mut linear = Vec::new();
    let mut add_hits = 0u32;
    for _ in 0..400_000_000u64 {
        if add_hits >= 12 {
            break;
        }
        let pv = rng.uniform(0.5, 2.0) * 1.0e6;
        let fv = rng.uniform(0.5, 2.0);
        if dr_add(fv, pv) != fv + pv {
            linear.push(ProbeCase {
                id: format!("lin-add-{add_hits}"),
                args: [0.0, -120.0, pv, fv, 0.0]
                    .iter()
                    .map(|v| WitnessArg::Scalar(format_bits_hex(*v)))
                    .collect(),
            });
            add_hits += 1;
        }
    }
    let mut div_hits = 0u32;
    for _ in 0..400_000_000u64 {
        if div_hits >= 12 {
            break;
        }
        let pv = rng.uniform(100.0, 1.0e6);
        let pmt = -rng.uniform(10.0, 5000.0);
        let s = -pv; // fv = 0 -> numerator = -(0+pv)
        if dr_div(s, pmt) != s / pmt {
            linear.push(ProbeCase {
                id: format!("lin-div-{div_hits}"),
                args: [0.0, pmt, pv, 0.0, 0.0]
                    .iter()
                    .map(|v| WitnessArg::Scalar(format_bits_hex(*v)))
                    .collect(),
            });
            div_hits += 1;
        }
    }
    println!("linear-branch probes: add={add_hits} div={div_hits}");
    let path = out_dir.join("pool-linear.json");
    std::fs::write(&path, serde_json::to_string_pretty(&linear).unwrap()).unwrap();
    println!("wrote {}", path.display());
}
