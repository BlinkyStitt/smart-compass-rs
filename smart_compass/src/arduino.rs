//

/// TODO: use generics
pub fn constrain(x: u8, low: u8, high: u8) -> u8 {
    if x <= low {
        return low;
    }
    if x >= high {
        return high;
    }
    x
}

pub fn map<N: num::Num + Copy>(x: N, in_min: N, in_max: N, out_min: N, out_max: N) -> N {
    return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}
