#![no_main]
#![no_std]

use hal::prelude::*;
use nrf52840_hal as hal;

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
        // TODO: Do something nice like blink red etc.?
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let peripherals = hal::pac::Peripherals::take().unwrap();
    let port0 = hal::gpio::p0::Parts::new(peripherals.P0);
    let mut led_red = port0.p0_26.into_push_pull_output(hal::gpio::Level::Low);
    let mut led_green = port0.p0_30.into_push_pull_output(hal::gpio::Level::Low);
    let mut led_blue = port0.p0_06.into_push_pull_output(hal::gpio::Level::Low);

    //let core_peripherals = hal::pac::CorePeripherals::take().unwrap();

    // TIMER0 is reserved by Softdevice?
    //let mut timer = hal::Timer::new(peripherals.TIMER0);
    let mut timer = hal::Timer::new(peripherals.TIMER1);
    // There seems to be more to timers that I don't get yet.
    // https://devzone.nordicsemi.com/f/nordic-q-a/1160/soft-device-and-timers---how-do-they-work-together
    // Let's do cycle delays instead

    loop {
        led_green.set_state(PinState::High).unwrap();
        led_blue.set_state(PinState::High).unwrap();
        led_red.set_state(PinState::Low).unwrap();
        timer.delay_ms(1000u32);

        led_red.set_state(PinState::High).unwrap();
        led_blue.set_state(PinState::High).unwrap();
        led_green.set_state(PinState::Low).unwrap();
        timer.delay_ms(1000u32);

        led_red.set_state(PinState::High).unwrap();
        led_green.set_state(PinState::High).unwrap();
        led_blue.set_state(PinState::Low).unwrap();
        timer.delay_ms(1000u32);
    }
}
