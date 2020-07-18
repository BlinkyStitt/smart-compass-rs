mod networked;
mod patterns;

use feather_m0::prelude::_embedded_hal_spi_FullDuplex;

use smart_leds::{brightness, gamma, RGB8, SmartLedsWrite};
use ws2812_spi::Ws2812;

// TODO: better trait bounds
// TODO: return the result instead of unwrapping?
pub fn draw<L>(leds: &mut Ws2812<L>, data: &[RGB8], led_brightness: u8) -> Result<(), L::Error>
    where
        L: _embedded_hal_spi_FullDuplex<u8>
{
    // correct colors
    let data = gamma(data.iter().cloned());

    // dim the lights
    let data = brightness(data, led_brightness);

    // display
    leds.write(data)
}
