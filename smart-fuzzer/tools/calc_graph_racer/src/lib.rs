//! calc_graph_racer — W109 calculation-graph search over Excel oracle bits.
//!
//! Three cooperating pieces:
//! * [`dsl`] — candidate calculation graphs as data (serde JSON);
//! * [`eval`] — bit-faithful evaluator (strict binary64 and x87 extended via
//!   `oxfunc_core::excel_numeric::research`);
//! * [`score`] + [`enumerate`] — lexicographic bit-exact scoring and the
//!   layered-search enumerators (association / store mask / eval model);
//! * [`scheduler`] — per-row surviving-candidate state, distinguishing-input
//!   search, and elimination against oracle answers.
//!
//! Acceptance fixtures (`tests/`) require the racer to rediscover the two
//! already-signed-off identifications — TBILLYIELD's association and POWER's
//! x87 staging — from witnesses alone before it is trusted on open rows.

pub mod dsl;
pub mod enumerate;
pub mod eval;
pub mod scheduler;
pub mod score;
