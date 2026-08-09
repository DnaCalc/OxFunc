//! Generate blind held-out EFFECT/RRI batches after the W109 banked-grid model race.
//!
//! Selection is oracle-free: rows are chosen only where surviving candidate graphs
//! disagree (plus controls where they collapse). Excel answers are captured later
//! through `Run-W109BulkBatch.ps1 -NoCache`.
use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::power_fn::power_kernel;
use std::collections::BTreeSet;

fn hex(x: f64) -> String {
    format!("0x{:016x}", x.to_bits())
}

fn x87_binexp_lsb(base: f64, mut n: u64) -> f64 {
    let mut acc = 1.0;
    let mut b = base;
    while n > 0 {
        if n & 1 == 1 {
            acc = rx::x87_mul(acc, b);
        }
        n >>= 1;
        if n > 0 {
            b = rx::x87_mul(b, b);
        }
    }
    acc
}

fn binexp_lsb(base: f64, mut n: u64) -> f64 {
    let mut acc = 1.0;
    let mut b = base;
    while n > 0 {
        if n & 1 == 1 {
            acc *= b;
        }
        n >>= 1;
        if n > 0 {
            b *= b;
        }
    }
    acc
}

fn effect_models(nominal: f64, periods: f64) -> [u64; 4] {
    let n = periods as u64;
    let base = 1.0 + nominal / periods;
    [
        (x87_binexp_lsb(base, n) - 1.0).to_bits(),
        (binexp_lsb(base, n) - 1.0).to_bits(),
        (rx::excel_pow_positive(base, periods) - 1.0).to_bits(),
        (base.powf(periods) - 1.0).to_bits(),
    ]
}

fn rri_models(periods: f64, pv: f64, fv: f64) -> [u64; 4] {
    let base = fv / pv;
    let reciprocal = 1.0 / periods;
    [
        (power_kernel(base, reciprocal).unwrap() - 1.0).to_bits(),
        (rx::excel_exp(rx::excel_ln(base) / periods) - 1.0).to_bits(),
        (rx::excel_exp(reciprocal * rx::excel_ln(base)) - 1.0).to_bits(),
        (base.powf(reciprocal) - 1.0).to_bits(),
    ]
}

fn nominal_models(effect_rate: f64, periods: f64) -> [u64; 4] {
    let base = 1.0 + effect_rate;
    let reciprocal = 1.0 / periods;
    let power_inner = power_kernel(base, reciprocal).unwrap() - 1.0;
    let native_inner = base.powf(reciprocal) - 1.0;
    let ln_div_inner = rx::excel_exp(rx::excel_ln(base) / periods) - 1.0;
    [
        (periods * power_inner).to_bits(),
        rx::x87_mul(periods, power_inner).to_bits(),
        (periods * native_inner).to_bits(),
        (periods * ln_div_inner).to_bits(),
    ]
}

