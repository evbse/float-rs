use core::ffi::{CStr, c_char, c_double, c_float};

#[link(name = "float")]
extern "C" {
    fn from_bytes_f32(d: *const c_char) -> c_float;
    fn from_bytes_f64(d: *const c_char) -> c_double;
}

pub fn from_bytes_f32_c(d: &CStr) -> f32 {
    unsafe { from_bytes_f32(d.as_ptr()) }
}
pub fn from_bytes_f64_c(d: &CStr) -> f64 {
    unsafe { from_bytes_f64(d.as_ptr()) }
}
