extern crate float;

use core::ffi::{CStr};

fn main() {
    let d = "1.0902420340782359E+27";
    let c = unsafe { CStr::from_bytes_with_nul_unchecked(d.as_bytes()) };
    println!("{:?}", float::ffi::from_bytes_f32_c(c));

    let d = "1.0902420340782359E+57";
    let c = unsafe { CStr::from_bytes_with_nul_unchecked(d.as_bytes()) };
    println!("{:?}", float::ffi::from_bytes_f64_c(c));

    let f: f32 = 1.0902420340782359E+27;
    let mut d = Vec::<u8>::with_capacity(32);
    let len = float::ffi::to_bytes_f32_c(d.as_mut_ptr().cast(), f);
    unsafe { d.set_len(len) };
    println!("{:?}", core::str::from_utf8(&d).unwrap());

    let f: f64 = 1.0902420340782359E+57;
    let mut d = Vec::<u8>::with_capacity(64);
    let len = float::ffi::to_bytes_f64_c(d.as_mut_ptr().cast(), f);
    unsafe { d.set_len(len) };
    println!("{:?}", core::str::from_utf8(&d).unwrap());
}
