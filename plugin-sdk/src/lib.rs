// rust 
// plugin-sdk/src/lib.rs

// 1. Define the raw FFI interface (previously plugin-abi)
#[link(wasm_import_module = "api")]
unsafe extern "C" {
    fn _api_say_hello();
}

// 2. Expose the clean, safe public API
/// Logs a current business workflow phase.
pub fn api_say_hello() {
    unsafe {
        _api_say_hello();
    }
}