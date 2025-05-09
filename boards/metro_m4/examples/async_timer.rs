#![no_std]
#![no_main]

#[cfg(not(feature = "use_semihosting"))]
use panic_halt as _;
#[cfg(feature = "use_semihosting")]
use panic_semihosting as _;

use bsp::{hal, pac, pin_alias};
use hal::fugit::MillisDurationU32;
use hal::{
    clock::GenericClockController, ehal::digital::StatefulOutputPin, pac::Tc4, timer::TimerCounter,
};
use metro_m4 as bsp;

atsamd_hal::bind_interrupts!(struct Irqs {
    TC4 => atsamd_hal::timer::InterruptHandler<Tc4>;
});

#[embassy_executor::main]
async fn main(_s: embassy_executor::Spawner) {
    let mut peripherals = pac::Peripherals::take().unwrap();
    let _core = pac::CorePeripherals::take().unwrap();

    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.gclk,
        &mut peripherals.mclk,
        &mut peripherals.osc32kctrl,
        &mut peripherals.oscctrl,
        &mut peripherals.nvmctrl,
    );
    let pins = bsp::Pins::new(peripherals.port);
    let mut red_led: bsp::RedLed = pin_alias!(pins.red_led).into();

    // configure a clock for the TC4 and TC5 peripherals
    let timer_clock = clocks.gclk0();
    let tc45 = &clocks.tc4_tc5(&timer_clock).unwrap();

    // instantiate a timer object for the TC4 peripheral
    let timer = TimerCounter::tc4_(tc45, peripherals.tc4, &mut peripherals.mclk);
    let mut timer = timer.into_future(Irqs);

    loop {
        timer
            .delay(MillisDurationU32::from_ticks(500).convert())
            .await;
        red_led.toggle().unwrap();
    }
}
