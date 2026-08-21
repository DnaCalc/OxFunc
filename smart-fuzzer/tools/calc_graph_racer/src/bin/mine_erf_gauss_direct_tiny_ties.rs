//! Offline disagreement mining for the 480 aggregate-tied GAUSS direct-tiny graphs.
//!
//! The only oracle answers read are the two named GAUSS discovery banks used by
//! the parent route/store checkpoint. Candidate-only searches never inspect an
//! answer and stay inside the identified `abs(x) <= 1e-15` direct-route domain.
//! No heldout path is named or read and this program performs no COM activity.
//!
//! Usage:
//!   mine_erf_gauss_direct_tiny_ties <OxFunc-root>

mod parent {
    include!("erf_gauss_tie_research/common.rs");

    const DIRECT_LIMIT_BITS: u64 = 1.0e-15_f64.to_bits();

    #[derive(Clone, Copy, Debug)]
    struct Candidate {
        ordinal: usize,
        cfg: BodyCfg,
        site: HalfSite,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct ScoreKey {
        reverse_exact: usize,
        max_ulp: u64,
        sum_ulp: u64,
    }

    #[derive(Clone, Copy, Debug)]
    struct ExistingRow {
        input_bits: u64,
        expected_bits: u64,
    }

    fn enumerate_candidates() -> Vec<Candidate> {
        let x_modes = [XMode::Extended, XMode::Stored];
        let gam_modes = [
            GamMode::Binary64,
            GamMode::Extended,
            GamMode::ExtendedReturn53,
        ];
        let inner_modes = [
            InnerMode::ExtendedCompensated,
            InnerMode::ExtendedDirect,
            InnerMode::Binary64Compensated,
            InnerMode::Binary64Direct,
        ];
        let associations = [Assoc::WgThenInner, Assoc::WThenGInner, Assoc::WInnerThenG];
        let w_modes = [WMode::X87Continuous, WMode::InputZ];
        let sites = [
            HalfSite::StoredReturn,
            HalfSite::ExtendedReturn,
            HalfSite::GFactor,
            HalfSite::WFactor,
            HalfSite::InnerFactor,
        ];
        let mut candidates = Vec::new();
        for x in x_modes {
            for series_53 in [false, true] {
                for j_53 in [false, true] {
                    for gam in gam_modes {
                        for g_53 in [false, true] {
                            for inner in inner_modes {
                                for assoc in associations {
                                    for first_product_53 in [false, true] {
                                        for w in w_modes {
                                            let cfg = BodyCfg {
                                                x,
                                                series_53,
                                                j_53,
                                                gam,
                                                g_53,
                                                inner,
                                                assoc,
                                                first_product_53,
                                                w,
                                            };
                                            for site in sites {
                                                candidates.push(Candidate {
                                                    ordinal: candidates.len(),
                                                    cfg,
                                                    site,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        candidates
    }

    fn predict_bits(input_bits: u64, candidate: Candidate) -> u64 {
        let input = f64::from_bits(input_bits);
        let z = input.abs() * std::f64::consts::FRAC_1_SQRT_2;
        let mut got = body_parts(z, candidate.cfg)
            .map(|(w, g, inner)| combine(w, g, inner, candidate.cfg, candidate.site))
            .unwrap_or(0.0);
        if input.is_sign_negative() {
            got = -got;
        }
        flush_subnormal(got).to_bits()
    }

    fn score(rows: &[ExistingRow], candidate: Candidate) -> ScoreKey {
        let mut exact = 0usize;
        let mut max_ulp = 0u64;
        let mut sum_ulp = 0u64;
        for row in rows {
            let got = predict_bits(row.input_bits, candidate);
            let delta = distance(got, row.expected_bits);
            exact += usize::from(delta == 0);
            max_ulp = max_ulp.max(delta);
            sum_ulp = sum_ulp.saturating_add(delta);
        }
        ScoreKey {
            reverse_exact: rows.len() - exact,
            max_ulp,
            sum_ulp,
        }
    }

    fn load_rows(root: &str) -> Vec<ExistingRow> {
        let exact = load_gauss(
            root,
            "answers-gauss-exact-discovery-v1.json",
            "batch-gauss-exact-discovery-v1.json",
            8_192,
        );
        let route = load_gauss(
            root,
            "answers-gauss-route-discovery-v1.json",
            "batch-gauss-route-discovery-v1.json",
            1_024,
        );
        let mut all = exact;
        for (input, expected) in route {
            assert!(
                all.insert(input, expected).is_none(),
                "GAUSS discovery overlap"
            );
        }
        let rows: Vec<_> = all
            .into_iter()
            .filter_map(|(input_bits, expected_bits)| {
                let input = f64::from_bits(input_bits);
                (input.abs() <= 1.0e-15).then_some(ExistingRow {
                    input_bits,
                    expected_bits,
                })
            })
            .collect();
        assert_eq!(rows.len(), 3_158, "direct-route discovery count drifted");
        rows
    }

    fn choose_answer_blind_rows(outputs: &[Vec<u64>], input_bits: &[u64]) -> Vec<usize> {
        let mut groups = vec![(0..outputs.len()).collect::<Vec<_>>()];
        let mut selected = Vec::new();
        let mut unused: BTreeSet<usize> = (0..input_bits.len()).collect();
        loop {
            let unresolved_before: usize = groups
                .iter()
                .map(|group| group.len() * group.len().saturating_sub(1) / 2)
                .sum();
            if unresolved_before == 0 {
                break;
            }
            let best = unused
                .iter()
                .filter_map(|&row| {
                    let unresolved_after: usize = groups
                        .iter()
                        .map(|group| {
                            let mut buckets: BTreeMap<u64, usize> = BTreeMap::new();
                            for &candidate in group {
                                *buckets.entry(outputs[candidate][row]).or_default() += 1;
                            }
                            buckets
                                .values()
                                .map(|count| count * count.saturating_sub(1) / 2)
                                .sum::<usize>()
                        })
                        .sum();
                    let gain = unresolved_before - unresolved_after;
                    (gain > 0).then_some((gain, std::cmp::Reverse(input_bits[row]), row))
                })
                .max();
            let Some((_, _, row)) = best else {
                break;
            };
            unused.remove(&row);
            selected.push(row);
            let mut next = Vec::new();
            for group in groups {
                let mut buckets: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
                for candidate in group {
                    buckets
                        .entry(outputs[candidate][row])
                        .or_default()
                        .push(candidate);
                }
                next.extend(buckets.into_values());
            }
            groups = next;
        }
        selected
    }

    fn add_ladder(pool: &mut BTreeSet<u64>, center: u64, radius: i64) {
        for offset in -radius..=radius {
            let bits = center.wrapping_add_signed(offset);
            let value = f64::from_bits(bits);
            if value.is_finite() && value.abs() <= 1.0e-15 {
                pool.insert(bits);
                pool.insert(bits ^ (1u64 << 63));
            }
        }
    }

    fn answer_blind_pool(existing: &BTreeSet<u64>) -> Vec<u64> {
        let mut pool = BTreeSet::new();
        for center in [
            0.0_f64.to_bits(),
            f64::MIN_POSITIVE.to_bits(),
            f64::EPSILON.to_bits(),
            (4.0 * f64::EPSILON).to_bits(),
            1.0e-16_f64.to_bits(),
            5.0e-16_f64.to_bits(),
            DIRECT_LIMIT_BITS,
        ] {
            add_ladder(&mut pool, center, 512);
        }
        for exponent in 1u64..=0x3cc {
            for mantissa in [
                0u64,
                1,
                0x0005_5555_5555_5555,
                0x000a_aaaa_aaaa_aaaa,
                0x000f_ffff_ffff_ffff,
            ] {
                let bits = (exponent << 52) | mantissa;
                let value = f64::from_bits(bits);
                if value <= 1.0e-15 {
                    pool.insert(bits);
                    pool.insert(bits | (1u64 << 63));
                }
            }
        }
        let mut state = 0x4733_3037_5449_4553u64;
        while pool.len() < 32_768 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let bits = state & 0x7fff_ffff_ffff_ffff;
            let value = f64::from_bits(bits);
            if value.is_finite() && value <= 1.0e-15 {
                pool.insert(bits);
                pool.insert(bits | (1u64 << 63));
            }
        }
        pool.retain(|bits| !existing.contains(bits));
        pool.into_iter().collect()
    }

    fn behavior_classes(outputs: Vec<Vec<u64>>) -> (Vec<Vec<u64>>, Vec<Vec<usize>>) {
        let mut by_outputs: BTreeMap<Vec<u64>, Vec<usize>> = BTreeMap::new();
        for (candidate, vector) in outputs.into_iter().enumerate() {
            by_outputs.entry(vector).or_default().push(candidate);
        }
        let mut vectors = Vec::with_capacity(by_outputs.len());
        let mut members = Vec::with_capacity(by_outputs.len());
        for (vector, group) in by_outputs {
            vectors.push(vector);
            members.push(group);
        }
        (vectors, members)
    }

    fn print_axes(group: &[usize], tied: &[Candidate]) {
        let count = |predicate: &dyn Fn(Candidate) -> bool| {
            group
                .iter()
                .filter(|&&member| predicate(tied[member]))
                .count()
        };
        println!(
            "    axes x[E={},S={}] series53[F={},T={}] j53[F={},T={}] g53[F={},T={}] fp53[F={},T={}] w[X87={},Z={}]",
            count(&|c| c.cfg.x == XMode::Extended),
            count(&|c| c.cfg.x == XMode::Stored),
            count(&|c| !c.cfg.series_53),
            count(&|c| c.cfg.series_53),
            count(&|c| !c.cfg.j_53),
            count(&|c| c.cfg.j_53),
            count(&|c| !c.cfg.g_53),
            count(&|c| c.cfg.g_53),
            count(&|c| !c.cfg.first_product_53),
            count(&|c| c.cfg.first_product_53),
            count(&|c| c.cfg.w == WMode::X87Continuous),
            count(&|c| c.cfg.w == WMode::InputZ),
        );
        println!(
            "    inner[EC={},ED={},BC={},BD={}] assoc[WgThenInner={},WThenGInner={},WInnerThenG={}] half[SR={},ER={},G={},W={},I={}]",
            count(&|c| c.cfg.inner == InnerMode::ExtendedCompensated),
            count(&|c| c.cfg.inner == InnerMode::ExtendedDirect),
            count(&|c| c.cfg.inner == InnerMode::Binary64Compensated),
            count(&|c| c.cfg.inner == InnerMode::Binary64Direct),
            count(&|c| c.cfg.assoc == Assoc::WgThenInner),
            count(&|c| c.cfg.assoc == Assoc::WThenGInner),
            count(&|c| c.cfg.assoc == Assoc::WInnerThenG),
            count(&|c| c.site == HalfSite::StoredReturn),
            count(&|c| c.site == HalfSite::ExtendedReturn),
            count(&|c| c.site == HalfSite::GFactor),
            count(&|c| c.site == HalfSite::WFactor),
            count(&|c| c.site == HalfSite::InnerFactor),
        );
    }

    pub(super) fn run(root: &str) {
        let rows = load_rows(root);
        let candidates = enumerate_candidates();
        assert_eq!(candidates.len(), 11_520, "candidate graph count drifted");

        let mut scored: Vec<_> = candidates
            .iter()
            .copied()
            .map(|candidate| (score(&rows, candidate), candidate))
            .collect();
        scored.sort_by_key(|(key, candidate)| (*key, candidate.ordinal));
        let best_key = scored[0].0;
        let tied: Vec<_> = scored
            .iter()
            .take_while(|(key, _)| *key == best_key)
            .map(|(_, candidate)| *candidate)
            .collect();
        assert_eq!(best_key.reverse_exact, 336);
        assert_eq!(best_key.max_ulp, 1);
        assert_eq!(best_key.sum_ulp, 336);
        assert_eq!(tied.len(), 480);
        println!(
            "parent race reproduced: graphs={} rows={} best={}/{} max={} sum={} ties={}",
            candidates.len(),
            rows.len(),
            rows.len() - best_key.reverse_exact,
            rows.len(),
            best_key.max_ulp,
            best_key.sum_ulp,
            tied.len()
        );

        let existing_outputs: Vec<Vec<u64>> = tied
            .iter()
            .map(|candidate| {
                rows.iter()
                    .map(|row| predict_bits(row.input_bits, *candidate))
                    .collect()
            })
            .collect();
        let (existing_vectors, existing_members) = behavior_classes(existing_outputs);
        let existing_sizes: BTreeMap<usize, usize> =
            existing_members
                .iter()
                .fold(BTreeMap::new(), |mut counts, group| {
                    *counts.entry(group.len()).or_default() += 1;
                    counts
                });
        let disagreeing_existing = (0..rows.len())
            .filter(|&row| {
                existing_vectors
                    .iter()
                    .map(|vector| vector[row])
                    .collect::<BTreeSet<_>>()
                    .len()
                    > 1
            })
            .count();
        assert_eq!(existing_vectors.len(), 1);
        assert_eq!(existing_sizes, BTreeMap::from([(480, 1)]));
        assert_eq!(disagreeing_existing, 0);
        println!(
            "existing direct-route per-row behavior: classes={} multiplicities={existing_sizes:?} disagreeing_rows={disagreeing_existing}/{}",
            existing_vectors.len(),
            rows.len()
        );
        let existing_bits: Vec<_> = rows.iter().map(|row| row.input_bits).collect();
        let existing_selected = choose_answer_blind_rows(&existing_vectors, &existing_bits);
        assert!(existing_selected.is_empty());
        println!(
            "existing answer-blind greedy separator: rows={} bits={:?}",
            existing_selected.len(),
            existing_selected
                .iter()
                .map(|&row| format!("0x{:016x}", existing_bits[row]))
                .collect::<Vec<_>>()
        );

        let existing_set: BTreeSet<_> = existing_bits.iter().copied().collect();
        let pool = answer_blind_pool(&existing_set);
        let mut pool_disagreements = Vec::new();
        let mut pool_outputs = vec![Vec::new(); tied.len()];
        for input_bits in pool.iter().copied() {
            let outputs: Vec<_> = tied
                .iter()
                .map(|candidate| predict_bits(input_bits, *candidate))
                .collect();
            let distinct = outputs.iter().copied().collect::<BTreeSet<_>>().len();
            if distinct > 1 {
                pool_disagreements.push(input_bits);
                for (candidate, output) in outputs.into_iter().enumerate() {
                    pool_outputs[candidate].push(output);
                }
            }
        }
        let (pool_vectors, pool_members) = behavior_classes(pool_outputs);
        let pool_sizes: BTreeMap<usize, usize> =
            pool_members
                .iter()
                .fold(BTreeMap::new(), |mut counts, group| {
                    *counts.entry(group.len()).or_default() += 1;
                    counts
                });
        let pool_selected = choose_answer_blind_rows(&pool_vectors, &pool_disagreements);
        assert_eq!(pool.len(), 30_032);
        assert_eq!(pool_disagreements.len(), 14);
        assert_eq!(pool_vectors.len(), 2);
        assert_eq!(pool_sizes, BTreeMap::from([(80, 1), (400, 1)]));
        assert_eq!(pool_selected.len(), 1);
        assert_eq!(pool_disagreements[pool_selected[0]], 0x02e6_4367_549e_b209);
        assert!(
            pool_disagreements
                .iter()
                .all(|bits| pool_disagreements.contains(&(bits ^ (1u64 << 63))))
        );
        println!(
            "candidate-only pool: inputs={} disagreements={} behavior_classes={} multiplicities={pool_sizes:?}",
            pool.len(),
            pool_disagreements.len(),
            pool_vectors.len()
        );
        println!(
            "candidate-only greedy separator: rows={} bits={:?}",
            pool_selected.len(),
            pool_selected
                .iter()
                .map(|&row| format!("0x{:016x}", pool_disagreements[row]))
                .collect::<Vec<_>>()
        );
        println!(
            "candidate-only disagreement bits={:?}",
            pool_disagreements
                .iter()
                .map(|bits| format!("0x{bits:016x}"))
                .collect::<Vec<_>>()
        );
        for &row in &pool_selected {
            let output_counts = pool_vectors.iter().enumerate().fold(
                BTreeMap::<u64, usize>::new(),
                |mut counts, (class, vector)| {
                    *counts.entry(vector[row]).or_default() += pool_members[class].len();
                    counts
                },
            );
            assert_eq!(
                output_counts,
                BTreeMap::from([(0x02d1_c377_56a9_7d07, 80), (0x02d1_c377_56a9_7d08, 400)])
            );
            println!(
                "  separator input=0x{:016x} output_graph_counts={:?}",
                pool_disagreements[row],
                output_counts
                    .into_iter()
                    .map(|(bits, count)| (format!("0x{bits:016x}"), count))
                    .collect::<Vec<_>>()
            );
        }

        let mut combined_outputs = Vec::with_capacity(tied.len());
        for candidate in 0..tied.len() {
            let mut vector = existing_vectors[existing_members
                .iter()
                .position(|group| group.contains(&candidate))
                .expect("existing class")]
            .clone();
            vector.extend_from_slice(
                &pool_vectors[pool_members
                    .iter()
                    .position(|group| group.contains(&candidate))
                    .expect("pool class")],
            );
            combined_outputs.push(vector);
        }
        let (_, combined_members) = behavior_classes(combined_outputs);
        let combined_sizes: BTreeMap<usize, usize> =
            combined_members
                .iter()
                .fold(BTreeMap::new(), |mut counts, group| {
                    *counts.entry(group.len()).or_default() += 1;
                    counts
                });
        assert_eq!(combined_members.len(), 2);
        assert_eq!(combined_sizes, BTreeMap::from([(80, 1), (400, 1)]));
        println!(
            "combined behavioral equivalence: classes={} multiplicities={combined_sizes:?}",
            combined_members.len()
        );

        for (class, group) in combined_members.iter().enumerate() {
            let representative = tied[group[0]];
            println!(
                "  class={class:03} members={} representative=g{:05} half={:?} {:?}",
                group.len(),
                representative.ordinal,
                representative.site,
                representative.cfg
            );
            print_axes(group, &tied);
        }
    }
}

fn main() {
    let root = std::env::args().nth(1).expect("OxFunc root");
    parent::run(&root);
}
