#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(optin_builtin_traits)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

const MMIO_BASE: usize = 0x3F000000;

const GPIO_BASE: usize = MMIO_BASE + 0x200000;

const GPIO_FSEL1: *mut u32 = (GPIO_BASE + 0x04) as *mut u32;
const GPIO_SET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
const GPIO_CLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;

#[cfg(not(test))]
mod init;

pub mod console;
pub mod mutex;
pub mod shell;

use console::kprintln;

// FIXME: You need to add dependencies here to
// test your drivers (Phase 2). Add them as needed.
extern crate pi;
use crate::shell::shell;
use pi::timer;
use core::time::Duration;
use pi::gpio::Gpio;
use pi::uart::MiniUart;
use core::fmt::Write;




unsafe fn kmain() -> ! {
    /*
    let timer = Duration::from_nanos(1000000);
    let mut a = Gpio::new(5).into_output(); 
    let mut b = Gpio::new(6).into_output();
    let mut c = Gpio::new(13).into_output();
    let mut d = Gpio::new(19).into_output();
    let mut e = Gpio::new(26).into_output();
    loop {  
        a.set();
        c.set();
        e.set();
        b.clear();
        d.clear();
        timer::spin_sleep(timer);
        a.clear();
        c.clear();
        e.clear();
        b.set();
        d.set();
        timer::spin_sleep(timer);
    }
    */
    shell("->");
}
