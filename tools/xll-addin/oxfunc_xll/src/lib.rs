#![allow(non_snake_case)]

use std::ffi::c_void;

use oxfunc_core::coercion::CoercionError;
use oxfunc_core::functions::abs::{AbsEvalError, abs_kernel};
use oxfunc_core::functions::abs_surface::eval_abs_scalar_value;
use oxfunc_core::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
use oxfunc_core::value::{CallArgValue, EvalValue, ExcelText, ReferenceLike, Value, WorksheetErrorCode};
use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    GetModuleFileNameW, GetModuleHandleExW,
};

const XLERR_NULL: i32 = 0;
const XLERR_DIV0: i32 = 7;
const XLERR_VALUE: i32 = 15;
const XLERR_REF: i32 = 23;
const XLERR_NAME: i32 = 29;
const XLERR_NUM: i32 = 36;
const XLERR_NA: i32 = 42;
const XLERR_GETTING_DATA: i32 = 43;

const ARG_TAG_MISSING: i32 = 0;
const ARG_TAG_EMPTY: i32 = 1;
const ARG_TAG_NUMBER: i32 = 2;
const ARG_TAG_TEXT: i32 = 3;
const ARG_TAG_LOGICAL: i32 = 4;
const ARG_TAG_ERROR: i32 = 5;

const RESULT_TAG_NUMBER: i32 = 1;
const RESULT_TAG_ERROR: i32 = 2;

#[repr(C)]
pub struct OxFuncShimArg {
    pub tag: i32,
    pub number: f64,
    pub logical: i32,
    pub text_ptr: *const u16,
    pub text_len: u32,
    pub error_code: i32,
}

#[repr(C)]
pub struct OxFuncShimResult {
    pub tag: i32,
    pub number: f64,
    pub error_code: i32,
}

unsafe extern "system" {
    fn oxfunc_register_all(module_path: *const u16) -> i32;
}

struct NoReferenceResolver;

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

fn map_excel_err_to_ws(code: i32) -> WorksheetErrorCode {
    match code {
        XLERR_NULL => WorksheetErrorCode::Null,
        XLERR_DIV0 => WorksheetErrorCode::Div0,
        XLERR_VALUE => WorksheetErrorCode::Value,
        XLERR_REF => WorksheetErrorCode::Ref,
        XLERR_NAME => WorksheetErrorCode::Name,
        XLERR_NUM => WorksheetErrorCode::Num,
        XLERR_NA => WorksheetErrorCode::NA,
        XLERR_GETTING_DATA => WorksheetErrorCode::GettingData,
        _ => WorksheetErrorCode::Value,
    }
}

fn map_ws_err_to_excel(code: WorksheetErrorCode) -> i32 {
    match code {
        WorksheetErrorCode::Null => XLERR_NULL,
        WorksheetErrorCode::Div0 => XLERR_DIV0,
        WorksheetErrorCode::Value => XLERR_VALUE,
        WorksheetErrorCode::Ref => XLERR_REF,
        WorksheetErrorCode::Name => XLERR_NAME,
        WorksheetErrorCode::Num => XLERR_NUM,
        WorksheetErrorCode::NA => XLERR_NA,
        WorksheetErrorCode::GettingData => XLERR_GETTING_DATA,
        WorksheetErrorCode::Spill => XLERR_VALUE,
        WorksheetErrorCode::Calc => XLERR_VALUE,
        WorksheetErrorCode::Field => XLERR_VALUE,
        WorksheetErrorCode::Blocked => XLERR_VALUE,
        WorksheetErrorCode::Connect => XLERR_VALUE,
    }
}

fn map_ref_resolution_to_ws(e: &RefResolutionError) -> WorksheetErrorCode {
    match e {
        RefResolutionError::CapabilityDenied { .. } => WorksheetErrorCode::Ref,
        RefResolutionError::UnresolvedReference { .. } => WorksheetErrorCode::Ref,
        RefResolutionError::EvalTimeDerefNotAllowed => WorksheetErrorCode::Ref,
        RefResolutionError::ProviderFailure { .. } => WorksheetErrorCode::Value,
    }
}

fn map_coercion_to_ws(e: &CoercionError) -> WorksheetErrorCode {
    match e {
        CoercionError::WorksheetError(code) => *code,
        CoercionError::RefResolution(err) => map_ref_resolution_to_ws(err),
        CoercionError::MissingArg => WorksheetErrorCode::Value,
        CoercionError::EmptyCell => WorksheetErrorCode::Value,
        CoercionError::NonNumericText(_) => WorksheetErrorCode::Value,
        CoercionError::UnsupportedValueKind(_) => WorksheetErrorCode::Value,
    }
}

