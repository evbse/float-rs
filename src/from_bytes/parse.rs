use crate::from_bytes::common::{Float, extended_to_float};
use crate::from_bytes::fast::{fast};
use crate::from_bytes::moderate::{moderate};
use crate::from_bytes::slow::{slow};

#[derive(Default)]
pub(crate) struct Number<'a> {
    pub exp: i32,
    pub mant: u64,
    pub neg: bool,
    pub many_digits: bool,
    pub integer: &'a [u8],
    pub fraction: &'a [u8],
}

pub fn parse<'a, F>(d: &'a [u8]) -> F
where
    F: Float,
{
    let tokens = parse_into_tokens(&d).unwrap();
    if let Some(value) = fast::<F>(&tokens) {
        return value;
    }

    let mut fp = moderate::<F>(&tokens);
    if fp.exp < 0 {
        fp.exp -= F::INVALID_FP;
        fp = slow::<F>(&tokens, fp);
    }

    let mut f = extended_to_float::<F>(fp);
    if tokens.neg {
        f = -f;
    }
    f
}

fn is_integer(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

fn read_u64(d: &[u8]) -> u64 {
    let mut val = [0; 8];
    val[..8].copy_from_slice(&d[..8]);
    let val = u64::from_le_bytes(val);
    val
}

fn parse_eight_digits_unrolled_(mut val: u64) -> u32 {
    let mask = 0x000000ff000000ff;
    let mul1 = 0x000f424000000064;
    let mul2 = 0x0000271000000001;
    val -= 0x3030303030303030;
    val = (val * 10) + (val >> 8);
    val = (val & mask)
        .wrapping_mul(mul1)
        .wrapping_add(((val >> 16) & mask).wrapping_mul(mul2))
        >> 32;
    val as u32
}

pub(crate) fn parse_eight_digits_unrolled(d: &[u8]) -> u32 {
    parse_eight_digits_unrolled_(read_u64(d))
}

fn is_made_of_eight_digits_fast_(val: u64) -> bool {
    (val.wrapping_add(0x4646464646464646) | val.wrapping_sub(0x3030303030303030))
        & 0x8080808080808080
        == 0
}

pub(crate) fn is_made_of_eight_digits_fast(d: &[u8]) -> bool {
    is_made_of_eight_digits_fast_(read_u64(d))
}

pub(crate) fn parse_into_tokens(mut d: &[u8]) -> Option<Number> {
    if d.len() == 0 {
        return None;
    }
    let mut out = Number::default();
    if d[0] == b'-' {
        out.neg = true;
        d = &d[1..];
        if d.len() == 0 {
            return None;
        }
        if !is_integer(d[0]) && d[0] != b'.' {
            return None;
        }
    }
    let start_digits = d;

    let mut i: u64 = 0;
    // while d.len() >= 8 && is_made_of_eight_digits_fast(&d[..8]) {
    //     i = i.wrapping_mul(100000000).wrapping_add(parse_eight_digits_unrolled(&d[..8]) as u64);
    //     d = &d[8..];
    // }
    while d.len() >= 1 && is_integer(d[0]) {
        i = i.wrapping_mul(10).wrapping_add((d[0] - b'0') as u64);
        d = &d[1..];
    }
    let mut digit_count = (start_digits.len() - d.len()) as i32;
    out.integer = &start_digits[..digit_count as usize];
    let mut exponent = 0;
    if d.len() >= 1 && d[0] == b'.' {
        d = &d[1..];
        let before = d;
        while d.len() >= 8 && is_made_of_eight_digits_fast(&d[..8]) {
            i = i
                .wrapping_mul(100000000)
                .wrapping_add(parse_eight_digits_unrolled(&d[..8]) as u64);
            d = &d[8..];
        }
        while d.len() >= 1 && is_integer(d[0]) {
            i = i.wrapping_mul(10).wrapping_add((d[0] - b'0') as u64);
            d = &d[1..];
        }
        exponent = d.len().wrapping_sub(before.len()) as i32;
        out.fraction = &before[..(before.len() - d.len()) as usize];
        digit_count -= exponent;
    }
    if digit_count == 0 {
        return None;
    }
    let mut exp_number = 0;
    if d.len() >= 1 && (b'e' == d[0] || b'E' == d[0]) {
        d = &d[1..];
        let mut neg_exp = false;
        if d.len() >= 1 && b'-' == d[0] {
            neg_exp = true;
            d = &d[1..];
        } else if d.len() >= 1 && b'+' == d[0] {
            d = &d[1..];
        }
        if d.len() == 0 || !is_integer(d[0]) {
            return None;
        } else {
            while d.len() >= 1 && is_integer(d[0]) {
                if exp_number < 0x10000000 {
                    exp_number = 10 * exp_number + (d[0] - b'0') as i32;
                }
                d = &d[1..];
            }
            if neg_exp {
                exp_number = -exp_number;
            }
            exponent += exp_number;
        }
    }

    if digit_count > 19 {
        let mut start = start_digits;
        while start.len() >= 1 && (start[0] == b'0' || start[0] == b'.') {
            if start[0] == b'0' {
                digit_count -= 1;
            }
            start = &start[1..];
        }
        if digit_count > 19 {
            out.many_digits = true;
            i = 0;
            d = out.integer;
            let minimal_nineteen_digit_integer = 1000000000000000000;
            while i < minimal_nineteen_digit_integer && d.len() >= 1 {
                i = i * 10 + (d[0] - b'0') as u64;
                d = &d[1..];
            }
            if i >= minimal_nineteen_digit_integer {
                exponent = d.len() as i32 + exp_number;
            } else {
                d = out.fraction;
                while i < minimal_nineteen_digit_integer && d.len() >= 1 {
                    i = i * 10 + (d[0] - b'0') as u64;
                    d = &d[1..];
                }
                exponent = d.len().wrapping_sub(out.fraction.len()) as i32 + exp_number;
            }
        }
    }
    out.exp = exponent;
    out.mant = i;
    Some(out)
}
