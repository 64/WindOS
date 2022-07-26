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

impl BootAllocator {
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

        let ptr = addr as *mut u8;
        unsafe {
            ptr.write_bytes(0, requested_size);
        }
        ptr
    }

    pub fn dump_regions(&self) {
        trace!("dumping memory regions...");
        for (i, rg) in self.regions.iter().enumerate() {
            trace!(
                "    region: {:#x}-{:#x} of size {:#x} {}",
                rg.start,
                rg.start + rg.size,
                rg.size,
                if i == 0 { "(prekernel)" } else { "" },
            );
        }
    }
}
