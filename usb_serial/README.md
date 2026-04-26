# USB Serial

Advertises the device as serial USB device.
Still cycles through the LEDs but uses a bit more advanced interrupt based timer handling, so the device doesn't busy loop all the time.

Additionally, the serial port is monitored to write everything back that comes in.
On sending `r` over the port, the device will go back into bootloader mode.

After serial DFU flashing on macOS, the app CDC device often does not re-enumerate until the board
is unplugged. Current workaround is a one-shot extra reset at startup: first boot writes a marker
to `GPREGRET` and resets, second boot clears marker and continues. `GPREGRET` survives system
reset, but not power cycle.

Build & run:
```sh
./build-and-flash.sh usb_serial
```

You can now interface with it on serial

Basic monitoring on Mac/Linux:
```sh
screen /dev/tty.usbmodemwumpf11
```
(`ctrl+a+k` to exit again)

Easier and more advanced interfacing using [tio](https://github.com/tio/tio):

List devices:
```sh
tio --list
```
Connect:
```sh
tio /dev/tty.usbmodemwumpf11
```
