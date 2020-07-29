/// A massive "thank you" to Dan Garcia!
/// FastLED has helped me and so many others make beautiful lights.
/// But now I'm using rust, and I can't use https://fastled.io
use smart_leds::RGB8;

// TODO: generic type for leds?
pub fn fadeToBlackBy(leds: &mut [RGB8], amount: u8) {
    for led in leds.iter_mut() {
        // TODO: is there a better way to do saturating subtraction for leds?
        if led.r > amount {
            led.r -= amount;
        } else {
            led.r = 0;
        }
        if led.g > amount {
            led.g -= amount;
        } else {
            led.g = 0;
        }
        if led.b > amount {
            led.b -= amount;
        } else {
            led.b = 0;
        }
    }
}
