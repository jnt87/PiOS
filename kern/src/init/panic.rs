use core::panic::PanicInfo;
use crate::console::{kprint, kprintln, CONSOLE};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    kprintln!("-----PANIC-----");
    if let Some(location) = _info.location() {
        kprintln!("FILE: {:?}", location.file());
        kprintln!("LINE: {:?}", location.line());
        kprintln!("COL: {:?}", location.column());
    }
    kprintln!("");
    if let pload = _info.payload().downcast_ref::<&str>().unwrap() {
        kprintln!("{:?}", pload);
    } else {
        kprintln!("panic occured");
    }
    loop {}
}
