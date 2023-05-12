pub(crate) trait LoHi {
    type O;
    fn low(self) -> Self::O;
    fn high(self) -> Self::O;
}
impl LoHi for u64 {
    type O = u32;
    fn low(self) -> Self::O {
        self as Self::O
    }
    fn high(self) -> Self::O {
        (self >> 32) as Self::O
    }
}
impl LoHi for u128 {
    type O = u64;
    fn low(self) -> Self::O {
        self as Self::O
    }
    fn high(self) -> Self::O {
        (self >> 64) as Self::O
    }
}

pub trait Float: Sealed {}

pub trait Sealed: Copy {
    fn is_nonfinite(self) -> bool;
    fn format_nonfinite(self) -> &'static str;
    unsafe fn write_to_buffer(self, buffer: *mut u8) -> usize;
}

pub(crate) const NAN: &str = "NaN";
pub(crate) const INFINITY: &str = "inf";
pub(crate) const NEG_INFINITY: &str = "-inf";

pub(crate) fn floor_log10_pow2(e: i32) -> i32 {
    const MIN_EXPONENT: i32 = -2620;
    const MAX_EXPONENT: i32 = 2620;
    debug_assert!(MIN_EXPONENT <= e && e <= MAX_EXPONENT);
    (e * 315653) >> 20
}

pub(crate) fn floor_log2_pow10(e: i32) -> i32 {
    const MIN_EXPONENT: i32 = -1233;
    const MAX_EXPONENT: i32 = 1233;
    debug_assert!(MIN_EXPONENT <= e && e <= MAX_EXPONENT);
    (e * 1741647) >> 19
}

pub(crate) fn floor_log10_pow2_minus_log10_4_over_3(e: i32) -> i32 {
    const MIN_EXPONENT: i32 = -2985;
    const MAX_EXPONENT: i32 = 2936;
    debug_assert!(MIN_EXPONENT <= e && e <= MAX_EXPONENT);
    (e * 631305 - 261663) >> 21
}

