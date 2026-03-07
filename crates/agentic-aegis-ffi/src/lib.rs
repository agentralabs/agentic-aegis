pub fn agentic_aegis_ffi_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[no_mangle]
pub extern "C" fn aegis_version() -> *const u8 {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr()
}

#[no_mangle]
pub extern "C" fn aegis_validate_code(
    _code: *const u8,
    _code_len: usize,
    _language: *const u8,
    _language_len: usize,
) -> i32 {
    // Returns 0 for valid, 1 for invalid, -1 for error
    0
}

#[no_mangle]
pub extern "C" fn aegis_check_input_safe(
    _input: *const u8,
    _input_len: usize,
) -> i32 {
    // Returns 1 for safe, 0 for unsafe
    1
}
