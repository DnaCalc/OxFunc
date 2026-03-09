use crate::value::{EvalValue, ReferenceKind, ReferenceLike};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolverCapabilities {
    pub allow_eval_time_deref: bool,
    pub allow_three_d_refs: bool,
    pub allow_structured_refs: bool,
    pub allow_spill_anchor_refs: bool,
    pub allow_external_refs: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallerContext {
    pub prefix: Option<String>,
    pub row: usize,
    pub col: usize,
}

impl ResolverCapabilities {
    pub const fn permissive_local() -> Self {
        Self {
            allow_eval_time_deref: true,
            allow_three_d_refs: true,
            allow_structured_refs: true,
            allow_spill_anchor_refs: true,
            allow_external_refs: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefResolutionError {
    EvalTimeDerefNotAllowed,
    CapabilityDenied {
        kind: ReferenceKind,
        capability: &'static str,
    },
    UnresolvedReference {
        target: String,
    },
    ProviderFailure {
        detail: String,
    },
}

pub trait ReferenceResolver {
    fn capabilities(&self) -> ResolverCapabilities;
    fn resolve_reference(&self, reference: &ReferenceLike)
    -> Result<EvalValue, RefResolutionError>;

    fn caller_context(&self) -> Option<CallerContext> {
        None
    }
}

pub fn normalize_reference(reference: &ReferenceLike) -> ReferenceLike {
    ReferenceLike {
        kind: reference.kind,
        target: reference.target.trim().to_string(),
    }
}

pub fn resolve_eval_value(
    resolver: &impl ReferenceResolver,
    reference: &ReferenceLike,
) -> Result<EvalValue, RefResolutionError> {
    let normalized = normalize_reference(reference);
    let caps = resolver.capabilities();

    if !caps.allow_eval_time_deref {
        return Err(RefResolutionError::EvalTimeDerefNotAllowed);
    }

    match normalized.kind {
        ReferenceKind::ThreeD if !caps.allow_three_d_refs => {
            return Err(RefResolutionError::CapabilityDenied {
                kind: normalized.kind,
                capability: "allow_three_d_refs",
            });
        }
        ReferenceKind::Structured if !caps.allow_structured_refs => {
            return Err(RefResolutionError::CapabilityDenied {
                kind: normalized.kind,
                capability: "allow_structured_refs",
            });
        }
        ReferenceKind::SpillAnchor if !caps.allow_spill_anchor_refs => {
            return Err(RefResolutionError::CapabilityDenied {
                kind: normalized.kind,
                capability: "allow_spill_anchor_refs",
            });
        }
        _ => {}
    }

    if !caps.allow_external_refs && normalized.target.contains('[') {
        return Err(RefResolutionError::CapabilityDenied {
            kind: normalized.kind,
            capability: "allow_external_refs",
        });
    }

    resolver.resolve_reference(&normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{EvalValue, ReferenceKind, ReferenceLike};

    struct MockResolver {
        caps: ResolverCapabilities,
        resolved: Option<EvalValue>,
    }

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            self.caps
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            match &self.resolved {
                Some(v) => Ok(v.clone()),
                None => Err(RefResolutionError::UnresolvedReference {
                    target: reference.target.clone(),
                }),
            }
        }
    }

    #[test]
    fn normalize_reference_trims_target() {
        let input = ReferenceLike {
            kind: ReferenceKind::A1,
            target: "  Sheet1!A1  ".to_string(),
        };
        let got = normalize_reference(&input);
        assert_eq!(got.target, "Sheet1!A1");
    }

    #[test]
    fn resolve_rejects_three_d_when_capability_disabled() {
        let resolver = MockResolver {
            caps: ResolverCapabilities {
                allow_eval_time_deref: true,
                allow_three_d_refs: false,
                allow_structured_refs: true,
                allow_spill_anchor_refs: true,
                allow_external_refs: false,
            },
            resolved: Some(EvalValue::Number(1.0)),
        };

        let input = ReferenceLike {
            kind: ReferenceKind::ThreeD,
            target: "Sheet1:Sheet2!A1".to_string(),
        };

        let got = resolve_eval_value(&resolver, &input);
        assert_eq!(
            got,
            Err(RefResolutionError::CapabilityDenied {
                kind: ReferenceKind::ThreeD,
                capability: "allow_three_d_refs"
            })
        );
    }

    #[test]
    fn resolve_rejects_external_reference_when_disallowed() {
        let resolver = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved: Some(EvalValue::Number(1.0)),
        };
        let input = ReferenceLike {
            kind: ReferenceKind::A1,
            target: "[External.xlsx]Sheet1!A1".to_string(),
        };

        let got = resolve_eval_value(&resolver, &input);
        assert_eq!(
            got,
            Err(RefResolutionError::CapabilityDenied {
                kind: ReferenceKind::A1,
                capability: "allow_external_refs"
            })
        );
    }

    #[test]
    fn resolve_passes_normalized_reference_to_provider() {
        let resolver = MockResolver {
            caps: ResolverCapabilities::permissive_local(),
            resolved: Some(EvalValue::Number(3.0)),
        };

        let input = ReferenceLike {
            kind: ReferenceKind::A1,
            target: "  A1 ".to_string(),
        };

        let got = resolve_eval_value(&resolver, &input);
        assert_eq!(got, Ok(EvalValue::Number(3.0)));
    }
}
