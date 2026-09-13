mod adapter {
    use std::ffi::{CStr, c_char};
    unsafe extern "C" {
        fn bench_cmark_init() -> i32;
        fn bench_cmark_options(flags: u32) -> i32;
        fn bench_cmark_render(
            input: *const c_char,
            len: usize,
            flags: u32,
            options: i32,
        ) -> *mut c_char;
        fn bench_cmark_free(output: *mut c_char);
    }
    pub struct Html(*mut c_char);
    impl AsRef<[u8]> for Html {
        fn as_ref(&self) -> &[u8] {
            // The public C API returns an owned NUL-terminated HTML allocation.
            unsafe { CStr::from_ptr(self.0).to_bytes() }
        }
    }
    impl Drop for Html {
        fn drop(&mut self) {
            unsafe {
                bench_cmark_free(self.0);
            }
        }
    }
    pub struct Renderer {
        flags: u32,
        options: i32,
    }
    impl Renderer {
        pub fn new(flags: u32) -> Self {
            unsafe {
                assert_eq!(bench_cmark_init(), 1);
                Self {
                    flags,
                    options: bench_cmark_options(flags),
                }
            }
        }
        pub fn render(&self, input: &str) -> Html {
            let output = unsafe {
                bench_cmark_render(input.as_ptr().cast(), input.len(), self.flags, self.options)
            };
            assert!(!output.is_null());
            Html(output)
        }
        pub fn options(&self) -> String {
            format!(
                "flags={}; cmark_options={}; owned native C output and default allocator",
                self.flags, self.options
            )
        }
    }
}
