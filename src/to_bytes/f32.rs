use core::{mem, ptr, slice, str};

use crate::to_bytes::{
    common::{
        floor_log10_pow2, floor_log10_pow2_minus_log10_4_over_3, floor_log2_pow10, func, Float,
        LoHi, INFINITY, NAN, NEG_INFINITY,
    },
    to_chars::write_f32 as to_buffer,
};

const EXPONENT_MASK: u32 = 0x7f800000;
const MANTISSA_MASK: u32 = 0x007fffff;
const SIGN_MASK: u32 = 0x80000000;

const MAX_BUFFER_LEN: usize = 1 + 9 + 1 + 1 + 1 + 2;

const MANTISSA_BITS: usize = 23;
const EXPONENT_BITS: usize = 8;
const MIN_EXPONENT: i32 = -126;
const EXPONENT_BIAS: i32 = -127;

const CARRIER_BITS: usize = 32;
const KAPPA: u32 = 1;

const BIG_DIVISOR: u32 = 100;
const SMALL_DIVISOR: u32 = 10;

const MAGIC_NUMBER: u32 = 6554;
const SHIFT_AMOUNT: i32 = 16;

const SHORTER_INTERVAL_TIE_LOWER_THRESHOLD: i32 = -35;
const SHORTER_INTERVAL_TIE_UPPER_THRESHOLD: i32 = -35;

type Wide = u64;

const MIN_K: i32 = -31;
const MAX_K: i32 = 46;

unsafe fn get(k: i32) -> Wide {
    debug_assert!(k >= MIN_K && k <= MAX_K);
    *CACHE.get_unchecked((k - MIN_K) as usize)
}

static CACHE: [Wide; 78] = [
    0x81ceb32c4b43fcf5,
    0xa2425ff75e14fc32,
    0xcad2f7f5359a3b3f,
    0xfd87b5f28300ca0e,
    0x9e74d1b791e07e49,
    0xc612062576589ddb,
    0xf79687aed3eec552,
    0x9abe14cd44753b53,
    0xc16d9a0095928a28,
    0xf1c90080baf72cb2,
    0x971da05074da7bef,
    0xbce5086492111aeb,
    0xec1e4a7db69561a6,
    0x9392ee8e921d5d08,
    0xb877aa3236a4b44a,
    0xe69594bec44de15c,
    0x901d7cf73ab0acda,
    0xb424dc35095cd810,
    0xe12e13424bb40e14,
    0x8cbccc096f5088cc,
    0xafebff0bcb24aaff,
    0xdbe6fecebdedd5bf,
    0x89705f4136b4a598,
    0xabcc77118461cefd,
    0xd6bf94d5e57a42bd,
    0x8637bd05af6c69b6,
    0xa7c5ac471b478424,
    0xd1b71758e219652c,
    0x83126e978d4fdf3c,
    0xa3d70a3d70a3d70b,
    0xcccccccccccccccd,
    0x8000000000000000,
    0xa000000000000000,
    0xc800000000000000,
    0xfa00000000000000,
    0x9c40000000000000,
    0xc350000000000000,
    0xf424000000000000,
    0x9896800000000000,
    0xbebc200000000000,
    0xee6b280000000000,
    0x9502f90000000000,
    0xba43b74000000000,
    0xe8d4a51000000000,
    0x9184e72a00000000,
    0xb5e620f480000000,
    0xe35fa931a0000000,
    0x8e1bc9bf04000000,
    0xb1a2bc2ec5000000,
    0xde0b6b3a76400000,
    0x8ac7230489e80000,
    0xad78ebc5ac620000,
    0xd8d726b7177a8000,
    0x878678326eac9000,
    0xa968163f0a57b400,
    0xd3c21bcecceda100,
    0x84595161401484a0,
    0xa56fa5b99019a5c8,
    0xcecb8f27f4200f3a,
    0x813f3978f8940985,
    0xa18f07d736b90be6,
    0xc9f2c9cd04674edf,
    0xfc6f7c4045812297,
    0x9dc5ada82b70b59e,
    0xc5371912364ce306,
    0xf684df56c3e01bc7,
    0x9a130b963a6c115d,
    0xc097ce7bc90715b4,
    0xf0bdc21abb48db21,
    0x96769950b50d88f5,
    0xbc143fa4e250eb32,
    0xeb194f8e1ae525fe,
    0x92efd1b8d0cf37bf,
    0xb7abc627050305ae,
    0xe596b7b0c643c71a,
    0x8f7e32ce7bea5c70,
    0xb35dbf821ae4f38c,
    0xe0352f62a19e306f,
];

func!(f32, u32, u64);
