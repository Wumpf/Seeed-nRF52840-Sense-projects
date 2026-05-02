#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts, gpio,
    interrupt::{self, typelevel::Interrupt as _},
    peripherals,
    usb::{self},
};
use embassy_usb::class::cdc_acm::CdcAcmClass;
use embassy_usb::{UsbDevice, driver::EndpointError};
use nrf_softdevice::{Softdevice, ble};
use static_cell::StaticCell;

bind_interrupts!(
    struct Irqs {
        USBD => usb::InterruptHandler<peripherals::USBD>;
    }
);

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    reset_into_dfu();
}

const USB_PACKAGE_SIZE: usize = 64;

/// Resets the device into Device Firmware Update mode (DFU).
fn reset_into_dfu() -> ! {
    // Via https://github.com/adafruit/Adafruit_nRF52_Bootloader#how-to-use
    // This should allow us to reset into DFU/serial bootloader mode after reset.

    // Bootloader: enter CDC/serial DFU on next reset.
    const GPREGRET_ENTER_SERIAL_DFU: u8 = 0x4E;
    // Bootloader: enter UF2 + CDC bootloader on next reset.
    #[allow(dead_code)]
    const GPREGRET_ENTER_UF2_DFU: u8 = 0x57;
    // Bootloader: enter OTA DFU mode on next reset.
    #[allow(dead_code)]
    const GPREGRET_ENTER_OTA_DFU: u8 = 0xA8;

    // Clear GPREGRET then set exact bootloader value.
    unsafe {
        nrf_softdevice::raw::sd_power_gpregret_clr(0, 0xff);
        nrf_softdevice::raw::sd_power_gpregret_set(0, GPREGRET_ENTER_SERIAL_DFU as u32);
    }
    cortex_m::peripheral::SCB::sys_reset();
}

type UsbDriver = usb::Driver<'static, &'static usb::vbus_detect::SoftwareVbusDetect>;

#[embassy_executor::task]
async fn softdevice_task(
    sd: &'static Softdevice,
    vbus: &'static usb::vbus_detect::SoftwareVbusDetect,
) -> ! {
    sd.run_with_callback(|event| {
        use nrf_softdevice::SocEvent;

        // Forward USB events.
        match event {
            SocEvent::PowerUsbDetected => vbus.detected(true),
            SocEvent::PowerUsbRemoved => vbus.detected(false),
            SocEvent::PowerUsbPowerReady => vbus.ready(),
            _ => {}
        }
    })
    .await
}

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, UsbDriver>) -> ! {
    device.run().await;
}

#[embassy_executor::task]
async fn usb_read_write_task(mut class: CdcAcmClass<'static, UsbDriver>) -> ! {
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
        embassy_time::Timer::after(embassy_time::Duration::from_millis(500)).await;

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
    class: &mut CdcAcmClass<'static, UsbDriver>,
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
    // Per https://github.com/embassy-rs/nrf-softdevice/tree/nrf-softdevice-v0.1.0#interrupt-priority
    // Interrupt priorities 0, 1 and 4 are reserved by the Softdevice, so we have to use 2 or 3 for all interrupts.
    let mut config = embassy_nrf::config::Config::default();
    //config.gpiote_interrupt_priority = embassy_nrf::interrupt::Priority::P2;
    config.time_interrupt_priority = interrupt::Priority::P2;
    let peripherals = embassy_nrf::init(config);
    interrupt::typelevel::USBD::set_priority(interrupt::Priority::P2);

    let config = softdevice_config();
    let sd = Softdevice::enable(&config);

    let vbus = setup_usb(&spawner, peripherals.USBD);
    spawner.spawn(softdevice_task(sd, vbus).unwrap());

    setup_blinking_leds(
        &spawner,
        peripherals.P0_26,
        peripherals.P0_30,
        peripherals.P0_06,
    );

    setup_ble_advertising(sd).await;
}

