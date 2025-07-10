macro_rules! impl_dispatcher {
    ($macro:ident; $($name:ident, $($feature:tt),*;)*) => {
        #[derive(Clone, Copy, Debug)]
        pub enum SimdDispatcher {
            Baseline,
            $($name),*
        }

        impl SimdDispatcher {
            pub fn new() -> Self {
                $(
                    if true $(&& $macro!($feature))* {
                        return Self::$name;
                    }
                )*

                Self::Baseline
            }

            #[inline(always)]
            pub fn dispatch(&self, f: impl FnOnce()) {
                match *self {
                    $(Self::$name => {
                        $(#[target_feature(enable = $feature)])*
                        unsafe fn __dispatch(f: impl FnOnce()) { f() }
                        unsafe { __dispatch(f) }
                    }),*

                    Self::Baseline => f()
                }
            }
        }
    };
}

#[cfg(target_arch = "x86_64")]
impl_dispatcher! {
    is_x86_feature_detected;
    Avx512, "avx512f", "avx512bw", "avx512cd", "avx512dq", "avx512vl";
    Avx2, "avx2", "fma";
    Avx, "avx";
    Sse42, "sse4.1", "sse4.2";
}

#[cfg(target_arch = "aarch64")]
impl_dispatcher! {
    is_aarch64_feature_detected;
    Neon, "neon";
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
impl_dispatcher! {
    __;
}
