use log::info;

use crate::boot_allocator::BootAllocator;

#[repr(align(4096))]
pub struct PageTable {
    entries: [u64; 512],
}

pub const VALID: u64 = 1;
pub const READ: u64 = 1 << 1;
pub const WRITE: u64 = 1 << 2;
pub const EXECUTE: u64 = 1 << 3;
pub const GLOBAL: u64 = 1 << 5;
pub const DIRTY: u64 = 1 << 6;
pub const ACCESSED: u64 = 1 << 7;
pub const PTE_PPN_MASK: u64 = 0xff_ffff_ffff_fc00;
pub const USEFUL_FLAGS_MASK: u64 = 0x3f;

impl PageTable {
    pub fn new() -> Self {
        Self { entries: [0; 512] }
    }

    pub fn new_at(addr: *mut u8) -> &'static mut Self {
        assert!(addr as usize & 0xfff == 0);
        unsafe {
            let addr = addr as *mut PageTable;
            addr.write_volatile(Self::new());
            &mut *addr
        }
    }

    pub fn linear_map_all(&mut self) {
        // Linearly map 64GiB of physical memory to 0xffff_ffc0_0000_0000
        let off = 256;
        let flags = VALID | READ | WRITE | GLOBAL | DIRTY | ACCESSED;
        for i in off..(off + 64) {
            let ppn_2 = (i - off << 28) as u64;
            self.entries[i] = ppn_2 | flags;
        }
    }

    pub fn map_page(&mut self, alloc: &mut BootAllocator, virt: u64, flags: u64) -> *mut u8 {
        // info!("mapping {:#x} with flags {:#b}", virt, flags);
        assert!((flags & VALID == VALID) && flags & (READ | WRITE | EXECUTE) != 0);
        self.do_map(alloc, virt & !0xfff, flags & USEFUL_FLAGS_MASK, 2)
    }

    fn do_map(
        &mut self,
        alloc: &mut BootAllocator,
        virt: u64,
        flags: u64,
        depth: usize,
    ) -> *mut u8 {
        let vpn = [
            (virt >> 12) & 0x1ff,
            (virt >> 21) & 0x1ff,
            (virt >> 30) & 0x1ff,
        ];

        let leaf = depth == 0;
        let entry = &mut self.entries[vpn[depth] as usize];

        let target = if leaf {
            if *entry & VALID == 0 {
                assert!(*entry & VALID == 0);
                let addr = alloc.alloc(0x1000) as u64;
                // trace!("allocating new page at {:#x}", addr);
                Some(addr)
            } else {
                unreachable!()
                // let existing_flags = *entry & USEFUL_FLAGS_MASK;
                // // trace!("hit existing mapping with flags {:#b}",
                // existing_flags); assert_eq!(existing_flags,
                // flags); Some(*entry & PTE_PPN_MASK << 2)
            }
        } else if *entry & VALID == 0 {
            let addr = alloc.alloc(0x1000);
            PageTable::new_at(addr);
            Some(addr as u64)
        } else {
            None
        };

        if let Some(target) = target {
            // Write an entry into the page table.
            *entry = (target >> 2 & PTE_PPN_MASK) | if leaf { flags } else { 0 };
        }
        *entry |= VALID;

        if leaf {
            target.unwrap() as *mut u8
        } else {
            // Read out the address of the page table, and recurse down.
            let addr = (*entry & PTE_PPN_MASK) << 2;
            let page_table = unsafe { &mut *(addr as *mut PageTable) };
            assert!(!core::ptr::eq(self, page_table));
            page_table.do_map(alloc, virt, flags, depth - 1)
        }
    }
}
