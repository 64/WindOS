#![no_std]
#![no_main]
#![feature(slice_take)]
#![feature(int_roundings)]
#![feature(extern_types)]
#![feature(fn_align)]

mod boot_allocator;
mod loader;
mod logging;
mod page_table;
mod panic;

use log::{info, LevelFilter};
use riscv::register::{mtvec::TrapMode, scause, sepc, stval, stvec};

use self::logging::UartLogger;
use core::fmt::Write;

core::arch::global_asm!(include_str!("boot.asm"));

fn phys_to_virt(phys: u64) -> u64 {
    phys + 0xffff_ffc0_0000_0000
}

#[no_mangle]
extern "C" fn prekernel_main(hart_id: u64, dtb: *const u8) {
    unsafe {
        stvec::write(trap_handler as usize, TrapMode::Direct);
    }

    logging::init(LevelFilter::Trace);
    info!("Booting WindOS...");

    let fdt = unsafe { fdt::Fdt::from_ptr(dtb).unwrap() };
    let mut boot_allocator = boot_allocator::BootAllocator::from_fdt(&fdt);

    let mut root_table = page_table::PageTable::new_at(boot_allocator.alloc(0x1000));
    root_table.linear_map_all();

    let (entry, new_sp) = loader::load_kernel(&mut root_table, &mut boot_allocator);
    let stvec = entry;
    assert!(stvec & 0b11 == 0);

    let satp = (8 << 60) | (root_table as *const page_table::PageTable as usize >> 12);

    unsafe {
        // Set the trap handler to the kernel's entry point. When paging is enabled, the
        // next instruction fetch will trap and enter the kernel.
        core::arch::asm!(
            "
                mv sp, {sp}
                csrw stvec, {stvec}
                csrw satp, {satp}
                unimp
            ",
            stvec = in(reg) stvec,
            satp = in(reg) satp,
            sp = in(reg) new_sp,
            in("a0") hart_id,
            in("a1") phys_to_virt(dtb as u64),
            in("a2") phys_to_virt(boot_allocator as *mut _ as u64),
            options(noreturn, nostack)
        );
    }
}

#[repr(align(4))]
fn trap_handler() {
    let _ = writeln!(
        UartLogger,
        concat!(
            "---------------------------\n",
            "[\x1b[31mPRE-KERNEL PANIC\x1b[0m] \x1b[31m{:?}\x1b[0m with stval = {:#x}, sepc = {:#x}",
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