fn map_abs_error_to_ws(e: &AbsEvalError) -> WorksheetErrorCode {
    match e {
        AbsEvalError::ArityMismatch { .. } => WorksheetErrorCode::Value,
        AbsEvalError::Coercion(err) => map_coercion_to_ws(err),
    }
}

unsafe fn call_arg_from_shim(arg: &OxFuncShimArg) -> CallArgValue {
    match arg.tag {
        ARG_TAG_MISSING => CallArgValue::MissingArg,
        ARG_TAG_EMPTY => CallArgValue::EmptyCell,
        ARG_TAG_NUMBER => CallArgValue::Eval(EvalValue::Number(arg.number)),
        ARG_TAG_LOGICAL => CallArgValue::Eval(EvalValue::Logical(arg.logical != 0)),
        ARG_TAG_TEXT => {
            let len = usize::try_from(arg.text_len).unwrap_or(0);
            let units = if arg.text_ptr.is_null() || len == 0 {
                Vec::new()
            } else {
                // SAFETY: Caller guarantees pointer points to at least text_len UTF-16 units.
                unsafe { std::slice::from_raw_parts(arg.text_ptr, len) }.to_vec()
            };
            CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(units)))
        }
        ARG_TAG_ERROR => CallArgValue::Eval(EvalValue::Error(map_excel_err_to_ws(arg.error_code))),
        _ => CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Value)),
    }
}

fn result_number(n: f64) -> OxFuncShimResult {
    OxFuncShimResult {
        tag: RESULT_TAG_NUMBER,
        number: n,
        error_code: 0,
    }
}

fn result_error(code: WorksheetErrorCode) -> OxFuncShimResult {
    OxFuncShimResult {
        tag: RESULT_TAG_ERROR,
        number: 0.0,
        error_code: map_ws_err_to_excel(code),
    }
}

fn current_module_path_wide_nul() -> Option<Vec<u16>> {
    let mut module: HMODULE = std::ptr::null_mut();
    // SAFETY: FROM_ADDRESS means this parameter is treated as an address inside the module.
    let ok = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            xlAutoOpen as *const c_void as *const u16,
            &mut module,
        )
    };
    if ok == 0 || module.is_null() {
        return None;
    }

    let mut buffer = vec![0u16; 32768];
    // SAFETY: `buffer` is valid and writable for the specified length.
    let len = unsafe { GetModuleFileNameW(module, buffer.as_mut_ptr(), buffer.len() as u32) };
    if len == 0 {
        return None;
    }
    let len_usize = usize::try_from(len).ok()?;
    let mut out = buffer[..len_usize].to_vec();
    out.push(0);
    Some(out)
}

#[unsafe(no_mangle)]
pub extern "system" fn oxfunc_abs_eval_shim(
    arg: *const OxFuncShimArg,
    out: *mut OxFuncShimResult,
) -> i32 {
    if arg.is_null() || out.is_null() {
        return 0;
    }

    // SAFETY: Pointers validated above and owned by caller for call duration.
    let call_arg = unsafe { call_arg_from_shim(&*arg) };
    let resolver = NoReferenceResolver;
    let args = [call_arg];
    let result = match eval_abs_scalar_value(&args, &resolver) {
        Ok(EvalValue::Number(n)) => result_number(n),
        Ok(EvalValue::Error(code)) => result_error(code),
        Ok(_) => result_error(WorksheetErrorCode::Value),
        Err(e) => result_error(map_abs_error_to_ws(&e)),
    };

    // SAFETY: Output pointer validated above.
    unsafe {
        *out = result;
    }
    1
}

#[unsafe(no_mangle)]
pub extern "system" fn OX_ABS_Q(value: f64) -> f64 {
    abs_kernel(value)
}

#[unsafe(no_mangle)]
pub extern "system" fn OX_PI() -> f64 {
    match oxfunc_core::functions::pi::eval_pi(&[]) {
        Ok(Value::Number(n)) => n,
        _ => std::f64::consts::PI,
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn xlAutoOpen() -> i32 {
    let module_path = match current_module_path_wide_nul() {
        Some(p) => p,
        None => return 0,
    };
    // SAFETY: C++ bridge is compiled into this binary and expects a NUL-terminated UTF-16 path.
    let ok = unsafe { oxfunc_register_all(module_path.as_ptr()) };
    if ok == 1 { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "system" fn xlAutoClose() -> i32 {
    1
}
