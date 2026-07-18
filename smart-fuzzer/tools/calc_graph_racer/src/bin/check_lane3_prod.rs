//! Lane-3 production re-scorer. Mode argv[1]:
//!   chidist   stdin "x df"          -> chisq_dist_rt_kernel
//!   gammadist stdin "x a b cum"     -> gamma_dist_kernel (real routing,
//!                                      incl. the integer-shape fast path)
//!   gratio_p  stdin "x a b cum"     -> regularized_gamma_p(a, x/b) (the
//!                                      W109-identified path, fast path
//!                                      BYPASSED — for divergence checks)
//!   poisson   stdin "k lam cum"     -> poisson_dist_kernel
use oxfunc_core::functions::beta_gamma_stats_family::gamma_dist_kernel;
use oxfunc_core::functions::chi_f_t_family::chisq_dist_rt_kernel;
use oxfunc_core::functions::discrete_dist_family::poisson_dist_kernel;
use oxfunc_core::functions::special_math_common::regularized_gamma_p;
use std::io::BufRead;

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let p: Vec<f64> = line
            .split_whitespace()
            .map(|s| f64::from_bits(u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()))
            .collect();
        let r = match (mode.as_str(), p.len()) {
            ("chidist", 2) => chisq_dist_rt_kernel(p[0], p[1]).map_err(|_| ()),
            ("gammadist", 4) => gamma_dist_kernel(p[0], p[1], p[2], p[3] != 0.0).map_err(|_| ()),
            ("gratio_p", 4) => {
                if p[3] != 0.0 {
                    Ok(regularized_gamma_p(p[1], p[0] / p[2]))
                } else {
                    gamma_dist_kernel(p[0], p[1], p[2], false).map_err(|_| ())
                }
            }
            ("poisson", 3) => poisson_dist_kernel(p[0], p[1], p[2] != 0.0).map_err(|_| ()),
            _ => Err(()),
        };
        match r {
            Ok(v) => println!("{:016x}", v.to_bits()),
            Err(()) => println!("ERR"),
        }
    }
}
