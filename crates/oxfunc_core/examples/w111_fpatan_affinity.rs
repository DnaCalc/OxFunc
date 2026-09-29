//! Public own-process CPU-affinity experiment; never reads another process.
use core::arch::asm;
#[link(name="kernel32")]
unsafe extern "system" {
    fn GetCurrentThread()->*mut core::ffi::c_void;
    fn SetThreadAffinityMask(thread:*mut core::ffi::c_void,mask:usize)->usize;
}
fn fpatan(x:f64)->u64 {
    let mut saved=0u16;let cw=0x033fu16;let mut value=0.0f64;
    unsafe {asm!("fnstcw word ptr [{saved}]","fldcw word ptr [{cw}]","fld qword ptr [{x}]","fld1","fpatan","fstp qword ptr [{value}]","fldcw word ptr [{saved}]",
        saved=in(reg)&mut saved,cw=in(reg)&cw,x=in(reg)&x,value=in(reg)&mut value,options(nostack,preserves_flags));}
    value.to_bits()
}
fn main(){
    let inputs=[0xc0000d685e592174u64,0xc0001dc1079c3c90,0xc00422f5b8083883,0xbff188b0ca40f8e7];
    let thread=unsafe{GetCurrentThread()};let mut observations=Vec::new();
    for cpu in 0..usize::BITS {
        let saved=unsafe{SetThreadAffinityMask(thread,1usize<<cpu)};
        if saved==0 {continue;}
        let bits:Vec<_>=inputs.iter().map(|b|format!("0x{:016x}",fpatan(f64::from_bits(*b)))).collect();
        assert_ne!(unsafe{SetThreadAffinityMask(thread,saved)},0);
        observations.push(serde_json::json!({"logical_cpu":cpu,"bits":bits}));
    }
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({"inputs":inputs.iter().map(|b|format!("0x{b:016x}")).collect::<Vec<_>>(),"observations":observations})).unwrap());
}