async fn setup_ble_advertising(sd: &'static Softdevice) {
    let mut config = ble::peripheral::Config::default();
    config.interval = 50; // Advertising interval in 0.625us units.

    // Legacy means it's BLE 4.x compatible.
    static ADV_DATA: ble::advertisement_builder::LegacyAdvertisementPayload =
        ble::advertisement_builder::LegacyAdvertisementBuilder::new()
            .flags(&[
                ble::advertisement_builder::Flag::GeneralDiscovery,
                ble::advertisement_builder::Flag::LE_Only,
            ])
            .services_16(
                ble::advertisement_builder::ServiceList::Complete,
                // if there were a lot of these there may not be room for the full name
                &[ble::advertisement_builder::ServiceUuid16::HEALTH_THERMOMETER],
            )
            .short_name("hello")
            .build();

    // Full name is visible once connected.
    static SCAN_DATA: ble::advertisement_builder::LegacyAdvertisementPayload =
        ble::advertisement_builder::LegacyAdvertisementBuilder::new()
            .full_name("Wumpf says hi with more words")
            .build();

    let adv = ble::peripheral::NonconnectableAdvertisement::ScannableUndirected {
        adv_data: &ADV_DATA,
        scan_data: &SCAN_DATA,
    };

    ble::peripheral::advertise(sd, adv, &config).await.unwrap();
}

fn setup_usb(
    spawner: &Spawner,
    usbd: embassy_nrf::Peri<'static, peripherals::USBD>,
) -> &'static usb::vbus_detect::SoftwareVbusDetect {
    // Enable USB events on softdevice.
    unsafe {
        nrf_softdevice::raw::sd_power_usbdetected_enable(1);
        nrf_softdevice::raw::sd_power_usbremoved_enable(1);
        nrf_softdevice::raw::sd_power_usbpwrrdy_enable(1);
    };

    // Create the driver.
    // We can't use usb::vbus_detect::HardwareVbusDetect with SoftDevice, so we have to feed in status ourselves.
    // This happens as part of the `softdevice_task` callback, which is called on USB events.
    let mut usbregstatus: u32 = 0;
    unsafe {
        nrf_softdevice::raw::sd_power_usbregstatus_get(&mut usbregstatus);
    }
    let usb_detected = (usbregstatus & 1) != 0;
    let power_ready = (usbregstatus & (1 << 1)) != 0;
    static VBUS: StaticCell<usb::vbus_detect::SoftwareVbusDetect> = StaticCell::new();
    let vbus = &*VBUS.init(usb::vbus_detect::SoftwareVbusDetect::new(
        usb_detected,
        power_ready,
    ));

    let driver = usb::Driver::new(usbd, Irqs, vbus);

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

    let device = builder.build();

    spawner.spawn(usb_task(device).unwrap());
    spawner.spawn(usb_read_write_task(class).unwrap());

    vbus
}

fn setup_blinking_leds(
    spawner: &Spawner,
    red: embassy_nrf::Peri<'static, impl gpio::Pin>,
    green: embassy_nrf::Peri<'static, impl gpio::Pin>,
    blue: embassy_nrf::Peri<'static, impl gpio::Pin>,
) {
    let led_red = gpio::Output::new(red, gpio::Level::Low, gpio::OutputDrive::Standard);
    let led_green = gpio::Output::new(green, gpio::Level::Low, gpio::OutputDrive::Standard);
    let led_blue = gpio::Output::new(blue, gpio::Level::Low, gpio::OutputDrive::Standard);
    spawner.spawn(blink_task(led_red, led_green, led_blue).unwrap());
}

fn softdevice_config() -> nrf_softdevice::Config {
    use nrf_softdevice::raw;

    let name = b"BlueWumpf";

    nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8, // TODO: switch to external? NRF_CLOCK_LF_SRC_XTAL?
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        // Configure GAP (Generic Access Profile) connection resource.
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 6,
            event_length: 24,
        }),
        // Configure GATT (Generic Attribute Profile) connection resource.
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 256 }), // Bumps up the maximum transmission unit, allowing us to send more data in one packet.
        // Attribute table size.
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: raw::BLE_GATTS_ATTR_TAB_SIZE_DEFAULT,
        }),
        // Configure BLE roles.
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 3,  //raw::BLE_GAP_ROLE_COUNT_PERIPH_DEFAULT as _,
            central_role_count: 3, //raw::BLE_GAP_ROLE_COUNT_CENTRAL_DEFAULT as _,
            central_sec_count: 0,  //raw::BLE_GAP_ROLE_COUNT_CENTRAL_SEC_DEFAULT as _,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        // Configure GAP (Generic Access Profile) device name.
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: name.as_ptr() as *const u8 as _,
            current_len: name.len() as _,
            max_len: name.len() as _,
            write_perm: unsafe { core::mem::zeroed() }, // Not writable.
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                raw::BLE_GATTS_VLOC_STACK as u8,
            ),
        }),
        ..Default::default()
    }
}
