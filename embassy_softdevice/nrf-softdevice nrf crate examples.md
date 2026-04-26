LLM generated overview of the examples at https://github.com/embassy-rs/nrf-softdevice/tree/5949a5b1445cc907745c6449a35577e4544cd255/examples/src/bin

**Advertising / Discovery**
- `ble_advertise.rs`: simple BLE advertising without connections.
- `ble_scan.rs`: scans for nearby BLE advertisements.
**GATT Services**
- `ble_bas_central.rs`: central connecting to Battery Service and reading battery data.
- `ble_bas_peripheral.rs`: peripheral exposing Battery Service.
- `ble_bas_peripheral_notify.rs`: Battery Service peripheral with notifications.
- `ble_dis_bas_peripheral_builder.rs`: peripheral exposing Device Information Service plus Battery Service.
- `ble_peripheral_onoff.rs`: peripheral exposing simple on/off control characteristic.
**Bonding / Pairing**
- `ble_bond_central.rs`: central with pairing and bonding support.
- `ble_bond_peripheral.rs`: peripheral with pairing and bonding support.
**HID**
- `ble_keyboard_peripheral_builder.rs`: BLE HID keyboard peripheral using builder API.
- `ble_keyboard_peripheral_builder_macro.rs`: BLE HID keyboard peripheral using macro-generated definitions.
**L2CAP**
- `ble_l2cap_central.rs`: central using L2CAP CoC data channels.
- `ble_l2cap_peripheral.rs`: peripheral using L2CAP CoC data channels.
**Flash**
- `flash.rs`: SoftDevice-safe flash read/write/erase example.


**Glossary**
- `BLE`: Bluetooth Low Energy
- `BAS`: Battery Service
- `DIS`: Device Information Service
- `GATT`: Generic Attribute Profile
- `HID`: Human Interface Device
- `L2CAP`: Logical Link Control and Adaptation Protocol
- `CoC`: Credit-based Connection-oriented Channel
- `OTA`: Over-the-Air
- `GATT server`: device exposing services/characteristics
- `central`: BLE initiator/client, usually scanner/connector
- `peripheral`: BLE advertiser/server, usually accepts connections
- `bonding`: storing long-term pairing keys
- `notification`: server pushes characteristic updates without explicit read request