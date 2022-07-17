use log::info;
use xmas_elf::ElfFile;

use crate::boot_allocator::BootAllocator;
use crate::page_table::{PageTable, VALID, READ, GLOBAL, WRITE, EXECUTE};

const KERNEL_BINARY: &[u8] = include_bytes!(env!("CARGO_BIN_FILE_KERNEL"));

pub fn load_kernel(root: &mut PageTable, alloc: &mut BootAllocator) {
    info!("Loading kernel...");
    let load_base = 0xffff_ffff_8000_0000;

    let elf = ElfFile::new(KERNEL_BINARY).unwrap();
    for phdr in elf.program_iter() {
        info!("{phdr}");

        let base = load_base + phdr.virtual_addr();
        for virt in (base..base + phdr.mem_size()).step_by(0x1000) {
            let mut prot = 0;
            if phdr.flags().is_read() {
                prot |= READ;
            }
            if phdr.flags().is_write() {
                prot |= WRITE;
            }
            if phdr.flags().is_execute() {
                prot |= EXECUTE;
            }

            let phys = alloc.alloc(0x1000);
            info!("mapping {:#x} to {:#x}", virt, phys as u64);
            root.map_page(alloc, virt, phys as u64, VALID | GLOBAL | prot);
        }
    }
}
