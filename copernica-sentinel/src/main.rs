#![feature(default_alloc_error_handler)]
#![no_std]
#![no_main]
use {
    cortex_m_rt::{entry, exception},
    cortex_m::{asm},
    rtt_target::{rprintln, rtt_init_print},
    embedded_hal::{
        digital::v2::OutputPin,
    },
    nb::block,
    stm32f1xx_hal::{
        prelude::*,
        timer::Timer,
    },
};
#[entry]
fn main() -> ! {
    rtt_init_print!();
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f1xx_hal::pac::Peripherals::take().unwrap();
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);
    assert!(clocks.usbclk_valid());
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut led = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
    let mut timer = Timer::syst(cp.SYST, &clocks).start_count_down(3.hz());
    loop {
        rprintln!("Off");
        block!(timer.wait()).unwrap();
        led.set_high().unwrap();
        rprintln!("On");
        block!(timer.wait()).unwrap();
        led.set_low().unwrap();
    }
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
