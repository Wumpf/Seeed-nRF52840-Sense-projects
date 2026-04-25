# [RTIC](https://rtic.rs/2/book/en/preface.html) Demo

[RTIC](https://rtic.rs/2/book/en/preface.html) is a nice framework for single core scheduling using interrupts!


Code started as a "port" of https://github.com/rtic-rs/rtic/tree/v2.2.0/examples/nrf52840_blinky
I've only left the RTC version - unlike TIMER, RTC can still run in "System ON sleep"

On the topic of sleep modes, learn more here:
* https://docs.nordicsemi.com/bundle/ps_nrf52840/page/_tmp/nrf52840/autodita/CURRENT/parameters.i_sleep.html
* https://forum.seeedstudio.com/t/sleep-current-of-xiao-nrf52840-deep-sleep-vs-light-sleep/271841/40