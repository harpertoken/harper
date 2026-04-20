# Firmware Abstraction

Harper supports firmware operations for embedded devices.

## Supported Platforms

- ESP32
- STM32
- Raspberry Pi Pico

## Firmware Tool

Use `[FIRMWARE ...]` commands in chat:

```
[FIRMWARE list]              - List devices
[FIRMWARE info <device>]   - Device info
[FIRMWARE gpio <pin> <high|low>] - Control GPIO
[FIRMWARE i2c ...]       - I2C operations
[FIRMWARE spi ...]       - SPI operations
[FIRMWARE uart ...]      - UART operations
```

## Configuration

Add devices in config (`config/local.toml`):

```toml
[firmware]
enabled = true

[[firmware.devices]]
name = "my-esp32"
platform = "esp32"
port = "/dev/ttyUSB0"
```

## Programmatic Use

```rust
use harper_firmware::{Esp32Device, FirmwareRegistry};

let mut registry = FirmwareRegistry::new();
registry.register("esp32-1".to_string(), Box::new(
    Esp32Device::new("/dev/ttyUSB0", "my-esp32")
));
```

## Available Traits

- `GpioController` - GPIO pin control
- `I2cController` - I2C communication
- `SpiController` - SPI communication
- `UartController` - UART serial
- `PwmController` - PWM output
- `AdcController` - ADC input

See `lib/harper-firmware/examples/basic.rs` for more examples.
