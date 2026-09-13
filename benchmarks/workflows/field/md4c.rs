mod adapter {
    use std::ffi::c_void;
    unsafe extern "C" {
        fn md_html(
            input: *const u8,
            len: u32,
            output: extern "C" fn(*const u8, u32, *mut c_void),
            userdata: *mut c_void,
            parser_flags: u32,
            renderer_flags: u32,
        ) -> i32;
    }
    extern "C" fn append(data: *const u8, len: u32, userdata: *mut c_void) {
        if len != 0 {
            // Synchronous callback; the uniquely borrowed output stays live.
            unsafe {
                (&mut *userdata.cast::<Vec<u8>>())
                    .extend_from_slice(std::slice::from_raw_parts(data, len as usize));
            }
        }
    }
    pub struct Renderer;
    impl Renderer {
        pub fn new(flags: u32) -> Self {
            assert_eq!(flags, 7);
            Self
        }
        pub fn render(&self, input: &str) -> Vec<u8> {
            let mut out = Vec::new();
            let result = unsafe {
                md_html(
                    input.as_ptr(),
                    input.len().try_into().unwrap(),
                    append,
                    (&mut out as *mut Vec<u8>).cast(),
                    0x100 | 0x200 | 0x800,
                    0,
                )
            };
            assert_eq!(result, 0);
            out
        }
        pub fn options(&self) -> String {
            "tables=0x100; strikethrough=0x200; tasks=0x800; trusted HTML/URLs; system malloc; normal growing Vec output".into()
        }
    }
}
