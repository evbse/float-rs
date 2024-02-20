use core::{cmp, mem, ops, ptr, slice};

use crate::from_bytes::table_small::{LARGE_POW5, LARGE_POW5_STEP, SMALL_INT_POW5};

const BIGINT_BITS: usize = 4000;

pub(crate) const BIGINT_LIMBS: usize = BIGINT_BITS / LIMB_BITS;

pub(crate) struct StackVec {
    data: [mem::MaybeUninit<Limb>; BIGINT_LIMBS],
    length: usize,
}

impl StackVec {
    pub(crate) const fn new() -> Self {
        Self {
            length: 0,
            data: [mem::MaybeUninit::uninit(); BIGINT_LIMBS],
        }
    }

    pub(crate) fn try_from(x: &[Limb]) -> Option<Self> {
        let mut vec = Self::new();
        vec.try_extend(x)?;
        Some(vec)
    }

    pub(crate) unsafe fn set_len(&mut self, len: usize) {
        self.length = len;
    }

    const fn len(&self) -> usize {
        self.length
    }

    const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) const fn capacity(&self) -> usize {
        BIGINT_LIMBS
    }

    unsafe fn push_unchecked(&mut self, value: Limb) {
        unsafe {
            ptr::write(self.as_mut_ptr().add(self.len()), value);
            self.length += 1;
        }
    }

    pub(crate) fn try_push(&mut self, value: Limb) -> Option<()> {
        if self.len() < self.capacity() {
            unsafe { self.push_unchecked(value) };
            Some(())
        } else {
            None
        }
    }

    unsafe fn extend_unchecked(&mut self, slc: &[Limb]) {
        let index = self.len();
        let new_len = index + slc.len();
        let src = slc.as_ptr();
        unsafe {
            let dst = self.as_mut_ptr().add(index);
            ptr::copy_nonoverlapping(src, dst, slc.len());
            self.set_len(new_len);
        }
    }

    fn try_extend(&mut self, slc: &[Limb]) -> Option<()> {
        if self.len() + slc.len() <= self.capacity() {
            unsafe { self.extend_unchecked(slc) };
            Some(())
        } else {
            None
        }
    }

    unsafe fn resize_unchecked(&mut self, len: usize, value: Limb) {
        let old_len = self.len();
        if len > old_len {
            let count = len - old_len;
            for index in 0..count {
                unsafe {
                    let dst = self.as_mut_ptr().add(old_len + index);
                    ptr::write(dst, value);
                }
            }
        }
        self.length = len;
    }

    pub(crate) fn try_resize(&mut self, len: usize, value: Limb) -> Option<()> {
        if len > self.capacity() {
            None
        } else {
            unsafe { self.resize_unchecked(len, value) };
            Some(())
        }
    }

    pub(crate) fn hi64(&self) -> (u64, bool) {
        hi64(self)
    }

    pub(crate) fn from_u64(x: u64) -> Self {
        from_u64(x)
    }

    pub(crate) fn normalize(&mut self) {
        normalize(self)
    }

    pub(crate) fn add_small(&mut self, y: Limb) -> Option<()> {
        small_add(self, y)
    }

    pub(crate) fn mul_small(&mut self, y: Limb) -> Option<()> {
        small_mul(self, y)
    }
}

impl PartialEq for StackVec {
    fn eq(&self, other: &Self) -> bool {
        use core::ops::Deref;
        self.len() == other.len() && self.deref() == other.deref()
    }
}

impl Eq for StackVec {}

impl cmp::PartialOrd for StackVec {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(compare(self, other))
    }
}

impl cmp::Ord for StackVec {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        compare(self, other)
    }
}

impl ops::Deref for StackVec {
    type Target = [Limb];
    fn deref(&self) -> &[Limb] {
        unsafe {
            let ptr = self.data.as_ptr() as *const Limb;
            slice::from_raw_parts(ptr, self.len())
        }
    }
}

