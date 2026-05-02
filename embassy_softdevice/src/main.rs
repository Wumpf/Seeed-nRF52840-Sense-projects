#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts, gpio, pac, peripherals,
    usb::{self},
};
use embassy_usb::class::cdc_acm::CdcAcmClass;
use embassy_usb::{UsbDevice, driver::EndpointError};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<peripherals::USBD>;
    CLOCK_POWER => usb::vbus_detect::InterruptHandler;
});

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    reset_into_dfu();
}

const USB_PACKAGE_SIZE: usize = 64;

/// Bootloader: enter CDC/serial DFU on next reset.
const GPREGRET_ENTER_SERIAL_DFU: u8 = 0x4E;
/// Bootloader: enter UF2 + CDC bootloader on next reset.
#[allow(dead_code)]
const GPREGRET_ENTER_UF2_DFU: u8 = 0x57;
/// Bootloader: enter OTA DFU mode on next reset.
#[allow(dead_code)]
const GPREGRET_ENTER_OTA_DFU: u8 = 0xA8;
/// App-local marker: request one extra reset after DFU flashing.
// TODO: need this?
#[allow(dead_code)]
const GPREGRET_ONE_SHOT_RESET_MARKER: u8 = 0xA5;

/// Resets the device into Device Firmware Update mode (DFU).
fn reset_into_dfu() -> ! {
    // Via https://github.com/adafruit/Adafruit_nRF52_Bootloader#how-to-use
    // This should allow us to reset into DFU/serial bootloader mode after reset.
    pac::POWER
        .gpregret()
        .write(|w| w.set_gpregret(GPREGRET_ENTER_SERIAL_DFU));
    cortex_m::peripheral::SCB::sys_reset();

    // Use with softdevice?
    //    use cortex_m::peripheral::SCB;
    //    use nrf_softdevice::raw;

    //    const GPREGRET_ENTER_SERIAL_DFU: u8 = 0x4E;

    //    fn reset_into_dfu() -> ! {
    //        // Clear GPREGRET then set exact bootloader value.
    //        unsafe {
    //            let _ = raw::sd_power_gpregret_clr(0, 0xff);
    //            let _ = raw::sd_power_gpregret_set(0, GPREGRET_ENTER_SERIAL_DFU as u32);
    //        }

    //        SCB::sys_reset();
    //    }
}

type MyUsbDriver = usb::Driver<'static, usb::vbus_detect::HardwareVbusDetect>;

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, MyUsbDriver>) -> ! {
    device.run().await;
}

#[embassy_executor::task]
async fn usb_read_write_task(mut class: CdcAcmClass<'static, MyUsbDriver>) -> ! {
    // Reconnect if we loose connection.
    loop {
        class.wait_connection().await;
        let _ = echo_and_reset_on_r(&mut class).await;
    }
}

#[embassy_executor::task]
async fn blink_task(
    mut led_red: gpio::Output<'static>,
    mut led_green: gpio::Output<'static>,
    mut led_blue: gpio::Output<'static>,
) -> ! {
    loop {
        led_red.set_high();
        led_green.set_high();
        led_blue.set_high();
        embassy_time::Timer::after(embassy_time::Duration::from_millis(500)).await; // TODO: softdevice compatible?

        led_red.set_low();
        led_green.set_low();
        led_blue.set_low();
        embassy_time::Timer::after(embassy_time::Duration::from_millis(500)).await;
    }
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo_and_reset_on_r(
    class: &mut CdcAcmClass<'static, MyUsbDriver>,
) -> Result<(), Disconnected> {
    let mut buf = [0; USB_PACKAGE_SIZE];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        class.write_packet(data).await?;

        if data.contains(&b'r') {
            class.write_packet(b"\nresetting...").await?;
            // There doesn't seem to be a reliable way to ensure that the packet arrived.
            cortex_m::asm::delay(6_400_000); // ~100ms at 64 MHz

            reset_into_dfu();
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    // Enable high frequency oscillator.
    // TODO: is this going to be a problem with SoftDevice? gpt says I probably want to use softdevice wrappers for this.
    pac::CLOCK.tasks_hfclkstart().write_value(1);
    while pac::CLOCK.events_hfclkstarted().read() != 1 {}

    // Create the driver, from the HAL.
    let driver = usb::Driver::new(
        p.USBD,
        Irqs,
        usb::vbus_detect::HardwareVbusDetect::new(Irqs), // TODO: doesn't work with softdevice
    );

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Wumpftech");
    config.product = Some("Wumpftech Serial");
    config.serial_number = Some("wumpf1");
    config.max_packet_size_0 = USB_PACKAGE_SIZE as u8;

    // Create embassy-usb DeviceBuilder using the driver and config.
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static MSOS_DESC: StaticCell<[u8; 128]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();
    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        &mut CONFIG_DESC.init([0; 256])[..],
        &mut BOS_DESC.init([0; 256])[..],
        &mut MSOS_DESC.init([0; 128])[..],
        &mut CONTROL_BUF.init([0; 128])[..],
    );

    // Create classes on the builder.
    static STATE: StaticCell<embassy_usb::class::cdc_acm::State> = StaticCell::new();
    let state = STATE.init(embassy_usb::class::cdc_acm::State::new());
    let class = embassy_usb::class::cdc_acm::CdcAcmClass::new(&mut builder, state, 64);

    // Build the builder.
    let usb = builder.build();

    // LEDs
    let led_red = gpio::Output::new(p.P0_26, gpio::Level::Low, gpio::OutputDrive::Standard);
    let led_green = gpio::Output::new(p.P0_30, gpio::Level::Low, gpio::OutputDrive::Standard);
    let led_blue = gpio::Output::new(p.P0_06, gpio::Level::Low, gpio::OutputDrive::Standard);

    spawner.spawn(usb_task(usb).unwrap());
    spawner.spawn(usb_read_write_task(class).unwrap());
    spawner.spawn(blink_task(led_red, led_green, led_blue).unwrap());
}
