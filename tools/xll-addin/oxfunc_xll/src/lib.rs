#![allow(non_snake_case)]

use std::ffi::c_void;
use std::sync::OnceLock;

use oxfunc_core::functions::surface_dispatch::{
    eval_surface_q_nullary_number, eval_surface_q_unary_number, eval_surface_unary_scalar_value,
};
use oxfunc_core::resolver::{RefResolutionError, ReferenceResolver, ResolverCapabilities};
use oxfunc_core::value::{CallArgValue, EvalValue, ExcelText, ReferenceLike, WorksheetErrorCode};
use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    GetModuleFileNameW, GetModuleHandleExW, GetModuleHandleW, GetProcAddress,
};

type Rw = i32;
type Col = i32;
type IdSheet = usize;

const MD_CALLBACK_12: &[u8] = b"MdCallBack12\0";

const XLTYPE_NUM: u32 = 0x0001;
const XLTYPE_STR: u32 = 0x0002;
const XLTYPE_BOOL: u32 = 0x0004;
const XLTYPE_REF: u32 = 0x0008;
const XLTYPE_ERR: u32 = 0x0010;
const XLTYPE_MULTI: u32 = 0x0040;
const XLTYPE_MISSING: u32 = 0x0080;
const XLTYPE_NIL: u32 = 0x0100;
const XLTYPE_SREF: u32 = 0x0400;
const XLTYPE_INT: u32 = 0x0800;

const XLBIT_DLLFREE: u32 = 0x4000;
const XLTYPE_MASK: u32 = 0x0FFF;

const XLERR_NULL: i32 = 0;
const XLERR_DIV0: i32 = 7;
const XLERR_VALUE: i32 = 15;
const XLERR_REF: i32 = 23;
const XLERR_NAME: i32 = 29;
const XLERR_NUM: i32 = 36;
const XLERR_NA: i32 = 42;
const XLERR_GETTING_DATA: i32 = 43;

const XLRET_SUCCESS: i32 = 0;

const XL_SPECIAL: i32 = 0x4000;
const XL_FREE: i32 = XL_SPECIAL;
const XL_COERCE: i32 = 2 | XL_SPECIAL;
const XLF_REGISTER: i32 = 149;

type Excel12Proc = unsafe extern "system" fn(
    xlfn: i32,
    coper: i32,
    rgpxloper12: *mut *mut XLOPER12,
    xloper12_res: *mut XLOPER12,
) -> i32;

static EXCEL12_PROC: OnceLock<Option<Excel12Proc>> = OnceLock::new();