fn xorshift(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

fn push_effect(
    rows: &mut Vec<(f64, f64, [u64; 4], &'static str)>,
    seen: &mut BTreeSet<(u64, u64)>,
    nominal: f64,
    periods: f64,
    class: &'static str,
) {
    if nominal <= 0.0 || periods < 1.0 || !nominal.is_finite() {
        return;
    }
    if seen.insert((nominal.to_bits(), periods.to_bits())) {
        rows.push((nominal, periods, effect_models(nominal, periods), class));
    }
}

fn push_rri(
    rows: &mut Vec<(f64, f64, f64, [u64; 4], &'static str)>,
    seen: &mut BTreeSet<(u64, u64, u64)>,
    periods: f64,
    pv: f64,
    fv: f64,
    class: &'static str,
) {
    if periods <= 0.0 || pv <= 0.0 || fv <= 0.0 || !fv.is_finite() {
        return;
    }
    if seen.insert((periods.to_bits(), pv.to_bits(), fv.to_bits())) {
        rows.push((periods, pv, fv, rri_models(periods, pv, fv), class));
    }
}

fn main() {
    let mut effect = Vec::new();
    let mut effect_seen = BTreeSet::new();

    // Structured neighbors around the original exact-rate fingerprint, but none
    // of the original (nominal,npery) pairs are reused.
    for &n in &[
        7_u64, 9, 10, 11, 13, 15, 17, 20, 25, 31, 33, 47, 49, 63, 65, 99, 101, 199, 201, 257, 359,
    ] {
        for &k in &[7_i32, 8, 9, 12, 16, 20] {
            let r0 = 2.0_f64.powi(-k);
            for d in [-3_i64, -1, 0, 1, 3] {
                let rb = (r0.to_bits() as i64 + d) as u64;
                let r = f64::from_bits(rb);
                push_effect(
                    &mut effect,
                    &mut effect_seen,
                    r * n as f64,
                    n as f64,
                    "structured",
                );
            }
        }
    }

    // Deterministic broad search. Keep every graph-disagreement row until the
    // blind discriminator quota is full, then add collapse controls.
    let mut seed = 0x5e17_2026_0809_cafe_u64;
    let mut disagreements = Vec::new();
    let mut controls = Vec::new();
    for _ in 0..2_000_000 {
        let z = xorshift(&mut seed);
        let n = 2 + z % 359;
        let exp = -20.0 + 19.5 * (((z >> 11) & ((1 << 24) - 1)) as f64 / (1_u64 << 24) as f64);
        let r = 2.0_f64.powf(exp) * (1.0 + ((z >> 35) & 0xffff) as f64 / 65536.0);
        let nominal = r * n as f64;
        let models = effect_models(nominal, n as f64);
        if models.iter().any(|x| *x != models[0]) {
            if disagreements.len() < 180 {
                disagreements.push((nominal, n as f64));
            }
        } else if controls.len() < 60 {
            controls.push((nominal, n as f64));
        }
        if disagreements.len() == 180 && controls.len() == 60 {
            break;
        }
    }
    for (nominal, periods) in disagreements {
        push_effect(
            &mut effect,
            &mut effect_seen,
            nominal,
            periods,
            "disagreement",
        );
    }
    for (nominal, periods) in controls {
        push_effect(&mut effect, &mut effect_seen, nominal, periods, "control");
    }

    let mut rri = Vec::new();
    let mut rri_seen = BTreeSet::new();
    let n_values = [
        3.0, 5.0, 6.0, 7.0, 9.0, 10.0, 12.0, 15.0, 24.0, 31.0, 48.0, 63.0, 65.0, 100.0, 127.0,
    ];
    for &n in &n_values {
        for k in 5..=48_i32 {
            let b0 = (1.0 + 2.0_f64.powi(-k)).to_bits();
            for d in [-5_i64, -2, -1, 0, 1, 2, 5] {
                let base = f64::from_bits((b0 as i64 + d) as u64);
                push_rri(&mut rri, &mut rri_seen, n, 1.0, base, "near-one");
            }
        }
    }

    let mut rri_disagreements = Vec::new();
    let mut rri_controls = Vec::new();
    for _ in 0..3_000_000 {
        let z = xorshift(&mut seed);
        let n = (3 + z % 358) as f64;
        let near = ((z >> 9) & 1) == 0;
        let base = if near {
            let k = 4 + ((z >> 10) % 48) as i32;
            let center = (1.0 + 2.0_f64.powi(-k)).to_bits();
            f64::from_bits((center as i64 + (((z >> 23) % 65) as i64 - 32)) as u64)
        } else {
            2.0_f64.powf(-8.0 + 16.0 * (((z >> 11) & 0x00ff_ffff) as f64 / 16_777_216.0))
        };
        let pv = if ((z >> 8) & 3) == 0 {
            f64::from_bits(0x3ff0_0000_0000_0000 + ((z >> 17) & 0x000f_ffff_ffff))
        } else {
            1.0
        };
        let fv = base * pv;
        let models = rri_models(n, pv, fv);
        if models.iter().any(|x| *x != models[0]) {
            if rri_disagreements.len() < 220 {
                rri_disagreements.push((n, pv, fv));
            }
        } else if rri_controls.len() < 60 {
            rri_controls.push((n, pv, fv));
        }
        if rri_disagreements.len() == 220 && rri_controls.len() == 60 {
            break;
        }
    }
    for (n, pv, fv) in rri_disagreements {
        push_rri(&mut rri, &mut rri_seen, n, pv, fv, "disagreement");
    }
    for (n, pv, fv) in rri_controls {
        push_rri(&mut rri, &mut rri_seen, n, pv, fv, "control");
    }

    // Adjacent-family risk scan required by the campaign: NOMINAL has the same
    // fractional root followed by a multiply by npery. Derive an oracle-blind
    // subset from the already-selected RRI inputs and retain both graph
    // disagreements and collapse controls.
    let mut nominal = Vec::new();
    let mut nominal_seen = BTreeSet::new();
    let mut nominal_disagreement = 0usize;
    let mut nominal_control = 0usize;
    for (periods, pv, fv, _, _) in &rri {
        let base = fv / pv;
        let effect_rate = base - 1.0;
        if effect_rate <= 0.0 || !effect_rate.is_finite() {
            continue;
        }
        let key = (effect_rate.to_bits(), periods.to_bits());
        if !nominal_seen.insert(key) {
            continue;
        }
        let models = nominal_models(effect_rate, *periods);
        let differs = models.iter().any(|x| *x != models[0]);
        if differs && nominal_disagreement < 500 {
            nominal.push((*periods, effect_rate, models, "disagreement"));
            nominal_disagreement += 1;
        } else if !differs && nominal_control < 100 {
            nominal.push((*periods, effect_rate, models, "control"));
            nominal_control += 1;
        }
        if nominal_disagreement == 500 && nominal_control == 100 {
            break;
        }
    }

    let mut effect_json = String::from(
        "{\"function\":\"EFFECT\",\"row_id\":\"effect-heldout-20260809\",\"probes\":[",
    );
    let mut effect_meta = String::from(
        "id,class,nominal_bits,npery_bits,x87_dr_binexp,plain_binexp,pow_chain,native_powf\n",
    );
    for (i, (nominal, periods, models, class)) in effect.iter().enumerate() {
        if i > 0 {
            effect_json.push(',');
        }
        let id = format!("eff-ho-{i:04}");
        effect_json.push_str(&format!(
            "{{\"probe\":{{\"id\":\"{id}\",\"args\":[\"{}\",\"{}\"]}}}}",
            hex(*nominal),
            hex(*periods)
        ));
        effect_meta.push_str(&format!(
            "{id},{class},{},{},0x{:016x},0x{:016x},0x{:016x},0x{:016x}\n",
            hex(*nominal),
            hex(*periods),
            models[0],
            models[1],
            models[2],
            models[3]
        ));
    }
    effect_json.push_str("]}");

    let mut rri_json =
        String::from("{\"function\":\"RRI\",\"row_id\":\"rri-heldout-20260809\",\"probes\":[");
    let mut rri_meta = String::from(
        "id,class,nper_bits,pv_bits,fv_bits,power_kernel,ln_div_n,recip_mul_ln,native_powf\n",
    );
    for (i, (periods, pv, fv, models, class)) in rri.iter().enumerate() {
        if i > 0 {
            rri_json.push(',');
        }
        let id = format!("rri-ho-{i:04}");
        rri_json.push_str(&format!(
            "{{\"probe\":{{\"id\":\"{id}\",\"args\":[\"{}\",\"{}\",\"{}\"]}}}}",
            hex(*periods),
            hex(*pv),
            hex(*fv)
        ));
        rri_meta.push_str(&format!(
            "{id},{class},{},{},{},0x{:016x},0x{:016x},0x{:016x},0x{:016x}\n",
            hex(*periods),
            hex(*pv),
            hex(*fv),
            models[0],
            models[1],
            models[2],
            models[3]
        ));
    }
    rri_json.push_str("]}");

    let mut nominal_json = String::from(
        "{\"function\":\"NOMINAL\",\"row_id\":\"nominal-adjacent-scan-20260809\",\"probes\":[",
    );
    let mut nominal_meta = String::from(
        "id,class,nper_bits,effect_bits,power_plain_mul,power_x87_mul,native_plain_mul,ln_div_plain_mul\n",
    );
    for (i, (periods, effect_rate, models, class)) in nominal.iter().enumerate() {
        if i > 0 {
            nominal_json.push(',');
        }
        let id = format!("nom-ho-{i:04}");
        nominal_json.push_str(&format!(
            "{{\"probe\":{{\"id\":\"{id}\",\"args\":[\"{}\",\"{}\"]}}}}",
            hex(*effect_rate),
            hex(*periods)
        ));
        nominal_meta.push_str(&format!(
            "{id},{class},{},{},0x{:016x},0x{:016x},0x{:016x},0x{:016x}\n",
            hex(*periods),
            hex(*effect_rate),
            models[0],
            models[1],
            models[2],
            models[3]
        ));
    }
    nominal_json.push_str("]}");

    let root = "../../work/w109/G6-solvers";
    std::fs::write(
        format!("{root}/batch-effect-heldout-20260809.json"),
        effect_json,
    )
    .unwrap();
    std::fs::write(
        format!("{root}/batch-effect-heldout-20260809-meta.csv"),
        effect_meta,
    )
    .unwrap();
    std::fs::write(format!("{root}/batch-rri-heldout-20260809.json"), rri_json).unwrap();
    std::fs::write(
        format!("{root}/batch-rri-heldout-20260809-meta.csv"),
        rri_meta,
    )
    .unwrap();
    std::fs::write(
        format!("{root}/batch-nominal-adjacent-20260809.json"),
        nominal_json,
    )
    .unwrap();
    std::fs::write(
        format!("{root}/batch-nominal-adjacent-20260809-meta.csv"),
        nominal_meta,
    )
    .unwrap();
    println!("wrote EFFECT held-out: {} rows", effect.len());
    println!("wrote RRI held-out: {} rows", rri.len());
    println!("wrote NOMINAL adjacent scan: {} rows", nominal.len());
}
