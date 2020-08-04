//
use num::Num;

pub fn constrain<N: Num + core::cmp::PartialOrd>(x: N, low: N, high: N) -> N {
    if x <= low {
        return low;
    }
    if x >= high {
        return high;
    }
    x
}

pub fn map<N: Num + Copy>(x: N, in_min: N, in_max: N, out_min: N, out_max: N) -> N {
    return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}