#[repr(C)]
#[derive(Clone, Copy)]
pub struct XLRef12 {
    rw_first: Rw,
    rw_last: Rw,
    col_first: Col,
    col_last: Col,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct XLMRef12 {
    count: u16,
    reftbl: [XLRef12; 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct XlSRef12 {
    count: u16,
    r#ref: XLRef12,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct XlMRef12 {
    lpmref: *mut XLMRef12,
    id_sheet: IdSheet,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct XlArray12 {
    lparray: *mut XLOPER12,
    rows: Rw,
    columns: Col,
}

#[repr(C)]
#[derive(Clone, Copy)]
union XlFlowVal {
    level: i32,
    tbctrl: i32,
    id_sheet: IdSheet,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct XlFlow {
    valflow: XlFlowVal,
    rw: Rw,
    col: Col,
    xlflow: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
union XlBigDataHandle {
    lpb_data: *mut u8,
    hdata: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct XlBigData {
    h: XlBigDataHandle,
    cb_data: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union XLOPER12Value {
    num: f64,
    str: *mut u16,
    xbool: i32,
    err: i32,
    w: i32,
    sref: XlSRef12,
    mref: XlMRef12,
    array: XlArray12,
    flow: XlFlow,
    bigdata: XlBigData,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct XLOPER12 {
    pub val: XLOPER12Value,
    pub xltype: u32,
}

#[derive(Clone, Copy)]
struct RegistrationSpec {
    export_name: &'static str,
    type_text: &'static str,
    function_name: &'static str,
    arg_names: &'static str,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum ULiftPolicy {
    ScalarOnly,
    UnaryScalarOrArrayElementwise,
}

#[derive(Clone, Copy)]
struct UExportSpec {
    function_id: &'static str,
    registration: RegistrationSpec,
    lift_policy: ULiftPolicy,
}

#[derive(Clone, Copy)]
struct QUnaryNumberExportSpec {
    function_id: &'static str,
    registration: RegistrationSpec,
}

#[derive(Clone, Copy)]
struct QNullaryNumberExportSpec {
    function_id: &'static str,
    registration: RegistrationSpec,
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

include!(concat!(env!("OUT_DIR"), "/xll_generated_exports.rs"));

fn make_xloper_num(value: f64) -> XLOPER12 {
    XLOPER12 {
        val: XLOPER12Value { num: value },
        xltype: XLTYPE_NUM,
    }
}

fn make_xloper_err(error_code: i32) -> XLOPER12 {
    XLOPER12 {
        val: XLOPER12Value { err: error_code },
        xltype: XLTYPE_ERR,
    }
}

fn make_xloper_int(value: i32) -> XLOPER12 {
    XLOPER12 {
        val: XLOPER12Value { w: value },
        xltype: XLTYPE_INT,
    }
}

fn alloc_result(mut oper: XLOPER12) -> *mut XLOPER12 {
    oper.xltype |= XLBIT_DLLFREE;
    Box::into_raw(Box::new(oper))
}

fn alloc_result_multi(rows: i32, cols: i32, items: Vec<XLOPER12>) -> *mut XLOPER12 {
    let boxed_items: Box<[XLOPER12]> = items.into_boxed_slice();
    let items_ptr = Box::into_raw(boxed_items) as *mut XLOPER12;
    let oper = XLOPER12 {
        val: XLOPER12Value {
            array: XlArray12 {
                lparray: items_ptr,
                rows,
                columns: cols,
            },
        },
        xltype: XLTYPE_MULTI | XLBIT_DLLFREE,
    };
    Box::into_raw(Box::new(oper))
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

fn eval_value_to_xloper(value: EvalValue) -> XLOPER12 {
    match value {
        EvalValue::Number(n) => make_xloper_num(n),
        EvalValue::Error(code) => make_xloper_err(map_ws_err_to_excel(code)),
        _ => make_xloper_err(XLERR_VALUE),
    }
}

fn fetch_excel12_proc() -> Option<Excel12Proc> {
    let module = unsafe { GetModuleHandleW(std::ptr::null()) };
    if module.is_null() {
        return None;
    }
    let proc = unsafe { GetProcAddress(module, MD_CALLBACK_12.as_ptr()) }?;
    // SAFETY: `MdCallBack12` has the documented Excel C API callback signature.
    Some(unsafe { std::mem::transmute::<unsafe extern "system" fn() -> isize, Excel12Proc>(proc) })
}

fn excel12_proc() -> Option<Excel12Proc> {
    *EXCEL12_PROC.get_or_init(fetch_excel12_proc)
}

fn excel12v(xlfn: i32, result: *mut XLOPER12, opers: &mut [*mut XLOPER12]) -> i32 {
    let Some(proc) = excel12_proc() else {
        return 32;
    };

    let coper = i32::try_from(opers.len()).unwrap_or(i32::MAX);
    let opers_ptr = if opers.is_empty() {
        std::ptr::null_mut()
    } else {
        opers.as_mut_ptr()
    };
    // SAFETY: Callback pointer is fetched from Excel and arguments match the C API contract.
    unsafe { proc(xlfn, coper, opers_ptr, result) }
}

fn call_excel_free(temp: &mut XLOPER12) {
    let mut args = [temp as *mut XLOPER12];
    let _ = excel12v(XL_FREE, std::ptr::null_mut(), &mut args);
}

fn to_excel_pascal_wide(s: &str) -> Vec<u16> {
    let mut utf16: Vec<u16> = s.encode_utf16().collect();
    if utf16.len() > 32767 {
        utf16.truncate(32767);
    }
    let mut out = Vec::with_capacity(utf16.len() + 1);
    out.push(u16::try_from(utf16.len()).unwrap_or(32767));
    out.extend(utf16);
    out
}

struct TempXlString {
    data: Vec<u16>,
    oper: XLOPER12,
}

impl TempXlString {
    fn new(s: &str) -> Self {
        let mut data = to_excel_pascal_wide(s);
        let oper = XLOPER12 {
            val: XLOPER12Value {
                str: data.as_mut_ptr(),
            },
            xltype: XLTYPE_STR,
        };
        Self { data, oper }
    }

    fn oper_mut_ptr(&mut self) -> *mut XLOPER12 {
        let _ = self.data.len();
        &mut self.oper
    }
}

fn register_one(module_path: &str, spec: RegistrationSpec) -> bool {
    let mut dll = TempXlString::new(module_path);
    let mut proc = TempXlString::new(spec.export_name);
    let mut type_text = TempXlString::new(spec.type_text);
    let mut fn_name = TempXlString::new(spec.function_name);
    let mut arg_names = TempXlString::new(spec.arg_names);
    let mut category = TempXlString::new("OxFunc Bridge");
    let mut macro_type = make_xloper_int(1);
    let mut reg_id = XLOPER12 {
        val: XLOPER12Value { w: 0 },
        xltype: 0,
    };

    let mut args = [
        dll.oper_mut_ptr(),
        proc.oper_mut_ptr(),
        type_text.oper_mut_ptr(),
        fn_name.oper_mut_ptr(),
        arg_names.oper_mut_ptr(),
        &mut macro_type,
        category.oper_mut_ptr(),
    ];

    excel12v(XLF_REGISTER, &mut reg_id, &mut args) == XLRET_SUCCESS
}

fn register_all(module_path: &str) -> bool {
    GENERATED_REGISTRATION_SPECS
        .iter()
        .all(|spec| register_one(module_path, *spec))
}

fn current_module_path() -> Option<String> {
    let mut module: HMODULE = std::ptr::null_mut();
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
    let len = unsafe { GetModuleFileNameW(module, buffer.as_mut_ptr(), buffer.len() as u32) };
    if len == 0 {
        return None;
    }
    let len_usize = usize::try_from(len).ok()?;
    Some(String::from_utf16_lossy(&buffer[..len_usize]))
}

fn is_ref_type(xltype: u32) -> bool {
    let ty = xltype & XLTYPE_MASK;
    ty == XLTYPE_REF || ty == XLTYPE_SREF
}

fn coerce_reference_to_value(arg: *mut XLOPER12, out: &mut XLOPER12) -> bool {
    let mut args = [arg];
    excel12v(XL_COERCE, out as *mut XLOPER12, &mut args) == XLRET_SUCCESS
}

fn call_arg_from_xloper(value: *const XLOPER12) -> CallArgValue {
    if value.is_null() {
        return CallArgValue::MissingArg;
    }
    // SAFETY: Caller provides a valid pointer for call duration.
    let ty = unsafe { (*value).xltype & XLTYPE_MASK };
    match ty {
        XLTYPE_MISSING => CallArgValue::MissingArg,
        XLTYPE_NIL => CallArgValue::EmptyCell,
        XLTYPE_NUM => {
            // SAFETY: Union arm is valid because `xltype` is `xltypeNum`.
            CallArgValue::Eval(EvalValue::Number(unsafe { (*value).val.num }))
        }
        XLTYPE_BOOL => {
            // SAFETY: Union arm is valid because `xltype` is `xltypeBool`.
            CallArgValue::Eval(EvalValue::Logical(unsafe { (*value).val.xbool != 0 }))
        }
        XLTYPE_ERR => {
            // SAFETY: Union arm is valid because `xltype` is `xltypeErr`.
            CallArgValue::Eval(EvalValue::Error(map_excel_err_to_ws(unsafe {
                (*value).val.err
            })))
        }
        XLTYPE_STR => {
            // SAFETY: Union arm is valid because `xltype` is `xltypeStr`.
            let pstr = unsafe { (*value).val.str };
            if pstr.is_null() {
                return CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(
                    Vec::new(),
                )));
            }
            // SAFETY: `pstr` points to an Excel Pascal-style UTF-16 string.
            let len = usize::from(unsafe { *pstr });
            // SAFETY: `pstr` points to at least `len + 1` UTF-16 units.
            let chars = unsafe { std::slice::from_raw_parts(pstr.add(1), len) }.to_vec();
            CallArgValue::Eval(EvalValue::Text(ExcelText::from_utf16_code_units(chars)))
        }
        _ => CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Value)),
    }
}

fn eval_scalar(function_id: &str, arg: &CallArgValue) -> EvalValue {
    let resolver = NoReferenceResolver;
    match eval_surface_unary_scalar_value(function_id, arg, &resolver) {
        Ok(v) => v,
        Err(code) => EvalValue::Error(code),
    }
}

fn eval_u_export(spec: UExportSpec, arg: *mut XLOPER12) -> *mut XLOPER12 {
    let mut temp = XLOPER12 {
        val: XLOPER12Value { w: 0 },
        xltype: 0,
    };
    let mut used_temp = false;
    let mut value_ptr = arg;

    if !value_ptr.is_null() {
        // SAFETY: `value_ptr` originates from Excel and is valid for this call.
        let ty = unsafe { (*value_ptr).xltype };
        if is_ref_type(ty) {
            if !coerce_reference_to_value(value_ptr, &mut temp) {
                return alloc_result(make_xloper_err(XLERR_VALUE));
            }
            value_ptr = &mut temp;
            used_temp = true;
        }
    }

    // SAFETY: `value_ptr` either comes from Excel or from `temp` above.
    let value_ty = if value_ptr.is_null() {
        XLTYPE_MISSING
    } else {
        unsafe { (*value_ptr).xltype & XLTYPE_MASK }
    };

    let result = if value_ty == XLTYPE_MULTI {
        match spec.lift_policy {
            ULiftPolicy::ScalarOnly => alloc_result(make_xloper_err(XLERR_VALUE)),
            ULiftPolicy::UnaryScalarOrArrayElementwise => {
                // SAFETY: `value_ptr` points to xltypeMulti in this branch.
                let array = unsafe { (*value_ptr).val.array };
                let rows = array.rows;
                let cols = array.columns;
                let count = usize::try_from(rows.saturating_mul(cols)).unwrap_or(0);
                let mut mapped = Vec::with_capacity(count);
                for i in 0..count {
                    // SAFETY: `lparray` points to `rows*cols` contiguous XLOPER12 entries.
                    let item_ptr = unsafe { array.lparray.add(i) };
                    // SAFETY: item pointer is inside the `lparray` allocation.
                    let call_arg = call_arg_from_xloper(item_ptr);
                    let eval_value = eval_scalar(spec.function_id, &call_arg);
                    mapped.push(eval_value_to_xloper(eval_value));
                }
                alloc_result_multi(rows, cols, mapped)
            }
        }
    } else {
        // SAFETY: `value_ptr` is nullable and handled in converter.
        let call_arg = call_arg_from_xloper(value_ptr);
        let eval_value = eval_scalar(spec.function_id, &call_arg);
        alloc_result(eval_value_to_xloper(eval_value))
    };

    if used_temp {
        call_excel_free(&mut temp);
    }
    result
}

fn eval_q_unary_number_export(spec: QUnaryNumberExportSpec, value: f64) -> f64 {
    eval_surface_q_unary_number(spec.function_id, value).unwrap_or(f64::NAN)
}

fn eval_q_nullary_number_export(spec: QNullaryNumberExportSpec) -> f64 {
    eval_surface_q_nullary_number(spec.function_id).unwrap_or(f64::NAN)
}

#[unsafe(no_mangle)]
pub extern "system" fn xlAutoOpen() -> i32 {
    let Some(module_path) = current_module_path() else {
        return 0;
    };
    if register_all(&module_path) { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "system" fn xlAutoClose() -> i32 {
    1
}

#[unsafe(no_mangle)]
pub extern "system" fn xlAutoFree12(to_free: *mut XLOPER12) {
    if to_free.is_null() {
        return;
    }

    // SAFETY: `to_free` is allocated by this module for `xlbitDLLFree` returns.
    let boxed = unsafe { Box::from_raw(to_free) };
    let base_type = boxed.xltype & XLTYPE_MASK;
    if base_type == XLTYPE_MULTI {
        // SAFETY: Union access is valid for `xltypeMulti`.
        let array = unsafe { boxed.val.array };
        if !array.lparray.is_null() {
            let count = usize::try_from(array.rows.saturating_mul(array.columns)).unwrap_or(0);
            if count > 0 {
                // SAFETY: `lparray` was allocated from `Box<[XLOPER12]>` in `alloc_result_multi`.
                let raw_slice = std::ptr::slice_from_raw_parts_mut(array.lparray, count);
                // SAFETY: Ownership is transferred back to Rust for deallocation.
                unsafe {
                    drop(Box::from_raw(raw_slice));
                }
            }
        }
    }
}
