#![no_std]
#![no_main]

use log::{info, LevelFilter};

mod logging;
mod panic;

#[no_mangle]
extern "C" fn kernel_main(_hart_id: u64, dtb: *const u8) {
    let fdt = unsafe { fdt::Fdt::from_ptr(dtb).unwrap() };

    let mut loglevel = LevelFilter::Info;

    let chosen = fdt.chosen();
    if let Some(bootargs) = chosen.bootargs() {
        for arg in bootargs.split_ascii_whitespace() {
            let (k, v) = if let Some(kv) = arg.split_once('=') {
                kv
            } else {
                continue;
            };

            match k {
                "loglevel" => {
                    loglevel = v
                        .parse::<LevelFilter>()
                        .expect("unexpected loglevel argument");
                }
                _ => continue,
            }
        }
    }

    logging::init(loglevel);

    info!("Booting WindOS...");
}
