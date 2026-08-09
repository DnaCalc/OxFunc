//! Blind follow-up after the RRI n=2 POWER-wrapper discriminator and NOMINAL
//! adjacent-family discovery. Covers the raw-pow-vs-POWER split, fractional RRI
//! periods, NOMINAL truncation, ratio staging, and near-one cancellation.
use oxfunc_core::excel_numeric::research as rx;
use oxfunc_core::functions::power_fn::power_kernel;
use std::collections::BTreeSet;

fn hex(x: f64) -> String {
    format!("0x{:016x}", x.to_bits())
}

fn x87_sub(a: f64, b: f64) -> f64 {
    let value = rx::ext_sub(&rx::ext_from_f64(a), &rx::ext_from_f64(b), rx::CW_PC64_RN);
    rx::ext_to_f64(&value, rx::CW_PC53_RN)
}

fn rri_models(periods: f64, pv: f64, fv: f64) -> [u64; 5] {
    let base = fv / pv;
    let reciprocal = 1.0 / periods;
    let raw = rx::excel_pow_chain(base, reciprocal);
    [
        (raw - 1.0).to_bits(),
        x87_sub(raw, 1.0).to_bits(),
        (power_kernel(base, reciprocal).unwrap() - 1.0).to_bits(),
        (base.powf(reciprocal) - 1.0).to_bits(),
        (rx::excel_exp(rx::excel_ln(base) / periods) - 1.0).to_bits(),
    ]
}

fn nominal_models(effect_rate: f64, periods_arg: f64) -> [u64; 6] {
    let periods = periods_arg.trunc();
    let base = 1.0 + effect_rate;
    let reciprocal = 1.0 / periods;
    let raw_inner = rx::excel_pow_chain(base, reciprocal) - 1.0;
    let wrapper_inner = power_kernel(base, reciprocal).unwrap() - 1.0;
    let native_inner = base.powf(reciprocal) - 1.0;
    let ln_div_inner = rx::excel_exp(rx::excel_ln(base) / periods) - 1.0;
    [
        (periods * raw_inner).to_bits(),
        rx::x87_mul(periods, raw_inner).to_bits(),
        (periods * wrapper_inner).to_bits(),
        (periods * native_inner).to_bits(),
        (periods * ln_div_inner).to_bits(),
        (periods_arg * raw_inner).to_bits(),
    ]
}

