extern crate float;

use std::thread::spawn;

fn main() {
    const NTHREADS: u32 = 12;
    const STEP: u32 = u32::MAX / NTHREADS;

    let mut threads = Vec::with_capacity((NTHREADS - 1) as usize);
    for i in 0..NTHREADS - 1 {
        threads.push(spawn(move || {
            let mut b = float::to_bytes::f32::Buffer::new();
            // let mut b = Vec::<u8>::with_capacity(32);
            for j in STEP * i..STEP * (i + 1) {
                let f = f32::from_bits(j);
                if f.is_finite() {
                    let a: f32 = unsafe { core::mem::transmute(j) };
                    let s = b.format(a);
                    // let len = float::ffi::to_bytes_f32_c(b.as_mut_ptr().cast(), f);
                    // unsafe { b.set_len(len) };
                    // let b: f32 = float::ffi::from_bytes_f32_c(unsafe {
                    //     CStr::from_bytes_with_nul_unchecked(s.as_bytes())
                    // });
                    let b = float::from_bytes::parse::<f32>(s.as_bytes());
                    assert!(a == b);
                }
            }
        }));
    }

    let i = NTHREADS - 1;
    let mut b = float::to_bytes::f32::Buffer::new();
    // let mut b = Vec::<u8>::with_capacity(32);
    for j in STEP * i..=u32::MAX {
        let f = f32::from_bits(j);
        if f.is_finite() {
            let a: f32 = unsafe { core::mem::transmute(j) };
            let s = b.format(a);
            // let len = float::ffi::to_bytes_f32_c(b.as_mut_ptr().cast(), f);
            // unsafe { b.set_len(len) };
            // let b: f32 = float::ffi::from_bytes_f32_c(unsafe {
            //     CStr::from_bytes_with_nul_unchecked(s.as_bytes())
            // });
            let b = float::from_bytes::parse::<f32>(s.as_bytes());
            assert!(a == b);
        }
    }

    threads.into_iter().for_each(|t| t.join().unwrap());
}
