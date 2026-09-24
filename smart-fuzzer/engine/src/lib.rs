//! W112 parity driver (`sf`). This crate grows phase by phase (W112-P0..P6); today it holds
//! the offline half of `sf judge`: typed-bit comparison of production OxFunc against banked
//! Excel answers, and the ODR-FN-005 severity classifier that every later command reuses.
//!
//! Only typed-bit comparison against live-Excel answers is evidence (W112 rule 6.2). This
//! crate never changes production kernels (rule 6.1).

pub mod classify;
pub mod judge;
pub mod witness;
