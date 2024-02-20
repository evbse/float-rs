use core::ops;

use crate::from_bytes::table_small::{SMALL_F32_POW10, SMALL_F64_POW10};

pub trait Float:
    Copy
    + ops::Div<Output = Self>
    + ops::Mul<Output = Self>
    + ops::Neg<Output = Self>
    + private::Sealed
{
    const MAX_DIGITS: usize;

    const SIGN_MASK: u64;
    const EXP_MASK: u64;
    const HIDDEN_BIT_MASK: u64;
    const MANT_MASK: u64;

    const MANT_SIZE: i32;
    const EXP_BIAS: i32;
    const DENORMAL_EXP: i32;
    const MAX_EXP: i32;

    const CARRY_MASK: u64;

    const INVALID_FP: i32 = -0x8000;

    const MAX_MANTISSA_FAST_PATH: u64 = 2_u64 << Self::MANT_SIZE;

    const INFINITE_POWER: i32 = Self::MAX_EXP + Self::EXP_BIAS;

    const MIN_EXP_ROUND_TO_EVEN: i32;
    const MAX_EXP_ROUND_TO_EVEN: i32;

    const MIN_EXP: i32;

    const SMALLEST_POWER_OF_TEN: i32;

    const LARGEST_POWER_OF_TEN: i32;

    const MIN_EXP_FAST_PATH: i32;

    const MAX_EXP_FAST_PATH: i32;

    const MAX_EXP_DISGUISED_FAST_PATH: i32;

    fn from_u64(u: u64) -> Self;

    fn from_bits(u: u64) -> Self;
    fn to_bits(self) -> u64;

    unsafe fn pow_fast_path(exponent: usize) -> Self;

    fn is_denormal(self) -> bool {
        self.to_bits() & Self::EXP_MASK == 0
    }

    fn exponent(self) -> i32 {
        if self.is_denormal() {
            return Self::DENORMAL_EXP;
        }

        let bits = self.to_bits();
        let biased_e: i32 = ((bits & Self::EXP_MASK) >> Self::MANT_SIZE) as i32;
        biased_e - Self::EXP_BIAS
    }

    fn mantissa(self) -> u64 {
        let bits = self.to_bits();
        let s = bits & Self::MANT_MASK;
        if !self.is_denormal() {
            s + Self::HIDDEN_BIT_MASK
        } else {
            s
        }
    }
}

impl Float for f32 {
    const MAX_DIGITS: usize = 114;
    const SIGN_MASK: u64 = 0x80000000;
    const EXP_MASK: u64 = 0x7F800000;
    const HIDDEN_BIT_MASK: u64 = 0x00800000;
    const MANT_MASK: u64 = 0x007FFFFF;
    const MANT_SIZE: i32 = 23;
    const EXP_BIAS: i32 = 127 + Self::MANT_SIZE;
    const DENORMAL_EXP: i32 = 1 - Self::EXP_BIAS;
    const MAX_EXP: i32 = 0xFF - Self::EXP_BIAS;
    const CARRY_MASK: u64 = 0x1000000;
    const MIN_EXP_ROUND_TO_EVEN: i32 = -17;
    const MAX_EXP_ROUND_TO_EVEN: i32 = 10;
    const MIN_EXP: i32 = -127;
    const SMALLEST_POWER_OF_TEN: i32 = -65;
    const LARGEST_POWER_OF_TEN: i32 = 38;
    const MIN_EXP_FAST_PATH: i32 = -10;
    const MAX_EXP_FAST_PATH: i32 = 10;
    const MAX_EXP_DISGUISED_FAST_PATH: i32 = 17;

    unsafe fn pow_fast_path(exponent: usize) -> Self {
        unsafe { *SMALL_F32_POW10.get_unchecked(exponent) }
    }

    fn from_u64(u: u64) -> f32 {
        u as _
    }

    fn from_bits(u: u64) -> f32 {
        f32::from_bits(u as u32)
    }

    fn to_bits(self) -> u64 {
        f32::to_bits(self) as u64
    }
}

impl Float for f64 {
    const MAX_DIGITS: usize = 769;
    const SIGN_MASK: u64 = 0x8000000000000000;
    const EXP_MASK: u64 = 0x7FF0000000000000;
    const HIDDEN_BIT_MASK: u64 = 0x0010000000000000;
    const MANT_MASK: u64 = 0x000FFFFFFFFFFFFF;
    const MANT_SIZE: i32 = 52;
    const EXP_BIAS: i32 = 1023 + Self::MANT_SIZE;
    const DENORMAL_EXP: i32 = 1 - Self::EXP_BIAS;
    const MAX_EXP: i32 = 0x7FF - Self::EXP_BIAS;
    const CARRY_MASK: u64 = 0x20000000000000;
    const MIN_EXP_ROUND_TO_EVEN: i32 = -4;
    const MAX_EXP_ROUND_TO_EVEN: i32 = 23;
    const MIN_EXP: i32 = -1023;
    const SMALLEST_POWER_OF_TEN: i32 = -342;
    const LARGEST_POWER_OF_TEN: i32 = 308;
    const MIN_EXP_FAST_PATH: i32 = -22;
    const MAX_EXP_FAST_PATH: i32 = 22;
    const MAX_EXP_DISGUISED_FAST_PATH: i32 = 37;

    unsafe fn pow_fast_path(exponent: usize) -> Self {
        unsafe { *SMALL_F64_POW10.get_unchecked(exponent) }
    }

    fn from_u64(u: u64) -> f64 {
        u as _
    }

    fn from_bits(u: u64) -> f64 {
        f64::from_bits(u)
    }

    fn to_bits(self) -> u64 {
        f64::to_bits(self)
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ExtendedFloat {
    pub(crate) mant: u64,
    pub(crate) exp: i32,
}

pub(crate) fn extended_to_float<F: Float>(x: ExtendedFloat) -> F {
    let mut word = x.mant;
    word |= (x.exp as u64) << F::MANT_SIZE;
    F::from_bits(word)
}
