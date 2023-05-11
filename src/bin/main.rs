extern crate float;

use core::ffi::{CStr};

fn main() {
    let d = "1.0902420340782359E+27";
    let c = unsafe { CStr::from_bytes_with_nul_unchecked(d.as_bytes()) };
    println!("{:?}", float::ffi::from_bytes_f32_c(c));

    let d = "1.0902420340782359E+57";
    let c = unsafe { CStr::from_bytes_with_nul_unchecked(d.as_bytes()) };
    println!("{:?}", float::ffi::from_bytes_f64_c(c));
}
