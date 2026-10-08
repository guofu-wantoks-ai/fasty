use std::error::Error;
use wasmi::{Engine, Linker, Module, Store};

fn main() -> Result<(), Box<dyn Error>> {
    let wasm_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../target/wasm32-unknown-unknown/debug/examples_runner.wasm"
    );

    let wasm_bytes = std::fs::read(wasm_path)?;
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes[..])?;
    let mut store = Store::new(&engine, ());
    let mut linker = Linker::<()>::new(&engine);

    // Match the guest's import module "api" and function "_api_say_hello".
    linker.func_wrap("api", "_api_say_hello", || {
        println!("Hello, world! This function runs on the host.");
    })?;

    let instance = linker.instantiate_and_start(&mut store, &module)?;
    let run = instance.get_typed_func::<(), ()>(&store, "run")?;

    run.call(&mut store, ())?;

    Ok(())
}