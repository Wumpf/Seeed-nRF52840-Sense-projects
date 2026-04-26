# Use [Embassy](https://embassy.dev/) with nRF SoftDevice demo

**WIP!**

This is most of everything in [`rtic_serial`](../rtic_serial/) demo and then some more.

For more Embassy nrf52840 examples, look for the [official embassy examples](https://github.com/embassy-rs/embassy/tree/654cc771728fa3a1bfc98e7c66eadd9c213d5755/examples/nrf52840/src/bin).

Incorporates [nrf-softdevice bindings](https://github.com/embassy-rs/nrf-softdevice)
to do some very basic bluetooth operations.

Make sure to set the right SoftDevice feature flag depending on what version your device runs.
See main [README.md](../README.md) for details.

## Why not RTIC?

See notes on SoftDevice in [`rtic_serial`](../rtic_serial/README.md) README.