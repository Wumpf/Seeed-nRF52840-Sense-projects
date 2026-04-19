# USB Serial

Advertises the device as serial USB device.
Still cycles through the LEDs but uses a bit more advanced interrupt based timer handling, so the device doesn't busy loop all the time.

Additionally, the serial port is monitored to write everything back that comes in.
On sending `r` over the port, the device will go back into bootloader mode.

TODO: directly after loading the bootloader, the serial device may not be recognized.
This means effectively that every time you load a new program onto the device,
you have to first unplug & plug it again.

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