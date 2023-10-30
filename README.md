# Universal Wallpad Serial Bridge over MQTT (including the Broker)

[한국어](README_ko.md) | [English](README.md)

This project is just an RS485-MQTT broker, so it needs to be used in conjunction with other apps to build a smart home.

If you are using HomeAssistant, see the [HomeAssistant Connect Guide (Preparing)]() for more information.

See [MQTT Device Topic Reference (Preparing)]() when connecting to other MQTT clients.

Please submit a pull request for incorrect translations. Thank you.

## Quick Start


[Download](https://github.com/blkis0/universal-wallpad-bridge/releases) the version appropriate for your environment.

Create the ``'rumqttd.toml'`` file in the same folder and refer to [this](https://github.com/blkis0/universal-wallpad-bridge/rumqttd.toml) when writing.
You can use a different path with the ``--rumqttd=<path>`` option.

Open a terminal and enter the following command

Windows
```
./universal-wallpad-bridge.exe -m <Manufacturer> -- <Primary Serial Port (ex. COM1)>
```

Linux
```
./universal-wallpad-bridge -m <Manufacturer> -- <Primary Serial Port (ex. /dev/ttyUSB0)> 
```

### Usable Options

|Required|Key|Value: Type|Description|
|---|---|---|---|
|O||<Serial Port: string>|Primary Serial Port - connected to anything devices. (ex. COM1 or /dev/ttyUSB0, etc...)
|O|-m / --manufacturer|[Manufacturer: string]|Wall-pad Manufacturer - Select your devices [manufacturer](#manufacturer-specific-features).
|-|-f / --features|[Feature1: string] [Feature2: string]...|Available Feature Types - Select the [available devices type](#supported-apartments). (Separated by commas)
|-|-s / --second-port|<Serial Port: string>|Secondary Serial Port - connected to the digital doorlock and energy meter. (ex. COM2 or /dev/ttyUSB1, etc...)
|-|-r / --rumqttd|<Path: string>|MQTT Broker Setting Path - specified path for ``rumqttd.toml`` (Default: ./rumqttd.toml )
|-|-i / --interval|<Numeric: uint64>|Fetch Interval (Default: 2s)
|-|--log|<Path: string>|Logging all packets.
|-|-v||Print more various information.

### See also

- [MQTT Device Topic Reference (Preparing)]()
- [Repository of rumqttd (Github)](https://github.com/bytebeamio/rumqtt/tree/main/rumqttd)
- [How to use the app on the Docker (Preparing)]()

## Build

Before, Install [Rust Build Tools](https://www.rust-lang.org/tools/install) on your computer.

After, Follow these steps 

- Download and unextract or clone the project to your computer.
    ```bash
    git clone https://github.com/blkis0/universal-wallpad-bridge
    ```

- Move to the project folder and start building with the following code.
    ```bash
    cargo build --release
    ```

- Now, you can find an executable program in the ``target`` directory

<br>

To start the program immediately, you can use the following command.

```bash
cargo run --release -- -m <Manufacturer> -- <Primary Serial Port (ex. COM1 or /dev/ttyUSB0, etc...)>
```

## Compatibility

Currently, It is only available for some wall pads.

You can use ``Ctrl + F`` or ``F3`` to search for the wall pad manufacturer.

### Available Manufacturer

- HyundaiHT (hyundai_ht)

### Manufacturer-specific features

#### HyundaiHT (hyundai_ht)

|Available|Type Name|Name|Additional Features|Description
|---|---|---|---|---|
O|floor_heating|Floor Heating|-|Ready on the 4-room controller
O|ventilator|Central Ventilator|-|Enabled the Passthrough, unsupported on the wall pad, on the Device of Shinwoo Air Conditioning
O|living_room_lights|Living Room Ceiling Lights|-|-
O|realtime_energy_meter|Realtime Energy Meter|Auto Fetch|-
-|gas_valve|Gas Valve|-|Developing
-|elevator_call|Elevator Call|-|Developing
-|lights|Central Light Switch|-|Developing
-|doorlock|Digital Door Lock|-|Developing
<br>