fn main() {
    let period_values: [f64; 14] = [
        0.5, 0.75, 1.25, 1.5, 1.75, 2.0, 2.5, 3.25, 4.5, 7.5, 10.1, 16.5, 31.75, 64.25,
    ];
    let mut bases = vec![0.125, 0.5, 0.75, 1.25, 1.5, 2.0, 3.0, 10.0, 64.0];
    for k in 5..=48_i32 {
        let center = (1.0 + 2.0_f64.powi(-k)).to_bits();
        for d in [-7_i64, -2, -1, 0, 1, 2, 7] {
            bases.push(f64::from_bits((center as i64 + d) as u64));
        }
    }

    let pv_values = [
        1.0,
        f64::from_bits(0x3ff0_0000_0001_2345),
        f64::from_bits(0x4023_4567_89ab_cdef),
    ];
    let mut rri_rows = Vec::new();
    let mut rri_seen = BTreeSet::new();
    let mut disagreement = 0usize;
    let mut controls = 0usize;
    'rri: for &periods in &period_values {
        for &base in &bases {
            for &pv in &pv_values {
                let fv = base * pv;
                let key = (periods.to_bits(), pv.to_bits(), fv.to_bits());
                if !rri_seen.insert(key) {
                    continue;
                }
                let models = rri_models(periods, pv, fv);
                let differs = models.iter().any(|x| *x != models[0]);
                if differs && disagreement < 600 {
                    rri_rows.push((periods, pv, fv, models, "disagreement"));
                    disagreement += 1;
                } else if !differs && controls < 120 {
                    rri_rows.push((periods, pv, fv, models, "control"));
                    controls += 1;
                }
                if disagreement == 600 && controls == 120 {
                    break 'rri;
                }
            }
        }
    }

    let nominal_period_args: [f64; 12] = [
        1.0,
        1.999_999_999_999_999_8,
        2.0,
        2.25,
        2.999_999_999_999_999_6,
        3.0,
        3.75,
        4.0,
        7.9,
        8.0,
        16.5,
        64.25,
    ];
    let mut nominal_rows = Vec::new();
    let mut nominal_seen = BTreeSet::new();
    let mut nominal_disagreement = 0usize;
    let mut nominal_controls = 0usize;
    'nominal: for &periods_arg in &nominal_period_args {
        if periods_arg.trunc() < 1.0 {
            continue;
        }
        for &base in &bases {
            if base <= 1.0 {
                continue;
            }
            let effect_rate = base - 1.0;
            let key = (effect_rate.to_bits(), periods_arg.to_bits());
            if !nominal_seen.insert(key) {
                continue;
            }
            let models = nominal_models(effect_rate, periods_arg);
            let differs = models.iter().any(|x| *x != models[0]);
            if differs && nominal_disagreement < 500 {
                nominal_rows.push((effect_rate, periods_arg, models, "disagreement"));
                nominal_disagreement += 1;
            } else if !differs && nominal_controls < 100 {
                nominal_rows.push((effect_rate, periods_arg, models, "control"));
                nominal_controls += 1;
            }
            if nominal_disagreement == 500 && nominal_controls == 100 {
                break 'nominal;
            }
        }
    }

    let mut rri_json =
        String::from("{\"function\":\"RRI\",\"row_id\":\"rri-followup-20260809\",\"probes\":[");
    let mut rri_meta = String::from(
        "id,class,nper_bits,pv_bits,fv_bits,raw_plain_sub,raw_x87_sub,power_wrapper,native_powf,ln_div_n\n",
    );
    for (i, (periods, pv, fv, models, class)) in rri_rows.iter().enumerate() {
        if i > 0 {
            rri_json.push(',');
        }
        let id = format!("rri-fu-{i:04}");
        rri_json.push_str(&format!(
            "{{\"probe\":{{\"id\":\"{id}\",\"args\":[\"{}\",\"{}\",\"{}\"]}}}}",
            hex(*periods),
            hex(*pv),
            hex(*fv)
        ));
        rri_meta.push_str(&format!(
            "{id},{class},{},{},{},0x{:016x},0x{:016x},0x{:016x},0x{:016x},0x{:016x}\n",
            hex(*periods),
            hex(*pv),
            hex(*fv),
            models[0],
            models[1],
            models[2],
            models[3],
            models[4]
        ));
    }
    rri_json.push_str("]}");

    let mut nominal_json = String::from(
        "{\"function\":\"NOMINAL\",\"row_id\":\"nominal-followup-20260809\",\"probes\":[",
    );
    let mut nominal_meta = String::from(
        "id,class,effect_bits,nper_arg_bits,raw_plain_mul,raw_x87_mul,power_wrapper,native_powf,ln_div_n,arg_not_truncated\n",
    );
    for (i, (effect_rate, periods_arg, models, class)) in nominal_rows.iter().enumerate() {
        if i > 0 {
            nominal_json.push(',');
        }
        let id = format!("nom-fu-{i:04}");
        nominal_json.push_str(&format!(
            "{{\"probe\":{{\"id\":\"{id}\",\"args\":[\"{}\",\"{}\"]}}}}",
            hex(*effect_rate),
            hex(*periods_arg)
        ));
        nominal_meta.push_str(&format!(
            "{id},{class},{},{},0x{:016x},0x{:016x},0x{:016x},0x{:016x},0x{:016x},0x{:016x}\n",
            hex(*effect_rate),
            hex(*periods_arg),
            models[0],
            models[1],
            models[2],
            models[3],
            models[4],
            models[5]
        ));
    }
    nominal_json.push_str("]}");

    let root = "../../work/w109/G6-solvers";
    std::fs::write(format!("{root}/batch-rri-followup-20260809.json"), rri_json).unwrap();
    std::fs::write(
        format!("{root}/batch-rri-followup-20260809-meta.csv"),
        rri_meta,
    )
    .unwrap();
    std::fs::write(
        format!("{root}/batch-nominal-followup-20260809.json"),
        nominal_json,
    )
    .unwrap();
    std::fs::write(
        format!("{root}/batch-nominal-followup-20260809-meta.csv"),
        nominal_meta,
    )
    .unwrap();
    println!("wrote RRI follow-up: {} rows", rri_rows.len());
    println!("wrote NOMINAL follow-up: {} rows", nominal_rows.len());
}
