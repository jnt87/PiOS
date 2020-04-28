#![feature(alloc_error_handler)]
#![feature(const_fn)]
#![feature(decl_macro)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(optin_builtin_traits)]
#![feature(ptr_internals)]
#![feature(raw_vec_internals)]
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

const MMIO_BASE: usize = 0x3F000000;

const GPIO_BASE: usize = MMIO_BASE + 0x200000;

const GPIO_FSEL1: *mut u32 = (GPIO_BASE + 0x04) as *mut u32;
const GPIO_SET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
const GPIO_CLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;

#[cfg(not(test))]
mod init;

extern crate alloc;

pub mod allocator;
pub mod console;
pub mod fs;
pub mod mutex;
pub mod shell;
pub mod param;
pub mod process;
pub mod traps;
pub mod vm;

use console::kprintln;

// You need to add dependencies here to
extern crate pi;
use crate::shell::shell;
use pi::timer;
use core::time::Duration;
use pi::gpio::Gpio;
use pi::uart::MiniUart;
use core::fmt::Write;
use allocator::Allocator;
use fs::FileSystem;
use pi::atags::Atags;
use fs::sd::Sd;
use fat32::traits::BlockDevice;
use aarch64;
use aarch64::brk;

use process::GlobalScheduler;
use traps::irq::Irq;
use vm::VMManager;

#[cfg_attr(not(test), global_allocator)]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();
pub static FILESYSTEM: FileSystem = FileSystem::uninitialized();
pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();
pub static VMM: VMManager = VMManager::uninitialized();
pub static IRQ: Irq = Irq::uninitialized();

fn kmain() -> ! {
    unsafe {
        ALLOCATOR.initialize();
        FILESYSTEM.initialize();
    }
    for tag in Atags::get() {
        kprintln!("{:?}", tag);
    }
    unsafe { kprintln!("Current EL: {:?}", aarch64::current_el()) };
    unsafe{ asm!("brk 2" :::: "volatile"); }
    kprintln!("Welcome to cs3210!");
    loop {
        shell::shell("> ");
    }
}
