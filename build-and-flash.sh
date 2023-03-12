#!/bin/bash
set -e

COM_PORT=/dev/cu.usbmodem1101

while getopts 'abc:h' opt; do
  case "$opt" in
    ?|h)
      echo "Usage: $(basename $0) crate_name"
      exit 1
      ;;
  esac
done
shift "$(($OPTIND -1))"
CRATE=$1

cargo build -p $CRATE --release
arm-none-eabi-objcopy -O ihex target/thumbv7em-none-eabihf/release/$CRATE target/$CRATE.hex
adafruit-nrfutil dfu genpkg --dev-type 0x0052 --sd-req 0x0123 --application target/$CRATE.hex target/$CRATE.zip
adafruit-nrfutil --verbose dfu serial -pkg $CRATE.zip -p $COM_PORT -b 115200 --singlebank