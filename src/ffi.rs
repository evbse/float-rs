use core::ffi::{CStr, c_char, c_double, c_float, c_uint};

#[link(name = "float")]
extern "C" {
    fn from_bytes_f32(d: *const c_char) -> c_float;
    fn from_bytes_f64(d: *const c_char) -> c_double;

    fn to_bytes_f32(d: *mut c_char, f: c_float) -> c_uint;
    fn to_bytes_f64(d: *mut c_char, f: c_double) -> c_uint;
}

pub fn from_bytes_f32_c(d: &CStr) -> f32 {
    unsafe { from_bytes_f32(d.as_ptr()) }
}
pub fn from_bytes_f64_c(d: &CStr) -> f64 {
    unsafe { from_bytes_f64(d.as_ptr()) }
}

pub fn to_bytes_f32_c(d: *mut c_char, f: f32) -> usize {
    unsafe { to_bytes_f32(d, f as c_float) as usize }
}
pub fn to_bytes_f64_c(d: *mut c_char, f: f64) -> usize {
    unsafe { to_bytes_f64(d, f as c_double) as usize }
}
