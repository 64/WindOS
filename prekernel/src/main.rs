#![no_std]
#![no_main]
#![feature(int_roundings)]
#![feature(extern_types)]

mod boot_allocator;
mod logging;
mod page_table;
mod loader;
mod panic;

use xmas_elf::ElfFile;
use log::{info, LevelFilter};

core::arch::global_asm!(include_str!("boot.asm"));

#[no_mangle]
extern "C" fn prekernel_main(_hart_id: u64, dtb: *const u8) {
    logging::init(LevelFilter::Trace);
    info!("Booting WindOS...");

    let fdt = unsafe { fdt::Fdt::from_ptr(dtb).unwrap() };
    let mut boot_allocator = boot_allocator::BootAllocator::from_fdt(&fdt);

    let mut root_table = page_table::PageTable::new_at(boot_allocator.alloc(0x1000));
    root_table.linear_map_all();

    loader::load_kernel(&mut root_table, &mut boot_allocator);

    let satp = (8 << 60) | (root_table as *const page_table::PageTable as usize >> 12);
    info!("New SATP = {:#x}", satp);

    unsafe {
        core::arch::asm!(
            "
                csrc sstatus, {mxr}
                csrw satp, {satp}
                sfence.vma
            ",
            mxr = in(reg) 1 << 19,
            satp = in(reg) satp
        );
    }
    loop {}

    info!("Shutting down WindOS...");
    sbi::system_reset::system_reset(
        sbi::system_reset::ResetType::Shutdown,
        sbi::system_reset::ResetReason::NoReason,
    )
    .unwrap();
}
