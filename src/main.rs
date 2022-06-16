#![no_std]
#![no_main]

mod asm;
mod logging;
mod panic;

use log::{info, LevelFilter, trace, debug, warn, error};

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
                    loglevel = match v {
                        "trace" => LevelFilter::Trace,
                        "debug" => LevelFilter::Debug,
                        "info" => LevelFilter::Info,
                        "warn" => LevelFilter::Warn,
                        "error" => LevelFilter::Error,
                        _ => continue,
                    };
                }
                _ => continue,
            }
        }
    }

    logging::init(loglevel);

    info!("Booting WindOS...");

    debug!("This is a devicetree representation of a {}", fdt.root().model());
    debug!("...which is compatible with at least: {}", fdt.root().compatible().first());
    debug!("...and has {} CPU(s)", fdt.cpus().count());
    debug!(
        "...and has at least one memory location at: {:#X}",
        fdt.memory().regions().next().unwrap().starting_address as usize
    );

    let chosen = fdt.chosen();
    if let Some(bootargs) = chosen.bootargs() {
        debug!("The bootargs are: {:?}", bootargs);
    }

    if let Some(stdout) = chosen.stdout() {
        debug!("It would write stdout to: {}", stdout.name);
    }

    let soc = fdt.find_node("/soc");
    debug!("Does it have a `/soc` node? {}", if soc.is_some() { "yes" } else { "no" });
    if let Some(soc) = soc {
        debug!("...and it has the following children:");
        for child in soc.children() {
            debug!("    {}", child.name);
        }
    }

    sbi::system_reset::system_reset(
        sbi::system_reset::ResetType::Shutdown,
        sbi::system_reset::ResetReason::NoReason,
    )
    .unwrap();
}
