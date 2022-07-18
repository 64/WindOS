use core::mem;

use arrayvec::ArrayVec;
use log::trace;

pub const NREGIONS: usize = 16;
pub const PAGE_SIZE: usize = 0x1000;

pub struct BootAllocator {
    regions: ArrayVec<Region, NREGIONS>,
}

#[derive(Debug)]
struct Region {
    start: usize,
    size: usize,
}

extern "C" {
    type Symbol;
    static __prekernel_start_phys: Symbol;
    static __prekernel_end_phys: Symbol;
}

impl BootAllocator {
    pub fn from_fdt(fdt: &fdt::Fdt) -> &'static mut BootAllocator {
        let mut out = Self {
            regions: ArrayVec::new(),
        };

        let prekernel_start = unsafe { &__prekernel_start_phys as *const Symbol as usize };
        let prekernel_end = unsafe {
            (&__prekernel_end_phys as *const Symbol as usize).next_multiple_of(PAGE_SIZE)
        };

        // The first region holds the pre-kernel data. We don't allocate from here
        // until the main kernel starts.
        out.regions.push(Region {
            start: prekernel_start,
            size: prekernel_end - prekernel_start,
        });

        for rg in fdt.memory().regions() {
            let mut start = rg.starting_address as usize;
            let mut size = rg.size.unwrap();

            if start <= prekernel_start && prekernel_start <= start + size {
                let off = prekernel_end.saturating_sub(start);
                if off == 0 {
                    continue;
                }

                start = prekernel_end;
                size -= off;
            }

            out.regions.push(Region { start, size });
        }

        // Make an allocation to store ourselves in. This ensures that the memory for
        // BootAllocator itself isn't reclaimed by the kernel.
        let allocator = out.alloc(mem::size_of::<BootAllocator>()) as *mut BootAllocator;
        unsafe {
            allocator.write(out);
            &mut *allocator
        }
    }

    pub fn alloc(&mut self, requested_size: usize) -> *mut u8 {
        let size = requested_size.next_multiple_of(PAGE_SIZE);
        let rg = self.regions[1..]
            .iter_mut()
            .find(|rg| rg.size >= size)
            .expect("cannot satisfy allocation");

        let addr = rg.start;
        rg.size -= size;
        rg.start = rg.start + size;

        let ptr = addr as *mut u8;
        unsafe {
            ptr.write_bytes(0, requested_size);
        }
        ptr
    }
}
