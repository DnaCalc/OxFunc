#![allow(non_snake_case)]

use std::env;
use std::ffi::c_void;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use oxfunc_core::functions::a1_refs::{
    A1Reference, A1ReferenceNotation, EXCEL_MAX_COLS, EXCEL_MAX_ROWS, format_relative_target,
    parse_a1_reference,
};
use oxfunc_core::functions::surface_dispatch::{
    eval_surface_q_binary_number, eval_surface_q_nullary_number, eval_surface_q_unary_number, eval_surface_value_call,
};
use oxfunc_core::locale_format::current_excel_host_context;
use oxfunc_core::resolver::{
    CallerContext, RefResolutionError, ReferenceResolver, ResolverCapabilities,
};
use oxfunc_core::value::{
    ArrayCellValue, ArrayShape, CallArgValue, EvalArray, EvalValue, ExcelText, ReferenceKind,
    ReferenceLike, WorksheetErrorCode,
};
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
const XL_SHEET_ID: i32 = 4 | XL_SPECIAL;
const XL_SHEET_NM: i32 = 5 | XL_SPECIAL;
const XLF_REGISTER: i32 = 149;
const XLF_CALLER: i32 = 89;
const XLF_GET_CELL: i32 = 185;
const XLF_GET_WORKSPACE: i32 = 186;
const XLF_GET_DOCUMENT: i32 = 188;
const XLF_GET_WORKBOOK: i32 = 268;

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

#[derive(Clone, Copy)]
struct ExperimentalRegistrationSpec {
    export_name: &'static str,
    type_text: &'static str,
    function_name: &'static str,
    arg_names: &'static str,
    macro_type: i32,
}

#[derive(Clone, Copy)]
struct ManualRegistrationSpec {
    export_name: &'static str,
    type_text: &'static str,
    function_name: &'static str,
    arg_names: &'static str,
    macro_type: i32,
}

const FLAG_EXPERIMENT_REGISTRATION_SPECS: &[ExperimentalRegistrationSpec] = &[
    ExperimentalRegistrationSpec {
        export_name: "OX_NOW",
        type_text: "Q",
        function_name: "ox_NOW_F_BASE",
        arg_names: "",
        macro_type: 1,
    },
    ExperimentalRegistrationSpec {
        export_name: "OX_NOW",
        type_text: "Q!",
        function_name: "ox_NOW_F_VOL",
        arg_names: "",
        macro_type: 1,
    },
    ExperimentalRegistrationSpec {
        export_name: "OX_ABS",
        type_text: "QU",
        function_name: "ox_ABS_F_BASE",
        arg_names: "arg1",
        macro_type: 1,
    },
    ExperimentalRegistrationSpec {
        export_name: "OX_ABS",
        type_text: "QU$",
        function_name: "ox_ABS_F_TS",
        arg_names: "arg1",
        macro_type: 1,
    },
    ExperimentalRegistrationSpec {
        export_name: "OX_INDIRECT",
        type_text: "QUU",
        function_name: "ox_INDIRECT_F_BASE",
        arg_names: "arg1,arg2",
        macro_type: 1,
    },
    ExperimentalRegistrationSpec {
        export_name: "OX_INDIRECT",
        type_text: "QUU#",
        function_name: "ox_INDIRECT_F_MACRO",
        arg_names: "arg1,arg2",
        macro_type: 1,
    },
];

const MANUAL_PROBE_REGISTRATION_SPECS: &[ManualRegistrationSpec] = &[
    ManualRegistrationSpec {
        export_name: "OX_PROBE_RET_NIL",
        type_text: "Q",
        function_name: "ox_PROBE_RET_NIL",
        arg_names: "",
        macro_type: 1,
    },
    ManualRegistrationSpec {
        export_name: "OX_PROBE_ECHO",
        type_text: "QU",
        function_name: "ox_PROBE_ECHO",
        arg_names: "arg1",
        macro_type: 1,
    },
    ManualRegistrationSpec {
        export_name: "OX_PROBE_DESCRIBE",
        type_text: "QU",
        function_name: "ox_PROBE_DESCRIBE",
        arg_names: "arg1",
        macro_type: 1,
    },
    ManualRegistrationSpec {
        export_name: "OX_PROBE_RET_ARRAY_NIL",
        type_text: "Q",
        function_name: "ox_PROBE_RET_ARRAY_NIL",
        arg_names: "",
        macro_type: 1,
    },
    ManualRegistrationSpec {
        export_name: "OX_PROBE_ARRAY_DESC",
        type_text: "QU",
        function_name: "ox_PROBE_ARRAY_DESC",
        arg_names: "arg1",
        macro_type: 1,
    },
    ManualRegistrationSpec {
        export_name: "OX_GET_CELL",
        type_text: "QUU",
        function_name: "ox_GET_CELL",
        arg_names: "type_num,reference",
        macro_type: 1,
    },
    ManualRegistrationSpec {
        export_name: "OX_GET_DOCUMENT",
        type_text: "QUU",
        function_name: "ox_GET_DOCUMENT",
        arg_names: "type_num,name_text",
        macro_type: 1,
    },
    ManualRegistrationSpec {
        export_name: "OX_GET_WORKBOOK",
        type_text: "QUU",
        function_name: "ox_GET_WORKBOOK",
        arg_names: "type_num,name_text",
        macro_type: 1,
    },
    ManualRegistrationSpec {
        export_name: "OX_GET_WORKBOOK_ACTIVE",
        type_text: "QU",
        function_name: "ox_GET_WORKBOOK_ACTIVE",
        arg_names: "type_num",
        macro_type: 0,
    },
    ManualRegistrationSpec {
        export_name: "OX_GET_WORKSPACE",
        type_text: "QU",
        function_name: "ox_GET_WORKSPACE",
        arg_names: "type_num",
        macro_type: 1,
    },
];

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
    preserve_refs: bool,
    min_arity: usize,
    arg_count: usize,
}

