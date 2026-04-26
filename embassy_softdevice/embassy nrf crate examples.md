LLM generated overview of the examples at https://github.com/embassy-rs/embassy/tree/654cc771728fa3a1bfc98e7c66eadd9c213d5755/examples/nrf52840/src/bin

**Basics**
- `blinky.rs`: blink LED with Embassy timer.
- `timer.rs`: basic timer/delay usage.
- `rtc.rs`: RTC peripheral usage.
- `temp.rs`: read on-chip temperature sensor.
- `rng.rs`: use hardware random generator.
- `wdt.rs`: watchdog timer setup and feeding.
- `nvmc.rs`: internal flash erase/write with NVMC.
**Executor / Concurrency**
- `channel.rs`: task communication through channel.
- `channel_sender_receiver.rs`: split sender/receiver channel pattern.
- `mutex.rs`: shared state protected by async mutex.
- `pubsub.rs`: publish/subscribe messaging between tasks.
- `raw_spawn.rs`: low-level task spawning.
- `self_spawn.rs`: task respawns itself.
- `self_spawn_current_executor.rs`: self-spawn on current executor.
- `manually_create_executor.rs`: build executor manually.
- `executor_fairness_test.rs`: test fairness/scheduling behavior.
- `multiprio.rs`: multiple executor priorities.
**GPIO / Events / Interconnect**
- `gpiote_channel.rs`: GPIO event via GPIOTE channel.
- `gpiote_port.rs`: GPIO event via GPIOTE port event.
- `egu.rs`: use Event Generator Unit.
- `ppi.rs`: connect peripheral events/tasks with PPI.
**UART**
- `uart.rs`: basic UART I/O.
- `uart_idle.rs`: UART with idle-line detection.
- `uart_split.rs`: split UART TX/RX across tasks.
- `buffered_uart.rs`: buffered async UART.
**SPI / I2C**
- `spim.rs`: SPI master example.
- `spis.rs`: SPI slave example.
- `twim.rs`: I2C/TWI master example.
- `twim_lowpower.rs`: low-power I2C master.
- `twis.rs`: I2C/TWI slave example.
**PWM / Motion / Timing Output**
- `pwm.rs`: simple PWM output.
- `pwm_servo.rs`: servo control with PWM.
- `pwm_sequence.rs`: PWM sequence playback.
- `pwm_double_sequence.rs`: two PWM sequences.
- `pwm_sequence_ppi.rs`: PWM sequencing triggered via PPI.
- `pwm_sequence_ws2812b.rs`: drive WS2812B LEDs with PWM.
- `qdec.rs`: quadrature decoder input.
**Audio**
- `i2s_waveform.rs`: generate waveform over I2S.
- `i2s_monitor.rs`: monitor/capture I2S stream.
- `i2s_effect.rs`: process/apply effect on I2S data.
- `pdm.rs`: basic PDM microphone capture.
- `pdm_continuous.rs`: continuous PDM audio capture.
**ADC / Analog**
- `saadc.rs`: one-shot ADC sampling with SAADC.
- `saadc_continuous.rs`: continuous ADC sampling.
**Storage / External Memory**
- `qspi.rs`: QSPI flash access.
- `qspi_lowpower.rs`: low-power QSPI flash usage.
**USB**
- `usb_serial.rs`: USB CDC serial device.
- `usb_serial_multitask.rs`: multitask USB serial handling.
- `usb_serial_winusb.rs`: USB serial / WinUSB compatibility example.
- `usb_hid_keyboard.rs`: USB HID keyboard.
- `usb_hid_mouse.rs`: USB HID mouse.
- `usb_ethernet.rs`: USB Ethernet/RNDIS-like networking.
**Networking / Radios**
- `ethernet_enc28j60.rs`: ENC28J60 SPI Ethernet.
- `wifi_esp_hosted.rs`: ESP-hosted Wi-Fi integration.
- `ieee802154_send.rs`: send 802.15.4 frames.
- `ieee802154_receive.rs`: receive 802.15.4 frames.
- `sixlowpan.rs`: 6LoWPAN over 802.15.4.
- `nfct.rs`: NFC tag / NFCT peripheral usage.

**Glossary**
- `ADC`: Analog-to-Digital Converter
- `CDC`: Communications Device Class
- `EGU`: Event Generator Unit
- `GPIOTE`: GPIO Tasks and Events
- `HID`: Human Interface Device
- `I2C`: Inter-Integrated Circuit
- `I2S`: Inter-IC Sound
- `NFCT`: Near Field Communication Tag
- `NVMC`: Non-Volatile Memory Controller
- `PDM`: Pulse Density Modulation
- `PPI`: Programmable Peripheral Interconnect
- `PWM`: Pulse Width Modulation
- `QDEC`: Quadrature Decoder
- `QSPI`: Quad SPI
- `RNG`: Random Number Generator
- `RTC`: Real-Time Counter/Clock
- `SAADC`: Successive Approximation ADC
- `SPI`: Serial Peripheral Interface
- `SPIM`: SPI Master
- `SPIS`: SPI Slave
- `TWI`: Nordic name for I2C
- `TWIM`: TWI/I2C Master
- `TWIS`: TWI/I2C Slave
- `UART`: Universal Asynchronous Receiver/Transmitter
- `USB`: Universal Serial Bus
- `WS2812B`: addressable RGB LED protocol
- `6LoWPAN`: IPv6 over Low-Power Wireless Personal Area Networks
- `IEEE 802.15.4`: low-power wireless PHY/MAC used by Thread/6LoWPAN/Zigbee-class stacks
