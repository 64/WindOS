pub fn phys_to_virt<T>(p: u64) -> *const T {
    (p + 0xffff_ffc0_0000_0000) as *const T
}

pub fn phys_to_virt_mut<T>(p: u64) -> *mut T {
    (p + 0xffff_ffc0_0000_0000) as *mut T
}
