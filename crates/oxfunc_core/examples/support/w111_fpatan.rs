//! A direct public-ISA baseline independent of the evolving production candidate.
pub fn direct(x:f64)->f64 {
    let mut saved=0u16;let cw=0x033fu16;let mut value=0.0f64;
    // SAFETY: balanced stack, live locals, restored calling-thread control word.
    unsafe {core::arch::asm!(
        "fnstcw word ptr [{saved}]","fldcw word ptr [{cw}]",
        "fld qword ptr [{x}]","fld1","fpatan","fstp qword ptr [{value}]",
        "fldcw word ptr [{saved}]",
        saved=in(reg)&mut saved,cw=in(reg)&cw,x=in(reg)&x,value=in(reg)&mut value,
        options(nostack,preserves_flags));}
    value
}
