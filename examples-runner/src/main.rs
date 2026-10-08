// examples-runner/src/main.rs
// Target: wasm32-unknown-unknown

use plugin_sdk::{log_step, is_step_completed};

#[no_mangle]
pub extern "C" fn execute_bpm_workflow() {
    // Start of process
    log_step(10); 
    
    // Check if the 2-week manager approval step (ID: 20) is done
    if is_step_completed(20) {
        log_step(30); // Advance to final step
    }
}

fn main() {}
