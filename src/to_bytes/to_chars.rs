use core::ptr;

const RADIX_100_TABLE: [u8; 200] = [
    b'0', b'0', b'0', b'1', b'0', b'2', b'0', b'3', b'0', b'4',
    b'0', b'5', b'0', b'6', b'0', b'7', b'0', b'8', b'0', b'9',
    b'1', b'0', b'1', b'1', b'1', b'2', b'1', b'3', b'1', b'4',
    b'1', b'5', b'1', b'6', b'1', b'7', b'1', b'8', b'1', b'9',
    b'2', b'0', b'2', b'1', b'2', b'2', b'2', b'3', b'2', b'4',
    b'2', b'5', b'2', b'6', b'2', b'7', b'2', b'8', b'2', b'9',
    b'3', b'0', b'3', b'1', b'3', b'2', b'3', b'3', b'3', b'4',
    b'3', b'5', b'3', b'6', b'3', b'7', b'3', b'8', b'3', b'9',
    b'4', b'0', b'4', b'1', b'4', b'2', b'4', b'3', b'4', b'4',
    b'4', b'5', b'4', b'6', b'4', b'7', b'4', b'8', b'4', b'9',
    b'5', b'0', b'5', b'1', b'5', b'2', b'5', b'3', b'5', b'4',
    b'5', b'5', b'5', b'6', b'5', b'7', b'5', b'8', b'5', b'9',
    b'6', b'0', b'6', b'1', b'6', b'2', b'6', b'3', b'6', b'4',
    b'6', b'5', b'6', b'6', b'6', b'7', b'6', b'8', b'6', b'9',
    b'7', b'0', b'7', b'1', b'7', b'2', b'7', b'3', b'7', b'4',
    b'7', b'5', b'7', b'6', b'7', b'7', b'7', b'8', b'7', b'9',
    b'8', b'0', b'8', b'1', b'8', b'2', b'8', b'3', b'8', b'4',
    b'8', b'5', b'8', b'6', b'8', b'7', b'8', b'8', b'8', b'9',
    b'9', b'0', b'9', b'1', b'9', b'2', b'9', b'3', b'9', b'4',
    b'9', b'5', b'9', b'6', b'9', b'7', b'9', b'8', b'9', b'9',
];
const RADIX_100_HEAD_TABLE: [u8; 200] = [
    b'0', b'.', b'1', b'.', b'2', b'.', b'3', b'.', b'4', b'.',
    b'5', b'.', b'6', b'.', b'7', b'.', b'8', b'.', b'9', b'.',
    b'1', b'.', b'1', b'.', b'1', b'.', b'1', b'.', b'1', b'.',
    b'1', b'.', b'1', b'.', b'1', b'.', b'1', b'.', b'1', b'.',
    b'2', b'.', b'2', b'.', b'2', b'.', b'2', b'.', b'2', b'.',
    b'2', b'.', b'2', b'.', b'2', b'.', b'2', b'.', b'2', b'.',
    b'3', b'.', b'3', b'.', b'3', b'.', b'3', b'.', b'3', b'.',
    b'3', b'.', b'3', b'.', b'3', b'.', b'3', b'.', b'3', b'.',
    b'4', b'.', b'4', b'.', b'4', b'.', b'4', b'.', b'4', b'.',
    b'4', b'.', b'4', b'.', b'4', b'.', b'4', b'.', b'4', b'.',
    b'5', b'.', b'5', b'.', b'5', b'.', b'5', b'.', b'5', b'.',
    b'5', b'.', b'5', b'.', b'5', b'.', b'5', b'.', b'5', b'.',
    b'6', b'.', b'6', b'.', b'6', b'.', b'6', b'.', b'6', b'.',
    b'6', b'.', b'6', b'.', b'6', b'.', b'6', b'.', b'6', b'.',
    b'7', b'.', b'7', b'.', b'7', b'.', b'7', b'.', b'7', b'.',
    b'7', b'.', b'7', b'.', b'7', b'.', b'7', b'.', b'7', b'.',
    b'8', b'.', b'8', b'.', b'8', b'.', b'8', b'.', b'8', b'.',
    b'8', b'.', b'8', b'.', b'8', b'.', b'8', b'.', b'8', b'.',
    b'9', b'.', b'9', b'.', b'9', b'.', b'9', b'.', b'9', b'.',
    b'9', b'.', b'9', b'.', b'9', b'.', b'9', b'.', b'9', b'.',
];

unsafe fn write_1_digit(n: u32, buf: *mut u8) {
    ptr::copy_nonoverlapping(RADIX_100_TABLE.as_ptr().add(n as usize * 2 + 1), buf, 1);
}