impl ops::DerefMut for StackVec {
    fn deref_mut(&mut self) -> &mut [Limb] {
        unsafe {
            let ptr = self.data.as_mut_ptr() as *mut Limb;
            slice::from_raw_parts_mut(ptr, self.len())
        }
    }
}

impl ops::MulAssign<&[Limb]> for StackVec {
    fn mul_assign(&mut self, rhs: &[Limb]) {
        large_mul(self, rhs).unwrap();
    }
}

pub(crate) struct Bigint {
    pub(crate) data: StackVec,
}

impl Bigint {
    pub(crate) fn new() -> Self {
        Self {
            data: StackVec::new(),
        }
    }

    pub(crate) fn from_u64(value: u64) -> Self {
        Self {
            data: StackVec::from_u64(value),
        }
    }

    pub(crate) fn hi64(&self) -> (u64, bool) {
        self.data.hi64()
    }

    pub(crate) fn pow(&mut self, base: u32, exp: u32) -> Option<()> {
        if base % 5 == 0 {
            pow(&mut self.data, exp)?;
        }
        if base % 2 == 0 {
            shl(&mut self.data, exp as usize)?;
        }
        Some(())
    }

    pub(crate) fn bit_length(&self) -> u32 {
        bit_length(&self.data)
    }
}

impl ops::MulAssign<&Bigint> for Bigint {
    fn mul_assign(&mut self, rhs: &Bigint) {
        self.data *= &rhs.data;
    }
}

struct ReverseView<'a, T: 'a> {
    inner: &'a [T],
}

impl<'a, T> ops::Index<usize> for ReverseView<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        let len = self.inner.len();
        &(*self.inner)[len - index - 1]
    }
}

fn rview(x: &[Limb]) -> ReverseView<Limb> {
    ReverseView { inner: x }
}

pub(crate) fn compare(x: &[Limb], y: &[Limb]) -> cmp::Ordering {
    match x.len().cmp(&y.len()) {
        cmp::Ordering::Equal => {
            let iter = x.iter().rev().zip(y.iter().rev());
            for (&xi, yi) in iter {
                match xi.cmp(yi) {
                    cmp::Ordering::Equal => (),
                    ord => return ord,
                }
            }
            cmp::Ordering::Equal
        }
        ord => ord,
    }
}

pub(crate) fn normalize(x: &mut StackVec) {
    while let Some(&value) = x.get(x.len().wrapping_sub(1)) {
        if value == 0 {
            unsafe { x.set_len(x.len() - 1) };
        } else {
            break;
        }
    }
}

pub(crate) fn from_u64(x: u64) -> StackVec {
    let mut vec = StackVec::new();
    if LIMB_BITS == 32 {
        vec.try_push(x as Limb).unwrap();
        vec.try_push((x >> 32) as Limb).unwrap();
    } else {
        vec.try_push(x as Limb).unwrap();
    }
    vec.normalize();
    vec
}

fn nonzero(x: &[Limb], rindex: usize) -> bool {
    let len = x.len();
    let slc = &x[..len - rindex];
    slc.iter().rev().any(|&x| x != 0)
}

fn u32_to_hi64_1(r0: u32) -> (u64, bool) {
    u64_to_hi64_1(r0 as u64)
}

fn u32_to_hi64_2(r0: u32, r1: u32) -> (u64, bool) {
    let r0 = (r0 as u64) << 32;
    let r1 = r1 as u64;
    u64_to_hi64_1(r0 | r1)
}

fn u32_to_hi64_3(r0: u32, r1: u32, r2: u32) -> (u64, bool) {
    let r0 = r0 as u64;
    let r1 = (r1 as u64) << 32;
    let r2 = r2 as u64;
    u64_to_hi64_2(r0, r1 | r2)
}

fn u64_to_hi64_1(r0: u64) -> (u64, bool) {
    let ls = r0.leading_zeros();
    (r0 << ls, false)
}

