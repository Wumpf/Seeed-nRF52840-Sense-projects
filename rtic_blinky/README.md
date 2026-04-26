# [RTIC](https://rtic.rs/2/book/en/preface.html) Demo

[RTIC](https://rtic.rs/2/book/en/preface.html) is a nice framework for single core scheduling using interrupts!

Code started as a "port" of https://github.com/rtic-rs/rtic/tree/v2.2.0/examples/nrf52840_blinky
I've only left the RTC version - unlike TIMER, RTC can still run in "System ON sleep"

On the topic of sleep modes, learn more here:
* https://docs.nordicsemi.com/bundle/ps_nrf52840/page/_tmp/nrf52840/autodita/CURRENT/parameters.i_sleep.html
* https://forum.seeedstudio.com/t/sleep-current-of-xiao-nrf52840-deep-sleep-vs-light-sleep/271841/40

## No probe-rs & defmt

I've removed all probe & defmt features since unless you have the [expansion board](https://www.seeedstudio.com/Seeeduino-XIAO-Expansion-board-p-4746.html) or the [debug mate](https://www.seeedstudio.com/Seeed-Studio-XIAO-Debug-Mate-p-6588.html)
it's kinda hard to use any of that since the only way to interface is really the serial interface.

There's a crate for [serial defmt](https://github.com/gauteh/defmt-serial), but doesn't look fun to use,
might as well just regular text over serial unless application size becomes an issue.

## USB Re-enumeration After DFU

After serial DFU flashing on macOS, app CDC device often does not re-enumerate until the board is unplugged.
Current workaround is a one-shot extra reset at startup: first boot writes a marker to `GPREGRET` and resets,
second boot clears marker and continues (`GPREGRET` survives system reset).
