#![no_main]
#![no_std]

use embedded_hal::digital::{OutputPin, PinState};
use nrf52840_hal as hal;
use usb_device::class_prelude::UsbBusAllocator;

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

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    reset_into_dfu();
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

#[derive(Clone, Copy)]
enum LightState {
    Red = 0,
    Green = 1,
    Blue = 2,
}

// makes control transfers 8x faster says https://github.com/nrf-rs/nrf-hal/blob/master/examples/usb/src/bin/serial.rs
const MAX_PACKAGE_SIZE: usize = 64;

#[cortex_m_rt::entry]
fn main() -> ! {
    let peripherals = hal::pac::Peripherals::take().unwrap();

    // I kept having the problem of the serial port (USB CDC) not showing up directly after a
    // DFU-based flashing, unless I power-cycled the board. Resetting the device once more fixes it.
    //
    // We use GPREGRET as a one-shot marker: first boot writes GPREGRET_ONE_SHOT_RESET_MARKER and
    // resets, second boot sees it, clears it, and continues. GPREGRET survives a system reset, but not a power cycle.
    // See https://devzone.nordicsemi.com/f/nordic-q-a/1935/definitive-information-on-gpregret-register
    let power = unsafe { &*hal::pac::POWER::PTR };
    if power.gpregret.read().bits() != GPREGRET_ONE_SHOT_RESET_MARKER as u32 {
        power
            .gpregret
            .write(|w| unsafe { w.bits(GPREGRET_ONE_SHOT_RESET_MARKER as u32) });
        hal::pac::SCB::sys_reset();
    }
    power.gpregret.write(|w| unsafe { w.bits(0) });

    let port0 = hal::gpio::p0::Parts::new(peripherals.P0);
    let mut led_red = port0.p0_26.into_push_pull_output(hal::gpio::Level::Low);
    let mut led_green = port0.p0_30.into_push_pull_output(hal::gpio::Level::Low);
    let mut led_blue = port0.p0_06.into_push_pull_output(hal::gpio::Level::Low);

    let clocks = hal::clocks::Clocks::new(peripherals.CLOCK);
    let clocks = clocks.enable_ext_hfosc();
    let usb_peripheral = hal::usbd::UsbPeripheral::new(peripherals.USBD, &clocks);
    let usb_bus = UsbBusAllocator::new(hal::usbd::Usbd::new(usb_peripheral));
    let mut serial_port = usbd_serial::SerialPort::new(&usb_bus);

    let mut usb_device = usb_device::device::UsbDeviceBuilder::new(
        &usb_bus,
        usb_device::device::UsbVidPid(0x16c0, 0x27dd),
    )
    .strings(&[usb_device::device::StringDescriptors::default()
        .manufacturer("Wumpftech")
        .product("Wumpftech nRF52840")
        .serial_number("wumpf1")])
    .unwrap()
    .device_class(usbd_serial::USB_CLASS_CDC)
    .max_packet_size_0(MAX_PACKAGE_SIZE as _)
    .unwrap()
    .build();

    // TIMER0 is reserved by Softdevice, see https://github.com/embassy-rs/nrf-softdevice/issues/16#issuecomment-691745438
    let mut timer = hal::Timer::new(peripherals.TIMER1).into_periodic();
    timer.start(hal::Timer::<hal::pac::TIMER1>::TICKS_PER_SECOND);

    let mut light = LightState::Red;

    loop {
        if timer.reset_if_finished() {
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
                }
            }

            let _ = serial_port.write("Switched light to ".as_bytes());
            let _ = serial_port.write(&[b'0' + (light as u8)]);
            let _ = serial_port.write("\r\n".as_bytes());
        }

        if usb_device.poll(&mut [&mut serial_port]) {
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
    }
}