fn u64_to_hi64_2(r0: u64, r1: u64) -> (u64, bool) {
    let ls = r0.leading_zeros();
    let rs = 64 - ls;
    let v = match ls {
        0 => r0,
        _ => (r0 << ls) | (r1 >> rs),
    };
    let n = r1 << ls != 0;
    (v, n)
}

macro_rules! hi {
    (@1 $self:ident, $rview:ident, $t:ident, $fn:ident) => {{
        $fn($rview[0] as $t)
    }};

    (@2 $self:ident, $rview:ident, $t:ident, $fn:ident) => {{
        let r0 = $rview[0] as $t;
        let r1 = $rview[1] as $t;
        $fn(r0, r1)
    }};

    (@nonzero2 $self:ident, $rview:ident, $t:ident, $fn:ident) => {{
        let (v, n) = hi!(@2 $self, $rview, $t, $fn);
        (v, n || nonzero($self, 2 ))
    }};

    (@3 $self:ident, $rview:ident, $t:ident, $fn:ident) => {{
        let r0 = $rview[0] as $t;
        let r1 = $rview[1] as $t;
        let r2 = $rview[2] as $t;
        $fn(r0, r1, r2)
    }};

    (@nonzero3 $self:ident, $rview:ident, $t:ident, $fn:ident) => {{
        let (v, n) = hi!(@3 $self, $rview, $t, $fn);
        (v, n || nonzero($self, 3))
    }};
}

pub(crate) fn hi64(x: &[Limb]) -> (u64, bool) {
    let rslc = rview(x);
    match x.len() {
        0 => (0, false),
        1 if LIMB_BITS == 32 => hi!(@1 x, rslc, u32, u32_to_hi64_1),
        1 => hi!(@1 x, rslc, u64, u64_to_hi64_1),
        2 if LIMB_BITS == 32 => hi!(@2 x, rslc, u32, u32_to_hi64_2),
        2 => hi!(@2 x, rslc, u64, u64_to_hi64_2),
        _ if LIMB_BITS == 32 => hi!(@nonzero3 x, rslc, u32, u32_to_hi64_3),
        _ => hi!(@nonzero2 x, rslc, u64, u64_to_hi64_2),
    }
}

fn pow(x: &mut StackVec, mut exp: u32) -> Option<()> {
    while exp >= LARGE_POW5_STEP {
        large_mul(x, &LARGE_POW5)?;
        exp -= LARGE_POW5_STEP;
    }

    let small_step = if LIMB_BITS == 32 { 13 } else { 27 };
    let max_native = (5 as Limb).pow(small_step);
    while exp >= small_step {
        small_mul(x, max_native)?;
        exp -= small_step;
    }
    if exp != 0 {
        let small_power = unsafe { *SMALL_INT_POW5.get_unchecked(exp as usize) };
        small_mul(x, small_power as Limb)?;
    }
    Some(())
}

fn scalar_add(x: Limb, y: Limb) -> (Limb, bool) {
    x.overflowing_add(y)
}

fn scalar_mul(x: Limb, y: Limb, carry: Limb) -> (Limb, Limb) {
    let z: Wide = (x as Wide) * (y as Wide) + (carry as Wide);
    (z as Limb, (z >> LIMB_BITS) as Limb)
}

fn small_add_from(x: &mut StackVec, y: Limb, start: usize) -> Option<()> {
    let mut index = start;
    let mut carry = y;
    while carry != 0 && index < x.len() {
        let result = scalar_add(x[index], carry);
        x[index] = result.0;
        carry = result.1 as Limb;
        index += 1;
    }
    if carry != 0 {
        x.try_push(carry)?;
    }
    Some(())
}

pub(crate) fn small_add(x: &mut StackVec, y: Limb) -> Option<()> {
    small_add_from(x, y, 0)
}

pub(crate) fn small_mul(x: &mut StackVec, y: Limb) -> Option<()> {
    let mut carry = 0;
    for xi in x.iter_mut() {
        let result = scalar_mul(*xi, y, carry);
        *xi = result.0;
        carry = result.1;
    }
    if carry != 0 {
        x.try_push(carry)?;
    }
    Some(())
}

