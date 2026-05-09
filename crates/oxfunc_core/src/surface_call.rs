use crate::function::{
    ArgPreparationProfile, Arity, CoercionLiftProfile, DeterminismClass, FecDependencyProfile,
    FunctionMeta, HostInteractionClass, KernelSignatureClass, ThreadSafetyClass, VolatilityClass,
};
use crate::functions::call_register_id_family::RegisteredExternalProvider;
use crate::functions::callable_helpers::CallableInvoker;
use crate::functions::now_fn::NowProvider;
use crate::functions::rand_fn::RandomProvider;
use crate::functions::rtd_fn::RtdProvider;
use crate::functions::surface_dispatch::{
    SurfaceDispatchKey, eval_surface_value_call_with_dispatch_key, resolve_surface_dispatch_key,
};
use crate::host_info::HostInfoProvider;
use crate::locale_format::LocaleFormatContext;
use crate::registry::{FunctionEntry, FunctionRegistry, builtin_registry};
use crate::resolver::ReferenceResolver;
use crate::value::{CallArgValue, EvalValue, WorksheetErrorCode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceCallSiteResolveError {
    UnknownFunctionId(String),
    UnknownSurfaceName(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallableArgumentSpec {
    Fixed(usize),
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceCallHoistPolicy {
    /// Permit hoisting functions whose only FEC dependency is reference
    /// resolution, but only when the formula planner has separately proven the
    /// planned subtree contains no runtime-varying references.
    pub allow_ref_only_dependency: bool,
    pub allow_fixed_caller_context: bool,
    pub allow_fixed_locale: bool,
    pub allow_fixed_time: bool,
    pub allow_fixed_random: bool,
    pub allow_fixed_external: bool,
    pub allow_fixed_host_state: bool,
}

impl SurfaceCallHoistPolicy {
    pub const STRICT_CONTEXT_FREE: Self = Self {
        allow_ref_only_dependency: false,
        allow_fixed_caller_context: false,
        allow_fixed_locale: false,
        allow_fixed_time: false,
        allow_fixed_random: false,
        allow_fixed_external: false,
        allow_fixed_host_state: false,
    };

    pub const FIXED_RUNTIME_CONTEXT: Self = Self {
        allow_ref_only_dependency: true,
        allow_fixed_caller_context: true,
        allow_fixed_locale: true,
        allow_fixed_time: true,
        allow_fixed_random: true,
        allow_fixed_external: true,
        allow_fixed_host_state: true,
    };
}

impl Default for SurfaceCallHoistPolicy {
    fn default() -> Self {
        Self::STRICT_CONTEXT_FREE
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SurfaceCallScratch {
    call_args: Vec<CallArgValue>,
}

impl SurfaceCallScratch {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            call_args: Vec::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.call_args.clear();
    }

    pub fn call_args(&self) -> &[CallArgValue] {
        &self.call_args
    }

    pub fn call_args_mut(&mut self) -> &mut Vec<CallArgValue> {
        &mut self.call_args
    }

    pub fn push_arg(&mut self, arg: CallArgValue) {
        self.call_args.push(arg);
    }

    pub fn extend_args(&mut self, args: impl IntoIterator<Item = CallArgValue>) {
        self.call_args.extend(args);
    }

    pub fn capacity(&self) -> usize {
        self.call_args.capacity()
    }
}

const CALLABLE_NONE: [CallableArgumentSpec; 0] = [];
const CALLABLE_FIXED_1: [CallableArgumentSpec; 1] = [CallableArgumentSpec::Fixed(1)];
const CALLABLE_FIXED_2: [CallableArgumentSpec; 1] = [CallableArgumentSpec::Fixed(2)];
const CALLABLE_FIXED_3: [CallableArgumentSpec; 1] = [CallableArgumentSpec::Fixed(3)];
const CALLABLE_LAST: [CallableArgumentSpec; 1] = [CallableArgumentSpec::Last];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceCallSite {
    dispatch_key: SurfaceDispatchKey,
    meta: FunctionMeta,
    surface_name: String,
    canonical_surface_name: Option<String>,
    callable_argument_specs: &'static [CallableArgumentSpec],
}

impl SurfaceCallSite {
    pub fn from_function_id(function_id: &str) -> Result<Self, SurfaceCallSiteResolveError> {
        let dispatch_key = resolve_surface_dispatch_key(function_id).ok_or_else(|| {
            SurfaceCallSiteResolveError::UnknownFunctionId(function_id.to_string())
        })?;
        Ok(Self::from_dispatch_key(dispatch_key))
    }

    pub fn from_registry_entry(entry: &FunctionEntry) -> Result<Self, SurfaceCallSiteResolveError> {
        let dispatch_key =
            resolve_surface_dispatch_key(&entry.meta.function_id).ok_or_else(|| {
                SurfaceCallSiteResolveError::UnknownFunctionId(entry.meta.function_id.clone())
            })?;
        Ok(Self::from_dispatch_key(dispatch_key))
    }

    pub fn from_registry_function_id(
        registry: &FunctionRegistry,
        function_id: &str,
    ) -> Result<Self, SurfaceCallSiteResolveError> {
        let entry = registry.lookup_by_id(function_id).ok_or_else(|| {
            SurfaceCallSiteResolveError::UnknownFunctionId(function_id.to_string())
        })?;
        Self::from_registry_entry(entry)
    }

    pub fn from_registry_surface_name(
        registry: &FunctionRegistry,
        surface_name: &str,
    ) -> Result<Self, SurfaceCallSiteResolveError> {
        let entry = registry
            .lookup_by_surface_name(surface_name)
            .ok_or_else(|| {
                SurfaceCallSiteResolveError::UnknownSurfaceName(surface_name.to_string())
            })?;
        Self::from_registry_entry(entry)
    }

    pub fn from_surface_name(surface_name: &str) -> Result<Self, SurfaceCallSiteResolveError> {
        let entry = builtin_registry()
            .lookup_by_surface_name(surface_name)
            .ok_or_else(|| {
                SurfaceCallSiteResolveError::UnknownSurfaceName(surface_name.to_string())
            })?;
        let dispatch_key =
            resolve_surface_dispatch_key(&entry.meta.function_id).ok_or_else(|| {
                SurfaceCallSiteResolveError::UnknownFunctionId(entry.meta.function_id.clone())
            })?;
        Ok(Self::from_dispatch_key(dispatch_key))
    }

    fn from_dispatch_key(dispatch_key: SurfaceDispatchKey) -> Self {
        let meta = dispatch_key.meta();
        let entry = builtin_registry().lookup_by_id(meta.function_id);
        let surface_name = entry
            .map(|entry| entry.surface_name.clone())
            .unwrap_or_else(|| canonical_surface_name_from_id(meta.function_id).to_string());
        let canonical_surface_name = entry
            .and_then(|entry| entry.registry_metadata.canonical_surface_name.clone())
            .or_else(|| Some(surface_name.clone()));
        Self {
            dispatch_key,
            meta,
            surface_name,
            canonical_surface_name,
            callable_argument_specs: callable_argument_specs_for_id(meta.function_id),
        }
    }

    pub fn function_id(&self) -> &'static str {
        self.dispatch_key.function_id()
    }

    pub fn dispatch_key(&self) -> SurfaceDispatchKey {
        self.dispatch_key
    }

    pub fn catalog_index(&self) -> usize {
        self.dispatch_key.catalog_index()
    }

    pub fn is_invokable_by_value_path(&self) -> bool {
        true
    }

    pub fn surface_name(&self) -> &str {
        &self.surface_name
    }

    pub fn canonical_surface_name(&self) -> Option<&str> {
        self.canonical_surface_name.as_deref()
    }

    pub fn function_meta(&self) -> FunctionMeta {
        self.meta
    }

    pub fn arity(&self) -> Arity {
        self.meta.arity
    }

    pub fn arg_preparation_profile(&self) -> ArgPreparationProfile {
        self.meta.arg_preparation_profile
    }

    pub fn volatility(&self) -> VolatilityClass {
        self.meta.volatility
    }

    pub fn determinism(&self) -> DeterminismClass {
        self.meta.determinism
    }

    pub fn host_interaction(&self) -> HostInteractionClass {
        self.meta.host_interaction
    }

    pub fn thread_safety(&self) -> ThreadSafetyClass {
        self.meta.thread_safety
    }

    pub fn coercion_lift_profile(&self) -> CoercionLiftProfile {
        self.meta.coercion_lift_profile
    }

    pub fn kernel_signature_class(&self) -> KernelSignatureClass {
        self.meta.kernel_signature_class
    }

    pub fn fec_dependency_profile(&self) -> FecDependencyProfile {
        self.meta.fec_dependency_profile
    }

    pub fn surface_fec_dependency_profile(&self) -> FecDependencyProfile {
        self.meta.surface_fec_dependency_profile
    }

    pub fn callable_argument_specs(&self) -> &'static [CallableArgumentSpec] {
        self.callable_argument_specs
    }

    pub fn callable_argument_ordinals_for_arity(&self, argc: usize) -> Vec<usize> {
        let mut ordinals = Vec::new();
        for spec in self.callable_argument_specs {
            match *spec {
                CallableArgumentSpec::Fixed(index) if index < argc => ordinals.push(index),
                CallableArgumentSpec::Fixed(_) => {}
                CallableArgumentSpec::Last if argc > 0 => ordinals.push(argc - 1),
                CallableArgumentSpec::Last => {}
            }
        }
        ordinals
    }

    pub fn new_scratch(&self) -> SurfaceCallScratch {
        let capacity = if self.meta.arity.max == usize::MAX {
            self.meta.arity.min
        } else {
            self.meta.arity.max.min(8).max(self.meta.arity.min)
        };
        SurfaceCallScratch::with_capacity(capacity)
    }

    pub fn is_context_free_pure(&self) -> bool {
        self.is_hoistable_under(SurfaceCallHoistPolicy::STRICT_CONTEXT_FREE)
    }

    pub fn is_hoistable_under(&self, policy: SurfaceCallHoistPolicy) -> bool {
        determinism_allows_hoist(self.meta.determinism, policy)
            && volatility_allows_hoist(self.meta.volatility, policy)
            && host_interaction_allows_hoist(self.meta.host_interaction, policy)
            && adapter_fec_dependency_allows_hoist(self.meta.fec_dependency_profile, policy)
            && surface_fec_dependency_allows_hoist(
                self.meta.fec_dependency_profile,
                self.meta.surface_fec_dependency_profile,
                policy,
            )
    }

    pub fn invoke<R: ReferenceResolver>(
        &self,
        args: &[CallArgValue],
        runtime: &mut SurfaceCallRuntime<'_, R>,
    ) -> Result<EvalValue, WorksheetErrorCode> {
        eval_surface_value_call_with_dispatch_key(
            self.dispatch_key,
            args,
            runtime.resolver,
            runtime.effective_now_serial(),
            runtime.effective_random_value(),
            runtime.locale_ctx,
            runtime.host_info,
            runtime.callable_invoker,
            runtime.rtd_provider,
            runtime.registered_external_provider,
        )
    }

    pub fn invoke_scratch<R: ReferenceResolver>(
        &self,
        scratch: &SurfaceCallScratch,
        runtime: &mut SurfaceCallRuntime<'_, R>,
    ) -> Result<EvalValue, WorksheetErrorCode> {
        self.invoke(scratch.call_args(), runtime)
    }

    pub fn invoke_with_scratch_builder<R, F>(
        &self,
        scratch: &mut SurfaceCallScratch,
        runtime: &mut SurfaceCallRuntime<'_, R>,
        build_args: F,
    ) -> Result<EvalValue, WorksheetErrorCode>
    where
        R: ReferenceResolver,
        F: FnOnce(&mut Vec<CallArgValue>),
    {
        scratch.clear();
        build_args(scratch.call_args_mut());
        self.invoke_scratch(scratch, runtime)
    }
}

pub struct SurfaceCallRuntime<'a, R: ReferenceResolver> {
    pub resolver: &'a R,
    pub now_serial: Option<f64>,
    pub now_provider: Option<&'a dyn NowProvider>,
    pub random_value: Option<f64>,
    pub random_provider: Option<&'a dyn RandomProvider>,
    pub locale_ctx: Option<&'a LocaleFormatContext<'a>>,
    pub host_info: Option<&'a dyn HostInfoProvider>,
    pub callable_invoker: Option<&'a dyn CallableInvoker>,
    pub rtd_provider: Option<&'a dyn RtdProvider>,
    pub registered_external_provider: Option<&'a dyn RegisteredExternalProvider>,
}

impl<'a, R: ReferenceResolver> SurfaceCallRuntime<'a, R> {
    pub fn new(resolver: &'a R) -> Self {
        Self {
            resolver,
            now_serial: None,
            now_provider: None,
            random_value: None,
            random_provider: None,
            locale_ctx: None,
            host_info: None,
            callable_invoker: None,
            rtd_provider: None,
            registered_external_provider: None,
        }
    }

    pub fn with_now_serial(mut self, now_serial: f64) -> Self {
        self.now_serial = Some(now_serial);
        self.now_provider = None;
        self
    }

    pub fn with_now_provider(mut self, now_provider: &'a dyn NowProvider) -> Self {
        self.now_serial = None;
        self.now_provider = Some(now_provider);
        self
    }

    pub fn with_random_value(mut self, random_value: f64) -> Self {
        self.random_value = Some(random_value);
        self.random_provider = None;
        self
    }

    pub fn with_random_provider(mut self, random_provider: &'a dyn RandomProvider) -> Self {
        self.random_value = None;
        self.random_provider = Some(random_provider);
        self
    }

    pub fn with_locale_context(mut self, locale_ctx: &'a LocaleFormatContext) -> Self {
        self.locale_ctx = Some(locale_ctx);
        self
    }

    pub fn with_host_info(mut self, host_info: &'a dyn HostInfoProvider) -> Self {
        self.host_info = Some(host_info);
        self
    }

    pub fn with_callable_invoker(mut self, callable_invoker: &'a dyn CallableInvoker) -> Self {
        self.callable_invoker = Some(callable_invoker);
        self
    }

    pub fn with_rtd_provider(mut self, rtd_provider: &'a dyn RtdProvider) -> Self {
        self.rtd_provider = Some(rtd_provider);
        self
    }

    pub fn with_registered_external_provider(
        mut self,
        registered_external_provider: &'a dyn RegisteredExternalProvider,
    ) -> Self {
        self.registered_external_provider = Some(registered_external_provider);
        self
    }

    pub fn effective_now_serial(&self) -> Option<f64> {
        self.now_serial
            .or_else(|| self.now_provider.map(|provider| provider.now_serial()))
    }

    pub fn effective_random_value(&self) -> Option<f64> {
        self.random_value
            .or_else(|| self.random_provider.map(|provider| provider.random_unit()))
    }
}

fn canonical_surface_name_from_id(function_id: &str) -> &str {
    function_id.strip_prefix("FUNC.").unwrap_or(function_id)
}

fn callable_argument_specs_for_id(function_id: &str) -> &'static [CallableArgumentSpec] {
    match function_id {
        "FUNC.BYCOL" | "FUNC.BYROW" => &CALLABLE_FIXED_1,
        "FUNC.GROUPBY" | "FUNC.MAKEARRAY" | "FUNC.REDUCE" | "FUNC.SCAN" => &CALLABLE_FIXED_2,
        "FUNC.PIVOTBY" => &CALLABLE_FIXED_3,
        "FUNC.MAP" => &CALLABLE_LAST,
        _ => &CALLABLE_NONE,
    }
}

fn determinism_allows_hoist(determinism: DeterminismClass, policy: SurfaceCallHoistPolicy) -> bool {
    match determinism {
        DeterminismClass::Deterministic => true,
        DeterminismClass::PseudoRandom => policy.allow_fixed_random,
        DeterminismClass::TimeDependent => policy.allow_fixed_time,
        DeterminismClass::ExternalEventDependent => policy.allow_fixed_external,
    }
}

fn volatility_allows_hoist(volatility: VolatilityClass, policy: SurfaceCallHoistPolicy) -> bool {
    match volatility {
        VolatilityClass::NonVolatile => true,
        VolatilityClass::VolatileContextual => {
            policy.allow_fixed_caller_context
                || policy.allow_fixed_host_state
                || policy.allow_fixed_locale
                || policy.allow_ref_only_dependency
        }
        VolatilityClass::VolatileFull => {
            policy.allow_fixed_time || policy.allow_fixed_random || policy.allow_fixed_external
        }
    }
}

fn host_interaction_allows_hoist(
    host_interaction: HostInteractionClass,
    policy: SurfaceCallHoistPolicy,
) -> bool {
    match host_interaction {
        HostInteractionClass::None => true,
        HostInteractionClass::WorkbookState
        | HostInteractionClass::ApplicationState
        | HostInteractionClass::EnvironmentState => policy.allow_fixed_host_state,
        HostInteractionClass::ExternalProvider => policy.allow_fixed_external,
    }
}

fn adapter_fec_dependency_allows_hoist(
    dependency: FecDependencyProfile,
    policy: SurfaceCallHoistPolicy,
) -> bool {
    match dependency {
        FecDependencyProfile::None => true,
        FecDependencyProfile::RefOnly => policy.allow_ref_only_dependency,
        FecDependencyProfile::CallerContext => policy.allow_fixed_caller_context,
        FecDependencyProfile::TimeProvider => policy.allow_fixed_time,
        FecDependencyProfile::RandomProvider => policy.allow_fixed_random,
        FecDependencyProfile::ExternalProvider => policy.allow_fixed_external,
        FecDependencyProfile::LocaleProfile => policy.allow_fixed_locale,
        FecDependencyProfile::Composite => false,
    }
}

fn surface_fec_dependency_allows_hoist(
    adapter_dependency: FecDependencyProfile,
    surface_dependency: FecDependencyProfile,
    policy: SurfaceCallHoistPolicy,
) -> bool {
    match surface_dependency {
        FecDependencyProfile::Composite => {
            adapter_fec_dependency_allows_hoist(adapter_dependency, policy)
                && policy.allow_ref_only_dependency
        }
        other => adapter_fec_dependency_allows_hoist(other, policy),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::{ArgPreparationProfile, HostInteractionClass, VolatilityClass};
    use crate::functions::adapters::PreparedArgValue;
    use crate::functions::call_register_id_family::{
        RegisterIdRequest, RegisteredExternalDescriptor, RegisteredExternalOriginKind,
        RegisteredExternalProviderError, RegisteredProcedureSpec,
    };
    use crate::functions::callable_helpers::{CallableInvocationError, CallableInvoker};
    use crate::functions::rtd_fn::{RtdProvider, RtdProviderResult, RtdRequest};
    use crate::functions::surface_dispatch::{
        FUNC_ID_BYROW, FUNC_ID_CELL, FUNC_ID_GROUPBY, FUNC_ID_HSTACK, FUNC_ID_INDEX, FUNC_ID_MAP,
        FUNC_ID_NOW, FUNC_ID_OP_ADD, FUNC_ID_PI, FUNC_ID_PIVOTBY, FUNC_ID_RAND, FUNC_ID_REDUCE,
        FUNC_ID_REGISTER_ID, FUNC_ID_RTD, FUNC_ID_VALUE, FUNC_ID_VSTACK,
        eval_surface_value_call_with_callable,
    };
    use crate::host_info::{CellInfoQuery, HostInfoError, HostInfoProvider};
    use crate::locale_format::test_en_us_context;
    use crate::resolver::{RefResolutionError, ResolverCapabilities};
    use crate::value::{
        ArrayCellValue, CallableArityShape, CallableCaptureMode, EvalArray, ExcelText, LambdaValue,
        ReferenceKind, ReferenceLike,
    };

    struct NoReferenceResolver;
    struct TestCallableInvoker;
    struct TestHostInfoProvider;
    struct TestRtdProvider;
    struct TestRegisteredExternalProvider;

    impl ReferenceResolver for NoReferenceResolver {
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

    impl CallableInvoker for TestCallableInvoker {
        fn invoke(
            &self,
            callable: &LambdaValue,
            args: &[PreparedArgValue],
        ) -> Result<PreparedArgValue, CallableInvocationError> {
            match callable.callable_token.as_str() {
                "helper.add1" => match args.first() {
                    Some(PreparedArgValue::Eval(EvalValue::Number(n))) => {
                        Ok(PreparedArgValue::Eval(EvalValue::Number(n + 1.0)))
                    }
                    _ => Err(CallableInvocationError::Worksheet(
                        WorksheetErrorCode::Value,
                    )),
                },
                other => Err(CallableInvocationError::UnsupportedCallableToken(
                    other.to_string(),
                )),
            }
        }
    }

    impl HostInfoProvider for TestHostInfoProvider {
        fn query_cell_info(
            &self,
            query: CellInfoQuery,
            _reference: Option<&ReferenceLike>,
        ) -> Result<EvalValue, HostInfoError> {
            match query {
                CellInfoQuery::Filename => Ok(EvalValue::Text(text("Book1.xlsx"))),
                _ => Err(HostInfoError::UnsupportedCellInfoQuery(query)),
            }
        }
    }

    fn text(value: &str) -> ExcelText {
        ExcelText::from_utf16_code_units(value.encode_utf16().collect())
    }

    impl RtdProvider for TestRtdProvider {
        fn resolve_rtd(&self, _request: &RtdRequest) -> RtdProviderResult {
            RtdProviderResult::Value(EvalValue::Number(42.0))
        }
    }

    impl TestRegisteredExternalProvider {
        fn descriptor() -> RegisteredExternalDescriptor {
            RegisteredExternalDescriptor {
                stable_registration_id: "surface-call-test".to_string(),
                register_id: 77.0,
                origin_kind: RegisteredExternalOriginKind::WorksheetRegisterId,
                display_name: Some(text("GetTickCount")),
                library_name: text("Kernel32"),
                procedure: RegisteredProcedureSpec::Name(text("GetTickCount")),
                declared_type_text: Some(text("J!")),
            }
        }
    }

    impl RegisteredExternalProvider for TestRegisteredExternalProvider {
        fn resolve_register_id(
            &self,
            _request: &RegisterIdRequest,
        ) -> Result<RegisteredExternalDescriptor, RegisteredExternalProviderError> {
            Ok(Self::descriptor())
        }

        fn lookup_registered_external(
            &self,
            _register_id: f64,
        ) -> Result<RegisteredExternalDescriptor, RegisteredExternalProviderError> {
            Ok(Self::descriptor())
        }
    }

    fn text_arg(value: &str) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Text(text(value)))
    }

    fn num_arg(value: f64) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Number(value))
    }

    fn reference_arg(target: &str) -> CallArgValue {
        CallArgValue::Reference(ReferenceLike {
            kind: ReferenceKind::A1,
            target: target.to_string(),
        })
    }

    fn lambda_arg(token: &str, arity: usize) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Lambda(LambdaValue::helper_lambda(
            token.to_string(),
            CallableArityShape::exact(arity),
            CallableCaptureMode::NoCapture,
            "surface.call.test",
        )))
    }

    fn array_arg(rows: Vec<Vec<ArrayCellValue>>) -> CallArgValue {
        CallArgValue::Eval(EvalValue::Array(EvalArray::from_rows(rows).unwrap()))
    }

    fn assert_call_site_parity(
        function_id: &str,
        args: &[CallArgValue],
        callable_invoker: Option<&dyn CallableInvoker>,
        host_info: Option<&dyn HostInfoProvider>,
    ) {
        let resolver = NoReferenceResolver;
        let call_site = SurfaceCallSite::from_function_id(function_id).unwrap();
        let mut runtime = SurfaceCallRuntime::new(&resolver)
            .with_now_serial(46000.0)
            .with_random_value(0.5);
        runtime.callable_invoker = callable_invoker;
        runtime.host_info = host_info;

        let got = call_site.invoke(args, &mut runtime);
        let expected = eval_surface_value_call_with_callable(
            function_id,
            args,
            &resolver,
            Some(46000.0),
            Some(0.5),
            None,
            host_info,
            callable_invoker,
            None,
            None,
        );
        assert_eq!(got, expected);
    }

    fn assert_call_site_parity_with_providers(
        function_id: &str,
        args: &[CallArgValue],
        locale_ctx: Option<&LocaleFormatContext<'_>>,
        rtd_provider: Option<&dyn RtdProvider>,
        registered_external_provider: Option<&dyn RegisteredExternalProvider>,
    ) {
        let resolver = NoReferenceResolver;
        let call_site = SurfaceCallSite::from_function_id(function_id).unwrap();
        let mut runtime = SurfaceCallRuntime::new(&resolver)
            .with_now_serial(46000.0)
            .with_random_value(0.5);
        runtime.locale_ctx = locale_ctx;
        runtime.rtd_provider = rtd_provider;
        runtime.registered_external_provider = registered_external_provider;

        let got = call_site.invoke(args, &mut runtime);
        let expected = eval_surface_value_call_with_callable(
            function_id,
            args,
            &resolver,
            Some(46000.0),
            Some(0.5),
            locale_ctx,
            None,
            None,
            rtd_provider,
            registered_external_provider,
        );
        assert_eq!(got, expected);
    }

    #[test]
    fn surface_call_site_resolves_from_id_and_surface_name() {
        let by_id = SurfaceCallSite::from_function_id(FUNC_ID_HSTACK).unwrap();
        assert_eq!(by_id.function_id(), FUNC_ID_HSTACK);
        assert!(by_id.is_invokable_by_value_path());
        assert_eq!(by_id.surface_name(), "HSTACK");
        assert_eq!(by_id.canonical_surface_name(), Some("HSTACK"));
        assert_eq!(
            by_id.arg_preparation_profile(),
            ArgPreparationProfile::ValuesOnlyPreAdapter
        );

        let by_name = SurfaceCallSite::from_surface_name("hstack").unwrap();
        assert_eq!(by_name.function_id(), FUNC_ID_HSTACK);
        assert_eq!(by_name.function_meta(), by_id.function_meta());

        let from_registry_id =
            SurfaceCallSite::from_registry_function_id(builtin_registry(), FUNC_ID_HSTACK).unwrap();
        assert_eq!(from_registry_id.function_id(), FUNC_ID_HSTACK);

        let from_registry_name =
            SurfaceCallSite::from_registry_surface_name(builtin_registry(), "hstack").unwrap();
        assert_eq!(from_registry_name.function_id(), FUNC_ID_HSTACK);
    }

    #[test]
    fn every_builtin_registry_entry_resolves_to_uniform_dispatch_key() {
        for entry in builtin_registry().iter() {
            let call_site = SurfaceCallSite::from_registry_entry(entry)
                .expect("built-in registry entry must resolve to a surface dispatch key");
            assert_eq!(call_site.function_id(), entry.meta.function_id.as_str());
            assert_eq!(
                call_site.function_meta().function_id,
                entry.meta.function_id.as_str()
            );
            assert_eq!(call_site.surface_name(), entry.surface_name.as_str());
        }
    }

    #[test]
    fn surface_call_site_exposes_stable_planning_metadata() {
        let now = SurfaceCallSite::from_function_id(FUNC_ID_NOW).unwrap();
        assert_eq!(now.volatility(), VolatilityClass::VolatileFull);

        let cell = SurfaceCallSite::from_function_id(FUNC_ID_CELL).unwrap();
        assert_eq!(cell.host_interaction(), HostInteractionClass::WorkbookState);
        assert_eq!(
            cell.arg_preparation_profile(),
            ArgPreparationProfile::RefsVisibleInAdapter
        );

        let reduce = SurfaceCallSite::from_function_id(FUNC_ID_REDUCE).unwrap();
        assert_eq!(
            reduce.callable_argument_specs(),
            &[CallableArgumentSpec::Fixed(2)]
        );
        assert_eq!(reduce.callable_argument_ordinals_for_arity(3), vec![2]);

        let byrow = SurfaceCallSite::from_function_id(FUNC_ID_BYROW).unwrap();
        assert_eq!(byrow.callable_argument_ordinals_for_arity(2), vec![1]);

        let map = SurfaceCallSite::from_function_id(FUNC_ID_MAP).unwrap();
        assert_eq!(map.callable_argument_specs(), &[CallableArgumentSpec::Last]);
        assert_eq!(map.callable_argument_ordinals_for_arity(4), vec![3]);

        let groupby = SurfaceCallSite::from_function_id(FUNC_ID_GROUPBY).unwrap();
        assert_eq!(groupby.callable_argument_ordinals_for_arity(8), vec![2]);

        let pivotby = SurfaceCallSite::from_function_id(FUNC_ID_PIVOTBY).unwrap();
        assert_eq!(pivotby.callable_argument_ordinals_for_arity(9), vec![3]);
    }

    #[test]
    fn surface_call_site_invocation_matches_dispatcher_for_representative_functions() {
        assert_call_site_parity(
            FUNC_ID_OP_ADD,
            &[
                array_arg(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                ]]),
                num_arg(10.0),
            ],
            None,
            None,
        );

        assert_call_site_parity(
            FUNC_ID_INDEX,
            &[
                array_arg(vec![vec![
                    ArrayCellValue::Number(7.0),
                    ArrayCellValue::Number(11.0),
                ]]),
                num_arg(1.0),
                num_arg(2.0),
            ],
            None,
            None,
        );

        assert_call_site_parity(
            FUNC_ID_HSTACK,
            &[num_arg(1.0), num_arg(2.0), num_arg(3.0)],
            None,
            None,
        );

        assert_call_site_parity(
            FUNC_ID_VSTACK,
            &[num_arg(1.0), num_arg(2.0), num_arg(3.0)],
            None,
            None,
        );

        assert_call_site_parity(FUNC_ID_NOW, &[], None, None);
        assert_call_site_parity(FUNC_ID_RAND, &[], None, None);

        assert_call_site_parity(
            FUNC_ID_CELL,
            &[text_arg("address"), reference_arg("B2")],
            None,
            None,
        );
    }

    #[test]
    fn surface_call_site_invocation_matches_dispatcher_for_host_and_helper_calls() {
        let host_info = TestHostInfoProvider;
        assert_call_site_parity(
            FUNC_ID_CELL,
            &[text_arg("filename")],
            None,
            Some(&host_info),
        );

        let callable_invoker = TestCallableInvoker;
        assert_call_site_parity(
            FUNC_ID_MAP,
            &[
                array_arg(vec![vec![
                    ArrayCellValue::Number(1.0),
                    ArrayCellValue::Number(2.0),
                ]]),
                lambda_arg("helper.add1", 1),
            ],
            Some(&callable_invoker),
            None,
        );
    }

    #[test]
    fn surface_call_site_invocation_matches_dispatcher_for_provider_and_locale_cases() {
        let locale = test_en_us_context();
        assert_call_site_parity_with_providers(
            FUNC_ID_VALUE,
            &[text_arg("123.5")],
            Some(&locale),
            None,
            None,
        );

        let rtd_provider = TestRtdProvider;
        assert_call_site_parity_with_providers(
            FUNC_ID_RTD,
            &[text_arg("prog.id"), text_arg("server"), text_arg("topic")],
            None,
            Some(&rtd_provider),
            None,
        );

        let registered_external_provider = TestRegisteredExternalProvider;
        assert_call_site_parity_with_providers(
            FUNC_ID_REGISTER_ID,
            &[
                text_arg("Kernel32"),
                text_arg("GetTickCount"),
                text_arg("J!"),
            ],
            None,
            None,
            Some(&registered_external_provider),
        );
    }

    #[test]
    fn surface_call_scratch_reuses_argument_storage_for_repeated_invocation() {
        let resolver = NoReferenceResolver;
        let call_site = SurfaceCallSite::from_function_id(FUNC_ID_HSTACK).unwrap();
        let mut runtime = SurfaceCallRuntime::new(&resolver);
        let mut scratch = call_site.new_scratch();
        let initial_capacity = scratch.capacity();

        let got = call_site
            .invoke_with_scratch_builder(&mut scratch, &mut runtime, |args| {
                args.push(num_arg(1.0));
                args.push(num_arg(2.0));
                args.push(num_arg(3.0));
            })
            .unwrap();
        assert!(matches!(got, EvalValue::Array(_)));
        assert_eq!(scratch.capacity(), initial_capacity);

        let got = call_site
            .invoke_with_scratch_builder(&mut scratch, &mut runtime, |args| {
                args.push(num_arg(4.0));
                args.push(num_arg(5.0));
                args.push(num_arg(6.0));
            })
            .unwrap();
        assert!(matches!(got, EvalValue::Array(_)));
        assert_eq!(scratch.capacity(), initial_capacity);
    }

    #[test]
    fn surface_call_site_exposes_generic_hoistability_gate() {
        let pi = SurfaceCallSite::from_function_id(FUNC_ID_PI).unwrap();
        assert!(pi.is_context_free_pure());

        let now = SurfaceCallSite::from_function_id(FUNC_ID_NOW).unwrap();
        assert!(!now.is_context_free_pure());
        assert!(now.is_hoistable_under(SurfaceCallHoistPolicy::FIXED_RUNTIME_CONTEXT));

        let value = SurfaceCallSite::from_function_id(FUNC_ID_VALUE).unwrap();
        assert!(!value.is_context_free_pure());
        assert!(!value.is_hoistable_under(SurfaceCallHoistPolicy {
            allow_fixed_locale: true,
            ..SurfaceCallHoistPolicy::STRICT_CONTEXT_FREE
        }));
        assert!(value.is_hoistable_under(SurfaceCallHoistPolicy {
            allow_ref_only_dependency: true,
            allow_fixed_locale: true,
            ..SurfaceCallHoistPolicy::STRICT_CONTEXT_FREE
        }));
    }
}
