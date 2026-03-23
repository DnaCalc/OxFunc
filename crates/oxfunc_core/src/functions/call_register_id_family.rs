use crate::coercion::CoercionError;
use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::adapters::{
    PreparedArgValue, coerce_prepared_to_text, prepare_args_values_only,
};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, ExcelText, WorksheetErrorCode};

pub const CALL_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.CALL",
    arity: Arity { min: 1, max: 255 },
    determinism: DeterminismClass::ExternalEventDependent,
    volatility: VolatilityClass::VolatileContextual,
    host_interaction: HostInteractionClass::ApplicationState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::RefsVisibleInAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::ExternalProvider,
    surface_fec_dependency_profile: FecDependencyProfile::Composite,
};

pub const REGISTER_ID_META: FunctionMeta = FunctionMeta {
    function_id: "FUNC.REGISTER.ID",
    arity: Arity { min: 2, max: 3 },
    determinism: DeterminismClass::ExternalEventDependent,
    volatility: VolatilityClass::NonVolatile,
    host_interaction: HostInteractionClass::ApplicationState,
    thread_safety: ThreadSafetyClass::HostSerialized,
    arg_preparation_profile: ArgPreparationProfile::ValuesOnlyPreAdapter,
    coercion_lift_profile: CoercionLiftProfile::Custom,
    kernel_signature_class: KernelSignatureClass::Custom,
    fec_dependency_profile: FecDependencyProfile::ExternalProvider,
    surface_fec_dependency_profile: FecDependencyProfile::ExternalProvider,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisteredProcedureSpec {
    Name(ExcelText),
    Ordinal(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisteredExternalOriginKind {
    WorksheetRegisterId,
    HostRegisteredExternal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterIdRequest {
    pub library_name: ExcelText,
    pub procedure: RegisteredProcedureSpec,
    pub declared_type_text: Option<ExcelText>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegisteredExternalDescriptor {
    pub stable_registration_id: String,
    pub register_id: f64,
    pub origin_kind: RegisteredExternalOriginKind,
    pub display_name: Option<ExcelText>,
    pub library_name: ExcelText,
    pub procedure: RegisteredProcedureSpec,
    pub declared_type_text: Option<ExcelText>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegisteredExternalTarget {
    RegisterId(f64),
    Direct(RegisterIdRequest),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegisteredExternalCallRequest {
    pub target: RegisteredExternalTarget,
    pub invocation_args: Vec<CallArgValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegisteredExternalProviderError {
    UnsupportedRegisterIdResolution,
    UnsupportedRegisteredExternalLookup,
    UnsupportedRegisteredExternalInvocation,
    UnknownRegisterId(f64),
    WorksheetError(WorksheetErrorCode),
    ProviderFailure { detail: String },
}

pub trait RegisteredExternalProvider {
    fn resolve_register_id(
        &self,
        _request: &RegisterIdRequest,
    ) -> Result<RegisteredExternalDescriptor, RegisteredExternalProviderError> {
        Err(RegisteredExternalProviderError::UnsupportedRegisterIdResolution)
    }

    fn lookup_registered_external(
        &self,
        _register_id: f64,
    ) -> Result<RegisteredExternalDescriptor, RegisteredExternalProviderError> {
        Err(RegisteredExternalProviderError::UnsupportedRegisteredExternalLookup)
    }

    fn invoke_registered_external(
        &self,
        _descriptor: &RegisteredExternalDescriptor,
        _args: &[CallArgValue],
    ) -> Result<EvalValue, RegisteredExternalProviderError> {
        Err(RegisteredExternalProviderError::UnsupportedRegisteredExternalInvocation)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CallRegisterIdEvalError {
    ArityMismatch {
        expected_min: usize,
        expected_max: usize,
        actual: usize,
    },
    Preparation(CoercionError),
    UnsupportedProcedureValueKind,
    NonIntegralProcedureOrdinal(f64),
    ProviderMissing,
    Provider(RegisteredExternalProviderError),
}

fn coerce_prepared_to_procedure_spec(
    arg: &PreparedArgValue,
) -> Result<RegisteredProcedureSpec, CallRegisterIdEvalError> {
    match arg {
        PreparedArgValue::Eval(EvalValue::Text(text)) => {
            Ok(RegisteredProcedureSpec::Name(text.clone()))
        }
        PreparedArgValue::Eval(EvalValue::Number(n))
            if n.is_finite()
                && *n >= i32::MIN as f64
                && *n <= i32::MAX as f64
                && n.fract() == 0.0 =>
        {
            Ok(RegisteredProcedureSpec::Ordinal(*n as i32))
        }
        PreparedArgValue::Eval(EvalValue::Number(n)) => {
            Err(CallRegisterIdEvalError::NonIntegralProcedureOrdinal(*n))
        }
        _ => Err(CallRegisterIdEvalError::UnsupportedProcedureValueKind),
    }
}

pub fn parse_register_id_request(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<RegisterIdRequest, CallRegisterIdEvalError> {
    if !REGISTER_ID_META.arity.accepts(args.len()) {
        return Err(CallRegisterIdEvalError::ArityMismatch {
            expected_min: REGISTER_ID_META.arity.min,
            expected_max: REGISTER_ID_META.arity.max,
            actual: args.len(),
        });
    }

    let prepared =
        prepare_args_values_only(args, resolver).map_err(CallRegisterIdEvalError::Preparation)?;
    let library_name =
        coerce_prepared_to_text(&prepared[0]).map_err(CallRegisterIdEvalError::Preparation)?;
    let procedure = coerce_prepared_to_procedure_spec(&prepared[1])?;
    let declared_type_text = if prepared.len() >= 3 {
        Some(coerce_prepared_to_text(&prepared[2]).map_err(CallRegisterIdEvalError::Preparation)?)
    } else {
        None
    };

    Ok(RegisterIdRequest {
        library_name,
        procedure,
        declared_type_text,
    })
}

pub fn parse_call_request(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
) -> Result<RegisteredExternalCallRequest, CallRegisterIdEvalError> {
    if !CALL_META.arity.accepts(args.len()) {
        return Err(CallRegisterIdEvalError::ArityMismatch {
            expected_min: CALL_META.arity.min,
            expected_max: CALL_META.arity.max,
            actual: args.len(),
        });
    }

    let first = prepare_args_values_only(&args[..1], resolver)
        .map_err(CallRegisterIdEvalError::Preparation)?;
    if let PreparedArgValue::Eval(EvalValue::Number(register_id)) = &first[0] {
        return Ok(RegisteredExternalCallRequest {
            target: RegisteredExternalTarget::RegisterId(*register_id),
            invocation_args: args[1..].to_vec(),
        });
    }

    if args.len() == 2 {
        let target_prefix = prepare_args_values_only(&args[..2], resolver)
            .map_err(CallRegisterIdEvalError::Preparation)?;
        let library_name = coerce_prepared_to_text(&target_prefix[0])
            .map_err(CallRegisterIdEvalError::Preparation)?;
        let procedure = coerce_prepared_to_procedure_spec(&target_prefix[1])?;
        return Ok(RegisteredExternalCallRequest {
            target: RegisteredExternalTarget::Direct(RegisterIdRequest {
                library_name,
                procedure,
                declared_type_text: None,
            }),
            invocation_args: Vec::new(),
        });
    }

    let target_prefix = prepare_args_values_only(&args[..3], resolver)
        .map_err(CallRegisterIdEvalError::Preparation)?;
    let library_name =
        coerce_prepared_to_text(&target_prefix[0]).map_err(CallRegisterIdEvalError::Preparation)?;
    let procedure = coerce_prepared_to_procedure_spec(&target_prefix[1])?;
    let declared_type_text =
        coerce_prepared_to_text(&target_prefix[2]).map_err(CallRegisterIdEvalError::Preparation)?;

    Ok(RegisteredExternalCallRequest {
        target: RegisteredExternalTarget::Direct(RegisterIdRequest {
            library_name,
            procedure,
            declared_type_text: Some(declared_type_text),
        }),
        invocation_args: args[3..].to_vec(),
    })
}

pub fn eval_register_id_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    provider: Option<&dyn RegisteredExternalProvider>,
) -> Result<EvalValue, CallRegisterIdEvalError> {
    let request = parse_register_id_request(args, resolver)?;
    let provider = provider.ok_or(CallRegisterIdEvalError::ProviderMissing)?;
    let descriptor = provider
        .resolve_register_id(&request)
        .map_err(CallRegisterIdEvalError::Provider)?;
    Ok(EvalValue::Number(descriptor.register_id))
}

pub fn eval_call_surface(
    args: &[CallArgValue],
    resolver: &impl ReferenceResolver,
    provider: Option<&dyn RegisteredExternalProvider>,
) -> Result<EvalValue, CallRegisterIdEvalError> {
    let request = parse_call_request(args, resolver)?;
    let provider = provider.ok_or(CallRegisterIdEvalError::ProviderMissing)?;
    let descriptor = match &request.target {
        RegisteredExternalTarget::RegisterId(register_id) => provider
            .lookup_registered_external(*register_id)
            .map_err(CallRegisterIdEvalError::Provider)?,
        RegisteredExternalTarget::Direct(register_id_request) => provider
            .resolve_register_id(register_id_request)
            .map_err(CallRegisterIdEvalError::Provider)?,
    };

    provider
        .invoke_registered_external(&descriptor, &request.invocation_args)
        .map_err(CallRegisterIdEvalError::Provider)
}

pub fn map_call_register_id_error_to_ws(error: &CallRegisterIdEvalError) -> WorksheetErrorCode {
    match error {
        CallRegisterIdEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        CallRegisterIdEvalError::Preparation(CoercionError::WorksheetError(code)) => *code,
        CallRegisterIdEvalError::Preparation(_) => WorksheetErrorCode::Value,
        CallRegisterIdEvalError::UnsupportedProcedureValueKind => WorksheetErrorCode::Value,
        CallRegisterIdEvalError::NonIntegralProcedureOrdinal(_) => WorksheetErrorCode::Value,
        CallRegisterIdEvalError::ProviderMissing => WorksheetErrorCode::Value,
        CallRegisterIdEvalError::Provider(RegisteredExternalProviderError::WorksheetError(
            code,
        )) => *code,
        CallRegisterIdEvalError::Provider(_) => WorksheetErrorCode::Value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{ReferenceKind, ReferenceLike};

    struct MockResolver;

    impl ReferenceResolver for MockResolver {
        fn capabilities(&self) -> ResolverCapabilities {
            ResolverCapabilities::permissive_local()
        }

        fn resolve_reference(
            &self,
            reference: &ReferenceLike,
        ) -> Result<EvalValue, RefResolutionError> {
            Err(RefResolutionError::UnresolvedReference {
                target: reference.target.clone(),
            })
        }
    }

    #[derive(Default)]
    struct RecordingProvider {
        last_resolve: std::cell::RefCell<Option<RegisterIdRequest>>,
        last_lookup: std::cell::RefCell<Option<f64>>,
        last_invoke: std::cell::RefCell<Option<(RegisteredExternalDescriptor, Vec<CallArgValue>)>>,
    }

    impl RecordingProvider {
        fn sample_descriptor() -> RegisteredExternalDescriptor {
            RegisteredExternalDescriptor {
                stable_registration_id: "REG.tickcount".to_string(),
                register_id: -1439170560.0,
                origin_kind: RegisteredExternalOriginKind::WorksheetRegisterId,
                display_name: Some(ExcelText::from_interop_assignment("GetTickCount")),
                library_name: ExcelText::from_interop_assignment("Kernel32"),
                procedure: RegisteredProcedureSpec::Name(ExcelText::from_interop_assignment(
                    "GetTickCount",
                )),
                declared_type_text: Some(ExcelText::from_interop_assignment("J!")),
            }
        }
    }

    impl RegisteredExternalProvider for RecordingProvider {
        fn resolve_register_id(
            &self,
            request: &RegisterIdRequest,
        ) -> Result<RegisteredExternalDescriptor, RegisteredExternalProviderError> {
            self.last_resolve.replace(Some(request.clone()));
            Ok(Self::sample_descriptor())
        }

        fn lookup_registered_external(
            &self,
            register_id: f64,
        ) -> Result<RegisteredExternalDescriptor, RegisteredExternalProviderError> {
            self.last_lookup.replace(Some(register_id));
            Ok(Self::sample_descriptor())
        }

        fn invoke_registered_external(
            &self,
            descriptor: &RegisteredExternalDescriptor,
            args: &[CallArgValue],
        ) -> Result<EvalValue, RegisteredExternalProviderError> {
            self.last_invoke
                .replace(Some((descriptor.clone(), args.to_vec())));
            Ok(EvalValue::Number(123.0))
        }
    }

    fn text_arg(text: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(ExcelText::from_interop_assignment(text)))
    }

    fn number_arg(number: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(number))
    }

    #[test]
    fn parse_register_id_request_preserves_library_procedure_and_type_text() {
        let request = parse_register_id_request(
            &[
                text_arg("Kernel32"),
                text_arg("GetTickCount"),
                text_arg("J!"),
            ],
            &MockResolver,
        )
        .expect("request");

        assert_eq!(request.library_name.to_string_lossy(), "Kernel32");
        assert_eq!(
            request.procedure,
            RegisteredProcedureSpec::Name(ExcelText::from_interop_assignment("GetTickCount"))
        );
        assert_eq!(
            request
                .declared_type_text
                .as_ref()
                .expect("type text")
                .to_string_lossy(),
            "J!"
        );
    }

    #[test]
    fn parse_register_id_request_accepts_integral_ordinal_procedure() {
        let request = parse_register_id_request(
            &[text_arg("Kernel32"), number_arg(42.0), text_arg("J!")],
            &MockResolver,
        )
        .expect("request");

        assert_eq!(request.procedure, RegisteredProcedureSpec::Ordinal(42));
    }

    #[test]
    fn eval_register_id_surface_returns_descriptor_register_id() {
        let provider = RecordingProvider::default();
        let got = eval_register_id_surface(
            &[
                text_arg("Kernel32"),
                text_arg("GetTickCount"),
                text_arg("J!"),
            ],
            &MockResolver,
            Some(&provider),
        );
        assert_eq!(got, Ok(EvalValue::Number(-1439170560.0)));
        assert_eq!(
            provider
                .last_resolve
                .borrow()
                .as_ref()
                .expect("request")
                .library_name
                .to_string_lossy(),
            "Kernel32"
        );
    }

    #[test]
    fn eval_call_surface_by_register_id_looks_up_then_invokes() {
        let provider = RecordingProvider::default();
        let got = eval_call_surface(
            &[number_arg(-1439170560.0), number_arg(1.0)],
            &MockResolver,
            Some(&provider),
        );

        assert_eq!(got, Ok(EvalValue::Number(123.0)));
        assert_eq!(*provider.last_lookup.borrow(), Some(-1439170560.0));
        let (_, args) = provider.last_invoke.borrow().clone().expect("invoke");
        assert_eq!(args, vec![number_arg(1.0)]);
    }

    #[test]
    fn eval_call_surface_direct_resolves_then_invokes_and_preserves_raw_args() {
        let provider = RecordingProvider::default();
        let got = eval_call_surface(
            &[
                text_arg("Kernel32"),
                text_arg("GetTickCount"),
                text_arg("J!"),
                CallArgValue::Reference(ReferenceLike {
                    kind: ReferenceKind::A1,
                    target: "A1".to_string(),
                }),
            ],
            &MockResolver,
            Some(&provider),
        );

        assert_eq!(got, Ok(EvalValue::Number(123.0)));
        let request = provider.last_resolve.borrow().clone().expect("request");
        assert_eq!(request.library_name.to_string_lossy(), "Kernel32");
        let (_, args) = provider.last_invoke.borrow().clone().expect("invoke");
        assert_eq!(
            args,
            vec![CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::A1,
                target: "A1".to_string(),
            })]
        );
    }

    #[test]
    fn direct_call_can_omit_type_text_for_seeded_zero_arg_lane() {
        let got = parse_call_request(
            &[text_arg("Kernel32"), text_arg("GetTickCount")],
            &MockResolver,
        )
        .expect("request");
        assert_eq!(
            got.target,
            RegisteredExternalTarget::Direct(RegisterIdRequest {
                library_name: ExcelText::from_interop_assignment("Kernel32"),
                procedure: RegisteredProcedureSpec::Name(ExcelText::from_interop_assignment(
                    "GetTickCount",
                )),
                declared_type_text: None,
            })
        );
        assert!(got.invocation_args.is_empty());
    }
}