fn large_add_from(x: &mut StackVec, y: &[Limb], start: usize) -> Option<()> {
    if y.len() > x.len().saturating_sub(start) {
        x.try_resize(y.len() + start, 0)?;
    }

    let mut carry = false;
    for (index, &yi) in y.iter().enumerate() {
        let xi = x.get_mut(start + index).unwrap();

        let result = scalar_add(*xi, yi);
        *xi = result.0;
        let mut tmp = result.1;
        if carry {
            let result = scalar_add(*xi, 1);
            *xi = result.0;
            tmp |= result.1;
        }
        carry = tmp;
    }

    if carry {
        small_add_from(x, 1, y.len() + start)?;
    }
    Some(())
}

fn long_mul(x: &[Limb], y: &[Limb]) -> Option<StackVec> {
    let mut z = StackVec::try_from(x)?;
    if !y.is_empty() {
        let y0 = y[0];
        small_mul(&mut z, y0)?;

        for (index, &yi) in y.iter().enumerate().skip(1) {
            if yi != 0 {
                let mut zi = StackVec::try_from(x)?;
                small_mul(&mut zi, yi)?;
                large_add_from(&mut z, &zi, index)?;
            }
        }
    }

    z.normalize();
    Some(z)
}

pub(crate) fn large_mul(x: &mut StackVec, y: &[Limb]) -> Option<()> {
    if y.len() == 1 {
        small_mul(x, y[0])?;
    } else {
        *x = long_mul(y, x)?;
    }
    Some(())
}

fn shl_bits(x: &mut StackVec, n: usize) -> Option<()> {
    let rshift = LIMB_BITS - n;
    let lshift = n;
    let mut prev: Limb = 0;
    for xi in x.iter_mut() {
        let tmp = *xi;
        *xi <<= lshift;
        *xi |= prev >> rshift;
        prev = tmp;
    }

    let carry = prev >> rshift;
    if carry != 0 {
        x.try_push(carry)?;
    }

    Some(())
}

fn shl_limbs(x: &mut StackVec, n: usize) -> Option<()> {
    if n + x.len() > x.capacity() {
        None
    } else if !x.is_empty() {
        let len = n + x.len();
        unsafe {
            let src = x.as_ptr();
            let dst = x.as_mut_ptr().add(n);
            ptr::copy(src, dst, x.len());
            ptr::write_bytes(x.as_mut_ptr(), 0, n);
            x.set_len(len);
        }
        Some(())
    } else {
        Some(())
    }
}

fn shl(x: &mut StackVec, n: usize) -> Option<()> {
    let rem = n % LIMB_BITS;
    let div = n / LIMB_BITS;
    if rem != 0 {
        shl_bits(x, rem)?;
    }
    if div != 0 {
        shl_limbs(x, div)?;
    }
    Some(())
}

fn leading_zeros(x: &[Limb]) -> u32 {
    let length = x.len();
    if let Some(&value) = x.get(length.wrapping_sub(1)) {
        value.leading_zeros()
    } else {
        0
    }
}

fn bit_length(x: &[Limb]) -> u32 {
    let nlz = leading_zeros(x);
    LIMB_BITS as u32 * x.len() as u32 - nlz
}

#[cfg(all(target_pointer_width = "64", not(target_arch = "sparc")))]
pub(crate) type Limb = u64;
#[cfg(all(target_pointer_width = "64", not(target_arch = "sparc")))]
type Wide = u128;
#[cfg(all(target_pointer_width = "64", not(target_arch = "sparc")))]
pub(crate) const LIMB_BITS: usize = 64;

#[cfg(not(all(target_pointer_width = "64", not(target_arch = "sparc"))))]
pub(crate) type Limb = u32;
#[cfg(not(all(target_pointer_width = "64", not(target_arch = "sparc"))))]
type Wide = u64;
#[cfg(not(all(target_pointer_width = "64", not(target_arch = "sparc"))))]
pub(crate) const LIMB_BITS: usize = 32;
