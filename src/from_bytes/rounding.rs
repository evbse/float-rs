use crate::from_bytes::common::{Float, ExtendedFloat};

pub(crate) fn round<F, Cb>(fp: &mut ExtendedFloat, cb: Cb)
where
    F: Float,
    Cb: Fn(&mut ExtendedFloat, i32),
{
    let fp_inf = ExtendedFloat {
        mant: 0,
        exp: F::INFINITE_POWER,
    };

    let mantissa_shift = 64 - F::MANT_SIZE - 1;

    if -fp.exp >= mantissa_shift {
        let shift = -fp.exp + 1;
        cb(fp, shift.min(64));
        fp.exp = (fp.mant >= F::HIDDEN_BIT_MASK) as i32;
        return;
    }

    cb(fp, mantissa_shift);

    let carry_mask = F::CARRY_MASK;
    if fp.mant & carry_mask == carry_mask {
        fp.mant >>= 1;
        fp.exp += 1;
    }

    if fp.exp >= F::INFINITE_POWER {
        *fp = fp_inf;
        return;
    }

    fp.mant &= F::MANT_MASK;
}

pub(crate) fn round_nearest_tie_even<Cb>(fp: &mut ExtendedFloat, shift: i32, cb: Cb)
where
    Cb: Fn(bool, bool, bool) -> bool,
{
    let mask = match shift == 64 {
        true => u64::MAX,
        false => (1 << shift) - 1,
    };
    let halfway = match shift == 0 {
        true => 0,
        false => 1 << (shift - 1),
    };
    let truncated_bits = fp.mant & mask;
    let is_above = truncated_bits > halfway;
    let is_halfway = truncated_bits == halfway;

    fp.mant = match shift == 64 {
        true => 0,
        false => fp.mant >> shift,
    };
    fp.exp += shift;

    let is_odd = fp.mant & 1 == 1;

    fp.mant += cb(is_odd, is_halfway, is_above) as u64;
}

pub(crate) fn round_down(fp: &mut ExtendedFloat, shift: i32) {
    fp.mant = match shift == 64 {
        true => 0,
        false => fp.mant >> shift,
    };
    fp.exp += shift;
}
