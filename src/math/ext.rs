use std::iter::Step;
use std::ops::Neg;
use std::f64::consts::PI as PI_64;
use std::f32::consts::PI as PI_32;

use cgmath::{BaseNum, BaseFloat};

impl BaseFloatExt for f32 {
    const PI: Self = PI_32;
    const TWO: Self = 2.0;
    const TWO_PI: Self = PI_32 * 2.0;
}

impl BaseFloatExt for f64 {
    const PI: Self = PI_64;
    const TWO: Self = 2.0;
    const TWO_PI: Self = PI_64 * 2.0;
}

pub trait BaseFloatExt: BaseFloat {
    const PI: Self;
    const TWO: Self;
    const TWO_PI: Self;

    #[inline(always)]
    fn normalize_angle(self, center: Self) -> Self {
        self - Self::TWO_PI * ((self + Self::PI - center) / Self::TWO_PI).floor()
    }

    #[inline(always)]
    fn accurate_normalize_angle(self) -> Self {
        let (sin, cos) = self.sin_cos();
        sin.atan2(cos)
    }

    #[inline(always)]
    fn lerp(self, min: Self, max: Self) -> Self {
        (Self::one() - self) * min + self * max
    }

    #[inline(always)]
    fn clamp01(self) -> Self {
        self.clamp(Self::zero(), Self::one())
    }

    #[inline(always)]
    fn clamp(mut self, min: Self, max: Self) -> Self {
        if self < min { self = min; }
        if self > max { self = max; }
        self
    }
}

pub trait BaseNumExt: BaseNum + Neg<Output=Self> {
    #[inline(always)]
    fn abs(self) -> Self {
        if Self::zero() >= self {
            self
        } else {
            -self
        }
    }

    #[inline(always)]
    fn signum(self) -> Self {
        if Self::zero() == self {
            self
        } else if self > Self::one() {
            Self::one()
        } else {
            -Self::one()
        }
    }

    #[inline(always)]
    fn min(self, other: Self) -> Self {
        if other > self {
            self
        } else {
            other
        }
    }

    #[inline(always)]
    fn max(self, other: Self) -> Self {
        if other < self {
            self
        } else {
            other
        }
    }
}

impl<T> BaseNumExt for T where T: BaseNum + Neg<Output=Self> {}

pub trait BaseIntExt: BaseNumExt + Step {}

impl<T> BaseIntExt for T where T: BaseNumExt + Step {}
