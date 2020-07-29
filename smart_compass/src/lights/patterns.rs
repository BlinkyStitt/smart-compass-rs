/*
shifting

let shift_n = some number calculated from the elapsed time or the number of frames drawn

light_data.iter().cycle().skip(shift_n).take(256).cloned()

TODO: use https://docs.rs/microfft/0.3.0/microfft/ for sound reactive patterns?

TODO: "video" dimming like FastLED does?

TODO: some of these functions would be useful
 - http://fastled.io/docs/3.1/group__lib8tion.html
 - http://fastled.io/docs/3.1/group___dimming.html
 - http://fastled.io/docs/3.1/group___noise.html
*/
use smart_leds::RGB8;

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
pub fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}

// TODO: port these https://github.com/jasoncoon/esp8266-fastled-webserver/blob/fibonacci256/Map.h
