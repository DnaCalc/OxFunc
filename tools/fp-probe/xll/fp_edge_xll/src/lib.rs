#![allow(non_snake_case)]

use std::ffi::c_void;

use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    GetModuleFileNameW, GetModuleHandleExW,
};

unsafe extern "system" {
    fn oxfp_register_all(module_path: *const u16) -> i32;
}

fn payload_from_f64(payload: f64) -> u64 {
    if !payload.is_finite() {
        return 1;
    }
    let rounded = payload.round();
    if rounded <= 0.0 {
        1
    } else if rounded >= (u64::MAX as f64) {
        u64::MAX
    } else {
        rounded as u64
    }
}

fn qnan_with_payload(payload: u64) -> f64 {
    let masked = payload & 0x0007_FFFF_FFFF_FFFF;
    f64::from_bits(0x7FF8_0000_0000_0000 | masked)
}

fn snan_with_payload(payload: u64) -> f64 {
    let mut masked = payload & 0x0003_FFFF_FFFF_FFFF;
    if masked == 0 {
        masked = 1;
    }
    f64::from_bits(0x7FF0_0000_0000_0000 | masked)
}

fn current_module_path_wide_nul() -> Option<Vec<u16>> {
    let mut module: HMODULE = 0 as HMODULE;
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
pub extern "system" fn OXFP_NEG_ZERO() -> f64 {
    -0.0
}

#[unsafe(no_mangle)]
pub extern "system" fn OXFP_POS_INF() -> f64 {
    f64::INFINITY
}

#[unsafe(no_mangle)]
pub extern "system" fn OXFP_NEG_INF() -> f64 {
    f64::NEG_INFINITY
}

#[unsafe(no_mangle)]
pub extern "system" fn OXFP_QNAN(payload: f64) -> f64 {
    qnan_with_payload(payload_from_f64(payload))
}

#[unsafe(no_mangle)]
pub extern "system" fn OXFP_SNAN(payload: f64) -> f64 {
    snan_with_payload(payload_from_f64(payload))
}

#[unsafe(no_mangle)]
pub extern "system" fn OXFP_BITS_ECHO(value: f64) -> f64 {
    value
}

#[unsafe(no_mangle)]
pub extern "system" fn xlAutoOpen() -> i32 {
    let module_path = match current_module_path_wide_nul() {
        Some(p) => p,
        None => return 0,
    };
    // SAFETY: C++ bridge is compiled into this binary and expects a NUL-terminated UTF-16 path.
    let ok = unsafe { oxfp_register_all(module_path.as_ptr()) };
    if ok == 1 { 1 } else { 0 }
}

#[unsafe(no_mangle)]
pub extern "system" fn xlAutoClose() -> i32 {
    1
}
