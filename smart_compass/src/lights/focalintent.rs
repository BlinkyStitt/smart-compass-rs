//! Ports of FastLED helpers.
//!
//! A massive "thank you" to Dan Garcia!
//! FastLED has helped me and so many others make beautiful lights.
//! But now I'm using rust, and I can't use https://fastled.io
//!
//! TODO: some of these functions would be useful
//!  - http://fastled.io/docs/3.1/group__lib8tion.html
//!  - http://fastled.io/docs/3.1/group___dimming.html
//!  - http://fastled.io/docs/3.1/group___noise.html
//!
//! TODO: "video" dimming like FastLED does?
//!
//! TODO: theres a lot of casting between u8, u16, u32, and i16. I'm not sure it is all correct
use crate::ELAPSED_MS;
use smart_leds::RGB8;

// LIB8STATIC uint8_t blend8( uint8_t a, uint8_t b, uint8_t amountOfB)
pub fn blend8(a: u8, b: u8, amount_of_b: u8) -> u8 {
    // uint8_t amountOfA = 255 - amountOfB;
    let amount_of_a = 255 - amount_of_b;
    
    // partial = (a * amountOfA);
    // partial += a;
    let mut partial: u16 = a as u16 * (amount_of_a + 1) as u16;
    
    // partial += (b * amountOfB);
    // partial += b;
    partial += b as u16 * (amount_of_b + 1) as u16;
    
    // result = partial >> 8;
    // return result;
    (partial >> 8) as u8
}

