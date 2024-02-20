use crate::from_bytes::common::{Float, ExtendedFloat};
use crate::from_bytes::parse::{Number};
use crate::from_bytes::table_moderate::{POWER_OF_FIVE_128, SMALLEST_POWER_OF_FIVE};

pub(crate) fn moderate<F: Float>(num: &Number) -> ExtendedFloat {
    let mut fp = compute_float::<F>(num.exp, num.mant);
    if num.many_digits && fp.exp >= 0 && fp != compute_float::<F>(num.exp, num.mant + 1) {
        fp = compute_error::<F>(num.exp, num.mant);
    }
    fp
}

fn compute_float<F: Float>(q: i32, mut w: u64) -> ExtendedFloat {
    let fp_zero = ExtendedFloat { mant: 0, exp: 0 };
    let fp_inf = ExtendedFloat {
        mant: 0,
        exp: F::INFINITE_POWER,
    };

    if w == 0 || q < F::SMALLEST_POWER_OF_TEN {
        return fp_zero;
    } else if q > F::LARGEST_POWER_OF_TEN {
        return fp_inf;
    }
    let lz = w.leading_zeros() as i32;
    w <<= lz;
    let (lo, hi) = compute_product_approx(q, w, F::MANT_SIZE as usize + 3);
    // if lo == u64::MAX {
    //     let inside_safe_exponent = (q >= -27) && (q <= 55);
    //     if !inside_safe_exponent {
    //         return compute_error_scaled::<F>(q, hi, lz);
    //     }
    // }
    let upperbit = (hi >> 63) as i32;
    let mut mant = hi >> (upperbit + 64 - F::MANT_SIZE - 3);
    let mut power2 = power(q) + upperbit - lz - F::MIN_EXP;
    if power2 <= 0 {
        if -power2 + 1 >= 64 {
            return fp_zero;
        }
        mant >>= -power2 + 1;
        mant += mant & 1;
        mant >>= 1;
        power2 = (mant >= (1_u64 << F::MANT_SIZE)) as i32;
        return ExtendedFloat { mant, exp: power2 };
    }
    if lo <= 1
        && q >= F::MIN_EXP_ROUND_TO_EVEN
        && q <= F::MAX_EXP_ROUND_TO_EVEN
        && mant & 3 == 1
        && (mant << (upperbit + 64 - F::MANT_SIZE - 3)) == hi
    {
        mant &= !1_u64;
    }
    mant += mant & 1;
    mant >>= 1;
    if mant >= (2_u64 << F::MANT_SIZE) {
        mant = 1_u64 << F::MANT_SIZE;
        power2 += 1;
    }
    mant &= !(1_u64 << F::MANT_SIZE);
    if power2 >= F::INFINITE_POWER {
        return fp_inf;
    }
    ExtendedFloat { mant, exp: power2 }
}

fn compute_error<F: Float>(q: i32, mut w: u64) -> ExtendedFloat {
    let lz = w.leading_zeros() as i32;
    w <<= lz;
    let hi = compute_product_approx(q, w, F::MANT_SIZE as usize + 3).1;
    compute_error_scaled::<F>(q, hi, lz)
}

fn compute_error_scaled<F: Float>(q: i32, mut w: u64, lz: i32) -> ExtendedFloat {
    let hilz = (w >> 63) as i32 ^ 1;
    w <<= hilz;
    let power2 = power(q as i32) + F::EXP_BIAS - hilz - lz - 62;

    ExtendedFloat {
        mant: w,
        exp: power2 + F::INVALID_FP,
    }
}

fn power(q: i32) -> i32 {
    (q.wrapping_mul(152_170 + 65536) >> 16) + 63
}

fn full_mult(a: u64, b: u64) -> (u64, u64) {
    let r = (a as u128) * (b as u128);
    (r as u64, (r >> 64) as u64)
}

fn compute_product_approx(q: i32, w: u64, precision: usize) -> (u64, u64) {
    let mask = if precision < 64 {
        u64::MAX >> precision
    } else {
        u64::MAX
    };

    let index = (q - SMALLEST_POWER_OF_FIVE) as usize;
    let (lo5, hi5) = POWER_OF_FIVE_128[index];
    let (mut first_lo, mut first_hi) = full_mult(w, lo5);
    if first_hi & mask == mask {
        let (_, second_hi) = full_mult(w, hi5);
        first_lo = first_lo.wrapping_add(second_hi);
        if second_hi > first_lo {
            first_hi += 1;
        }
    }
    (first_lo, first_hi)
}
