# thermostat

## This is for my experimental purpose. I am NOT responsible and have no liabilities for anything happend with this program.

Use this at your own risk.

CAUTION: Do not use with appliances which will be critical in case of failure; follow what are stated in SwitchBot Plug mini manual.  
Again, I am not responsible for anything happend with this program.

## What is this?

Controlling BLE enabled smart plug with temperature/humidity sensor on Raspberry Pi.

Tested and developed with following hardwares.  
This might work on Raspberry Pi B+/2 B+ so on... but not tested yet.

## Hardware Requirements

* Raspberry Pi 3 (B+)
* DHT22 Environment Sensor
* SwitchBot Plug mini (JP)

## Software Requirements

* Raspberry Pi OS - bullseye (11), September 22nd, 2022
    * `build-essential` package group
* Rust 1.66.0
    * cargo 1.66.0

## How to build

1. copy/rename `.env.example` to `.env`
2. fill in `.env` parameters
3. `cargo build` or `cargo build -r`
    * `.env` file must be present before building program.

## `.env` explanation

```
SENSOR_PIN: GPIO Pin that DHT22 is connected to

SWITCH_PLUG_MAC: BLE MAC of SwitchBot Plug mini

UPPER_TEMP: temperature in celsius to trigger switch OFF

LOWER_TEMP: temperature in celsius to trigger switch ON

PROFILE: 'debug' value to enable extra debug logs for BLE device data
```

## Usage

example:

`sudo RUST_LOG='info' ./thermostat`

Root privilage is required to access GPIO data.

Set `RUST_LOG` environment variable to filter.  
See log crate or simple_logger crate for the value.

The built program will be at `./target/<build_profile>/thermostat` by the default cargo setting.

## License

MIT
