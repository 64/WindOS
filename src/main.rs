#![no_std]
#![no_main]
#![feature(int_roundings)]
#![feature(extern_types)]

mod asm;
mod boot_allocator;
mod consts;
mod logging;
mod panic;
mod utils;

use log::{debug, error, info, trace, warn, LevelFilter};

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

    trace!(
        "This is a devicetree representation of a {}",
        fdt.root().model()
    );
    trace!(
        "...which is compatible with at least: {}",
        fdt.root().compatible().first()
    );
    trace!("...and has {} CPU(s)", fdt.cpus().count());
    trace!(
        "...and has at least one memory location at: {:#X}",
        fdt.memory().regions().next().unwrap().starting_address as usize
    );

    let chosen = fdt.chosen();
    if let Some(bootargs) = chosen.bootargs() {
        trace!("The bootargs are: {:?}", bootargs);
    }

    if let Some(stdout) = chosen.stdout() {
        trace!("It would write stdout to: {}", stdout.name);
    }

    let soc = fdt.find_node("/soc");
    trace!(
        "Does it have a `/soc` node? {}",
        if soc.is_some() { "yes" } else { "no" }
    );
    if let Some(soc) = soc {
        trace!("...and it has the following children:");
        for child in soc.children() {
            trace!("    {}", child.name);
        }
    }

    let mut boot_allocator = boot_allocator::BootAllocator::from_fdt(&fdt);

    trace!("alloc phys: {:?}", boot_allocator.alloc(0x1000));

    info!("Shutting down WindOS...");
    sbi::system_reset::system_reset(
        sbi::system_reset::ResetType::Shutdown,
        sbi::system_reset::ResetReason::NoReason,
    )
    .unwrap();
}
