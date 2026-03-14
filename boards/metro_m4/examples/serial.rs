#![no_std]
#![no_main]

use metro_m4 as bsp;

use bsp::hal;

#[cfg(not(feature = "use_semihosting"))]
use panic_halt as _;
#[cfg(feature = "use_semihosting")]
use panic_semihosting as _;

use bsp::{entry, pin_alias};
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::ehal::delay::DelayNs;
use hal::pac::{CorePeripherals, Peripherals};
use hal::sercom_v2::uart::Usart;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    // Shadow PAC handle used for sercom_v2 `enable_from_pac` after individual
    // fields have been moved out of `peripherals` during setup.
    let peripherals_pac = unsafe { Peripherals::steal() };
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.gclk,
        &mut peripherals.mclk,
        &mut peripherals.osc32kctrl,
        &mut peripherals.oscctrl,
        &mut peripherals.nvmctrl,
    );

    let pins = bsp::Pins::new(peripherals.port);
    let uart_rx = pin_alias!(pins.uart_rx);
    let uart_tx = pin_alias!(pins.uart_tx);
    let mut delay = Delay::new(core.SYST, &mut clocks);
    let gclk0 = clocks.gclk0();
    let core_clock_hz = clocks.sercom3_core(&gclk0).unwrap().freq().to_Hz();
    let mut uart = Usart::default()
        .rx(uart_rx)
        .tx(uart_tx)
        .baud(9_600)
        .to_config()
        .enable_from_pac(&peripherals_pac, core_clock_hz);

    loop {
        for byte in b"Hello, world!" {
            uart.write_u8(*byte);
        }
        delay.delay_ms(1000);
    }
}
