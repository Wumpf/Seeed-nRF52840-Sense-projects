/*
 * Via https://github.com/embassy-rs/nrf-softdevice/blob/5949a5b1445cc907745c6449a35577e4544cd255/examples/memory-nrf52840.x
 */
MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* NRF52840 with Softdevice S140 7.3.0 */
  FLASH : ORIGIN = 0x00000000 + 156K, LENGTH = 1024K - 156K
  RAM : ORIGIN = 0x20000000 + 31K, LENGTH = 256K - 31K
}