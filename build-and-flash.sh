#!/bin/bash
set -e

COM_PORT=/dev/cu.usbmodem1101

cargo build --release
arm-none-eabi-objcopy -O ihex target/thumbv7em-none-eabihf/release/hello-world hello-world.hex
adafruit-nrfutil dfu genpkg --dev-type 0x0052 --sd-req 0x0123 --application hello-world.hex hello-world.zip
adafruit-nrfutil --verbose dfu serial -pkg hello-world.zip -p $COM_PORT -b 115200 --singlebank