//! Ports of helpful arduino functions.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constrain() {
        assert_eq!(constrain(0, 1, 3), 1);
        assert_eq!(constrain(1, 1, 3), 1);
        assert_eq!(constrain(2, 1, 3), 2);
        assert_eq!(constrain(3, 1, 3), 3);
        assert_eq!(constrain(4, 1, 3), 3);
    }

    #[test]
    fn test_map_f32() {
        assert_eq!(map(-1.0, -1.0, 1.0, 0.0, 2.0), 0.0);
        assert_eq!(map(0.0, -1.0, 1.0, 0.0, 2.0), 1.0);
        assert_eq!(map(1.0, -1.0, 1.0, 0.0, 2.0), 2.0);
        assert_eq!(map(2.0, -1.0, 1.0, 0.0, 2.0), 3.0);
    }
}