#[derive(Clone, Copy)]
struct QUnaryNumberExportSpec {
    function_id: &'static str,
    registration: RegistrationSpec,
}

#[derive(Clone, Copy)]
struct QBinaryNumberExportSpec {
    function_id: &'static str,
    registration: RegistrationSpec,
}

#[derive(Clone, Copy)]
struct QNullaryNumberExportSpec {
    function_id: &'static str,
    registration: RegistrationSpec,
}

struct ExcelReferenceResolver {
    caller: Option<CallerContext>,
}

impl ReferenceResolver for ExcelReferenceResolver {
    fn capabilities(&self) -> ResolverCapabilities {
        ResolverCapabilities::permissive_local()
    }

    fn resolve_reference(
        &self,
        reference: &ReferenceLike,
    ) -> Result<EvalValue, RefResolutionError> {
        resolve_reference_via_excel(reference)
    }

    fn caller_context(&self) -> Option<CallerContext> {
        self.caller.clone()
    }
}

include!(concat!(env!("OUT_DIR"), "/xll_generated_exports.rs"));

fn make_xloper_num(value: f64) -> XLOPER12 {
    XLOPER12 {
        val: XLOPER12Value { num: value },
        xltype: XLTYPE_NUM,
    }
}

fn make_xloper_bool(value: bool) -> XLOPER12 {
    XLOPER12 {
        val: XLOPER12Value {
            xbool: if value { 1 } else { 0 },
        },
        xltype: XLTYPE_BOOL,
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

fn make_xloper_str_from_utf16(units: &[u16]) -> XLOPER12 {
    let mut data = Vec::with_capacity(units.len().saturating_add(1));
    let text_len = units.len().min(32767);
    data.push(u16::try_from(text_len).unwrap_or(32767));
    data.extend_from_slice(&units[..text_len]);
    let ptr = Box::into_raw(data.into_boxed_slice()) as *mut u16;
    XLOPER12 {
        val: XLOPER12Value { str: ptr },
        xltype: XLTYPE_STR,
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

fn make_xloper_nil() -> XLOPER12 {
    XLOPER12 {
        val: XLOPER12Value { w: 0 },
        xltype: XLTYPE_NIL,
    }
}

fn make_xloper_text(s: &str) -> XLOPER12 {
    let units: Vec<u16> = s.encode_utf16().collect();
    make_xloper_str_from_utf16(&units)
}

struct OwnedReferenceOper {
    oper: XLOPER12,
    owned_mref: Option<*mut XLMRef12>,
}

impl OwnedReferenceOper {
    fn as_mut_ptr(&mut self) -> *mut XLOPER12 {
        &mut self.oper
    }

    fn into_result_oper(mut self) -> XLOPER12 {
        self.owned_mref = None;
        self.oper
    }
}

impl Drop for OwnedReferenceOper {
    fn drop(&mut self) {
        if let Some(ptr) = self.owned_mref.take() {
            unsafe {
                drop(Box::from_raw(ptr));
            }
        }
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

fn notation_from_bounds(
    start_row: usize,
    start_col: usize,
    end_row: usize,
    end_col: usize,
) -> A1ReferenceNotation {
    if start_row == 1 && end_row == EXCEL_MAX_ROWS {
        A1ReferenceNotation::WholeColumn
    } else if start_col == 1 && end_col == EXCEL_MAX_COLS {
        A1ReferenceNotation::WholeRow
    } else {
        A1ReferenceNotation::Rect
    }
}

fn reference_target_from_bounds(
    rw_first: i32,
    rw_last: i32,
    col_first: i32,
    col_last: i32,
    prefix: Option<String>,
) -> Option<String> {
    let start_row = usize::try_from(rw_first.saturating_add(1)).ok()?;
    let end_row = usize::try_from(rw_last.saturating_add(1)).ok()?;
    let start_col = usize::try_from(col_first.saturating_add(1)).ok()?;
    let end_col = usize::try_from(col_last.saturating_add(1)).ok()?;
    let reference = A1Reference {
        prefix,
        start_row,
        start_col,
        end_row,
        end_col,
        notation: notation_from_bounds(start_row, start_col, end_row, end_col),
    };
    format_relative_target(&reference)
}

fn reference_like_from_bounds(
    rw_first: i32,
    rw_last: i32,
    col_first: i32,
    col_last: i32,
    prefix: Option<String>,
) -> ReferenceLike {
    let target = reference_target_from_bounds(rw_first, rw_last, col_first, col_last, prefix)
        .unwrap_or_else(|| {
            let r1 = rw_first.saturating_add(1);
            let r2 = rw_last.saturating_add(1);
            let c1 = col_first.saturating_add(1);
            let c2 = col_last.saturating_add(1);
            if r1 == r2 && c1 == c2 {
                format!("R{r1}C{c1}")
            } else {
                format!("R{r1}C{c1}:R{r2}C{c2}")
            }
        });
    ReferenceLike {
        kind: if rw_first == rw_last && col_first == col_last {
            ReferenceKind::A1
        } else {
            ReferenceKind::Area
        },
        target,
    }
}

fn parse_pascal_utf16_string(value: *const XLOPER12) -> Option<String> {
    if value.is_null() {
        return None;
    }
    let ty = unsafe { (*value).xltype & XLTYPE_MASK };
    if ty != XLTYPE_STR {
        return None;
    }
    let pstr = unsafe { (*value).val.str };
    if pstr.is_null() {
        return Some(String::new());
    }
    let len = usize::from(unsafe { *pstr });
    let chars = unsafe { std::slice::from_raw_parts(pstr.add(1), len) };
    Some(String::from_utf16_lossy(chars))
}

fn area_refs_from_mref(mref: XlMRef12) -> Option<Vec<XLRef12>> {
    if mref.lpmref.is_null() {
        return None;
    }
    let count = usize::from(unsafe { (*mref.lpmref).count });
    if count == 0 {
        return None;
    }
    let first = unsafe { (*mref.lpmref).reftbl.as_ptr() };
    Some(unsafe { std::slice::from_raw_parts(first, count) }.to_vec())
}

fn call_excel_special(xlfn: i32, args: &mut [*mut XLOPER12]) -> Option<XLOPER12> {
    let mut out = XLOPER12 {
        val: XLOPER12Value { w: 0 },
        xltype: 0,
    };
    if excel12v(xlfn, &mut out, args) != XLRET_SUCCESS {
        return None;
    }
    Some(out)
}

fn sheet_name_from_reference_oper(reference: *mut XLOPER12) -> Option<String> {
    let mut args = [reference];
    let mut out = call_excel_special(XL_SHEET_NM, &mut args)?;
    let name = parse_pascal_utf16_string(&out);
    call_excel_free(&mut out);
    name
}

fn sheet_id_from_prefix(prefix: &str) -> Option<IdSheet> {
    let mut prefix_arg = TempXlString::new(prefix);
    let mut args = [prefix_arg.oper_mut_ptr()];
    let mut out = call_excel_special(XL_SHEET_ID, &mut args)?;
    let ty = out.xltype & XLTYPE_MASK;
    let id = match ty {
        XLTYPE_REF => Some(unsafe { out.val.mref.id_sheet }),
        _ => None,
    };
    call_excel_free(&mut out);
    id
}

fn caller_context_from_reference(reference: &ReferenceLike) -> Option<CallerContext> {
    let parsed = parse_a1_reference(&reference.target)?;
    Some(CallerContext {
        prefix: parsed.prefix,
        row: parsed.start_row,
        col: parsed.start_col,
    })
}

fn current_caller_context() -> Option<CallerContext> {
    let mut args = [];
    let mut out = call_excel_special(XLF_CALLER, &mut args)?;
    let caller = match out.xltype & XLTYPE_MASK {
        XLTYPE_SREF => {
            let sref = unsafe { out.val.sref };
            let target = reference_target_from_bounds(
                sref.r#ref.rw_first,
                sref.r#ref.rw_last,
                sref.r#ref.col_first,
                sref.r#ref.col_last,
                None,
            )?;
            caller_context_from_reference(&ReferenceLike {
                kind: ReferenceKind::A1,
                target,
            })
        }
        XLTYPE_REF => {
            let mref = unsafe { out.val.mref };
            let refs = area_refs_from_mref(mref)?;
            let first = refs.first()?;
            let target = reference_target_from_bounds(
                first.rw_first,
                first.rw_last,
                first.col_first,
                first.col_last,
                None,
            )?;
            caller_context_from_reference(&ReferenceLike {
                kind: ReferenceKind::Area,
                target,
            })
        }
        _ => None,
    };
    if matches!(out.xltype & XLTYPE_MASK, XLTYPE_REF | XLTYPE_SREF | XLTYPE_STR | XLTYPE_MULTI) {
        call_excel_free(&mut out);
    }
    caller
}

fn owned_reference_oper_from_a1(reference: &A1Reference) -> Option<OwnedReferenceOper> {
    let xl_ref = XLRef12 {
        rw_first: i32::try_from(reference.start_row.checked_sub(1)?).ok()?,
        rw_last: i32::try_from(reference.end_row.checked_sub(1)?).ok()?,
        col_first: i32::try_from(reference.start_col.checked_sub(1)?).ok()?,
        col_last: i32::try_from(reference.end_col.checked_sub(1)?).ok()?,
    };

    if let Some(prefix) = reference.prefix.as_deref() {
        if let Some(id_sheet) = sheet_id_from_prefix(prefix) {
            let mref = Box::new(XLMRef12 {
                count: 1,
                reftbl: [xl_ref],
            });
            let mref_ptr = Box::into_raw(mref);
            return Some(OwnedReferenceOper {
                oper: XLOPER12 {
                    val: XLOPER12Value {
                        mref: XlMRef12 {
                            lpmref: mref_ptr,
                            id_sheet,
                        },
                    },
                    xltype: XLTYPE_REF,
                },
                owned_mref: Some(mref_ptr),
            });
        }
    }

    Some(OwnedReferenceOper {
        oper: XLOPER12 {
            val: XLOPER12Value {
                sref: XlSRef12 {
                    count: 1,
                    r#ref: xl_ref,
                },
            },
            xltype: XLTYPE_SREF,
        },
        owned_mref: None,
    })
}

fn resolved_eval_from_call_arg(arg: CallArgValue) -> EvalValue {
    match arg {
        CallArgValue::Eval(value) => value,
        CallArgValue::EmptyCell | CallArgValue::MissingArg => EvalValue::Array(EvalArray::from_scalar(
            ArrayCellValue::EmptyCell,
        )),
        CallArgValue::Reference(reference) => EvalValue::Reference(reference),
    }
}

fn call_arg_to_xloper(arg: CallArgValue) -> XLOPER12 {
    match arg {
        CallArgValue::Eval(value) => eval_value_to_xloper(value),
        CallArgValue::MissingArg | CallArgValue::EmptyCell => make_xloper_nil(),
        CallArgValue::Reference(reference) => eval_value_to_xloper(EvalValue::Reference(reference)),
    }
}

fn clone_excel_return_to_owned(value: *const XLOPER12, preserve_refs: bool) -> XLOPER12 {
    call_arg_to_xloper(call_arg_from_xloper(value, preserve_refs))
}

fn free_excel_result_if_needed(value: &mut XLOPER12) {
    if matches!(value.xltype & XLTYPE_MASK, XLTYPE_REF | XLTYPE_SREF | XLTYPE_STR | XLTYPE_MULTI) {
        call_excel_free(value);
    }
}

fn probe_info_unary(xlfn: i32, arg1: *mut XLOPER12, preserve_refs: bool) -> *mut XLOPER12 {
    let mut args = [arg1];
    let Some(mut out) = call_excel_special(xlfn, &mut args) else {
        return alloc_result(make_xloper_err(XLERR_VALUE));
    };
    let cloned = clone_excel_return_to_owned(&out, preserve_refs);
    free_excel_result_if_needed(&mut out);
    alloc_result(cloned)
}

fn probe_info_binary(
    xlfn: i32,
    arg1: *mut XLOPER12,
    arg2: *mut XLOPER12,
    preserve_refs: bool,
) -> *mut XLOPER12 {
    let mut args = [arg1, arg2];
    let Some(mut out) = call_excel_special(xlfn, &mut args) else {
        return alloc_result(make_xloper_err(XLERR_VALUE));
    };
    let cloned = clone_excel_return_to_owned(&out, preserve_refs);
    free_excel_result_if_needed(&mut out);
    alloc_result(cloned)
}

fn resolve_reference_via_excel(reference: &ReferenceLike) -> Result<EvalValue, RefResolutionError> {
    let parsed = parse_a1_reference(&reference.target).ok_or_else(|| {
        RefResolutionError::UnresolvedReference {
            target: reference.target.clone(),
        }
    })?;
    let mut temp_ref = owned_reference_oper_from_a1(&parsed).ok_or_else(|| {
        RefResolutionError::ProviderFailure {
            detail: format!("unable to construct Excel reference for {}", reference.target),
        }
    })?;
    let mut out = XLOPER12 {
        val: XLOPER12Value { w: 0 },
        xltype: 0,
    };
    if !coerce_reference_to_value(temp_ref.as_mut_ptr(), &mut out) {
        return Err(RefResolutionError::UnresolvedReference {
            target: reference.target.clone(),
        });
    }
    let resolved = resolved_eval_from_call_arg(call_arg_from_xloper(&out, false));
    call_excel_free(&mut out);
    Ok(resolved)
}

fn eval_value_to_xloper(value: EvalValue) -> XLOPER12 {
    match value {
        EvalValue::Number(n) => make_xloper_num(n),
        EvalValue::Logical(b) => make_xloper_bool(b),
        EvalValue::Text(t) => make_xloper_str_from_utf16(t.utf16_code_units()),
        EvalValue::Error(code) => make_xloper_err(map_ws_err_to_excel(code)),
        EvalValue::Reference(reference) => {
            let Some(parsed) = parse_a1_reference(&reference.target) else {
                return make_xloper_err(XLERR_VALUE);
            };
            let Some(oper) = owned_reference_oper_from_a1(&parsed) else {
                return make_xloper_err(XLERR_REF);
            };
            oper.into_result_oper()
        }
        EvalValue::Array(array) => {
            let shape = array.shape();
            let items: Vec<XLOPER12> = array
                .iter_row_major()
                .map(|cell| match cell {
                    ArrayCellValue::Number(n) => make_xloper_num(*n),
                    ArrayCellValue::Text(t) => make_xloper_str_from_utf16(t.utf16_code_units()),
                    ArrayCellValue::Logical(b) => make_xloper_bool(*b),
                    ArrayCellValue::Error(code) => make_xloper_err(map_ws_err_to_excel(*code)),
                    ArrayCellValue::EmptyCell => make_xloper_nil(),
                })
                .collect();
            XLOPER12 {
                val: XLOPER12Value {
                    array: XlArray12 {
                        lparray: Box::into_raw(items.into_boxed_slice()) as *mut XLOPER12,
                        rows: i32::try_from(shape.rows).unwrap_or(i32::MAX),
                        columns: i32::try_from(shape.cols).unwrap_or(i32::MAX),
                    },
                },
                xltype: XLTYPE_MULTI,
            }
        }
        EvalValue::Lambda(_) => make_xloper_err(XLERR_VALUE),
    }
}

fn array_cell_from_call_arg(arg: CallArgValue) -> ArrayCellValue {
    match arg {
        CallArgValue::Eval(EvalValue::Number(n)) => ArrayCellValue::Number(n),
        CallArgValue::Eval(EvalValue::Text(t)) => ArrayCellValue::Text(t),
        CallArgValue::Eval(EvalValue::Logical(b)) => ArrayCellValue::Logical(b),
        CallArgValue::Eval(EvalValue::Error(code)) => ArrayCellValue::Error(code),
        CallArgValue::EmptyCell | CallArgValue::MissingArg => ArrayCellValue::EmptyCell,
        CallArgValue::Reference(_) | CallArgValue::Eval(EvalValue::Reference(_)) => {
            ArrayCellValue::Error(WorksheetErrorCode::Value)
        }
        CallArgValue::Eval(EvalValue::Array(_)) | CallArgValue::Eval(EvalValue::Lambda(_)) => {
            ArrayCellValue::Error(WorksheetErrorCode::Value)
        }
    }
}

fn current_excel_serial_utc() -> f64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    25569.0 + (now / 86400.0)
}

fn current_random_unit() -> f64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    (nanos % 1_000_000_000) as f64 / 1_000_000_000.0
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
    register_one_dynamic(
        module_path,
        spec.export_name,
        spec.type_text,
        spec.function_name,
        spec.arg_names,
        1,
    )
}

fn register_one_dynamic(
    module_path: &str,
    export_name: &str,
    type_text_text: &str,
    function_name_text: &str,
    arg_names_text: &str,
    macro_type_value: i32,
) -> bool {
    let mut dll = TempXlString::new(module_path);
    let mut proc = TempXlString::new(export_name);
    let mut type_text = TempXlString::new(type_text_text);
    let mut fn_name = TempXlString::new(function_name_text);
    let mut arg_names = TempXlString::new(arg_names_text);
    let mut category = TempXlString::new("OxFunc Bridge");
    let mut macro_type = make_xloper_int(macro_type_value);
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

fn flag_experiments_enabled() -> bool {
    let raw = env::var("OXFUNC_XLL_ENABLE_FLAG_EXPERIMENTS").unwrap_or_else(|_| "1".to_string());
    !matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "0" | "false" | "no" | "off"
    )
}

fn register_flag_experiment_aliases(module_path: &str) -> bool {
    FLAG_EXPERIMENT_REGISTRATION_SPECS
        .iter()
        .all(|spec| {
            register_one_dynamic(
                module_path,
                spec.export_name,
                spec.type_text,
                spec.function_name,
                spec.arg_names,
                spec.macro_type,
            )
        })
}

fn register_all(module_path: &str) -> bool {
    GENERATED_REGISTRATION_SPECS
        .iter()
        .all(|spec| register_one(module_path, *spec))
}

fn register_manual_probe_aliases(module_path: &str) -> bool {
    MANUAL_PROBE_REGISTRATION_SPECS.iter().all(|spec| {
        register_one_dynamic(
            module_path,
            spec.export_name,
            spec.type_text,
            spec.function_name,
            spec.arg_names,
            spec.macro_type,
        )
    })
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

fn call_arg_from_xloper(value: *const XLOPER12, preserve_refs: bool) -> CallArgValue {
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
        XLTYPE_INT => {
            // SAFETY: Union arm is valid because `xltype` is `xltypeInt`.
            CallArgValue::Eval(EvalValue::Number(unsafe { (*value).val.w as f64 }))
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
        XLTYPE_MULTI => {
            // SAFETY: Union arm is valid because `xltype` is `xltypeMulti`.
            let array = unsafe { (*value).val.array };
            let rows = usize::try_from(array.rows.max(0)).unwrap_or(0);
            let cols = usize::try_from(array.columns.max(0)).unwrap_or(0);
            if rows == 0 || cols == 0 || array.lparray.is_null() {
                return CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Value));
            }
            let len = rows.saturating_mul(cols);
            // SAFETY: Excel provides `rows * columns` contiguous XLOPER12 items.
            let items = unsafe { std::slice::from_raw_parts(array.lparray, len) };
            let cells = items
                .iter()
                .map(|item| array_cell_from_call_arg(call_arg_from_xloper(item, preserve_refs)))
                .collect();
            let shape = ArrayShape { rows, cols };
            let eval_array = EvalArray::new(shape, cells)
                .expect("excel multi arrays should match their declared dimensions");
            CallArgValue::Eval(EvalValue::Array(eval_array))
        }
        XLTYPE_SREF if preserve_refs => {
            // SAFETY: Union arm is valid because `xltype` is `xltypeSRef`.
            let sref = unsafe { (*value).val.sref };
            CallArgValue::Reference(reference_like_from_bounds(
                sref.r#ref.rw_first,
                sref.r#ref.rw_last,
                sref.r#ref.col_first,
                sref.r#ref.col_last,
                None,
            ))
        }
        XLTYPE_REF if preserve_refs => {
            // SAFETY: Union arm is valid because `xltype` is `xltypeRef`.
            let mref = unsafe { (*value).val.mref };
            let Some(refs) = area_refs_from_mref(mref) else {
                return CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Ref));
            };
            let targets: Option<Vec<String>> = refs
                .iter()
                .map(|entry| {
                    reference_target_from_bounds(
                        entry.rw_first,
                        entry.rw_last,
                        entry.col_first,
                        entry.col_last,
                        None,
                    )
                })
                .collect();
            let Some(targets) = targets else {
                return CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Ref));
            };
            let target = if targets.len() == 1 {
                targets[0].clone()
            } else {
                format!("({})", targets.join(","))
            };
            CallArgValue::Reference(ReferenceLike {
                kind: ReferenceKind::Area,
                target,
            })
        }
        _ => CallArgValue::Eval(EvalValue::Error(WorksheetErrorCode::Value)),
    }
}