macro_rules! func {
    ($f:ty, $t:ty, $w:ty) => {
        pub struct Buffer {
            bytes: [mem::MaybeUninit<u8>; MAX_BUFFER_LEN],
        }

        impl Buffer {
            pub fn new() -> Self {
                let bytes = [mem::MaybeUninit::<u8>::uninit(); MAX_BUFFER_LEN];
                Buffer { bytes }
            }

            pub fn format(&mut self, f: $f) -> &str {
                if f.is_nonfinite() {
                    f.format_nonfinite()
                } else {
                    unsafe {
                        let n = f.write_to_buffer(self.bytes.as_mut_ptr().cast());
                        debug_assert!(n <= self.bytes.len());
                        let slice = slice::from_raw_parts(self.bytes.as_ptr().cast(), n);
                        str::from_utf8_unchecked(slice)
                    }
                }
            }
        }

        impl Float for $f {}

        impl Sealed for $f {
            fn is_nonfinite(self) -> bool {
                let bits = self.to_bits();
                bits & EXPONENT_MASK == EXPONENT_MASK
            }

            fn format_nonfinite(self) -> &'static str {
                let bits = self.to_bits();
                if bits & MANTISSA_MASK != 0 {
                    NAN
                } else if bits & SIGN_MASK != 0 {
                    NEG_INFINITY
                } else {
                    INFINITY
                }
            }

            unsafe fn write_to_buffer(self, buffer: *mut u8) -> usize {
                let end = to_chars(self, buffer);
                end.offset_from(buffer) as usize
            }
        }

        fn extract_exponent_bits(u: $t) -> u32 {
            const EXPONENT_BITS_MASK: u32 = (1 << EXPONENT_BITS) - 1;
            (u >> MANTISSA_BITS) as u32 & EXPONENT_BITS_MASK
        }

        fn remove_exponent_bits(u: $t, exponent_bits: u32) -> $t {
            u ^ ((exponent_bits as $t) << MANTISSA_BITS)
        }

        fn remove_sign_bit_and_shift(u: $t) -> $t {
            u << 1
        }

        fn is_nonzero(u: $t) -> bool {
            (u << 1) != 0
        }

        fn is_positive(u: $t) -> bool {
            const SIGN_BIT: $t = (1 as $t) << (MANTISSA_BITS + EXPONENT_BITS);
            u < SIGN_BIT
        }

        fn is_negative(u: $t) -> bool {
            !is_positive(u)
        }

        fn has_even_mantissa_bits(u: $t) -> bool {
            u % 2 == 0
        }

        fn compute_mul(u: $t, cache: $w) -> ($t, bool) {
            let r = upper_bits(u, cache);
            (r.high(), r.low() == 0)
        }

        fn compute_delta(cache: $w, beta: i32) -> u32 {
            (cache.high() >> ((CARRIER_BITS - 1) as i32 - beta)) as u32
        }

        fn compute_mul_parity(u: $t, cache: $w, beta: i32) -> (bool, bool) {
            let r = lower_bits(u, cache);
            (
                (r.high() >> (CARRIER_BITS as i32 - beta)) & 1 != 0,
                (r.high() << beta | (r.low() >> (CARRIER_BITS as i32 - beta))) == 0,
            )
        }

        fn check_divisibility_and_divide_by_pow10(mut n: u32) -> (u32, bool) {
            n *= MAGIC_NUMBER;
            const MASK: u32 = (1 << SHIFT_AMOUNT) as u32 - 1;
            let is_divisible = (n & MASK) < MAGIC_NUMBER;
            n >>= SHIFT_AMOUNT;
            (n, is_divisible)
        }

        fn upper_bits(x: $t, y: $w) -> $w {
            let xyh = (x as $w) * (y.high() as $w);
            let xyl = (x as $w) * (y.low() as $w);

            xyh + (xyl >> CARRIER_BITS)
        }

        fn lower_bits(x: $t, y: $w) -> $w {
            // let xyh = (x as $w) * (y.high() as $w);
            // let xyl = (x as $w) * (y.low() as $w);

            // ((xyh + xyl.high() as $w) << CARRIER_BITS) | xyl.low() as $w

            (x as $w).wrapping_mul(y)
        }

        fn compute_nearest_normal(
            two_fc: $t,
            exponent: i32,
            has_even_mantissa_bits: bool,
        ) -> ($t, i32) {
            let include_left_endpoint = has_even_mantissa_bits;
            let include_right_endpoint = has_even_mantissa_bits;

            let minus_k = floor_log10_pow2(exponent) - KAPPA as i32;
            let cache = unsafe { get(-minus_k) };
            let beta = exponent + floor_log2_pow10(-minus_k);

            let deltai = compute_delta(cache, beta);
            let (zi, is_z_integer) = compute_mul((two_fc | 1) << beta, cache);

            let mut mantissa = zi / BIG_DIVISOR as $t;
            let mut r = (zi - BIG_DIVISOR as $t * mantissa) as u32;

            'small_divisor_case: loop {
                if r < deltai {
                    if r == 0 && (is_z_integer & !include_right_endpoint) {
                        mantissa -= 1;
                        r = BIG_DIVISOR;
                        break 'small_divisor_case;
                    }
                } else if r > deltai {
                    break 'small_divisor_case;
                } else {
                    let (xi_parity, is_x_integer) = compute_mul_parity(two_fc - 1, cache, beta);
                    if !(xi_parity | (is_x_integer & include_left_endpoint)) {
                        break 'small_divisor_case;
                    }
                }
                let exponent = minus_k + KAPPA as i32 + 1;

                return (mantissa, exponent);
            }

            mantissa *= 10;
            let exponent = minus_k + KAPPA as i32;

            let dist = r - (deltai / 2) + (SMALL_DIVISOR / 2);
            let approx_y_parity = ((dist ^ (SMALL_DIVISOR / 2)) & 1) != 0;

            let (dist, is_divisible_by_small_divisor) =
                check_divisibility_and_divide_by_pow10(dist);

            mantissa += dist as $t;

            if is_divisible_by_small_divisor {
                let (yi_parity, is_y_integer) = compute_mul_parity(two_fc, cache, beta);
                if yi_parity != approx_y_parity {
                    mantissa -= 1;
                } else {
                    if prefer_round_down(mantissa) & is_y_integer {
                        mantissa -= 1;
                    }
                }
            }

            (mantissa, exponent)
        }

        fn compute_nearest_shorter(exponent: i32) -> ($t, i32) {
            let include_left_endpoint = true;
            let include_right_endpoint = true;

            let minus_k = floor_log10_pow2_minus_log10_4_over_3(exponent);
            let beta = exponent + floor_log2_pow10(-minus_k);

            let cache = unsafe { get(-minus_k) };

            let mut xi = compute_left_endpoint_for_shorter_interval_case(cache, beta);
            let mut zi = compute_right_endpoint_for_shorter_interval_case(cache, beta);

            if !include_right_endpoint && is_right_endpoint_integer_shorter_interval(exponent) {
                zi -= 1;
            }
            if !include_left_endpoint || !is_left_endpoint_integer_shorter_interval(exponent) {
                xi += 1;
            }

            let mantissa = zi / 10;

            if mantissa * 10 >= xi {
                let exponent = minus_k + 1;
                return (mantissa, exponent);
            }

            let mut mantissa = compute_round_up_for_shorter_interval_case(cache, beta);
            let exponent = minus_k;

            if prefer_round_down(mantissa)
                && exponent >= SHORTER_INTERVAL_TIE_LOWER_THRESHOLD
                && exponent <= SHORTER_INTERVAL_TIE_UPPER_THRESHOLD
            {
                mantissa -= 1;
            } else if mantissa < xi {
                mantissa += 1;
            }

            (mantissa, exponent)
        }

        fn compute_left_endpoint_for_shorter_interval_case(cache: $w, beta: i32) -> $t {
            (cache.high() - (cache.high() >> (MANTISSA_BITS + 2)))
                >> ((CARRIER_BITS - MANTISSA_BITS - 1) as i32 - beta)
        }

        fn compute_right_endpoint_for_shorter_interval_case(cache: $w, beta: i32) -> $t {
            (cache.high() + (cache.high() >> (MANTISSA_BITS + 1)))
                >> ((CARRIER_BITS - MANTISSA_BITS - 1) as i32 - beta)
        }

        fn compute_round_up_for_shorter_interval_case(cache: $w, beta: i32) -> $t {
            ((cache.high() >> ((CARRIER_BITS - MANTISSA_BITS - 2) as i32 - beta)) + 1) / 2
        }

        fn is_left_endpoint_integer_shorter_interval(exponent: i32) -> bool {
            const CASE_SHORTER_INTERVAL_LEFT_ENDPOINT_LOWER_THRESHOLD: i32 = 2;
            const CASE_SHORTER_INTERVAL_LEFT_ENDPOINT_UPPER_THRESHOLD: i32 = 3;

            exponent >= CASE_SHORTER_INTERVAL_LEFT_ENDPOINT_LOWER_THRESHOLD
                && exponent <= CASE_SHORTER_INTERVAL_LEFT_ENDPOINT_UPPER_THRESHOLD
        }

        fn is_right_endpoint_integer_shorter_interval(exponent: i32) -> bool {
            const CASE_SHORTER_INTERVAL_RIGHT_ENDPOINT_LOWER_THRESHOLD: i32 = 0;
            const CASE_SHORTER_INTERVAL_RIGHT_ENDPOINT_UPPER_THRESHOLD: i32 = 3;

            exponent >= CASE_SHORTER_INTERVAL_RIGHT_ENDPOINT_LOWER_THRESHOLD
                && exponent <= CASE_SHORTER_INTERVAL_RIGHT_ENDPOINT_UPPER_THRESHOLD
        }

        fn prefer_round_down(mantissa: $t) -> bool {
            mantissa % 2 != 0
        }

        fn to_decimal(x: $f) -> ($t, i32) {
            let br = x.to_bits();
            let exponent_bits = extract_exponent_bits(br);
            let signed_mantissa_bits = remove_exponent_bits(br, exponent_bits);

            let mut two_fc = remove_sign_bit_and_shift(signed_mantissa_bits);
            let mut exponent = exponent_bits as i32;

            if exponent != 0 {
                exponent += EXPONENT_BIAS - MANTISSA_BITS as i32;

                if two_fc == 0 {
                    return compute_nearest_shorter(exponent);
                }

                two_fc |= 1 << (MANTISSA_BITS + 1);
            } else {
                exponent = MIN_EXPONENT - MANTISSA_BITS as i32;
            }

            compute_nearest_normal(
                two_fc,
                exponent,
                has_even_mantissa_bits(signed_mantissa_bits),
            )
        }

        unsafe fn to_chars(x: $f, mut buffer: *mut u8) -> *mut u8 {
            let br = x.to_bits();
            let exponent_bits = extract_exponent_bits(br);
            let s = remove_exponent_bits(br, exponent_bits);

            if is_negative(s) {
                *buffer = b'-';
                buffer = buffer.add(1);
            }

            if is_nonzero(br) {
                let (significand, exponent) = to_decimal(x);
                to_buffer(significand, exponent, buffer)
            } else {
                ptr::copy_nonoverlapping(b"0E0".as_ptr(), buffer, 3);
                buffer.add(3)
            }
        }
    };
}
pub(crate) use func;