unsafe fn write_2_digits(n: u32, buf: *mut u8) {
    ptr::copy_nonoverlapping(RADIX_100_TABLE.as_ptr().add(n as usize * 2), buf, 2);
}

unsafe fn write_9_digits(mant: u32, mut exp: i32, mut buf: *mut u8) -> (i32, *mut u8) {
    if mant >= 1_0000_0000 {
        let mut prod = (mant as u64) * 1441151882;
        prod >>= 25;
        ptr::copy_nonoverlapping(
            RADIX_100_HEAD_TABLE.as_ptr().add((prod >> 32) as usize * 2),
            buf,
            2,
        );

        prod = ((prod as u32) as u64) * 100;
        write_2_digits((prod >> 32) as u32, buf.add(2));
        prod = ((prod as u32) as u64) * 100;
        write_2_digits((prod >> 32) as u32, buf.add(4));
        prod = ((prod as u32) as u64) * 100;
        write_2_digits((prod >> 32) as u32, buf.add(6));
        prod = ((prod as u32) as u64) * 100;
        write_2_digits((prod >> 32) as u32, buf.add(8));

        exp += 8;
        buf = buf.add(10);
    } else if mant >= 100_0000 {
        let mut prod = (mant as u64) * 281474978;
        prod >>= 16;
        let head_digits = (prod >> 32) as u32;
        exp += 6 + if head_digits >= 10 { 1 } else { 0 };

        ptr::copy_nonoverlapping(
            RADIX_100_HEAD_TABLE.as_ptr().add(head_digits as usize * 2),
            buf,
            2,
        );
        *buf.add(2) = *RADIX_100_TABLE.get_unchecked(head_digits as usize * 2 + 1);

        if prod as u32 <= (((1 as u64) << 32) / 100_0000) as u32 {
            buf = buf.add(
                1 + ((if head_digits >= 10 { 1 } else { 0 })
                    & (if *buf.add(2) > b'0' { 1 } else { 0 }))
                    * 2,
            );
        } else {
            buf = buf.add(if head_digits >= 10 { 1 } else { 0 });

            prod = ((prod as u32) as u64) * 100;
            write_2_digits((prod >> 32) as u32, buf.add(2));

            if prod as u32 <= (((1 as u64) << 32) / 1_0000) as u32 {
                buf = buf.add(3 + (if *buf.add(3) > b'0' { 1 } else { 0 }));
            } else {
                prod = ((prod as u32) as u64) * 100;
                write_2_digits((prod >> 32) as u32, buf.add(4));

                if prod as u32 <= (((1 as u64) << 32) / 100) as u32 {
                    buf = buf.add(5 + (if *buf.add(5) > b'0' { 1 } else { 0 }));
                } else {
                    prod = ((prod as u32) as u64) * 100;
                    write_2_digits((prod >> 32) as u32, buf.add(6));

                    buf = buf.add(7 + (if *buf.add(7) > b'0' { 1 } else { 0 }));
                }
            }
        }
    } else if mant >= 1_0000 {
        let mut prod = (mant as u64) * 429497;
        let head_digits = (prod >> 32) as u32;

        exp += 4 + (if head_digits >= 10 { 1 } else { 0 });

        ptr::copy_nonoverlapping(
            RADIX_100_HEAD_TABLE.as_ptr().add(head_digits as usize * 2),
            buf,
            2,
        );
        *buf.add(2) = *RADIX_100_TABLE.get_unchecked(head_digits as usize * 2 + 1);

        if prod as u32 <= (((1 as u64) << 32) / 1_0000) as u32 {
            buf = buf.add(
                1 + ((if head_digits >= 10 { 1 } else { 0 })
                    & (if *buf.add(2) > b'0' { 1 } else { 0 }))
                    * 2,
            );
        } else {
            buf = buf.add(if head_digits >= 10 { 1 } else { 0 });

            prod = ((prod as u32) as u64) * 100;
            write_2_digits((prod >> 32) as u32, buf.add(2));

            if prod as u32 <= (((1 as u64) << 32) / 100) as u32 {
                buf = buf.add(3 + (if *buf.add(3) > b'0' { 1 } else { 0 }));
            } else {
                prod = ((prod as u32) as u64) * 100;
                write_2_digits((prod >> 32) as u32, buf.add(4));

                buf = buf.add(5 + (if *buf.add(5) > b'0' { 1 } else { 0 }));
            }
        }
    } else if mant >= 100 {
        let mut prod = (mant as u64) * 42949673;
        let head_digits = (prod >> 32) as u32;

        exp += 2 + (if head_digits >= 10 { 1 } else { 0 });

        ptr::copy_nonoverlapping(
            RADIX_100_HEAD_TABLE.as_ptr().add(head_digits as usize * 2),
            buf,
            2,
        );
        *buf.add(2) = *RADIX_100_TABLE.get_unchecked(head_digits as usize * 2 + 1);

        if prod as u32 <= (((1 as u64) << 32) / 100) as u32 {
            buf = buf.add(
                1 + ((if head_digits >= 10 { 1 } else { 0 })
                    & (if *buf.add(2) > b'0' { 1 } else { 0 }))
                    * 2,
            );
        } else {
            buf = buf.add(if head_digits >= 10 { 1 } else { 0 });

            prod = ((prod as u32) as u64) * 100;
            write_2_digits((prod >> 32) as u32, buf.add(2));

            buf = buf.add(3 + (if *buf.add(3) > b'0' { 1 } else { 0 }));
        }
    } else {
        exp += if mant >= 10 { 1 } else { 0 };

        ptr::copy_nonoverlapping(RADIX_100_HEAD_TABLE.as_ptr().add(mant as usize * 2), buf, 2);
        *buf.add(2) = *RADIX_100_TABLE.get_unchecked(mant as usize * 2 + 1);

        buf = buf.add(
            1 + ((if mant >= 10 { 1 } else { 0 }) & (if *buf.add(2) > b'0' { 1 } else { 0 })) * 2,
        );
    }

    (exp, buf)
}

