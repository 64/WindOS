use core::panic::PanicInfo;

use log::error;

#[panic_handler]
fn abort(info: &PanicInfo) -> ! {
    error!("kernel panic: {info}");
    loop {}
}
