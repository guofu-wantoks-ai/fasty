// examples-runner/src/main.rs
// Target: wasm32-unknown-unknown

use plugin_sdk::{api_say_hello};

#[unsafe(no_mangle)]
pub extern "C" fn run() {
    // Start of process
    api_say_hello(); 
}