use crate::from_bytes::common::{Float};
use crate::from_bytes::parse::{Number};
use crate::from_bytes::table_small::{SMALL_INT_POW10};

fn is_fast_path<F: Float>(num: &Number) -> bool {
    F::MIN_EXP_FAST_PATH <= num.exp
        && num.exp <= F::MAX_EXP_DISGUISED_FAST_PATH
        && num.mant <= F::MAX_MANTISSA_FAST_PATH
        && !num.many_digits
}

pub(crate) fn fast<F: Float>(num: &Number) -> Option<F> {
    if is_fast_path::<F>(num) {
        let max_exponent = F::MAX_EXP_FAST_PATH;
        let mut f = if num.exp <= max_exponent {
            let value = F::from_u64(num.mant);
            if num.exp < 0 {
                value / unsafe { F::pow_fast_path((-num.exp) as _) }
            } else {
                value * unsafe { F::pow_fast_path(num.exp as _) }
            }
        } else {
            let shift = num.exp - max_exponent;
            let int_power = unsafe { *SMALL_INT_POW10.get_unchecked(shift as usize) };
            let mantissa = num.mant.checked_mul(int_power)?;
            if mantissa > F::MAX_MANTISSA_FAST_PATH {
                return None;
            }
            F::from_u64(mantissa) * unsafe { F::pow_fast_path(max_exponent as _) }
        };
        if num.neg {
            f = -f;
        }
        Some(f)
    } else {
        None
    }
}
