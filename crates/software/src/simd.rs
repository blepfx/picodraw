#[cfg(target_arch = "x86_64")]
pub fn dispatch<F: FnOnce()>(f: F) {
    if is_x86_feature_detected!("avx2") {
        unsafe { dispatch_avx2(f) }
    } else if is_x86_feature_detected!("avx") {
        unsafe { dispatch_avx1(f) }
    } else if is_x86_feature_detected!("sse4.2") {
        unsafe { dispatch_sse42(f) }
    } else {
        f()
    }

    #[target_feature(enable = "avx2")]
    unsafe fn dispatch_avx2<F: FnOnce()>(f: F) {
        f()
    }

    #[target_feature(enable = "avx")]
    unsafe fn dispatch_avx1<F: FnOnce()>(f: F) {
        f()
    }

    #[target_feature(enable = "sse4.2")]
    unsafe fn dispatch_sse42<F: FnOnce()>(f: F) {
        f()
    }
}

#[cfg(target_arch = "aarch64")]
pub fn dispatch<F: FnOnce()>(f: F) {
    if is_aarch64_feature_detected!("neon") {
        unsafe { dispatch_neon(f) }
    } else {
        f()
    }

    #[target_feature(enable = "neon")]
    unsafe fn dispatch_neon<F: FnOnce()>(f: F) {
        f()
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
pub fn dispatch<F: FnOnce()>(f: F) {
    f()
}
