use core::{cmp, ptr};

use log::info;
use xmas_elf::{
    program::{SegmentData, Type},
    ElfFile,
};

use crate::{
    boot_allocator::BootAllocator,
    page_table::{PageTable, EXECUTE, GLOBAL, READ, VALID, WRITE},
};

const KERNEL_BINARY: &[u8] = include_bytes!(env!("CARGO_BIN_FILE_KERNEL"));
const KERNEL_BASE: u64 = 0xffff_ffff_8000_0000;

pub fn load_kernel(root: &mut PageTable, alloc: &mut BootAllocator) -> (u64, u64) {
    info!("Loading kernel: {}", env!("CARGO_BIN_FILE_KERNEL"));

    let elf = ElfFile::new(KERNEL_BINARY).unwrap();
    for phdr in elf
        .program_iter()
        .filter(|phdr| phdr.get_type() == Ok(Type::Load))
    {
        // trace!("{phdr}");

        let base = phdr.virtual_addr();
        let mut data = match phdr.get_data(&elf) {
            Ok(SegmentData::Undefined(data)) => data,
            _ => panic!("no segment data"),
        };

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

            let data_to_copy = data.take(..cmp::min(data.len(), 0x1000)).unwrap();
            let addr = root.map_page(alloc, virt, VALID | GLOBAL | prot);

            unsafe {
                ptr::copy_nonoverlapping(data_to_copy.as_ptr(), addr, data_to_copy.len());
            }
        }
    }

    (elf.header.pt2.entry_point(), map_stack(root, alloc))
}

fn map_stack(root: &mut PageTable, alloc: &mut BootAllocator) -> u64 {
    let stack_pages = 8;
    for i in 0..stack_pages {
        let addr = KERNEL_BASE - 0x1000 * (i + 1) - 1;
        root.map_page(alloc, addr, VALID | GLOBAL | READ | WRITE);
    }

    // Map guard pages.
    root.map_page(alloc, KERNEL_BASE - 0x1000, VALID | GLOBAL | EXECUTE);
    root.map_page(
        alloc,
        KERNEL_BASE - 0x1000 * (stack_pages + 2),
        VALID | GLOBAL | EXECUTE,
    );

    KERNEL_BASE - 0x1000
}
