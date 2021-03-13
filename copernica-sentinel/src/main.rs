#![feature(default_alloc_error_handler)]
#![no_std]
#![no_main]
use cortex_m_rt::{entry, exception};
use cortex_m::{asm, peripheral::SCB};
use rtt_target::{rprintln, rtt_init_print};
#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello world with a stacktrace!");
    SCB::set_pendsv();
    rprintln!("after PendSV");
    exit()
}
#[exception]
fn PendSV() {
    rprintln!("PendSV");
    panic!()
}
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rprintln!("{}", info);
    exit()
}
fn exit() -> ! {
    loop {
        asm::bkpt()
    }
}
