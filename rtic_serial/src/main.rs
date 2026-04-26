#![no_main]
#![no_std]

use embedded_hal::digital::OutputPin;
use hal::clocks::{ExternalOscillator, Internal, LfOscStarted};
use hal::gpio::{Level, Output, Pin, PushPull};
use hal::usbd::{UsbPeripheral, Usbd};
use nrf52840_hal as hal;
use rtic_monotonics::nrf::rtc::prelude::*; // memory layout
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid};

type UsbBus = Usbd<UsbPeripheral<'static>>;
type SerialPort = usbd_serial::SerialPort<'static, UsbBus>;
type UsbSerialDevice = UsbDevice<'static, UsbBus>;

/// Bootloader: enter CDC/serial DFU on next reset.
const GPREGRET_ENTER_SERIAL_DFU: u8 = 0x4E;
/// Bootloader: enter UF2 + CDC bootloader on next reset.
#[allow(dead_code)]
const GPREGRET_ENTER_UF2_DFU: u8 = 0x57;
/// Bootloader: enter OTA DFU mode on next reset.
#[allow(dead_code)]
const GPREGRET_ENTER_OTA_DFU: u8 = 0xA8;
/// App-local marker: request one extra reset after DFU flashing.
const GPREGRET_ONE_SHOT_RESET_MARKER: u8 = 0xA5;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    reset_into_dfu();
}

nrf_rtc0_monotonic!(Mono);

// makes control transfers 8x faster says https://github.com/nrf-rs/nrf-hal/blob/master/examples/usb/src/bin/serial.rs
const MAX_PACKAGE_SIZE: usize = 64;

#[rtic::app(device = hal::pac, dispatchers = [SWI0_EGU0])]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        serial_port: SerialPort,
    }

    #[local]
    struct Local {
        led_blue: Pin<Output<PushPull>>,
        led_red: Pin<Output<PushPull>>,
        usb_device: UsbSerialDevice,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let device = cx.device;

        // I kept having the problem of the serial port (USB CDC) not showing up directly after
        // a DFU-based flashing, unless I power-cycled the board. Resetting the device once more
        // fixes it.
        //
        // We use GPREGRET as a one-shot marker: first boot writes GPREGRET_ONE_SHOT_RESET_MARKER
        // and resets, second boot sees it, clears it, and continues. GPREGRET survives a system reset, but not a power
        // cycle. See https://devzone.nordicsemi.com/f/nordic-q-a/1935/definitive-information-on-gpregret-register
        let power = device.POWER;
        if power.gpregret.read().bits() != GPREGRET_ONE_SHOT_RESET_MARKER as u32 {
            power
                .gpregret
                .write(|w| unsafe { w.bits(GPREGRET_ONE_SHOT_RESET_MARKER as u32) });
            hal::pac::SCB::sys_reset();
        }
        power.gpregret.write(|w| unsafe { w.bits(0) });

        // Setup clocks before starting USB and RTC-based monotonic.
        let clocks = hal::clocks::Clocks::new(device.CLOCK)
            .enable_ext_hfosc()
            .start_lfclk();
        let clocks =
            cortex_m::singleton!(: hal::clocks::Clocks<ExternalOscillator, Internal, LfOscStarted> = clocks)
                .unwrap();

        // USB types require 'static backing storage, so keep allocator in singleton memory.
        let usb_peripheral = UsbPeripheral::new(device.USBD, clocks);
        let usb_bus =
            cortex_m::singleton!(: UsbBusAllocator<UsbBus> = UsbBusAllocator::new(Usbd::new(usb_peripheral)))
                .unwrap();
        let serial_port = usbd_serial::SerialPort::new(usb_bus);
        let usb_device = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
            .strings(&[StringDescriptors::default()
                .manufacturer("Wumpftech")
                .product("Wumpftech nRF52840")
                .serial_number("wumpf1")])
            .unwrap()
            .device_class(usbd_serial::USB_CLASS_CDC)
            .max_packet_size_0(MAX_PACKAGE_SIZE as _)
            .unwrap()
            .build();

        // Initialize Monotonic
        Mono::start(device.RTC0);

        // Setup LED
        let port0 = hal::gpio::p0::Parts::new(device.P0);
        let led_blue = port0.p0_06.into_push_pull_output(Level::Low).degrade();
        let led_red = port0.p0_26.into_push_pull_output(Level::Low).degrade();

        // Schedule blinking task
        usb_poll::spawn().unwrap();
        blink_red::spawn().unwrap();
        blink_blue::spawn().unwrap();

        (
            Shared { serial_port },
            Local {
                led_blue,
                led_red,
                usb_device,
            },
        )
    }

    #[task(shared = [serial_port], local = [usb_device], priority = 2)]
    async fn usb_poll(mut cx: usb_poll::Context) {
        loop {
            cx.shared.serial_port.lock(|serial_port| {
                if cx.local.usb_device.poll(&mut [serial_port]) {
                    read_serial_port_package(serial_port);
                }
            });

            // Documentation of `poll` says it should be called at least every 10ms.
            // 10ms & 5ms delay are empircally too long, device won't be detected by host.
            Mono::delay(2.millis()).await;
        }
    }

    #[task(local = [led_red], shared = [serial_port])]
    async fn blink_red(mut cx: blink_red::Context) {
        let blink_red::LocalResources { led_red, .. } = cx.local;

        let mut blink_on = false;
        loop {
            blink_on = !blink_on;
            if blink_on {
                led_red.set_high().unwrap();
            } else {
                led_red.set_low().unwrap();
            }

            cx.shared.serial_port.lock(|serial_port| {
                let message: &[u8] = if blink_on { b"red on\n" } else { b"red off\n" };
                let _ = serial_port.write(message);
            });

            Mono::delay(1000.millis()).await;
        }
    }

    #[task(local = [led_blue])]
    async fn blink_blue(cx: blink_blue::Context) {
        let blink_blue::LocalResources { led_blue, .. } = cx.local;

        let mut blink_on = false;
        loop {
            blink_on = !blink_on;
            if blink_on {
                led_blue.set_high().unwrap();
            } else {
                led_blue.set_low().unwrap();
            }

            Mono::delay(100.millis()).await;
        }
    }
}

/// Reads a package from the serial port, echoes it back, and if it contains an `r`, resets into DFU mode.
fn read_serial_port_package(serial_port: &mut SerialPort) {
    let mut buf = [0u8; MAX_PACKAGE_SIZE];
    match serial_port.read(&mut buf) {
        Ok(count) if count > 0 => {
            // Echo back the received data.
            let mut write_offset = 0;
            while write_offset < count {
                match serial_port.write(&buf[write_offset..count]) {
                    Ok(len) => {
                        if len > 0 {
                            write_offset += len;
                        } else {
                            break;
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }

            // If there's an `r`, reset into DFU mode.
            if buf[..count].contains(&b'r') {
                reset_into_dfu();
            }
        }
        _ => {}
    }
}

/// Resets the device into Device Firmware Update mode (DFU).
fn reset_into_dfu() -> ! {
    let power = unsafe { &*hal::pac::POWER::PTR };

    // Via https://github.com/adafruit/Adafruit_nRF52_Bootloader#how-to-use
    // This should allow us to reset into DFU/serial bootloader mode after reset.
    power
        .gpregret
        .write(|w| unsafe { w.bits(GPREGRET_ENTER_SERIAL_DFU.into()) });
    hal::pac::SCB::sys_reset();
}
