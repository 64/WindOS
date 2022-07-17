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
    pub fn from_fdt(fdt: &fdt::Fdt) -> Self {
        let mut out = Self {
            regions: ArrayVec::new(),
        };

        let kernel_start = unsafe { &__prekernel_start_phys as *const Symbol as usize };
        let kernel_end = unsafe { (&__prekernel_end_phys as *const Symbol as usize).next_multiple_of(PAGE_SIZE) };

        for rg in fdt.memory().regions() {
            let mut start = rg.starting_address as usize;
            let mut size = rg.size.unwrap();

            if start <= kernel_start && kernel_start <= start + size {
                let off = kernel_end.saturating_sub(start);
                if off == 0 {
                    continue;
                }

                start = kernel_end;
                size -= off;
            }

            out.regions.push(Region { start, size });
        }

        trace!(target: "boot allocator", "dumping memory regions... kernel is at {:#x}..{:#x}", kernel_start, kernel_end);

        for rg in &out.regions {
            trace!(
                "    region: {:#x}-{:#x} of size {:#x}",
                rg.start, rg.start + rg.size,
                rg.size
            );
        }

        out
    }

    pub fn alloc(&mut self, requested_size: usize) -> *mut u8 {
        let size = requested_size.next_multiple_of(PAGE_SIZE);
        let rg = self
            .regions
            .iter_mut()
            .find(|rg| rg.size >= size)
            .expect("cannot satisfy allocation");

        let addr = rg.start;
        rg.size -= size;
        rg.start = rg.start + size;
        addr as *mut u8
    }
}
