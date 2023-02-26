Getting Started with Seeed Studio XIAO nRF52840
===============================================

I got [one of these little boards](https://www.seeedstudio.com/Seeed-XIAO-BLE-nRF52840-p-5201.html)
and wanted to start building things in Rust with it.
There didn't seem to be anything off-the-shelf to get started, so after some quite hard but 
rewarding reading up & tinkering I came up with this minimal example!

I puzzled these steps together mostly by looking at related projects

A bit challening was that most guides work with the [nRF52840-DK](https://www.seeedstudio.com/Seeed-XIAO-BLE-nRF52840-p-5201.html)
which you can flash directly with JTAG, meaning that you don't need to care about the bootloader
that much.

Software Prerequisites
-------------
* [rust](https://www.rust-lang.org/)
* `arm-none-eabi-objcopy` to convert from ELF to ihex (Intel .hex format)
  * rust linker doesn't come with this, on Mac you can install with 
    `brew install armmbed/formulae/arm-none-eabi-gcc`
* [Adafruit-nrfutil](https://github.com/adafruit/Adafruit_nRF52_nrfutil)
  * pip3 install adafruit-nrfutil
  * This is a Python3 port of an older version of Nordic's [`nrfutil`](https://www.nordicsemi.com/Products/Development-tools/nrf-util).
    I tinkered around with their newer one as well, but settled with Adafruit's version
    since it came with source (helped me figure out issues with `memory.x`!),
    is what Seeed uses in their Arduino IDE plugin and is *way* faster to run than the official one.

Preparation
-----------
Connect the device to your computer and double tap the reset button, can be a bit fiddly.
This will bring up a (virtual!) USB drive which contains amont other things a bootloader.txt file.

For me this looks like so:

```
UF2 Bootloader 0.6.2-12-g459adc9-dirty lib/nrfx (v2.0.0) lib/tinyusb (0.10.1-293-gaf8e5a90) lib/uf2 (remotes/origin/configupdate-9-gadbb8c7)
Model: Seeed XIAO nRF52840
Board-ID: Seeed_XIAO_nRF52840_Sense
Date: Nov 30 2021
SoftDevice: S140 7.3.0
```

The last line is important! SoftDevice is the proprietary bluetooth stack by Nordic,
and the way we're going to flash the device we don't want to overwrite it (and get prevented in parts).
Some more details on SoftDevice on the [Rust project](https://github.com/embassy-rs/nrf-softdevice)
that allows interfacing with it.

This directly leads to the [`memory.x`](memory.x) file which has values for this particular SoftDevice
version.
If you have a different version, you might need to adjust these numbers in there!


### Notes on bootloader

As it says in the `bootloader.txt` file, it is a UF2 bootloader.
Some more information about it and what's up with the simulated USB stick device can be found
at [Adafruit](https://learn.adafruit.com/adafruit-feather-m0-express-designed-for-circuit-python-circuitpython/uf2-bootloader-details).


Flashing & Iterating
--------------------
Every time you want to flash you need to double press the reset button.

> TODO: There should be some way to flash it without that. Need to figure if that's true and how.

All commands that are needed are in [`build-and-flash.sh`](build-and-flash.sh), but you'll probably
need to change `COM_PORT` to the correct port your device is connected on.
Easiest way to figure it out on Mac is to run `ls /dev/cu.*`

If your SoftDevice setting is different you'll need to adjust the `sd-req` parameter as well, see below.


### What's happening

Let's go through what this script does and what else is involved:

* `cargo build --release`
  * build in release mode, since debug is just way too big
  * Note that there's a bunch of important settings in [`.cargo/config.toml`](.cargo/config.toml)
    * Target architecture was set to `thumbv7em-none-eabihf`
    * `link.x` is provided by the `cortex-m` crate and sets all important linker settings. It also picks up our `memory.x` file automatically
    * `--nmagic` turns of page alignment on sections (TODO: Unclear if this is _really_ needed, saw it used a lot elsewhere)
  * TODO: Set settings to to make it smaller
* `arm-none-eabi-objcopy -O ihex target/thumbv7em-none-eabihf/release/hello-world hello-world.hex`
  * converts the ELF to ihex which the packaging tool can work with
* `adafruit-nrfutil dfu genpkg --dev-type 0x0052 --sd-req 0x0123 --application hello-world.hex hello-world.zip`
  * packages the ihex to a package that can be flashed.
  * `sd-req` parameter specifies the SoftDevice version.
    * To get a full list of which version gets which number run `nrfutil pkg generate --help`
      with the official (much slower) [`nrfutil`](https://www.nordicsemi.com/Products/Development-tools/nrf-util)
  * TODO: `dev-type` is a bit unclear, seems to be `hw-type` in the official tool but why would we then go hex 0x52 instead of integer 52?
    Got this directly from Adafruit nrfutil readme.
* `adafruit-nrfutil --verbose dfu serial -pkg hello-world.zip -p $COM_PORT -b 115200 --singlebank`
  * writes the package to the device and restarts it
