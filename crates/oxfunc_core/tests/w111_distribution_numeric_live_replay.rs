//! Exact public Excel replay; retained kernel discrepancies remain explicit.
use oxfunc_core::functions::{discrete_dist_family as d, normal_log_family as n};
use serde_json::Value;
use std::collections::BTreeMap;

fn replay(raw: &str, known: &str, expected_counts: (usize, usize)) {
    let data: Value = serde_json::from_str(raw).unwrap();
    let report: Value = serde_json::from_str(known).unwrap();
    let residuals: BTreeMap<&str, &Value> = report["misses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["id"].as_str().unwrap(), r))
        .collect();
    let mut counts = (0, 0);
    for row in data["witnesses"].as_array().unwrap() {
        let args: Vec<_> = row["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                f64::from_bits(
                    u64::from_str_radix(v.as_str().unwrap().trim_start_matches("0x"), 16).unwrap(),
                )
            })
            .collect();
        let actual = match data["function"].as_str().unwrap() {
            "NORM.DIST" => n::norm_dist_kernel(args[0], args[1], args[2], args[3] != 0.0),
            "NORM.S.DIST" => n::norm_s_dist_kernel(args[0], args[1] != 0.0),
            "GAUSS" => Ok(n::identified_gauss(args[0])),
            "BINOM.INV" => d::binom_inv_kernel(args[0], args[1], args[2]),
            "NEGBINOM.DIST" => d::negbinom_dist_kernel(args[0], args[1], args[2], args[3] != 0.0),
            "POISSON.DIST" => d::poisson_dist_kernel(args[0], args[1], args[2] != 0.0),
            "HYPGEOM.DIST" => {
                d::hypergeom_dist_kernel(args[0], args[1], args[2], args[3], args[4] != 0.0)
            }
            name => panic!("unbound function {name}"),
        };
        let actual = match actual {
            Ok(v) => format!("0x{:016x}", v.to_bits()),
            Err(e) => format!("error:{e:?}"),
        };
        let id = row["id"].as_str().unwrap();
        let expected = row["expected_bits"].as_str().unwrap();
        if let Some(open) = residuals.get(id) {
            assert_eq!(row["args"], open["args"]);
            assert_eq!(expected, open["expected"]);
            assert_eq!(actual, open["actual"], "changed residual {id}");
            assert_ne!(actual, expected);
            counts.1 += 1;
        } else {
            assert_eq!(actual, expected, "{id}");
            counts.0 += 1;
        }
    }
    assert_eq!(counts, expected_counts);
}
macro_rules! cohort {
    ($name:literal,$exact:expr,$open:expr) => {
        replay(
            include_str!(concat!(
                "../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/",
                $name,
                ".json"
            )),
            include_str!(concat!(
                "../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/",
                $name,
                "-production-replay.json"
            )),
            ($exact, $open),
        );
    };
}
#[test]
fn normal_density_operation_graph_matches_four_captured_cohorts() {
    cohort!("density-discovery", 5426, 0);
    cohort!("density-heldout", 6586, 0);
    cohort!("density-normalization", 128, 0);
    cohort!("density-subtraction", 144, 0);
}

#[test]
fn normal_density_frozen_production_matches_further_independent_packet() {
    cohort!("final-norm.dist", 6810, 0);
}
#[test]
fn discrete_domain_captures_preserve_open_interior_kernel_differences() {
    cohort!("domain-binom.inv", 402, 3);
    cohort!("domain-negbinom.dist", 234, 54);
    cohort!("domain-hypgeom.dist", 2185, 131);
    cohort!("negbinom-pin-controls", 8, 22);
}

#[test]
fn discrete_fresh_domain_and_branch_controls_preserve_numeric_residuals() {
    cohort!("final-binom.inv", 501, 0);
    cohort!("final-negbinom.dist", 115, 66);
    replay(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/final-hypgeom.dist.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/final-hypgeom.dist-raw-negative-refined-replay.json"),
        (968, 33),
    );
    cohort!("branch-hypgeom.dist", 446, 34);
    cohort!("branch-poisson.dist", 300, 12);
    replay(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/poisson-branch-heldout.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/poisson-publication/poisson-branch-heldout-current-replay.json"),
        (727, 17),
    );
}

#[test]
fn poisson_positive_count_publication_keeps_zero_count_subnormals_separate() {
    replay(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/poisson-publication/discovery-answers.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/poisson-publication/discovery-production-replay.json"),
        (959, 469),
    );
}

#[test]
fn poisson_frozen_publication_independent_boundaries_retain_backend_residuals() {
    replay(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/poisson-publication/heldout-answers.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/poisson-publication/heldout-production-replay.json"),
        (734, 366),
    );
}

#[test]
fn normal_cdf_staged_wrapper_preserves_open_erfc_backend_differences() {
    replay(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/integer-publication/heldout/answers/answers-norm.s.dist.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/cdf-research/full-norm.s.dist-production-replay.json"),
        (9271, 335),
    );
    replay(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/cdf-research/z-heldout-answers-norm.s.dist.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/cdf-research/z-heldout-norm.s.dist-production-replay.json"),
        (276, 236),
    );
    replay(
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/cdf-research/z-heldout-answers-gauss.json"),
        include_str!("../../../docs/function-lane/evidence/w111-broad-20260929/distribution-surface/cdf-research/z-heldout-gauss-production-replay.json"),
        (452, 60),
    );
}
