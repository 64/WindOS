#![no_std]
#![no_main]
#![feature(fn_align)]
#![feature(int_roundings)]

use log::{info, LevelFilter};
use riscv::register::{mtvec::TrapMode, scause, sepc, stval, stvec};

use crate::{boot_allocator::BootAllocator, logging::UartLogger};
use core::fmt::Write;

mod addr;
mod boot_allocator;
mod logging;
mod panic;

#[no_mangle]
#[repr(align(4))]
extern "C" fn kernel_main(_hart_id: u64, dtb: *const u8, alloc: &'static mut BootAllocator) {
    unsafe {
        core::arch::asm!("lla gp, __global_pointer$", options(nomem, nostack));
        stvec::write(early_trap_handler as usize, TrapMode::Direct);
    }

    parse_fdt(dtb);
    info!("Starting WindOS kernel...");

    alloc.dump_regions();

    unsafe {
        core::ptr::read_volatile(3 as *const u8);
    }

    info!("Shutting down WindOS...");
    sbi::system_reset::system_reset(
        sbi::system_reset::ResetType::Shutdown,
        sbi::system_reset::ResetReason::NoReason,
    )
    .unwrap();
}

#[repr(align(4))]
fn early_trap_handler() {
    let _ = writeln!(
        UartLogger,
        concat!(
            "---------------------------\n",
            "[\x1b[31mPANIC\x1b[0m] \x1b[31m{:?}\x1b[0m with stval = {:#x}, sepc = {:#x}",
        ),
        scause::read().cause(),
        stval::read(),
        sepc::read(),
    );

    let _ = sbi::system_reset::system_reset(
        sbi::system_reset::ResetType::Shutdown,
        sbi::system_reset::ResetReason::NoReason,
    );

    loop {}
}

fn parse_fdt(dtb: *const u8) {
    let fdt = unsafe { fdt::Fdt::from_ptr(dtb).unwrap() };

    let mut loglevel = LevelFilter::Info;

    let chosen = fdt.chosen();
    if let Some(bootargs) = chosen.bootargs() {
        for arg in bootargs.split_ascii_whitespace() {
            let (k, v) = if let Some(kv) = arg.split_once('=') {
                kv
            } else {
                continue;
            };

            match k {
                "loglevel" => {
                    loglevel = v
                        .parse::<LevelFilter>()
                        .expect("unexpected loglevel argument");
                }
                _ => continue,
            }
        }
    }

    logging::init(loglevel);
}
