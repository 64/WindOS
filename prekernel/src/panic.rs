use crate::logging::UartLogger;
use core::{fmt::Write, panic::PanicInfo};

#[panic_handler]
fn abort(info: &PanicInfo) -> ! {
    let _ = writeln!(UartLogger, "\x1b[31mPREKERNEL PANIC:\x1b[0m {info}");

    sbi::system_reset::system_reset(
        sbi::system_reset::ResetType::Shutdown,
        sbi::system_reset::ResetReason::NoReason,
    )
    .unwrap_or_else(|_| loop {});

    unreachable!()
}