// TODO: generic type for leds? Maybe using Iter?
pub fn fade_to_black_by(leds: &mut [RGB8], amount: u8) {
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

/// beat88 generates a 16-bit 'sawtooth' wave at a given BPM,
/// with BPM specified in Q8.8 fixed-point format; e.g.
/// for this function, 120 BPM MUST BE specified as
/// 120*256 = 30720.
/// If you just want to specify "120", use beat16 or beat8.
///
/// BPM is 'beats per minute', or 'beats per 60000ms'.
/// To avoid using the (slower) division operator, we
/// want to convert 'beats per 60000ms' to 'beats per 65536ms',
/// and then use a simple, fast bit-shift to divide by 65536.
///
/// The ratio 65536:60000 is 279.620266667:256; we'll call it 280:256.
/// The conversion is accurate to about 0.05%, more or less,
/// e.g. if you ask for "120 BPM", you'll get about "119.93".
pub fn beat88(bpm88: u16, time_base: u32) -> u16 {
    let now = unsafe { ELAPSED_MS };

    (((now - time_base) * bpm88 as u32 * 280) >> 16) as u16
}

/// beatsin88 generates a 16-bit sine wave at a given BPM,
/// that oscillates within a given range.
/// For this function, BPM MUST BE SPECIFIED as
/// a Q8.8 fixed-point value; e.g. 120BPM must be
/// specified as 120*256 = 30720.
/// If you just want to specify "120", use beatsin16 or beatsin8.
pub fn beatsin88(bpm88: u16, low: u16, high: u16, time_base: u32, phase_offset: u16) -> u16 {
    // uint16_t beat = beat88( beats_per_minute_88, timebase);
    let beat = beat88(bpm88, time_base);

    // uint16_t beatsin = (sin16( beat + phase_offset) + 32768);
    let beat_sin = (sin16(beat + phase_offset) + 32767 + 1) as u16;

    // uint16_t rangewidth = highest - lowest;
    let range_width = high - low;

    // uint16_t scaledbeat = scale16( beatsin, rangewidth);
    let scaledbeat = scale16(beat_sin, range_width);

    // uint16_t result = lowest + scaledbeat;
    // return result;
    low + scaledbeat
}

/// scale a 16-bit unsigned value by a 16-bit value,
/// considered as numerator of a fraction whose denominator
/// is 65536. In other words, it computes i * (scale / 65536)
pub fn scale16(i: u16, scale: u16) -> u16 {
    ((i as u32) * (1 + (scale as u32)) / 65536) as u16
}

/*
pub fn scale8_video() {
    todo!();
}
*/

/*
/// Fast 8-bit approximation of sin(x). This approximation never varies more than
/// 2% from the floating point value you'd get by doing
///
///     float s = (sin(x) * 128.0) + 128;
///
/// @param theta input angle from 0-255
/// @returns sin of theta, value between 0 and 255
pub fn sin8(theta: u8) -> u8 {
    const B_M16_INTERLEAVE: [u8; 8] = [ 0, 49, 49, 41, 90, 27, 117, 10 ];

    // uint8_t offset = theta;
    let mut offset = theta;
    // if( theta & 0x40 ) {
    if theta & 0x40 != 0 {
    //     offset = (uint8_t)255 - offset;
        offset = 255 - offset;
    }
    // offset &= 0x3F; // 0..63
    offset &= 0x3f;

    // uint8_t secoffset  = offset & 0x0F; // 0..15
    let mut secoffset = offset & 0x0F;
    // if( theta & 0x40) secoffset++;
    if theta & 0x40 != 0 {
        secoffset += 1;
    }

    // uint8_t section = offset >> 4; // 0..3
    let section = offset >> 4;
    // uint8_t s2 = section * 2;
    let s2 = section * 2;
    // const uint8_t* p = b_m16_interleave;
    // p += s2;
    // uint8_t b   =  *p;
    let b = B_M16_INTERLEAVE[s2 as usize];
    // p++;
    // uint8_t m16 =  *p;
    let m16 = B_M16_INTERLEAVE[(s2 + 1) as usize];

    // uint8_t mx = (m16 * secoffset) >> 4;
    let mx = (m16 * secoffset) >> 4;

    // int8_t y = mx + b;
    let mut y = mx + b;

    // if( theta & 0x80 ) y = -y;
    if theta & 0x80 != 0 {
        // TODO: is this correct?
        y = 255 - y;
    }

    // y += 128;
    y += 128;

    // return y;
    y
}
*/

/// Fast 16-bit approximation of sin(x). This approximation never varies more than
/// 0.69% from the floating point value you'd get by doing
///
/// "float s = sin(x) * 32767.0;"
///
/// @param theta input angle from 0-65535
/// @returns sin of theta, value between -32767 to 32767.
pub fn sin16(theta: u16) -> i16 {
    // static const uint16_t base[] =
    // { 0, 6393, 12539, 18204, 23170, 27245, 30273, 32137 };
    const BASE: [u16; 8] = [ 0, 6393, 12539, 18204, 23170, 27245, 30273, 32137 ];

    // static const uint8_t slope[] = { 49, 48, 44, 38, 31, 23, 14, 4 };
    const SLOPE: [u8; 8] = [ 49, 48, 44, 38, 31, 23, 14, 4 ];

    // uint16_t offset = (theta & 0x3FFF) >> 3; // 0..2047
    let mut offset: u16 = (theta & 0x3FFF) >> 3;

    // if( theta & 0x4000 ) offset = 2047 - offset;
    if theta & 0x4000 != 0 {
        offset = 2047 - offset;
    }

    // uint8_t section = offset / 256; // 0..7
    let section: u8 = (offset / 256) as u8;

    // uint16_t b   = base[section];
    let b: u16 = BASE[section as usize];
    // uint8_t  m   = slope[section];
    let m: u8 = SLOPE[section as usize];

    // uint8_t secoffset8 = (uint8_t)(offset) / 2;
    let secoffset8: u8 = (offset as u8) / 2;

    // uint16_t mx = m * secoffset8;
    let mx: u16 = (m as u16) * (secoffset8 as u16);
    // int16_t y = mx + b;
    let mut y: i16 = (mx as i16) + (b as i16);

    // if( theta & 0x8000 ) y = -y;
    if theta & 0x8000 != 0 {
        y = - y;
    }

    // return y;
    y
}

// CRGB& nblend( CRGB& existing, const CRGB& overlay, fract8 amountOfOverlay )
pub fn nblend(existing: &mut RGB8, overlay: &RGB8, amount_of_overlay: u8) {
    if amount_of_overlay == 0 {
        return;
    }

    if amount_of_overlay == 255 {
        existing.r = overlay.r;
        existing.g = overlay.g;
        existing.b = overlay.b;
        return;
    }

    existing.r = blend8(existing.r, overlay.r, amount_of_overlay);
    existing.g = blend8(existing.g, overlay.g, amount_of_overlay);
    existing.b = blend8(existing.b, overlay.b, amount_of_overlay);
    
    return;
}

/*
pub fn blur1d() {
    todo!();
}
*/

/*
pub fn blur2d() {
    todo!();
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale16() {
        assert_eq!(scale16(0, 0), 0);
        assert_eq!(scale16(0, u16::MAX), 0);
        assert_eq!(scale16(64, 32768), 32);
        assert_eq!(scale16(256, 32768), 128);
        assert_eq!(scale16(u16::MAX, u16::MAX), u16::MAX);
    }

    #[test]
    fn test_sin16() {
        assert_eq!(sin16(0), 0);
        assert_eq!(sin16(9), 0);
        assert_eq!(sin16(256), 784);
        assert_eq!(sin16(1024), 3136);
        assert_eq!(sin16(32761), 0);
        assert_eq!(sin16(36100), -10233);
        assert_eq!(sin16(49284), -32613);
        assert_eq!(sin16(64516), -3087);
    }
}
