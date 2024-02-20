use core::cmp;

use crate::from_bytes::bigint::{Bigint, Limb, LIMB_BITS};
use crate::from_bytes::common::{Float, ExtendedFloat, extended_to_float};
use crate::from_bytes::parse::{parse_eight_digits_unrolled, Number};
use crate::from_bytes::rounding::{round, round_down, round_nearest_tie_even};
use crate::from_bytes::table_small::{SMALL_INT_POW10};

pub(crate) fn slow<'a, F>(tokens: &Number, fp: ExtendedFloat) -> ExtendedFloat
where
    F: Float,
{
    let sci_exp = scientific_exponent(&tokens);

    let (bigmant, digits) = parse_mantissa(&tokens, F::MAX_DIGITS);
    let exponent = sci_exp + 1 - digits as i32;
    if exponent >= 0 {
        positive_digit_comp::<F>(bigmant, exponent)
    } else {
        negative_digit_comp::<F>(bigmant, fp, exponent)
    }
}

fn positive_digit_comp<F: Float>(mut bigmant: Bigint, exponent: i32) -> ExtendedFloat {
    bigmant.pow(10, exponent as u32).unwrap();

    let (mant, is_truncated) = bigmant.hi64();
    let exp = bigmant.bit_length() as i32 - 64 + F::EXP_BIAS;
    let mut fp = ExtendedFloat { mant, exp };

    round::<F, _>(&mut fp, |f, s| {
        round_nearest_tie_even(f, s, |is_odd, is_halfway, is_above| {
            is_above || (is_halfway && is_truncated) || (is_odd && is_halfway)
        });
    });
    fp
}

fn negative_digit_comp<F: Float>(
    bigmant: Bigint,
    mut fp: ExtendedFloat,
    exponent: i32,
) -> ExtendedFloat {
    let mut real_digits = bigmant;
    let real_exp = exponent;

    let mut b = fp;
    round::<F, _>(&mut b, round_down);
    let b = extended_to_float::<F>(b);

    let theor = bh(b);
    let mut theor_digits = Bigint::from_u64(theor.mant);
    let theor_exp = theor.exp;

    let binary_exp = theor_exp - real_exp;
    let halfradix_exp = -real_exp;
    if halfradix_exp != 0 {
        theor_digits.pow(5, halfradix_exp as u32).unwrap();
    }
    if binary_exp > 0 {
        theor_digits.pow(2, binary_exp as u32).unwrap();
    } else if binary_exp < 0 {
        real_digits.pow(2, -binary_exp as u32).unwrap();
    }

    let ord = real_digits.data.cmp(&theor_digits.data);
    round::<F, _>(&mut fp, |f, s| {
        round_nearest_tie_even(f, s, |is_odd, _, _| match ord {
            cmp::Ordering::Greater => true,
            cmp::Ordering::Less => false,
            cmp::Ordering::Equal if is_odd => true,
            cmp::Ordering::Equal => false,
        });
    });
    fp
}

fn parse_mantissa(tok: &Number, max_digits: usize) -> (Bigint, usize) {
    let mut counter = 0;
    let mut count = 0;
    let mut value: Limb = 0;
    let mut out = Bigint::new();
    let step = if LIMB_BITS == 64 { 19 } else { 9 };

    let mut d = tok.integer;
    d = skip_zeros(d);
    while d.len() >= 1 {
        while d.len() >= 8 && step - counter >= 8 && max_digits - count >= 8 {
            value = value * 100000000 + parse_eight_digits_unrolled(&d[..8]) as u64;
            d = &d[8..];
            counter += 8;
            count += 8;
        }
        while d.len() >= 1 && counter < step && count < max_digits {
            value = value * 10 + (d[0] - b'0') as Limb;
            d = &d[1..];
            counter += 1;
            count += 1;
        }
        if count == max_digits {
            add_native(
                &mut out,
                unsafe { *SMALL_INT_POW10.get_unchecked(counter) },
                value,
            );
            let mut truncated = is_truncated(d);
            if tok.fraction.len() >= 1 {
                truncated |= is_truncated(tok.fraction);
            }
            if truncated {
                add_native(&mut out, 10, 1);
                count += 1;
            }
            return (out, count);
        } else {
            add_native(
                &mut out,
                unsafe { *SMALL_INT_POW10.get_unchecked(counter) },
                value,
            );
            counter = 0;
            value = 0;
        }
    }

    d = tok.fraction;
    if d.len() >= 1 {
        if count == 0 {
            d = skip_zeros(d);
        }
        while d.len() >= 1 {
            while d.len() >= 8 && step - counter >= 8 && max_digits - count >= 8 {
                value = value * 100000000 + parse_eight_digits_unrolled(&d[..8]) as u64;
                d = &d[8..];
                counter += 8;
                count += 8;
            }
            while d.len() >= 1 && counter < step && count < max_digits {
                value = value * 10 + (d[0] - b'0') as Limb;
                d = &d[1..];
                counter += 1;
                count += 1;
            }
            if count == max_digits {
                add_native(
                    &mut out,
                    unsafe { *SMALL_INT_POW10.get_unchecked(counter) },
                    value,
                );
                let truncated = is_truncated(d);
                if truncated {
                    add_native(&mut out, 10, 1);
                    count += 1;
                }
                return (out, count);
            } else {
                add_native(
                    &mut out,
                    unsafe { *SMALL_INT_POW10.get_unchecked(counter) },
                    value,
                );
                counter = 0;
                value = 0;
            }
        }
    }

    if counter != 0 {
        add_native(
            &mut out,
            unsafe { *SMALL_INT_POW10.get_unchecked(counter) },
            value,
        );
    }

    (out, count)
}

fn scientific_exponent(num: &Number) -> i32 {
    let mut mant = num.mant;
    let mut exp = num.exp;
    while mant >= 10000 {
        mant /= 10000;
        exp += 4;
    }
    while mant >= 100 {
        mant /= 100;
        exp += 2;
    }
    while mant >= 10 {
        mant /= 10;
        exp += 1;
    }
    exp as i32
}

fn b<F: Float>(float: F) -> ExtendedFloat {
    ExtendedFloat {
        mant: float.mantissa(),
        exp: float.exponent(),
    }
}

fn bh<F: Float>(float: F) -> ExtendedFloat {
    let fp = b(float);
    ExtendedFloat {
        mant: (fp.mant << 1) + 1,
        exp: fp.exp - 1,
    }
}

fn skip_zeros(mut d: &[u8]) -> &[u8] {
    while d.len() >= 8 {
        let mut val = [0; 8];
        val[..8].copy_from_slice(&d[..8]);
        let val = u64::from_le_bytes(val);
        if val != 0x3030303030303030 {
            break;
        }
        d = &d[8..];
    }
    while d.len() >= 1 {
        if d[0] != b'0' {
            break;
        }
        d = &d[1..];
    }
    d
}

fn is_truncated(mut d: &[u8]) -> bool {
    while d.len() >= 8 {
        let mut val = [0; 8];
        val[..8].copy_from_slice(&d[..8]);
        let val = u64::from_le_bytes(val);
        if val != 0x3030303030303030 {
            return true;
        }
        d = &d[8..];
    }
    while d.len() >= 1 {
        if d[0] != b'0' {
            return true;
        }
        d = &d[1..];
    }
    false
}

fn add_native(big: &mut Bigint, power: Limb, value: Limb) {
    big.data.mul_small(power).unwrap();
    big.data.add_small(value).unwrap();
}
