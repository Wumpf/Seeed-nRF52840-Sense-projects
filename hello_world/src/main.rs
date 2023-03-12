#![no_main]
#![no_std]

use hal::prelude::*;
use nrf52840_hal as hal;

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    reset_into_dfu();
}

/// Resets the device into Device Firmware Update mode (DFU).
fn reset_into_dfu() -> ! {
    // Via https://devzone.nordicsemi.com/f/nordic-q-a/50839/start-dfu-mode-or-open_bootloader-from-application-by-function-call
    unsafe { (*hal::pac::POWER::PTR).gpregret.write(|w| w.bits(0xB1)) };
    hal::pac::SCB::sys_reset();
}

#[derive(Clone, Copy)]
enum LightState {
    Red = 0,
    Green = 1,
    Blue = 2,
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let peripherals = hal::pac::Peripherals::take().unwrap();
    let port0 = hal::gpio::p0::Parts::new(peripherals.P0);
    let mut led_red = port0.p0_26.into_push_pull_output(hal::gpio::Level::Low);
    let mut led_green = port0.p0_30.into_push_pull_output(hal::gpio::Level::Low);
    let mut led_blue = port0.p0_06.into_push_pull_output(hal::gpio::Level::Low);

    // TIMER0 is reserved by Softdevice?
    // There seems to be more to timers that I don't get yet.
    // https://devzone.nordicsemi.com/f/nordic-q-a/1160/soft-device-and-timers---how-do-they-work-together
    let mut timer = hal::Timer::new(peripherals.TIMER1).into_periodic();
    timer.start(hal::Timer::<hal::pac::TIMER0>::TICKS_PER_SECOND);

    let mut light = LightState::Red;

    loop {
        light = match light {
            LightState::Red => LightState::Green,
            LightState::Green => LightState::Blue,
            LightState::Blue => LightState::Red,
        };
        match light {
            LightState::Red => {
                led_red.set_state(PinState::Low).unwrap();
                led_green.set_state(PinState::High).unwrap();
                led_blue.set_state(PinState::High).unwrap();
            }
            LightState::Green => {
                led_red.set_state(PinState::High).unwrap();
                led_green.set_state(PinState::Low).unwrap();
                led_blue.set_state(PinState::High).unwrap();
            }
            LightState::Blue => {
                led_red.set_state(PinState::High).unwrap();
                led_green.set_state(PinState::High).unwrap();
                led_blue.set_state(PinState::Low).unwrap();
                //reset_into_dfu();
            }
        }

        while timer.wait().is_err() {
            continue;
        }
    }
}
