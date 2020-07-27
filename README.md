# Smart Compass V1

## Setup

```sh
brew cask install gcc-arm-embedded

# for on-chip debugging
# Forewarning: Semihosting operations are very slow. Like, each WRITE operation can take hundreds of milliseconds
brew install open-ocd

# for samd11, samd21:
rustup target add thumbv6m-none-eabi

# for samd51, same54:
rustup target add thumbv7em-none-eabihf

# for uploading the code onto devices with bootloaders
cargo install cargo-hf2
```

## Development

1. Hit the device reset button twice.

2. Build and upload one of the programs:

    ```sh
    # OR
    cd smart_compass_feather_m0
    cargo hf2 --release --bin test_lights

    # OR if hf2 doesn't work
    cd smart_compass_feather_m0

    cargo build --release --bin test_lights

    arm-none-eabi-objcopy -O binary \
        ../target/thumbv6m-none-eabi/release/test_lights \
        ../target/thumbv6m-none-eabi/release/test_lights.bin

    # note: if Mac Catalina is trying to delete this file, open system preferences > Security and there should be a buttoon to allow arm-none-eabi-objcopy.

    # plug in the feather_m0

    # MAYBE: stty -F /dev/cu.usbmodem14201 ospeed 1200

    # double press the reset button

    ~/Library/Arduino15/packages/arduino/tools/bossac/1.7.0/bossac -i -d -U true -i -e -w -v \
        --port=cu.usbmodem14201 \
        ../target/thumbv6m-none-eabi/release/test_lights.bin -R

    # OR
    cd smart_compass_feather_m0
    cargo hf2 --release --bin smart_compass

    # OR
    cd smart_compass_stm32f3_discovery
    cargo hf2 --release --bin smart_compass
    ```

## Reading

- <https://docs.rs/cortex-m-semihosting/0.3.5/cortex_m_semihosting/>
- <https://github.com/atsamd-rs/atsamd/blob/master/boards/feather_m0/README.md>
- <https://github.com/atsamd-rs/atsamd>
- <https://crates.io/crates/cargo-hf2>
- <https://github.com/smart-leds-rs/smart-leds-samples/blob/master/stm32f0-examples/examples/stm32f0_ws2812_spi_blink.rs>
- <https://docs.rs/crate/adafruit_gps/0.3.6/source/examples/simple.rs>
- <https://gitter.im/smart-leds-rs/community?at=5c926bc7fcaf7b5f73e7158c>
