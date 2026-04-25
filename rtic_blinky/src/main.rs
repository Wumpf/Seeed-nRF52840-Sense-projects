#![no_main]
#![no_std]

use core::sync::atomic::AtomicUsize;

use cortex_m_semihosting::debug;
use defmt_rtt as _; // global logger
use nrf52840_hal as hal;
use panic_probe as _;
use rtic_monotonics::nrf::rtc::prelude::*; // memory layout

// Same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

// defmt needs a timestamp. Use a simple monotonic counter.
static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!(
    "{=usize}",
    COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
);

/// Hardfault handler.
///
/// Terminates the application and makes a semihosting-capable debug tool exit
/// with an error. This seems better than the default, which is to spin in a
/// loop.
#[cortex_m_rt::exception]
unsafe fn HardFault(_frame: &cortex_m_rt::ExceptionFrame) -> ! {
    loop {
        debug::exit(debug::EXIT_FAILURE);
    }
}

nrf_rtc0_monotonic!(Mono);

#[rtic::app(device = hal::pac, dispatchers = [SWI0_EGU0])]
mod app {
    use super::*;

    use embedded_hal::digital::OutputPin;
    use hal::gpio::{Level, Output, Pin, PushPull};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Pin<Output<PushPull>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        // Configure low frequency clock
        hal::clocks::Clocks::new(cx.device.CLOCK).start_lfclk();

        // Initialize Monotonic
        Mono::start(cx.device.RTC0);

        // Setup LED
        let port0 = hal::gpio::p0::Parts::new(cx.device.P0);
        let led = port0.p0_06.into_push_pull_output(Level::Low).degrade();

        // Schedule the blinking task
        blink::spawn().ok();

        (Shared {}, Local { led })
    }

    #[task(local = [led])]
    async fn blink(cx: blink::Context) {
        let blink::LocalResources { led, .. } = cx.local;

        let mut next_tick = Mono::now();
        let mut blink_on = false;
        loop {
            let now = Mono::now();
            let now_ms: fugit::SecsDurationU64 = now.duration_since_epoch().convert();
            defmt::println!("Timer {} ({})", now_ms, now.ticks());

            blink_on = !blink_on;
            if blink_on {
                led.set_high().unwrap();
            } else {
                led.set_low().unwrap();
            }

            next_tick += 1000.millis();
            Mono::delay_until(next_tick).await;
        }
    }
}
