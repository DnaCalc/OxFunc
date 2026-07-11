pub mod capability;
pub mod coercion;
/// W108 Excel-faithful elementary numeric core (exp/expm1/log/log1p/sqrt).
/// Internal to the crate; financial/math kernels call these `pub(crate)` fns.
/// Under the `research-x87` feature the module is public so the W109
/// calculation-graph search tooling can reach `excel_numeric::research`; every
/// non-research item inside stays `pub(crate)` either way.
#[cfg(not(feature = "research-x87"))]
mod excel_numeric;
#[cfg(feature = "research-x87")]
pub mod excel_numeric;
#[macro_use]
pub mod function;
pub mod function_call;
pub mod functions;
pub mod host_info;
pub mod locale_format;
pub mod registry;
mod registry_context_seed;
mod registry_help_seed;
mod registry_signature_seed;
pub mod resolver;
pub mod semantic_kernel;
pub mod value;
pub mod xll_export_specs;
