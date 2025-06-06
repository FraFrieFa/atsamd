#![no_std]
#![no_main]

//! This example shows a simple transfer operation with an slave device.
//! The ItsyBitsy will send a simple Hello World message, and the slave
//! is expected to send a response. After the transaction, the response
//! from the slave is echoed in the default UART.

#[cfg(not(feature = "use_semihosting"))]
use panic_halt as _;

#[cfg(feature = "use_semihosting")]
use panic_semihosting as _;

use itsybitsy_m4 as bsp;

use bsp::{
    entry,
    hal::{
        clock::GenericClockController,
        delay::Delay,
        nb,
        pac::{CorePeripherals, Peripherals},
        prelude::*,
    },
    spi_master,
};

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.gclk,
        &mut peripherals.mclk,
        &mut peripherals.osc32kctrl,
        &mut peripherals.oscctrl,
        &mut peripherals.nvmctrl,
    );
    let pins = bsp::Pins::new(peripherals.port);
    let mut delay = Delay::new(core.SYST, &mut clocks);
    let mut serial = bsp::uart(
        &mut clocks,
        115200.Hz(),
        peripherals.sercom3,
        &mut peripherals.mclk,
        pins.d0_rx,
        pins.d1_tx,
    );
    let mut spi1 = spi_master(
        &mut clocks,
        4.MHz(),
        peripherals.sercom1,
        &mut peripherals.mclk,
        pins.sck,
        pins.mosi,
        pins.miso,
    );
    let mut red_led = pins.d13.into_push_pull_output();
    let mut cs = pins.a2.into_push_pull_output();
    let message = b"hello world";
    loop {
        cs.set_low().unwrap();
        if let Ok(slave_msg) = spi1.transfer(&mut message.clone()) {
            cs.set_high().unwrap();
            for c in slave_msg {
                let _ = nb::block!(serial.write(*c));
            }
        }
        delay.delay_ms(200u8);
        red_led.set_high().unwrap();
        delay.delay_ms(200u8);
        red_led.set_low().unwrap();
    }
}
