#!/bin/bash -eux
# TODO: is there some to make cargo do this for us?

[ "$(basename $(pwd))" = "smart_compass_feather_m0" ]

target_bin=$1

cargo build --release --bin "$target_bin"

arm-none-eabi-objcopy -O binary \
    "../target/thumbv6m-none-eabi/release/$target_bin" \
    "../target/thumbv6m-none-eabi/release/$target_bin.bin"

# note: if Mac Catalina is trying to delete this command, open system preferences > Security and there should be a buttoon to allow arm-none-eabi-objcopy.

# plug in the feather_m0
# double press the reset button
# TODO: do this with stty 

# TODO: we might need an offset - https://users.rust-lang.org/t/getting-started-with-feather-m0-solved/38962/2
# arduino didn't include an offset though so maybe it is properly detected
# NOTE: if you don't have Arduino IDE installed, you can get bossac with `brew cask install bossa`
# TODO: option to specify what port to use
~/Library/Arduino15/packages/arduino/tools/bossac/1.7.0/bossac \
    -e -w -v -R \
    "../target/thumbv6m-none-eabi/release/$target_bin.bin"

echo "Success!"
