pub const SQRT_2PI: f64 = 2.506_628_274_631_000_7;

pub fn phi_kernel(x: f64) -> f64 {
    (-0.5 * x * x).exp() / SQRT_2PI
}

pub fn erf_approx(x: f64) -> f64 {
    libm::erf(x)
}

pub fn gauss_kernel(x: f64) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    0.5 * erf_approx(x / std::f64::consts::SQRT_2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phi_and_gauss_match_excel_probe_lanes() {
        assert!((phi_kernel(0.0) - 0.398942280401433).abs() < 1e-12);
        assert!((phi_kernel(1.0) - 0.241970724519143).abs() < 1e-12);
        assert!(gauss_kernel(0.0).abs() < 1e-12);
        assert!((gauss_kernel(1.0) - 0.341344746068543).abs() < 1e-7);
    }
}