pub(crate) unsafe fn write_f32(mant: u32, mut exp: i32, mut buf: *mut u8) -> *mut u8 {
    let (exp_, buf_) = write_9_digits(mant, exp, buf);
    exp = exp_;
    buf = buf_;

    if exp < 0 {
        ptr::copy_nonoverlapping(b"E-".as_ptr(), buf, 2);
        buf = buf.add(2);
        exp = -exp;
    } else {
        ptr::copy_nonoverlapping(b"E".as_ptr(), buf, 1);
        buf = buf.add(1);
    }

    if exp >= 10 {
        write_2_digits(exp as u32, buf);
        buf = buf.add(2);
    } else {
        write_1_digit(exp as u32, buf);
        buf = buf.add(1);
    }

    buf
}

pub(crate) unsafe fn write_f64(mant: u64, mut exp: i32, mut buf: *mut u8) -> *mut u8 {
    let lmant;
    let rmant;

    if mant >= 1_0000_0000 {
        lmant = (mant / 1_0000_0000) as u32;
        rmant = (mant as u32) - lmant.wrapping_mul(1_0000_0000);
    } else {
        lmant = mant as u32;
        rmant = 0;
    }

    if rmant == 0 {
        let (exp_, buf_) = write_9_digits(lmant, exp, buf);
        exp = exp_;
        buf = buf_;
    } else {
        if lmant >= 1_0000_0000 {
            let mut prod = (lmant as u64) * 1441151882;
            prod >>= 25;
            ptr::copy_nonoverlapping(
                RADIX_100_HEAD_TABLE
                    .as_ptr()
                    .add(((prod >> 32) as u32) as usize * 2),
                buf,
                2,
            );

            prod = ((prod as u32) as u64) * 100;
            write_2_digits((prod >> 32) as u32, buf.add(2));
            prod = ((prod as u32) as u64) * 100;
            write_2_digits((prod >> 32) as u32, buf.add(4));
            prod = ((prod as u32) as u64) * 100;
            write_2_digits((prod >> 32) as u32, buf.add(6));
            prod = ((prod as u32) as u64) * 100;
            write_2_digits((prod >> 32) as u32, buf.add(8));

            prod = (rmant as u64) * 281474978;
            prod >>= 16;
            prod += 1;
            write_2_digits((prod >> 32) as u32, buf.add(10));
            prod = ((prod as u32) as u64) * 100;
            write_2_digits((prod >> 32) as u32, buf.add(12));
            prod = ((prod as u32) as u64) * 100;
            write_2_digits((prod >> 32) as u32, buf.add(14));
            prod = ((prod as u32) as u64) * 100;
            write_2_digits((prod >> 32) as u32, buf.add(16));

            exp += 16;
            buf = buf.add(18);
        } else {
            if lmant >= 100_0000 {
                let mut prod = (lmant as u64) * 281474978;
                prod >>= 16;
                let head_digits = (prod >> 32) as u32;

                ptr::copy_nonoverlapping(
                    RADIX_100_HEAD_TABLE.as_ptr().add(head_digits as usize * 2),
                    buf,
                    2,
                );
                *buf.add(2) = *RADIX_100_TABLE.get_unchecked(head_digits as usize * 2 + 1);

                exp += 6 + (if head_digits >= 10 { 1 } else { 0 });
                buf = buf.add(if head_digits >= 10 { 1 } else { 0 });

                prod = ((prod as u32) as u64) * 100;
                write_2_digits((prod >> 32) as u32, buf.add(2));
                prod = ((prod as u32) as u64) * 100;
                write_2_digits((prod >> 32) as u32, buf.add(4));
                prod = ((prod as u32) as u64) * 100;
                write_2_digits((prod >> 32) as u32, buf.add(6));

                buf = buf.add(8);
            } else if lmant >= 1_0000 {
                let mut prod = (lmant as u64) * 429497;
                let head_digits = (prod >> 32) as u32;

                ptr::copy_nonoverlapping(
                    RADIX_100_HEAD_TABLE.as_ptr().add(head_digits as usize * 2),
                    buf,
                    2,
                );
                *buf.add(2) = *RADIX_100_TABLE.get_unchecked(head_digits as usize * 2 + 1);

                exp += 4 + (if head_digits >= 10 { 1 } else { 0 });
                buf = buf.add(if head_digits >= 10 { 1 } else { 0 });

                prod = ((prod as u32) as u64) * 100;
                write_2_digits((prod >> 32) as u32, buf.add(2));
                prod = ((prod as u32) as u64) * 100;
                write_2_digits((prod >> 32) as u32, buf.add(4));

                buf = buf.add(6);
            } else if lmant >= 100 {
                let mut prod = (lmant as u64) * 42949673;
                let head_digits = (prod >> 32) as u32;

                ptr::copy_nonoverlapping(
                    RADIX_100_HEAD_TABLE.as_ptr().add(head_digits as usize * 2),
                    buf,
                    2,
                );
                *buf.add(2) = *RADIX_100_TABLE.get_unchecked(head_digits as usize * 2 + 1);

                exp += 2 + (if head_digits >= 10 { 1 } else { 0 });
                buf = buf.add(if head_digits >= 10 { 1 } else { 0 });

                prod = ((prod as u32) as u64) * 100;
                write_2_digits((prod >> 32) as u32, buf.add(2));

                buf = buf.add(4);
            } else {
                ptr::copy_nonoverlapping(
                    RADIX_100_HEAD_TABLE.as_ptr().add(lmant as usize * 2),
                    buf,
                    2,
                );
                *buf.add(2) = *RADIX_100_TABLE.get_unchecked(lmant as usize * 2 + 1);

                exp += if lmant >= 10 { 1 } else { 0 };
                buf = buf.add(2 + (if lmant >= 10 { 1 } else { 0 }));
            }

            let mut prod = (rmant as u64) * 281474978;
            prod >>= 16;
            prod += 1;
            write_2_digits((prod >> 32) as u32, buf);

            if prod as u32 <= (((1 as u64) << 32) / 100_0000) as u32 {
                buf = buf.add(1 + (if *buf.add(1) > b'0' { 1 } else { 0 }));
            } else {
                prod = ((prod as u32) as u64) * 100;
                write_2_digits((prod >> 32) as u32, buf.add(2));

                if prod as u32 <= (((1 as u64) << 32) / 1_0000) as u32 {
                    buf = buf.add(3 + (if *buf.add(3) > b'0' { 1 } else { 0 }));
                } else {
                    prod = ((prod as u32) as u64) * 100;
                    write_2_digits((prod >> 32) as u32, buf.add(4));

                    if prod as u32 <= (((1 as u64) << 32) / 100) as u32 {
                        buf = buf.add(5 + (if *buf.add(5) > b'0' { 1 } else { 0 }));
                    } else {
                        prod = ((prod as u32) as u64) * 100;
                        write_2_digits((prod >> 32) as u32, buf.add(6));
                        buf = buf.add(7 + (if *buf.add(7) > b'0' { 1 } else { 0 }));
                    }
                }
            }
        }
    }

    if exp < 0 {
        ptr::copy_nonoverlapping(b"E-".as_ptr(), buf, 2);
        buf = buf.add(2);
        exp = -exp;
    } else {
        ptr::copy_nonoverlapping(b"E".as_ptr(), buf, 1);
        buf = buf.add(1);
    }

    if exp >= 100 {
        let mut prod = (exp as u32) * 6554;
        let d1 = prod >> 16;
        prod = ((prod as u16) as u32) * 5;
        let d2 = prod >> 15;
        write_2_digits(d1, buf);
        write_1_digit(d2, buf.add(2));
        buf = buf.add(3);
    } else if exp >= 10 {
        write_2_digits(exp as u32, buf);
        buf = buf.add(2);
    } else {
        write_1_digit(exp as u32, buf);
        buf = buf.add(1);
    }

    buf
}
