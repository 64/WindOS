#![no_std]
#![no_main]

core::arch::global_asm!(include_str!("asm/boot.asm"));

use core::panic::PanicInfo;

#[panic_handler]
fn abort(_info: &PanicInfo) -> ! {
    loop {}
}
