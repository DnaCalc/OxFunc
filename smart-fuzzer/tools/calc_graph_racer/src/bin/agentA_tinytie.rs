//! Agent A (2026-08-21 ERF/GAUSS swarm): hardware answer key for the 14
//! tiny-tie separator inputs under the two tied prediction classes from
//! `ERF_GAUSS_DIRECT_TINY_TIE_MINING_OFFLINE_CHECKPOINT_20260809.md`.
//!
//! Discovery-only: reads no oracle answers, names no heldout, performs no COM
//! activity. Reuses the frozen tie-research module byte-for-byte (SHA256
//! 3049E37155C920F55225EEAE174E288396F2F387D52281BE0A6BC4444C362BD1).
//!
//! Class 80  = XMode::Extended, gam Binary64, g extended, assoc WInnerThenG,
//!             first product stored to binary64, w = X87Continuous
//!             (register-continuous FYL2X -> F2XM1 exp(0.5*ln(z^2)) recovery,
//!             then the RN53 store of w*inner before the g multiply).
//! Class 400 = XMode::Extended, gam Binary64, g extended, w = InputZ
//!             (direct reuse of the stored GAUSS argument z).
//!
//! Usage: agentA_tinytie

mod parent {
    include!("erf_gauss_tie_research/common.rs");

    const SEPARATORS: [u64; 14] = [
        0x02e6_4367_549e_b209,
        0x04d4_c623_1fc3_00e4,
        0x0506_37ff_cc5e_d176,
        0x08a5_7db8_b14a_5222,
        0x0cb5_1ffb_4e2d_7c5f,
        0x0dd6_7b5b_55e4_d187,
        0x1006_3804_74b3_4294,
        0x82e6_4367_549e_b209,
        0x84d4_c623_1fc3_00e4,
        0x8506_37ff_cc5e_d176,
        0x88a5_7db8_b14a_5222,
        0x8cb5_1ffb_4e2d_7c5f,
        0x8dd6_7b5b_55e4_d187,
        0x9006_3804_74b3_4294,
    ];

    fn predict_bits(input_bits: u64, cfg: BodyCfg, site: HalfSite) -> u64 {
        let input = f64::from_bits(input_bits);
        let z = input.abs() * std::f64::consts::FRAC_1_SQRT_2;
        let mut got = body_parts(z, cfg)
            .map(|(w, g, inner)| combine(w, g, inner, cfg, site))
            .unwrap_or(0.0);
        if input.is_sign_negative() {
            got = -got;
        }
        flush_subnormal(got).to_bits()
    }

    pub(super) fn run() {
        let class80 = BodyCfg {
            x: XMode::Extended,
            series_53: false,
            j_53: false,
            gam: GamMode::Binary64,
            g_53: false,
            inner: InnerMode::ExtendedCompensated,
            assoc: Assoc::WInnerThenG,
            first_product_53: true,
            w: WMode::X87Continuous,
        };
        let class400 = BodyCfg {
            x: XMode::Extended,
            series_53: false,
            j_53: false,
            gam: GamMode::Binary64,
            g_53: false,
            inner: InnerMode::ExtendedCompensated,
            assoc: Assoc::WgThenInner,
            first_product_53: false,
            w: WMode::InputZ,
        };
        let h = store(&gam1_half(GamMode::Binary64));
        println!("gam1_half(binary64 per-op) h bits = 0x{:016x}", h.to_bits());
        println!(
            "{:>18} {:>18} {:>18} {:>18} {:>18} {:>5}",
            "input", "z_bits", "rn53_w80_bits", "class80_bits", "class400_bits", "wflip"
        );
        for bits in SEPARATORS {
            let input = f64::from_bits(bits);
            let z = input.abs() * std::f64::consts::FRAC_1_SQRT_2;
            let (w80, _, inner80) = body_parts(z, class80).expect("nonzero tiny x");
            let w80_stored = store(&ext_mul(&w80, &inner80, CW));
            let p80 = predict_bits(bits, class80, HalfSite::StoredReturn);
            let p400 = predict_bits(bits, class400, HalfSite::StoredReturn);
            let flip = (w80_stored.to_bits() as i64).wrapping_sub(z.to_bits() as i64);
            println!(
                "0x{bits:016x} 0x{:016x} 0x{:016x} 0x{p80:016x} 0x{p400:016x} {flip:+}",
                z.to_bits(),
                w80_stored.to_bits(),
            );
        }
    }
}

fn main() {
    parent::run();
}
