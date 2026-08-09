//! W109 G6-01: isolate PMT's timing-factor node without modelling `em`.
//!
//! At a power-of-two rate and `fv=0`, the final multiplication by rate is an
//! exact exponent shift.  Pairing otherwise identical type-0/type-1 calls
//! therefore cancels every upstream PMT unknown and distinguishes
//! `q / (1+r)` from `q * RN(1/(1+r))` directly from worksheet outputs.

use calc_graph_racer::eval::parse_bits_hex;
use calc_graph_racer::score::{WitnessArg, WitnessSet};
use std::collections::BTreeMap;

const ANSWER_DIR: &str = "../../work/w109/G6-solvers";
const HELDOUT_BATCH: &str = "../../work/w109/G6-solvers/batch-pmt-tf-metamer-heldout-20260809.json";
const HELDOUT_ANSWERS: &str =
    "../../work/w109/G6-solvers/answers-pmt-tf-metamer-heldout-20260809.json";
const HELDOUT_NAME: &str = "answers-pmt-tf-metamer-heldout-20260809.json";

#[derive(Clone)]
struct Observed {
    source: String,
    id: String,
    value: f64,
}

fn scalar_args(witness: &calc_graph_racer::score::Witness) -> Option<[f64; 5]> {
    let values = witness
        .args
        .iter()
        .filter_map(|arg| match arg {
            WitnessArg::Scalar(text) => parse_bits_hex(text),
            _ => None,
        })
        .collect::<Vec<_>>();
    (values.len() == 5).then(|| values.try_into().unwrap())
}

fn positive_power_of_two(value: f64) -> bool {
    value.is_sign_positive() && value.is_normal() && value.to_bits() & ((1_u64 << 52) - 1) == 0
}

fn validate_heldout_capture() {
    let batch: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(HELDOUT_BATCH).expect("read PMT timing heldout batch"),
    )
    .expect("parse PMT timing heldout batch");
    let answers: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(HELDOUT_ANSWERS).expect("read PMT timing heldout answers"),
    )
    .expect("parse PMT timing heldout answers");
    assert_eq!(batch["function"], "PMT");
    assert_eq!(answers["function"], "PMT");
    let probes = batch["probes"].as_array().expect("batch probes array");
    let witnesses = answers["witnesses"]
        .as_array()
        .expect("answer witnesses array");
    assert_eq!(probes.len(), 1_536);
    assert_eq!(witnesses.len(), probes.len());
    for (index, (wrapped, witness)) in probes.iter().zip(witnesses).enumerate() {
        let probe = &wrapped["probe"];
        assert_eq!(probe["id"], witness["id"], "id mismatch at {index}");
        assert_eq!(
            probe["args"], witness["args"],
            "argument mismatch at {index}"
        );
    }
    let provenance = &answers["capture_provenance"];
    assert_eq!(provenance["schema_version"], "w109-capture-provenance-v1");
    assert_eq!(provenance["environment"]["excel_version"], "16.0");
    assert_eq!(provenance["environment"]["excel_build"], "20228");
    assert_eq!(provenance["environment"]["excel_bitness"], "64-bit");
    assert_eq!(provenance["environment"]["workbook_compatibility"], "2");
    assert_eq!(
        provenance["environment"]["excel_input_plumbing"],
        "cell_value2_bulk"
    );
    assert_eq!(provenance["oracle_cache"]["mode"], "no_cache");
    assert_eq!(provenance["oracle_cache"]["hits"], 0);
    assert_eq!(provenance["oracle_cache"]["misses"], 0);
    assert_eq!(provenance["runner"]["version"], "w109-bulk-batch-v2");
}

#[derive(Default)]
struct Score {
    pairs: usize,
    reciprocal: usize,
    divide: usize,
    discriminators: usize,
}

impl Score {
    fn record(&mut self, reciprocal_hit: bool, divide_hit: bool, differs: bool) {
        self.pairs += 1;
        self.reciprocal += usize::from(reciprocal_hit);
        self.divide += usize::from(divide_hit);
        self.discriminators += usize::from(differs);
    }
}