fn eval_surface_value(function_id: &str, args: &[CallArgValue]) -> EvalValue {
    let resolver = ExcelReferenceResolver {
        caller: current_caller_context(),
    };
    match eval_surface_value_call(
        function_id,
        args,
        &resolver,
        Some(current_excel_serial_utc()),
        Some(current_random_unit()),
        Some(&current_excel_host_context()),
    ) {
        Ok(v) => v,
        Err(code) => EvalValue::Error(code),
    }
}

fn describe_eval_value(value: &EvalValue) -> String {
    match value {
        EvalValue::Number(_) => "number".to_string(),
        EvalValue::Text(text) => {
            if text.utf16_code_units().is_empty() {
                "text(\"\")".to_string()
            } else {
                "text".to_string()
            }
        }
        EvalValue::Logical(_) => "logical".to_string(),
        EvalValue::Error(code) => format!("error({code:?})"),
        EvalValue::Reference(reference) => {
            format!("reference({:?}:{})", reference.kind, reference.target)
        }
        EvalValue::Array(array) => {
            let shape = array.shape();
            let parts = array
                .iter_row_major()
                .map(|cell| match cell {
                    ArrayCellValue::Number(_) => "number".to_string(),
                    ArrayCellValue::Text(text) => {
                        if text.utf16_code_units().is_empty() {
                            "text(\"\")".to_string()
                        } else {
                            "text".to_string()
                        }
                    }
                    ArrayCellValue::Logical(_) => "logical".to_string(),
                    ArrayCellValue::Error(code) => format!("error({code:?})"),
                    ArrayCellValue::EmptyCell => "empty_cell".to_string(),
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("array({}x{})[{}]", shape.rows, shape.cols, parts)
        }
        EvalValue::Lambda(_) => "lambda".to_string(),
    }
}

fn describe_call_arg(arg: &CallArgValue) -> String {
    match arg {
        CallArgValue::MissingArg => "missing_arg".to_string(),
        CallArgValue::EmptyCell => "empty_cell".to_string(),
        CallArgValue::Reference(reference) => {
            format!("reference({:?}:{})", reference.kind, reference.target)
        }
        CallArgValue::Eval(value) => describe_eval_value(value),
    }
}

fn probe_echo(raw: *mut XLOPER12) -> *mut XLOPER12 {
    if raw.is_null() {
        return alloc_result(make_xloper_err(XLERR_VALUE));
    }

    let ty = unsafe { (*raw).xltype & XLTYPE_MASK };
    match ty {
        XLTYPE_NIL => alloc_result(make_xloper_nil()),
        XLTYPE_NUM => alloc_result(make_xloper_num(unsafe { (*raw).val.num })),
        XLTYPE_INT => alloc_result(make_xloper_num(unsafe { (*raw).val.w as f64 })),
        XLTYPE_BOOL => alloc_result(make_xloper_bool(unsafe { (*raw).val.xbool != 0 })),
        XLTYPE_ERR => alloc_result(make_xloper_err(unsafe { (*raw).val.err })),
        XLTYPE_STR => match call_arg_from_xloper(raw, false) {
            CallArgValue::Eval(EvalValue::Text(text)) => {
                alloc_result(make_xloper_str_from_utf16(text.utf16_code_units()))
            }
            _ => alloc_result(make_xloper_err(XLERR_VALUE)),
        },
        XLTYPE_MULTI => match call_arg_from_xloper(raw, false) {
            CallArgValue::Eval(EvalValue::Array(array)) => {
                alloc_result(eval_value_to_xloper(EvalValue::Array(array)))
            }
            _ => alloc_result(make_xloper_err(XLERR_VALUE)),
        },
        XLTYPE_SREF | XLTYPE_REF => match call_arg_from_xloper(raw, true) {
            CallArgValue::Reference(reference) => {
                alloc_result(eval_value_to_xloper(EvalValue::Reference(reference)))
            }
            _ => alloc_result(make_xloper_err(XLERR_VALUE)),
        },
        _ => alloc_result(make_xloper_err(XLERR_VALUE)),
    }
}

fn probe_describe(raw: *mut XLOPER12) -> *mut XLOPER12 {
    let description = describe_call_arg(&call_arg_from_xloper(raw, true));
    alloc_result(make_xloper_text(&description))
}

fn probe_ret_array_nil() -> *mut XLOPER12 {
    let items = vec![
        make_xloper_nil(),
        make_xloper_num(1.0),
        make_xloper_text("x"),
        make_xloper_nil(),
    ];
    alloc_result_multi(2, 2, items)
}

fn probe_array_desc(raw: *mut XLOPER12) -> *mut XLOPER12 {
    let description = match call_arg_from_xloper(raw, false) {
        CallArgValue::Eval(EvalValue::Array(array)) => describe_eval_value(&EvalValue::Array(array)),
        other => describe_call_arg(&other),
    };
    alloc_result(make_xloper_text(&description))
}

fn raw_arg_is_missing(raw: *mut XLOPER12) -> bool {
    if raw.is_null() {
        return true;
    }
    // SAFETY: `raw` originates from Excel and is valid for this call.
    unsafe { ((*raw).xltype & XLTYPE_MASK) == XLTYPE_MISSING }
}

fn effective_u_arg_len(spec: UExportSpec, raw_args: &[*mut XLOPER12]) -> usize {
    let trimmed = raw_args
        .iter()
        .rposition(|raw| !raw_arg_is_missing(*raw))
        .map(|idx| idx + 1)
        .unwrap_or(0);
    trimmed.max(spec.min_arity)
}

fn eval_u_export(spec: UExportSpec, raw_args: &[*mut XLOPER12]) -> *mut XLOPER12 {
    if raw_args.len() != spec.arg_count {
        return alloc_result(make_xloper_err(XLERR_VALUE));
    }

    if raw_args.len() == 1 && matches!(spec.lift_policy, ULiftPolicy::UnaryScalarOrArrayElementwise) {
        let raw = raw_args[0];
        let mut temp = XLOPER12 {
            val: XLOPER12Value { w: 0 },
            xltype: 0,
        };
        let mut value_ptr = raw;
        let mut used_temp = false;
        if !value_ptr.is_null() {
            // SAFETY: `value_ptr` originates from Excel and is valid for this call.
            let ty = unsafe { (*value_ptr).xltype };
            if is_ref_type(ty) && !spec.preserve_refs {
                if !coerce_reference_to_value(value_ptr, &mut temp) {
                    return alloc_result(make_xloper_err(XLERR_VALUE));
                }
                value_ptr = &mut temp;
                used_temp = true;
            }
        }

        let value_ty = if value_ptr.is_null() {
            XLTYPE_MISSING
        } else {
            // SAFETY: `value_ptr` is valid for this scope.
            unsafe { (*value_ptr).xltype & XLTYPE_MASK }
        };

        if value_ty == XLTYPE_MULTI {
            // SAFETY: `value_ptr` points to xltypeMulti in this branch.
            let array = unsafe { (*value_ptr).val.array };
            let rows = array.rows;
            let cols = array.columns;
            let count = usize::try_from(rows.saturating_mul(cols)).unwrap_or(0);
            let mut mapped = Vec::with_capacity(count);
            for i in 0..count {
                // SAFETY: `lparray` points to `rows*cols` contiguous XLOPER12 entries.
                let item_ptr = unsafe { array.lparray.add(i) };
                let call_arg = call_arg_from_xloper(item_ptr, false);
                let eval_value = eval_surface_value(spec.function_id, &[call_arg]);
                mapped.push(eval_value_to_xloper(eval_value));
            }
            if used_temp {
                call_excel_free(&mut temp);
            }
            return alloc_result_multi(rows, cols, mapped);
        }

        let call_arg = call_arg_from_xloper(value_ptr, spec.preserve_refs);
        let eval_value = eval_surface_value(spec.function_id, &[call_arg]);
        if used_temp {
            call_excel_free(&mut temp);
        }
        return alloc_result(eval_value_to_xloper(eval_value));
    }

    let effective_len = effective_u_arg_len(spec, raw_args);
    let mut args = Vec::with_capacity(effective_len);
    for raw in raw_args.iter().take(effective_len) {
        let mut temp = XLOPER12 {
            val: XLOPER12Value { w: 0 },
            xltype: 0,
        };
        let mut value_ptr = *raw;
        let mut used_temp = false;

        if !value_ptr.is_null() {
            // SAFETY: `value_ptr` originates from Excel and is valid for this call.
            let ty = unsafe { (*value_ptr).xltype };
            if is_ref_type(ty) && !spec.preserve_refs {
                if !coerce_reference_to_value(value_ptr, &mut temp) {
                    return alloc_result(make_xloper_err(XLERR_VALUE));
                }
                value_ptr = &mut temp;
                used_temp = true;
            }
        }

        let call_arg = call_arg_from_xloper(value_ptr, spec.preserve_refs);
        if used_temp {
            call_excel_free(&mut temp);
        }
        args.push(call_arg);
    }

    let eval_value = eval_surface_value(spec.function_id, &args);
    alloc_result(eval_value_to_xloper(eval_value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn missing_xloper() -> XLOPER12 {
        XLOPER12 {
            val: XLOPER12Value { w: 0 },
            xltype: XLTYPE_MISSING,
        }
    }

    fn num_xloper(value: f64) -> XLOPER12 {
        XLOPER12 {
            val: XLOPER12Value { num: value },
            xltype: XLTYPE_NUM,
        }
    }

    #[test]
    fn effective_u_arg_len_trims_trailing_missing_args() {
        let present = num_xloper(1.0);
        let missing1 = missing_xloper();
        let missing2 = missing_xloper();
        let raw_args = [
            (&present as *const XLOPER12).cast_mut(),
            (&missing1 as *const XLOPER12).cast_mut(),
            (&missing2 as *const XLOPER12).cast_mut(),
        ];
        let spec = UExportSpec {
            function_id: "FUNC.TEXTJOIN",
            registration: RegistrationSpec {
                export_name: "OX_TEXTJOIN",
                type_text: "QUUU",
                function_name: "ox_TEXTJOIN",
                arg_names: "",
            },
            lift_policy: ULiftPolicy::ScalarOnly,
            preserve_refs: false,
            min_arity: 3,
            arg_count: 3,
        };
        assert_eq!(effective_u_arg_len(spec, &raw_args), 1.max(spec.min_arity));
    }

    #[test]
    fn effective_u_arg_len_keeps_internal_missing_args() {
        let present1 = num_xloper(1.0);
        let missing = missing_xloper();
        let present2 = num_xloper(2.0);
        let raw_args = [
            (&present1 as *const XLOPER12).cast_mut(),
            (&missing as *const XLOPER12).cast_mut(),
            (&present2 as *const XLOPER12).cast_mut(),
        ];
        let spec = UExportSpec {
            function_id: "FUNC.XLOOKUP",
            registration: RegistrationSpec {
                export_name: "OX_XLOOKUP",
                type_text: "QUUU",
                function_name: "ox_XLOOKUP",
                arg_names: "",
            },
            lift_policy: ULiftPolicy::ScalarOnly,
            preserve_refs: false,
            min_arity: 3,
            arg_count: 3,
        };
        assert_eq!(effective_u_arg_len(spec, &raw_args), 3);
    }
}

fn eval_q_unary_number_export(spec: QUnaryNumberExportSpec, value: f64) -> f64 {
    eval_surface_q_unary_number(spec.function_id, value).unwrap_or(f64::NAN)
}

fn eval_q_binary_number_export(spec: QBinaryNumberExportSpec, lhs: f64, rhs: f64) -> f64 {
    eval_surface_q_binary_number(spec.function_id, lhs, rhs).unwrap_or(f64::NAN)
}

fn eval_q_nullary_number_export(spec: QNullaryNumberExportSpec) -> f64 {
    eval_surface_q_nullary_number(spec.function_id).unwrap_or(f64::NAN)
}

#[unsafe(no_mangle)]
pub extern "system" fn OX_PROBE_RET_NIL() -> *mut XLOPER12 {
    alloc_result(make_xloper_nil())
}

#[unsafe(no_mangle)]
pub extern "system" fn OX_PROBE_ECHO(arg1: *mut XLOPER12) -> *mut XLOPER12 {
    probe_echo(arg1)
}

#[unsafe(no_mangle)]
pub extern "system" fn OX_PROBE_DESCRIBE(arg1: *mut XLOPER12) -> *mut XLOPER12 {
    probe_describe(arg1)
}

#[unsafe(no_mangle)]
pub extern "system" fn OX_PROBE_RET_ARRAY_NIL() -> *mut XLOPER12 {
    probe_ret_array_nil()
}

#[unsafe(no_mangle)]
pub extern "system" fn OX_PROBE_ARRAY_DESC(arg1: *mut XLOPER12) -> *mut XLOPER12 {
    probe_array_desc(arg1)
}

#[unsafe(no_mangle)]
pub extern "system" fn OX_GET_CELL(
    type_num: *mut XLOPER12,
    reference: *mut XLOPER12,
) -> *mut XLOPER12 {
    probe_info_binary(XLF_GET_CELL, type_num, reference, true)
}

#[unsafe(no_mangle)]
pub extern "system" fn OX_GET_DOCUMENT(
    type_num: *mut XLOPER12,
    name_text: *mut XLOPER12,
) -> *mut XLOPER12 {
    probe_info_binary(XLF_GET_DOCUMENT, type_num, name_text, true)
}

#[unsafe(no_mangle)]
pub extern "system" fn OX_GET_WORKBOOK(
    type_num: *mut XLOPER12,
    name_text: *mut XLOPER12,
) -> *mut XLOPER12 {
    probe_info_binary(XLF_GET_WORKBOOK, type_num, name_text, true)
}

#[unsafe(no_mangle)]
pub extern "system" fn OX_GET_WORKBOOK_ACTIVE(type_num: *mut XLOPER12) -> *mut XLOPER12 {
    probe_info_unary(XLF_GET_WORKBOOK, type_num, true)
}

#[unsafe(no_mangle)]
pub extern "system" fn OX_GET_WORKSPACE(type_num: *mut XLOPER12) -> *mut XLOPER12 {
    probe_info_unary(XLF_GET_WORKSPACE, type_num, true)
}

#[unsafe(no_mangle)]
pub extern "system" fn xlAutoOpen() -> i32 {
    let Some(module_path) = current_module_path() else {
        return 0;
    };
    if !register_all(&module_path) {
        return 0;
    }
    if !register_manual_probe_aliases(&module_path) {
        return 0;
    }
    if flag_experiments_enabled() && !register_flag_experiment_aliases(&module_path) {
        return 0;
    }
    1
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
    if base_type == XLTYPE_STR {
        // SAFETY: Union access is valid for `xltypeStr`.
        let pstr = unsafe { boxed.val.str };
        if !pstr.is_null() {
            // SAFETY: First code unit stores pascal-string length.
            let len = usize::from(unsafe { *pstr });
            // SAFETY: Allocation originates from `Box<[u16]>` in `make_xloper_str_from_utf16`.
            let raw_slice = std::ptr::slice_from_raw_parts_mut(pstr, len.saturating_add(1));
            unsafe {
                drop(Box::from_raw(raw_slice));
            }
        }
    }
    if base_type == XLTYPE_REF {
        let mref = unsafe { boxed.val.mref };
        if !mref.lpmref.is_null() {
            unsafe {
                drop(Box::from_raw(mref.lpmref));
            }
        }
    }
}










