// rust 
// plugin-sdk/src/lib.rs

// 1. Define the raw FFI interface (previously plugin-abi)
#[link(wasm_import_module = "api")]
unsafe extern "C" {
    fn host_log_step(step_id: i32);
    fn host_get_step_status(step_id: i32) -> i32;
}

// 2. Expose the clean, safe public API
/// Logs a current business workflow phase.
pub fn log_step(step_id: i32) {
    unsafe {
        host_log_step(step_id);
    }
}

/// Checks whether an external process or human approval step has been completed.
pub fn is_step_completed(step_id: i32) -> bool {
    let result = unsafe { host_get_step_status(step_id) };
    result == 1
}