fn main() {
    validate_heldout_capture();
    let mut pairs: BTreeMap<[u64; 4], [Option<Observed>; 2]> = BTreeMap::new();
    let mut sources = std::fs::read_dir(ANSWER_DIR)
        .expect("read PMT answer directory")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            (name.starts_with("answers-pmt-") && name.ends_with(".json"))
                .then_some((name, entry.path()))
        })
        .collect::<Vec<_>>();
    sources.sort_by(|a, b| a.0.cmp(&b.0));

    let mut parsed_sources = 0usize;
    let mut candidate_rows = 0usize;
    for (source, path) in sources {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(set) = serde_json::from_str::<WitnessSet>(&text) else {
            continue;
        };
        if set.function != "PMT" {
            continue;
        }
        parsed_sources += 1;
        for witness in &set.witnesses {
            let Some([rate, periods, present, future, timing]) = scalar_args(witness) else {
                continue;
            };
            if future != 0.0 || !positive_power_of_two(rate) || (timing != 0.0 && timing != 1.0) {
                continue;
            }
            let Some(expected) = parse_bits_hex(&witness.expected_bits) else {
                continue;
            };
            candidate_rows += 1;
            let key = [
                rate.to_bits(),
                periods.to_bits(),
                present.to_bits(),
                future.to_bits(),
            ];
            let slot = &mut pairs.entry(key).or_default()[timing as usize];
            if let Some(prior) = slot {
                assert_eq!(
                    prior.value.to_bits(),
                    expected.to_bits(),
                    "conflicting oracle outputs for {source}/{} and {}/{}",
                    witness.id.as_deref().unwrap_or("<missing-id>"),
                    prior.source,
                    prior.id,
                );
            } else {
                *slot = Some(Observed {
                    source: source.clone(),
                    id: witness
                        .id
                        .clone()
                        .unwrap_or_else(|| "<missing-id>".to_owned()),
                    value: expected,
                });
            }
        }
    }
    println!(
        "audited {parsed_sources} PMT answer files; {candidate_rows} power-of-two/fv=0/type rows; {} unique tuple keys",
        pairs.len()
    );

    let mut all = Score::default();
    let mut heldout = Score::default();
    let mut banked = Score::default();
    let mut printed = 0usize;
    for (key, observations) in &pairs {
        let [Some(type0), Some(type1)] = observations else {
            continue;
        };
        let rate = f64::from_bits(key[0]);
        let reciprocal = type0.value * (1.0 / (1.0 + rate));
        let divide = type0.value / (1.0 + rate);
        let reciprocal_hit = reciprocal.to_bits() == type1.value.to_bits();
        let divide_hit = divide.to_bits() == type1.value.to_bits();
        let differs = reciprocal.to_bits() != divide.to_bits();
        all.record(reciprocal_hit, divide_hit, differs);
        if type0.source == HELDOUT_NAME && type1.source == HELDOUT_NAME {
            heldout.record(reciprocal_hit, divide_hit, differs);
        } else {
            banked.record(reciprocal_hit, divide_hit, differs);
        }
        if differs {
            if printed < 12 {
                println!(
                    "discriminator {} / {}: r={:#018x} n={:#018x} pv={:#018x} type0={:#018x} Excel-type1={:#018x} recip={:#018x} divide={:#018x}",
                    format!("{}/{}", type0.source, type0.id),
                    format!("{}/{}", type1.source, type1.id),
                    key[0],
                    key[1],
                    key[2],
                    type0.value.to_bits(),
                    type1.value.to_bits(),
                    reciprocal.to_bits(),
                    divide.to_bits(),
                );
                printed += 1;
            }
        }
    }
    println!(
        "PMT timing metamer banked: pairs={}; reciprocal-multiply={}/{}; true-divide={}/{}; discriminating-pairs={}",
        banked.pairs,
        banked.reciprocal,
        banked.pairs,
        banked.divide,
        banked.pairs,
        banked.discriminators,
    );
    println!(
        "PMT timing metamer frozen heldout: pairs={}; reciprocal-multiply={}/{}; true-divide={}/{}; discriminating-pairs={}",
        heldout.pairs,
        heldout.reciprocal,
        heldout.pairs,
        heldout.divide,
        heldout.pairs,
        heldout.discriminators,
    );
    println!(
        "PMT timing metamer combined: pairs={}; reciprocal-multiply={}/{}; true-divide={}/{}; discriminating-pairs={}",
        all.pairs, all.reciprocal, all.pairs, all.divide, all.pairs, all.discriminators,
    );
    assert!(banked.pairs >= 64);
    assert_eq!(heldout.pairs, 768);
    assert_eq!(all.reciprocal, all.pairs);
    assert!(all.divide < all.pairs);
}
